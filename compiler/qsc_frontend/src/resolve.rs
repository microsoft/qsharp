// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::compile::PackageId;
use miette::Diagnostic;
use qsc_ast::{
    ast,
    visit::{self as ast_visit, Visitor as AstVisitor},
};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    hir,
    visit::{self as hir_visit, Visitor as HirVisitor},
};
use std::{
    collections::{HashMap, HashSet},
    mem, vec,
};
use thiserror::Error;

const PRELUDE: &[&str] = &[
    "Microsoft.Quantum.Canon",
    "Microsoft.Quantum.Core",
    "Microsoft.Quantum.Intrinsic",
];

pub type Resolutions<Id> = IndexMap<Id, Res<Id>>;

/// A resolution. This connects a usage of a name with the declaration of that name by uniquely
/// identifying the node that declared it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Res<Id> {
    Internal(Id),
    External(PackageId, hir::NodeId),
}

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("`{0}` not found in this scope")]
    NotFound(String, #[label("not found")] Span),

    #[error("`{0}` is ambiguous")]
    Ambiguous(
        String,
        #[label("ambiguous name")] Span,
        #[label("could refer to the item in this namespace")] Span,
        #[label("could also refer to the item in this namespace")] Span,
    ),
}

pub(super) struct Resolver<'a> {
    resolutions: Vec<(ast::NodeId, Res<ast::NodeId>)>,
    tys: HashMap<&'a str, HashMap<&'a str, Res<ast::NodeId>>>,
    terms: HashMap<&'a str, HashMap<&'a str, Res<ast::NodeId>>>,
    opens: HashMap<&'a str, HashMap<&'a str, Span>>,
    namespace: &'a str,
    locals: Vec<HashMap<&'a str, ast::NodeId>>,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub(super) fn drain_resolutions(&mut self) -> vec::Drain<(ast::NodeId, Res<ast::NodeId>)> {
        self.resolutions.drain(..)
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn add_global_callable(&mut self, decl: &'a ast::CallableDecl) {
        let res = Res::Internal(decl.name.id);
        self.resolutions.push((decl.name.id, res));
        self.terms
            .entry(self.namespace)
            .or_default()
            .insert(&decl.name.name, res);
    }

    pub(super) fn into_resolutions(self) -> (Resolutions<ast::NodeId>, Vec<Error>) {
        (self.resolutions.into_iter().collect(), self.errors)
    }

    fn resolve_ty(&mut self, path: &ast::Path) {
        match resolve(&self.tys, &self.opens, self.namespace, &[], path) {
            Ok(id) => self.resolutions.push((path.id, id)),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &ast::Path) {
        match resolve(&self.terms, &self.opens, self.namespace, &self.locals, path) {
            Ok(id) => self.resolutions.push((path.id, id)),
            Err(err) => self.errors.push(err),
        }
    }

    fn with_pat(&mut self, pat: &'a ast::Pat, f: impl FnOnce(&mut Self)) {
        let mut env = HashMap::new();
        self.with_scope(&mut env, |resolver| {
            resolver.bind(pat);
            f(resolver);
        });
    }

    pub(super) fn with_scope(
        &mut self,
        scope: &mut HashMap<&'a str, ast::NodeId>,
        f: impl FnOnce(&mut Self),
    ) {
        self.locals.push(mem::take(scope));
        f(self);
        *scope = self
            .locals
            .pop()
            .expect("scope symmetry should be preserved");
    }

    fn bind(&mut self, pat: &'a ast::Pat) {
        match &pat.kind {
            ast::PatKind::Bind(name, _) => {
                let env = self
                    .locals
                    .last_mut()
                    .expect("binding should have environment");
                self.resolutions.push((name.id, Res::Internal(name.id)));
                env.insert(name.name.as_str(), name.id);
            }
            ast::PatKind::Discard(_) | ast::PatKind::Elided => {}
            ast::PatKind::Paren(pat) => self.bind(pat),
            ast::PatKind::Tuple(pats) => pats.iter().for_each(|p| self.bind(p)),
        }
    }
}

