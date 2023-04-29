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
use qsc_hir::hir::{self, ItemId, LocalItemId, PackageId, PrimTy};
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
    PrimTy(PrimTy),
    /// The unit type.
    UnitTy,
}

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("`{0}` not found in this scope")]
    NotFound(String, #[label] Span),

    #[error("`{name}` could refer to the item in `{first_open}` or `{second_open}`")]
    Ambiguous {
        name: String,
        first_open: String,
        second_open: String,
        #[label("ambiguous name")]
        name_span: Span,
        #[label("found in this namespace")]
        first_open_span: Span,
        #[label("and also in this namespace")]
        second_open_span: Span,
    },
}

struct Open {
    namespace: Rc<str>,
    span: Span,
}

pub(super) struct Resolver {
    resolutions: Resolutions,
    global_tys: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    global_terms: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    parent_namespace: Rc<str>,
    opens: Vec<HashMap<Rc<str>, Vec<Open>>>,
    tys: Vec<HashMap<Rc<str>, ItemId>>,
    terms: Vec<HashMap<Rc<str>, ItemId>>,
    vars: Vec<HashMap<Rc<str>, ast::NodeId>>,
    next_item_id: LocalItemId,
    errors: Vec<Error>,
}

impl Resolver {
    pub(super) fn resolutions(&self) -> &Resolutions {
        &self.resolutions
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn into_resolutions(self) -> (Resolutions, Vec<Error>) {
        (self.resolutions, self.errors)
    }

    fn resolve_ty(&mut self, path: &ast::Path) {
        let local = |name| {
            self.tys
                .iter()
                .rev()
                .find_map(|scope| scope.get(name).copied().map(Res::Item))
        };

        match resolve(
            &self.global_tys,
            &self.opens,
            &self.parent_namespace,
            local,
            path,
        ) {
            Ok(id) => self.resolutions.insert(path.id, id),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &ast::Path) {
        let local = |name| {
            let mut local_vars = self.vars.iter().rev();
            let mut local_terms = self.terms.iter().rev();
            loop {
                let var_env = local_vars.next();
                let term_env = local_terms.next();

                if let Some(env) = var_env {
                    if let Some(&id) = env.get(name) {
                        return Some(Res::Local(id));
                    }
                }

                if let Some(env) = term_env {
                    if let Some(&id) = env.get(name) {
                        return Some(Res::Item(id));
                    }
                }

                if var_env.is_none() && term_env.is_none() {
                    return None;
                }
            }
        };

        match resolve(
            &self.global_terms,
            &self.opens,
            &self.parent_namespace,
            local,
            path,
        ) {
            Ok(id) => self.resolutions.insert(path.id, id),
            Err(err) => self.errors.push(err),
        }
    }

    fn with_pat(&mut self, pat: &ast::Pat, f: impl FnOnce(&mut Self)) {
        self.with_scope(|resolver| {
            resolver.bind_pat(pat);
            f(resolver);
        });
    }

    fn with_scope(&mut self, f: impl FnOnce(&mut Self)) {
        self.add_scope();
        f(self);
        self.vars.pop().expect("variable scope should be symmetric");
        self.terms.pop().expect("term scope should be symmetric");
        self.tys.pop().expect("type scope should be symmetric");
        self.opens.pop().expect("open scope should be symmetric");
    }

    pub(super) fn add_scope(&mut self) {
        self.opens.push(HashMap::new());
        self.tys.push(HashMap::new());
        self.terms.push(HashMap::new());
        self.vars.push(HashMap::new());
    }

    fn bind_pat(&mut self, pat: &ast::Pat) {
        match &pat.kind {
            ast::PatKind::Bind(name, _) => {
                let scope = self.vars.last_mut().expect("binding should have scope");
                self.resolutions.insert(name.id, Res::Local(name.id));
                scope.insert(Rc::clone(&name.name), name.id);
            }
            ast::PatKind::Discard(_) | ast::PatKind::Elided => {}
            ast::PatKind::Paren(pat) => self.bind_pat(pat),
            ast::PatKind::Tuple(pats) => pats.iter().for_each(|p| self.bind_pat(p)),
        }
    }

    fn bind_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::Callable(decl) => {
                let item_id = ItemId {
                    package: None,
                    item: self.next_item_id,
                };
                self.next_item_id = self.next_item_id.successor();
                self.resolutions.insert(decl.name.id, Res::Item(item_id));

                let scope = self.terms.last_mut().expect("binding should have scope");
                scope.insert(Rc::clone(&decl.name.name), item_id);
            }
            ast::ItemKind::Open(name, alias) => {
                let alias = alias.as_ref().map_or("".into(), |a| Rc::clone(&a.name));
                self.opens
                    .last_mut()
                    .expect("open item should have scope")
                    .entry(alias)
                    .or_default()
                    .push(Open {
                        namespace: Rc::clone(&name.name),
                        span: name.span,
                    });
            }
            ast::ItemKind::Ty(name, _) if self.resolutions.get(name.id).is_none() => {
                // new type, who dis?
                let item_id = ItemId {
                    package: None,
                    item: self.next_item_id,
                };
                self.next_item_id = self.next_item_id.successor();
                self.resolutions.insert(name.id, Res::Item(item_id));

                let ty_scope = self.tys.last_mut().expect("binding should have scope");
                ty_scope.insert(Rc::clone(&name.name), item_id);

                let term_scope = self.terms.last_mut().expect("binding should have scope");
                term_scope.insert(Rc::clone(&name.name), item_id);
            }
            ast::ItemKind::Ty(_, _) => todo!(),
            ast::ItemKind::Err => {}
        }
    }
}

