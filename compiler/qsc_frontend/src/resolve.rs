// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast,
    visit::{self as ast_visit, Visitor as AstVisitor},
};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    hir::{self, DefId, PackageDefId, PackageId},
    visit::Visitor as HirVisitor,
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

pub(super) type Resolutions = IndexMap<ast::NodeId, Res>;

/// A resolution. This connects a usage of a name with the declaration of that name by uniquely
/// identifying the node that declared it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum Res {
    /// A resolution to a global definition.
    Def(DefId),
    /// A resolution to a local variable.
    Local(ast::NodeId),
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
    resolutions: Resolutions,
    tys: HashMap<&'a str, HashMap<&'a str, DefId>>,
    terms: HashMap<&'a str, HashMap<&'a str, DefId>>,
    opens: HashMap<&'a str, HashMap<&'a str, Span>>,
    namespace: &'a str,
    locals: Vec<HashMap<&'a str, ast::NodeId>>,
    errors: Vec<Error>,
    next_def_id: PackageDefId,
}

impl<'a> Resolver<'a> {
    pub(super) fn resolutions(&self) -> &Resolutions {
        &self.resolutions
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn add_global_callable(&mut self, decl: &'a ast::CallableDecl) {
        let def = self.next_def_id;
        self.next_def_id = PackageDefId(def.0 + 1);
        let def_id = DefId { package: None, def };
        self.resolutions.insert(decl.name.id, Res::Def(def_id));
        self.terms
            .entry(self.namespace)
            .or_default()
            .insert(&decl.name.name, def_id);
    }

    pub(super) fn into_resolutions(self) -> (Resolutions, Vec<Error>) {
        (self.resolutions, self.errors)
    }

    fn resolve_ty(&mut self, path: &ast::Path) {
        match resolve(&self.tys, &self.opens, self.namespace, &[], path) {
            Ok(id) => self.resolutions.insert(path.id, id),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &ast::Path) {
        match resolve(&self.terms, &self.opens, self.namespace, &self.locals, path) {
            Ok(id) => self.resolutions.insert(path.id, id),
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
                self.resolutions.insert(name.id, Res::Local(name.id));
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
    resolutions: Resolutions,
    tys: HashMap<&'a str, HashMap<&'a str, DefId>>,
    terms: HashMap<&'a str, HashMap<&'a str, DefId>>,
    package: Option<PackageId>,
    namespace: &'a str,
    next_def_id: PackageDefId,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new() -> Self {
        Self {
            resolutions: Resolutions::new(),
            tys: HashMap::new(),
            terms: HashMap::new(),
            package: None,
            namespace: "",
            next_def_id: PackageDefId(0),
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
            next_def_id: self.next_def_id,
        }
    }
}

impl<'a> AstVisitor<'a> for GlobalTable<'a> {
    fn visit_namespace(&mut self, namespace: &'a ast::Namespace) {
        self.namespace = &namespace.name.name;
        let def = self.next_def_id;
        self.next_def_id = PackageDefId(def.0 + 1);
        let def_id = DefId {
            package: self.package,
            def,
        };
        self.resolutions.insert(namespace.name.id, Res::Def(def_id));
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
                let def = self.next_def_id;
                self.next_def_id = PackageDefId(def.0 + 1);
                let def_id = DefId {
                    package: self.package,
                    def,
                };
                self.resolutions.insert(decl.name.id, Res::Def(def_id));
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&decl.name.name, def_id);
            }
            ast::ItemKind::Ty(name, _) => {
                let def = self.next_def_id;
                self.next_def_id = PackageDefId(def.0 + 1);
                let def_id = DefId {
                    package: self.package,
                    def,
                };
                self.resolutions.insert(name.id, Res::Def(def_id));
                self.tys
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, def_id);
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, def_id);
            }
            ast::ItemKind::Err | ast::ItemKind::Open(..) => {}
        }
    }
}

impl<'a> HirVisitor<'a> for GlobalTable<'a> {
    fn visit_package(&mut self, package: &'a hir::Package) {
        let package_id = self
            .package
            .expect("package ID should be set before visiting HIR");

        for (def, item) in package.items.iter() {
            if item.visibility.map(|v| v.kind) == Some(hir::VisibilityKind::Internal) {
                continue;
            }

            let Some(parent) = item.parent else { continue; };

            let def_id = DefId {
                package: Some(package_id),
                def,
            };

            let hir::ItemKind::Namespace(namespace, _) = &package.items.get(parent).expect("").kind else {
                panic!("parent item is not a namespace");
            };

            match &item.kind {
                hir::ItemKind::Callable(decl) => {
                    self.terms
                        .entry(&namespace.name)
                        .or_default()
                        .insert(&decl.name.name, def_id);
                }
                hir::ItemKind::Ty(name, _) => {
                    self.tys
                        .entry(&namespace.name)
                        .or_default()
                        .insert(&name.name, def_id);
                    self.terms
                        .entry(&namespace.name)
                        .or_default()
                        .insert(&name.name, def_id);
                }
                hir::ItemKind::Err | hir::ItemKind::Namespace(..) => {}
            }
        }
    }
}

fn resolve(
    globals: &HashMap<&str, HashMap<&str, DefId>>,
    opens: &HashMap<&str, HashMap<&str, Span>>,
    parent: &str,
    locals: &[HashMap<&str, ast::NodeId>],
    path: &ast::Path,
) -> Result<Res, Error> {
    let name = path.name.name.as_str();
    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    if namespace.is_empty() {
        if let Some(&id) = locals.iter().rev().find_map(|env| env.get(name)) {
            // Locals shadow everything.
            return Ok(Res::Local(id));
        } else if let Some(&id) = globals.get(parent).and_then(|env| env.get(name)) {
            // Items in the parent namespace shadow opens.
            return Ok(Res::Def(id));
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
        assert!(candidates.len() <= 1, "ambiguity in prelude resolution");
        if let Some(id) = single(candidates) {
            return Ok(Res::Def(id));
        }
    }

    if open_candidates.is_empty() {
        if let Some(&id) = globals.get(namespace).and_then(|env| env.get(name)) {
            // An unopened global is the last resort.
            return Ok(Res::Def(id));
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
            .map(Res::Def)
            .ok_or_else(|| Error::NotFound(name.to_string(), path.span))
    }
}

fn resolve_implicit_opens<'a>(
    globals: &HashMap<&str, HashMap<&str, DefId>>,
    namespaces: impl IntoIterator<Item = &'a &'a str>,
    name: &str,
) -> HashSet<DefId> {
    let mut candidates = HashSet::new();
    for namespace in namespaces {
        if let Some(&id) = globals.get(namespace).and_then(|env| env.get(name)) {
            candidates.insert(id);
        }
    }
    candidates
}

fn resolve_explicit_opens<'a>(
    globals: &HashMap<&str, HashMap<&str, DefId>>,
    namespaces: impl IntoIterator<Item = (&'a &'a str, &'a Span)>,
    name: &str,
) -> HashMap<DefId, Span> {
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