impl<'a> AstVisitor<'a> for Resolver<'a> {
    fn visit_namespace(&mut self, namespace: &'a ast::Namespace) {
        self.opens = HashMap::new();
        self.namespace = &namespace.name.name;
        for item in &namespace.items {
            if let ast::ItemKind::Open(name, alias) = &item.kind {
                let alias = alias.as_ref().map_or("", |a| &a.name);
                self.opens
                    .entry(alias)
                    .or_default()
                    .insert(&name.name, name.span);
            }
        }

        ast_visit::walk_namespace(self, namespace);
        self.namespace = "";
    }

    fn visit_callable_decl(&mut self, decl: &'a ast::CallableDecl) {
        self.with_pat(&decl.input, |resolver| {
            ast_visit::walk_callable_decl(resolver, decl);
        });
    }

    fn visit_spec_decl(&mut self, decl: &'a ast::SpecDecl) {
        if let ast::SpecBody::Impl(input, block) = &decl.body {
            self.with_pat(input, |resolver| resolver.visit_block(block));
        } else {
            ast_visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_ty(&mut self, ty: &'a ast::Ty) {
        if let ast::TyKind::Path(path) = &ty.kind {
            self.resolve_ty(path);
        } else {
            ast_visit::walk_ty(self, ty);
        }
    }

    fn visit_block(&mut self, block: &'a ast::Block) {
        self.with_scope(&mut HashMap::new(), |resolver| {
            ast_visit::walk_block(resolver, block);
        });
    }

    fn visit_stmt(&mut self, stmt: &'a ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Local(_, pat, _) => {
                ast_visit::walk_stmt(self, stmt);
                self.bind(pat);
            }
            ast::StmtKind::Qubit(_, pat, init, block) => {
                ast_visit::walk_qubit_init(self, init);
                self.bind(pat);
                if let Some(block) = block {
                    ast_visit::walk_block(self, block);
                }
            }
            ast::StmtKind::Empty | ast::StmtKind::Expr(..) | ast::StmtKind::Semi(..) => {
                ast_visit::walk_stmt(self, stmt);
            }
        }
    }

    fn visit_expr(&mut self, expr: &'a ast::Expr) {
        match &expr.kind {
            ast::ExprKind::For(pat, iter, block) => {
                self.visit_expr(iter);
                self.with_pat(pat, |resolver| resolver.visit_block(block));
            }
            ast::ExprKind::Repeat(repeat, cond, fixup) => {
                self.with_scope(&mut HashMap::new(), |resolver| {
                    repeat
                        .stmts
                        .iter()
                        .for_each(|stmt| resolver.visit_stmt(stmt));
                    resolver.visit_expr(cond);
                    if let Some(block) = fixup.as_ref() {
                        block
                            .stmts
                            .iter()
                            .for_each(|stmt| resolver.visit_stmt(stmt));
                    }
                });
            }
            ast::ExprKind::Lambda(_, input, output) => {
                self.with_pat(input, |resolver| resolver.visit_expr(output));
            }
            ast::ExprKind::Path(path) => self.resolve_term(path),
            _ => ast_visit::walk_expr(self, expr),
        }
    }
}

pub(super) struct GlobalTable<'a> {
    resolutions: Vec<(ast::NodeId, Res<ast::NodeId>)>,
    tys: HashMap<&'a str, HashMap<&'a str, Res<ast::NodeId>>>,
    terms: HashMap<&'a str, HashMap<&'a str, Res<ast::NodeId>>>,
    package: Option<PackageId>,
    namespace: &'a str,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new() -> Self {
        Self {
            resolutions: Vec::new(),
            tys: HashMap::new(),
            terms: HashMap::new(),
            package: None,
            namespace: "",
        }
    }

    pub(super) fn set_package(&mut self, package: PackageId) {
        self.package = Some(package);
    }

    pub(super) fn into_resolver(self) -> Resolver<'a> {
        Resolver {
            resolutions: self.resolutions,
            tys: self.tys,
            terms: self.terms,
            opens: HashMap::new(),
            namespace: "",
            locals: Vec::new(),
            errors: Vec::new(),
        }
    }
}

impl<'a> AstVisitor<'a> for GlobalTable<'a> {
    fn visit_namespace(&mut self, namespace: &'a ast::Namespace) {
        self.namespace = &namespace.name.name;
        ast_visit::walk_namespace(self, namespace);
        self.namespace = "";
    }

    fn visit_item(&mut self, item: &'a ast::Item) {
        assert!(
            self.package.is_none(),
            "package ID should not be set before visiting AST"
        );

        match &item.kind {
            ast::ItemKind::Callable(decl) => {
                let res = Res::Internal(decl.name.id);
                self.resolutions.push((decl.name.id, res));
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&decl.name.name, res);
            }
            ast::ItemKind::Ty(name, _) => {
                let res = Res::Internal(name.id);
                self.resolutions.push((name.id, res));
                self.tys
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, res);
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, res);
            }
            ast::ItemKind::Err | ast::ItemKind::Open(..) => {}
        }
    }
}