impl AstVisitor<'_> for Resolver {
    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        self.parent_namespace = Rc::clone(&namespace.name.name);
        let mut opens: HashMap<_, Vec<_>> = HashMap::new();
        for item in &namespace.items {
            if let ast::ItemKind::Open(name, alias) = &item.kind {
                let alias = alias.as_ref().map_or("".into(), |a| Rc::clone(&a.name));
                opens.entry(alias).or_default().push(Open {
                    namespace: Rc::clone(&name.name),
                    span: name.span,
                });
            }
        }

        self.opens.push(opens);
        ast_visit::walk_namespace(self, namespace);
        self.opens.pop().expect("open scope should be symmetric");
        self.parent_namespace = "".into();
    }

    fn visit_item(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::Open(name, alias) => {
                let alias = alias.as_ref().map_or("".into(), |a| Rc::clone(&a.name));
                self.opens
                    .last_mut()
                    .expect("open item should have scope")
                    .entry(alias)
                    .or_default()
                    .push(Open {
                        namespace: Rc::clone(&name.name),
                        span: name.span,
                    });
            }
            ast::ItemKind::Ty(name, _) if self.resolutions.get(name.id).is_none() => {
                // new type, who dis?
                let item_id = ItemId {
                    package: None,
                    item: self.next_item_id,
                };
                self.next_item_id = self.next_item_id.successor();
                self.resolutions.insert(name.id, Res::Item(item_id));

                let ty_scope = self.tys.last_mut().expect("binding should have scope");
                ty_scope.insert(Rc::clone(&name.name), item_id);

                let term_scope = self.terms.last_mut().expect("binding should have scope");
                term_scope.insert(Rc::clone(&name.name), item_id);
            }
            ast::ItemKind::Ty(..) | ast::ItemKind::Callable(_) | ast::ItemKind::Err => {}
        }

        ast_visit::walk_item(self, item);
    }

    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        if self.resolutions.get(decl.name.id).is_none() {
            // new callable, who dis?
            let item_id = ItemId {
                package: None,
                item: self.next_item_id,
            };
            self.next_item_id = self.next_item_id.successor();
            self.resolutions.insert(decl.name.id, Res::Item(item_id));

            let scope = self.terms.last_mut().expect("binding should have scope");
            scope.insert(Rc::clone(&decl.name.name), item_id);
        }

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
        self.with_scope(|resolver| {
            for stmt in &block.stmts {
                if let ast::StmtKind::Item(item) = &stmt.kind {
                    resolver.bind_item(item);
                }
            }

            ast_visit::walk_block(resolver, block);
        });
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Item(item) => {
                let vars = mem::take(&mut self.vars);
                self.visit_item(item);
                self.vars = vars;
            }
            ast::StmtKind::Local(_, pat, _) => {
                ast_visit::walk_stmt(self, stmt);
                self.bind_pat(pat);
            }
            ast::StmtKind::Qubit(_, pat, init, block) => {
                ast_visit::walk_qubit_init(self, init);
                self.bind_pat(pat);
                if let Some(block) = block {
                    ast_visit::walk_block(self, block);
                }
            }
            ast::StmtKind::Empty | ast::StmtKind::Expr(_) | ast::StmtKind::Semi(_) => {
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
                self.with_scope(|resolver| {
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
                ("BigInt".into(), Res::PrimTy(PrimTy::BigInt)),
                ("Bool".into(), Res::PrimTy(PrimTy::Bool)),
                ("Double".into(), Res::PrimTy(PrimTy::Double)),
                ("Int".into(), Res::PrimTy(PrimTy::Int)),
                ("Pauli".into(), Res::PrimTy(PrimTy::Pauli)),
                ("Qubit".into(), Res::PrimTy(PrimTy::Qubit)),
                ("Range".into(), Res::PrimTy(PrimTy::Range)),
                ("Result".into(), Res::PrimTy(PrimTy::Result)),
                ("String".into(), Res::PrimTy(PrimTy::String)),
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
            global_tys: self.tys,
            global_terms: self.terms,
            parent_namespace: "".into(),
            opens: Vec::new(),
            tys: Vec::new(),
            terms: Vec::new(),
            vars: Vec::new(),
            next_item_id: self.next_item_id,
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

fn resolve<'a>(
    globals: &HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    opens: &[HashMap<Rc<str>, Vec<Open>>],
    parent_namespace: &Rc<str>,
    local: impl Fn(&'a str) -> Option<Res>,
    path: &'a ast::Path,
) -> Result<Res, Error> {
    let name = path.name.name.as_ref();
    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    if namespace.is_empty() {
        if let Some(res) = local(name) {
            // Locals shadow everything.
            return Ok(res);
        } else if let Some(&res) = globals.get(parent_namespace).and_then(|env| env.get(name)) {
            // Items in the parent namespace shadow opens.
            return Ok(res);
        }
    }

    // Explicit opens shadow prelude and unopened globals.
    let open_candidates = opens
        .iter()
        .rev()
        .find_map(|scope| {
            let open_namespaces = scope.get(namespace)?;
            let candidates = resolve_explicit_opens(globals, open_namespaces, name);
            if candidates.is_empty() {
                None
            } else {
                Some(candidates)
            }
        })
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
        let mut opens: Vec<_> = open_candidates.into_values().collect();
        opens.sort_unstable_by_key(|open| open.span);
        Err(Error::Ambiguous {
            name: name.to_string(),
            first_open: opens[0].namespace.to_string(),
            second_open: opens[1].namespace.to_string(),
            name_span: path.span,
            first_open_span: opens[0].span,
            second_open_span: opens[1].span,
        })
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
        if let Some(&res) = globals.get(namespace).and_then(|env| env.get(name)) {
            candidates.insert(res);
        }
    }
    candidates
}

fn resolve_explicit_opens<'a>(
    globals: &HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    opens: impl IntoIterator<Item = &'a Open>,
    name: &str,
) -> HashMap<Res, &'a Open> {
    let mut candidates = HashMap::new();
    for open in opens {
        if let Some(&res) = globals.get(&open.namespace).and_then(|env| env.get(name)) {
            candidates.insert(res, open);
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
