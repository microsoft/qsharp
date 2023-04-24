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
use qsc_hir::hir::{self, ItemId, LocalItemId, PackageId};
use std::{
    collections::{HashMap, HashSet},
    mem,
    rc::Rc,
    vec,
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
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum Res {
    /// A global item.
    Item(ItemId),
    /// A local variable.
    Local(ast::NodeId),
    /// A primitive type.
    PrimTy(hir::PrimTy),
    /// The unit type.
    UnitTy,
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

pub(super) struct Resolver {
    resolutions: Resolutions,
    tys: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    terms: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    opens: HashMap<Rc<str>, HashMap<Rc<str>, Span>>,
    namespace: Rc<str>,
    next_item_id: LocalItemId,
    locals: Vec<HashMap<Rc<str>, ast::NodeId>>,
    errors: Vec<Error>,
}

impl Resolver {
    pub(super) fn resolutions(&self) -> &Resolutions {
        &self.resolutions
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn add_global_callable(&mut self, decl: &ast::CallableDecl) {
        let res = Res::Item(ItemId {
            package: None,
            item: self.next_item_id,
        });
        self.next_item_id = self.next_item_id.successor();
        self.resolutions.insert(decl.name.id, res);
        self.terms
            .entry(Rc::clone(&self.namespace))
            .or_default()
            .insert(Rc::clone(&decl.name.name), res);
    }

    pub(super) fn into_resolutions(self) -> (Resolutions, Vec<Error>) {
        (self.resolutions, self.errors)
    }

    fn resolve_ty(&mut self, path: &ast::Path) {
        match resolve(&self.tys, &self.opens, &self.namespace, &[], path) {
            Ok(id) => self.resolutions.insert(path.id, id),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &ast::Path) {
        match resolve(
            &self.terms,
            &self.opens,
            &self.namespace,
            &self.locals,
            path,
        ) {
            Ok(id) => self.resolutions.insert(path.id, id),
            Err(err) => self.errors.push(err),
        }
    }

    fn with_pat(&mut self, pat: &ast::Pat, f: impl FnOnce(&mut Self)) {
        let mut env = HashMap::new();
        self.with_scope(&mut env, |resolver| {
            resolver.bind(pat);
            f(resolver);
        });
    }

    pub(super) fn with_scope(
        &mut self,
        scope: &mut HashMap<Rc<str>, ast::NodeId>,
        f: impl FnOnce(&mut Self),
    ) {
        self.locals.push(mem::take(scope));
        f(self);
        *scope = self
            .locals
            .pop()
            .expect("scope symmetry should be preserved");
    }

    fn bind(&mut self, pat: &ast::Pat) {
        match &pat.kind {
            ast::PatKind::Bind(name, _) => {
                let env = self
                    .locals
                    .last_mut()
                    .expect("binding should have environment");
                self.resolutions.insert(name.id, Res::Local(name.id));
                env.insert(Rc::clone(&name.name), name.id);
            }
            ast::PatKind::Discard(_) | ast::PatKind::Elided => {}
            ast::PatKind::Paren(pat) => self.bind(pat),
            ast::PatKind::Tuple(pats) => pats.iter().for_each(|p| self.bind(p)),
        }
    }
}

impl AstVisitor<'_> for Resolver {
    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        self.opens = HashMap::new();
        self.namespace = Rc::clone(&namespace.name.name);
        for item in &namespace.items {
            if let ast::ItemKind::Open(name, alias) = &item.kind {
                let alias = alias.as_ref().map_or("".into(), |a| Rc::clone(&a.name));
                self.opens
                    .entry(alias)
                    .or_default()
                    .insert(Rc::clone(&name.name), name.span);
            }
        }

        ast_visit::walk_namespace(self, namespace);
        self.namespace = "".into();
    }

    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        self.with_pat(&decl.input, |resolver| {
            ast_visit::walk_callable_decl(resolver, decl);
        });
    }

    fn visit_spec_decl(&mut self, decl: &ast::SpecDecl) {
        if let ast::SpecBody::Impl(input, block) = &decl.body {
            self.with_pat(input, |resolver| resolver.visit_block(block));
        } else {
            ast_visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_ty(&mut self, ty: &ast::Ty) {
        if let ast::TyKind::Path(path) = &ty.kind {
            self.resolve_ty(path);
        } else {
            ast_visit::walk_ty(self, ty);
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        self.with_scope(&mut HashMap::new(), |resolver| {
            ast_visit::walk_block(resolver, block);
        });
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
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

    fn visit_expr(&mut self, expr: &ast::Expr) {
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

pub(super) struct GlobalTable {
    resolutions: Resolutions,
    tys: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    terms: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    next_item_id: LocalItemId,
}

impl GlobalTable {
    pub(super) fn new() -> Self {
        let tys = HashMap::from([(
            "Microsoft.Quantum.Core".into(),
            HashMap::from([
                ("BigInt".into(), Res::PrimTy(hir::PrimTy::BigInt)),
                ("Bool".into(), Res::PrimTy(hir::PrimTy::Bool)),
                ("Double".into(), Res::PrimTy(hir::PrimTy::Double)),
                ("Int".into(), Res::PrimTy(hir::PrimTy::Int)),
                ("Pauli".into(), Res::PrimTy(hir::PrimTy::Pauli)),
                ("Qubit".into(), Res::PrimTy(hir::PrimTy::Qubit)),
                ("Range".into(), Res::PrimTy(hir::PrimTy::Range)),
                ("Result".into(), Res::PrimTy(hir::PrimTy::Result)),
                ("String".into(), Res::PrimTy(hir::PrimTy::String)),
                ("Unit".into(), Res::UnitTy),
            ]),
        )]);

        Self {
            resolutions: Resolutions::new(),
            tys,
            terms: HashMap::new(),
            next_item_id: LocalItemId::default(),
        }
    }

    pub(super) fn add_local_package(&mut self, package: &ast::Package) {
        for namespace in &package.namespaces {
            let item_id = self.next_item_id();
            self.resolutions
                .insert(namespace.name.id, Res::Item(item_id));

            for item in &namespace.items {
                match &item.kind {
                    ast::ItemKind::Callable(decl) => {
                        let res = Res::Item(self.next_item_id());
                        self.resolutions.insert(decl.name.id, res);
                        self.terms
                            .entry(Rc::clone(&namespace.name.name))
                            .or_default()
                            .insert(Rc::clone(&decl.name.name), res);
                    }
                    ast::ItemKind::Ty(name, _) => {
                        let res = Res::Item(self.next_item_id());
                        self.resolutions.insert(name.id, res);
                        self.tys
                            .entry(Rc::clone(&namespace.name.name))
                            .or_default()
                            .insert(Rc::clone(&name.name), res);
                        self.terms
                            .entry(Rc::clone(&namespace.name.name))
                            .or_default()
                            .insert(Rc::clone(&name.name), res);
                    }
                    ast::ItemKind::Err | ast::ItemKind::Open(..) => {}
                }
            }
        }
    }

    pub(super) fn add_external_package(&mut self, id: PackageId, package: &hir::Package) {
        for item in package.items.values() {
            if item.visibility.map(|v| v.kind) == Some(hir::VisibilityKind::Internal) {
                continue;
            }
            let Some(parent) = item.parent else { continue; };
            let hir::ItemKind::Namespace(namespace, _) =
                &package.items.get(parent).expect("parent should exist").kind else { continue; };

            let res = Res::Item(ItemId {
                package: Some(id),
                item: item.id,
            });

            match &item.kind {
                hir::ItemKind::Callable(decl) => {
                    self.terms
                        .entry(Rc::clone(&namespace.name))
                        .or_default()
                        .insert(Rc::clone(&decl.name.name), res);
                }
                hir::ItemKind::Ty(name, _) => {
                    self.tys
                        .entry(Rc::clone(&namespace.name))
                        .or_default()
                        .insert(Rc::clone(&name.name), res);
                    self.terms
                        .entry(Rc::clone(&namespace.name))
                        .or_default()
                        .insert(Rc::clone(&name.name), res);
                }
                hir::ItemKind::Err | hir::ItemKind::Namespace(..) => {}
            }
        }
    }

    pub(super) fn into_resolver(self) -> Resolver {
        Resolver {
            resolutions: self.resolutions,
            tys: self.tys,
            terms: self.terms,
            opens: HashMap::new(),
            namespace: "".into(),
            next_item_id: self.next_item_id,
            locals: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn next_item_id(&mut self) -> ItemId {
        let item = ItemId {
            package: None,
            item: self.next_item_id,
        };
        self.next_item_id = self.next_item_id.successor();
        item
    }
}

fn resolve(
    globals: &HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    opens: &HashMap<Rc<str>, HashMap<Rc<str>, Span>>,
    parent: &Rc<str>,
    locals: &[HashMap<Rc<str>, ast::NodeId>],
    path: &ast::Path,
) -> Result<Res, Error> {
    let name = path.name.name.as_ref();
    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    if namespace.is_empty() {
        if let Some(&node) = locals.iter().rev().find_map(|env| env.get(name)) {
            // Locals shadow everything.
            return Ok(Res::Local(node));
        } else if let Some(&res) = globals.get(parent).and_then(|env| env.get(name)) {
            // Items in the parent namespace shadow opens.
            return Ok(res);
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
        if let Some(res) = single(candidates) {
            return Ok(res);
        }
    }

    if open_candidates.is_empty() {
        if let Some(&res) = globals.get(namespace).and_then(|env| env.get(name)) {
            // An unopened global is the last resort.
            return Ok(res);
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

fn resolve_implicit_opens(
    globals: &HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    namespaces: impl IntoIterator<Item = impl AsRef<str>>,
    name: &str,
) -> HashSet<Res> {
    let mut candidates = HashSet::new();
    for namespace in namespaces {
        let namespace = namespace.as_ref();
        if let Some(&id) = globals.get(namespace).and_then(|env| env.get(name)) {
            candidates.insert(id);
        }
    }
    candidates
}

fn resolve_explicit_opens<'a>(
    globals: &HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    namespaces: impl IntoIterator<Item = (impl AsRef<str>, &'a Span)>,
    name: &str,
) -> HashMap<Res, Span> {
    let mut candidates = HashMap::new();
    for (namespace, &span) in namespaces {
        let namespace = namespace.as_ref();
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