impl<'a> HirVisitor<'a> for GlobalTable<'a> {
    fn visit_namespace(&mut self, namespace: &'a hir::Namespace) {
        self.namespace = &namespace.name.name;
        hir_visit::walk_namespace(self, namespace);
        self.namespace = "";
    }

    fn visit_item(&mut self, item: &'a hir::Item) {
        let package = self
            .package
            .expect("package ID should be set before visiting HIR");

        if item.visibility.map(|v| v.kind) == Some(hir::VisibilityKind::Internal) {
            return;
        }

        match &item.kind {
            hir::ItemKind::Callable(decl) => {
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&decl.name.name, Res::External(package, decl.name.id));
            }
            hir::ItemKind::Ty(name, _) => {
                let res = Res::External(package, name.id);
                self.tys
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, res);
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, res);
            }
            hir::ItemKind::Err | hir::ItemKind::Open(..) => {}
        }
    }
}

fn resolve(
    globals: &HashMap<&str, HashMap<&str, Res<ast::NodeId>>>,
    opens: &HashMap<&str, HashMap<&str, Span>>,
    parent: &str,
    locals: &[HashMap<&str, ast::NodeId>],
    path: &ast::Path,
) -> Result<Res<ast::NodeId>, Error> {
    let name = path.name.name.as_str();
    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    if namespace.is_empty() {
        if let Some(&id) = locals.iter().rev().find_map(|env| env.get(name)) {
            // Locals shadow everything.
            return Ok(Res::Internal(id));
        } else if let Some(&id) = globals.get(parent).and_then(|env| env.get(name)) {
            // Items in the parent namespace shadow opens.
            return Ok(id);
        }
    }

    // Explicit opens shadow prelude and unopened globals.
    let open_candidates = opens
        .get(namespace)
        .map(|open_namespaces| resolve_explicit_opens(globals, open_namespaces, name))
        .unwrap_or_default();

    if open_candidates.is_empty() && namespace.is_empty() {
        // Prelude shadows unopened globals.
        let candidates = resolve_implicit_opens(globals, PRELUDE, name);
        assert!(candidates.len() <= 1, "Ambiguity in prelude resolution.");
        if let Some(id) = single(candidates) {
            return Ok(id);
        }
    }

    if open_candidates.is_empty() {
        if let Some(&id) = globals.get(namespace).and_then(|env| env.get(name)) {
            // An unopened global is the last resort.
            return Ok(id);
        }
    }

    if open_candidates.len() > 1 {
        let mut spans: Vec<_> = open_candidates.into_values().collect();
        spans.sort();
        Err(Error::Ambiguous(
            name.to_string(),
            path.span,
            spans[0],
            spans[1],
        ))
    } else {
        single(open_candidates.into_keys())
            .ok_or_else(|| Error::NotFound(name.to_string(), path.span))
    }
}

fn resolve_implicit_opens<'a>(
    globals: &HashMap<&str, HashMap<&str, Res<ast::NodeId>>>,
    namespaces: impl IntoIterator<Item = &'a &'a str>,
    name: &str,
) -> HashSet<Res<ast::NodeId>> {
    let mut candidates = HashSet::new();
    for namespace in namespaces {
        if let Some(&id) = globals.get(namespace).and_then(|env| env.get(name)) {
            candidates.insert(id);
        }
    }
    candidates
}

fn resolve_explicit_opens<'a>(
    globals: &HashMap<&str, HashMap<&str, Res<ast::NodeId>>>,
    namespaces: impl IntoIterator<Item = (&'a &'a str, &'a Span)>,
    name: &str,
) -> HashMap<Res<ast::NodeId>, Span> {
    let mut candidates = HashMap::new();
    for (&namespace, &span) in namespaces {
        if let Some(&id) = globals.get(namespace).and_then(|env| env.get(name)) {
            candidates.insert(id, span);
        }
    }
    candidates
}

fn single<T>(xs: impl IntoIterator<Item = T>) -> Option<T> {
    let mut xs = xs.into_iter();
    let x = xs.next();
    match xs.next() {
        None => x,
        Some(_) => None,
    }
}
