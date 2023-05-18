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
    global,
    hir::{self, ItemId, LocalItemId, PackageId, PrimTy},
};
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    vec,
};
use thiserror::Error;

const PRELUDE: &[&str] = &[
    "Microsoft.Quantum.Canon",
    "Microsoft.Quantum.Core",
    "Microsoft.Quantum.Intrinsic",
];

pub(super) struct Table {
    pub(super) names: IndexMap<ast::NodeId, Res>,
    pub(super) next_id: LocalItemId,
}

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

struct Scope {
    kind: ScopeKind,
    opens: HashMap<Rc<str>, Vec<Open>>,
    tys: HashMap<Rc<str>, ItemId>,
    terms: HashMap<Rc<str>, ItemId>,
    vars: HashMap<Rc<str>, ast::NodeId>,
}

impl Scope {
    fn new(kind: ScopeKind) -> Self {
        Self {
            kind,
            opens: HashMap::new(),
            tys: HashMap::new(),
            terms: HashMap::new(),
            vars: HashMap::new(),
        }
    }

    fn item(&self, kind: NameKind, name: &str) -> Option<&ItemId> {
        let items = match kind {
            NameKind::Ty => &self.tys,
            NameKind::Term => &self.terms,
        };
        items.get(name)
    }
}

struct GlobalScope {
    tys: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
    terms: HashMap<Rc<str>, HashMap<Rc<str>, Res>>,
}

impl GlobalScope {
    fn get(&self, kind: NameKind, namespace: &str, name: &str) -> Option<&Res> {
        let namespaces = match kind {
            NameKind::Ty => &self.tys,
            NameKind::Term => &self.terms,
        };
        namespaces.get(namespace).and_then(|items| items.get(name))
    }
}

#[derive(Eq, PartialEq)]
enum ScopeKind {
    Namespace(Rc<str>),
    Callable,
    Block,
}

#[derive(Clone, Copy)]
enum NameKind {
    Ty,
    Term,
}

struct Open {
    namespace: Rc<str>,
    span: Span,
}

pub(super) struct Resolver {
    resolutions: Table,
    globals: GlobalScope,
    scopes: Vec<Scope>,
    errors: Vec<Error>,
}

impl Resolver {
    pub(super) fn new(globals: GlobalTable) -> Self {
        Self {
            resolutions: globals.resolutions,
            globals: globals.scope,
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub(super) fn with_persistent_local_scope(globals: GlobalTable) -> Self {
        Self {
            resolutions: globals.resolutions,
            globals: globals.scope,
            scopes: vec![Scope::new(ScopeKind::Block)],
            errors: Vec::new(),
        }
    }

    pub(super) fn resolutions(&self) -> &Table {
        &self.resolutions
    }

    pub(super) fn resolutions_mut(&mut self) -> &mut Table {
        &mut self.resolutions
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn into_resolutions(self) -> (Table, Vec<Error>) {
        (self.resolutions, self.errors)
    }

    fn resolve(&mut self, kind: NameKind, path: &ast::Path) {
        match resolve(kind, &self.globals, &self.scopes, path) {
            Ok(id) => self.resolutions.names.insert(path.id, id),
            Err(err) => self.errors.push(err),
        }
    }

    fn with_scope(&mut self, kind: ScopeKind, f: impl FnOnce(&mut Self)) {
        self.scopes.push(Scope::new(kind));
        f(self);
        self.scopes
            .pop()
            .expect("pushed scope should be the last element on the stack");
    }

    fn with_pat(&mut self, kind: ScopeKind, pat: &ast::Pat, f: impl FnOnce(&mut Self)) {
        self.with_scope(kind, |resolver| {
            resolver.bind_pat(pat);
            f(resolver);
        });
    }

    fn bind_pat(&mut self, pat: &ast::Pat) {
        match &pat.kind {
            ast::PatKind::Bind(name, _) => {
                let scope = self.scopes.last_mut().expect("binding should have scope");
                self.resolutions.names.insert(name.id, Res::Local(name.id));
                scope.vars.insert(Rc::clone(&name.name), name.id);
            }
            ast::PatKind::Discard(_) | ast::PatKind::Elided => {}
            ast::PatKind::Paren(pat) => self.bind_pat(pat),
            ast::PatKind::Tuple(pats) => pats.iter().for_each(|p| self.bind_pat(p)),
        }
    }

    fn bind_open(&mut self, name: &ast::Ident, alias: Option<&ast::Ident>) {
        let alias = alias.as_ref().map_or("".into(), |a| Rc::clone(&a.name));
        let scope = self.scopes.last_mut().expect("open item should have scope");
        scope.opens.entry(alias).or_default().push(Open {
            namespace: Rc::clone(&name.name),
            span: name.span,
        });
    }

    fn bind_local_item_if_new(&mut self, item: &ast::Item) {
        match &item.kind {
            ast::ItemKind::Open(name, alias) => self.bind_open(name, alias.as_ref()),
            ast::ItemKind::Callable(decl) if !self.resolutions.names.contains_key(decl.name.id) => {
                let id = next_item_id(&mut self.resolutions.next_id);
                self.resolutions.names.insert(decl.name.id, Res::Item(id));
                let scope = self.scopes.last_mut().expect("binding should have scope");
                scope.terms.insert(Rc::clone(&decl.name.name), id);
            }
            ast::ItemKind::Ty(name, _) if !self.resolutions.names.contains_key(name.id) => {
                let id = next_item_id(&mut self.resolutions.next_id);
                self.resolutions.names.insert(name.id, Res::Item(id));
                let scope = self.scopes.last_mut().expect("binding should have scope");
                scope.tys.insert(Rc::clone(&name.name), id);
                scope.terms.insert(Rc::clone(&name.name), id);
            }
            ast::ItemKind::Callable(..) | ast::ItemKind::Ty(..) | ast::ItemKind::Err => {}
        }
    }
}

impl AstVisitor<'_> for Resolver {
    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        if !self.resolutions.names.contains_key(namespace.name.id) {
            self.resolutions.names.insert(
                namespace.name.id,
                Res::Item(next_item_id(&mut self.resolutions.next_id)),
            );

            for item in &namespace.items {
                bind_global_item(
                    &mut self.resolutions.names,
                    &mut self.globals,
                    &namespace.name.name,
                    || next_item_id(&mut self.resolutions.next_id),
                    item,
                );
            }
        }

        let kind = ScopeKind::Namespace(Rc::clone(&namespace.name.name));
        self.with_scope(kind, |resolver| {
            for item in &namespace.items {
                if let ast::ItemKind::Open(name, alias) = &item.kind {
                    resolver.bind_open(name, alias.as_ref());
                }
            }

            ast_visit::walk_namespace(resolver, namespace);
        });
    }

    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        self.with_pat(ScopeKind::Callable, &decl.input, |resolver| {
            ast_visit::walk_callable_decl(resolver, decl);
        });
    }

    fn visit_spec_decl(&mut self, decl: &ast::SpecDecl) {
        if let ast::SpecBody::Impl(input, block) = &decl.body {
            self.with_pat(ScopeKind::Block, input, |resolver| {
                resolver.visit_block(block);
            });
        } else {
            ast_visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_ty(&mut self, ty: &ast::Ty) {
        if let ast::TyKind::Path(path) = &ty.kind {
            self.resolve(NameKind::Ty, path);
        } else {
            ast_visit::walk_ty(self, ty);
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        self.with_scope(ScopeKind::Block, |resolver| {
            for stmt in &block.stmts {
                if let ast::StmtKind::Item(item) = &stmt.kind {
                    resolver.bind_local_item_if_new(item);
                }
            }

            ast_visit::walk_block(resolver, block);
        });
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match &stmt.kind {
            ast::StmtKind::Item(item) => {
                self.bind_local_item_if_new(item);
                self.visit_item(item);
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
                self.with_pat(ScopeKind::Block, pat, |resolver| {
                    resolver.visit_block(block);
                });
            }
            ast::ExprKind::Repeat(repeat, cond, fixup) => {
                self.with_scope(ScopeKind::Block, |resolver| {
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
                self.with_pat(ScopeKind::Block, input, |resolver| {
                    resolver.visit_expr(output);
                });
            }
            ast::ExprKind::Path(path) => self.resolve(NameKind::Term, path),
            ast::ExprKind::TernOp(ast::TernOp::Update, container, index, replace) => {
                self.visit_expr(container);
                if !is_field_update(&self.globals, &self.scopes, index) {
                    self.visit_expr(index);
                }
                self.visit_expr(replace);
            }
            _ => ast_visit::walk_expr(self, expr),
        }
    }
}

pub(super) struct GlobalTable {
    resolutions: Table,
    scope: GlobalScope,
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

        let terms = HashMap::new();
        Self {
            resolutions: Table {
                names: IndexMap::new(),
                next_id: LocalItemId::default(),
            },
            scope: GlobalScope { tys, terms },
        }
    }

    pub(super) fn add_local_package(&mut self, package: &ast::Package) {
        for namespace in &package.namespaces {
            self.resolutions.names.insert(
                namespace.name.id,
                Res::Item(next_item_id(&mut self.resolutions.next_id)),
            );

            for item in &namespace.items {
                bind_global_item(
                    &mut self.resolutions.names,
                    &mut self.scope,
                    &namespace.name.name,
                    || next_item_id(&mut self.resolutions.next_id),
                    item,
                );
            }
        }
    }

    pub(super) fn add_external_package(&mut self, id: PackageId, package: &hir::Package) {
        for global in global::iter_package(Some(id), package)
            .filter(|global| global.visibility == hir::Visibility::Public)
        {
            match global.kind {
                global::Kind::Ty(ty) => {
                    self.scope
                        .tys
                        .entry(global.namespace)
                        .or_default()
                        .insert(global.name, Res::Item(ty.id));
                }
                global::Kind::Term(term) => {
                    self.scope
                        .terms
                        .entry(global.namespace)
                        .or_default()
                        .insert(global.name, Res::Item(term.id));
                }
            }
        }
    }
}

/// Tries to extract a field name from an expression in cases where it is syntactically ambiguous
/// whether the expression is a field name or a variable name. This applies to the index operand in
/// a ternary update operator.
pub(super) fn extract_field_name<'a>(
    names: &IndexMap<ast::NodeId, Res>,
    expr: &'a ast::Expr,
) -> Option<&'a Rc<str>> {
    // Follow the same reasoning as `is_field_update`.
    match &expr.kind {
        ast::ExprKind::Path(path)
            if path.namespace.is_none() && !matches!(names.get(path.id), Some(Res::Local(_))) =>
        {
            Some(&path.name.name)
        }
        _ => None,
    }
}

fn is_field_update(globals: &GlobalScope, scopes: &[Scope], index: &ast::Expr) -> bool {
    // Disambiguate the update operator by looking at the index expression. If it's an
    // unqualified path that doesn't resolve to a local, assume that it's meant to be a field name.
    match &index.kind {
        ast::ExprKind::Path(path) if path.namespace.is_none() => !matches!(
            resolve(NameKind::Term, globals, scopes, path),
            Ok(Res::Local(_))
        ),
        _ => false,
    }
}

fn bind_global_item(
    names: &mut IndexMap<ast::NodeId, Res>,
    scope: &mut GlobalScope,
    namespace: &Rc<str>,
    next_id: impl FnOnce() -> ItemId,
    item: &ast::Item,
) {
    match &item.kind {
        ast::ItemKind::Callable(decl) => {
            let res = Res::Item(next_id());
            names.insert(decl.name.id, res);
            scope
                .terms
                .entry(Rc::clone(namespace))
                .or_default()
                .insert(Rc::clone(&decl.name.name), res);
        }
        ast::ItemKind::Ty(name, _) => {
            let res = Res::Item(next_id());
            names.insert(name.id, res);
            scope
                .tys
                .entry(Rc::clone(namespace))
                .or_default()
                .insert(Rc::clone(&name.name), res);
            scope
                .terms
                .entry(Rc::clone(namespace))
                .or_default()
                .insert(Rc::clone(&name.name), res);
        }
        ast::ItemKind::Err | ast::ItemKind::Open(..) => {}
    }
}

fn next_item_id(local_id: &mut LocalItemId) -> ItemId {
    let item_id = ItemId {
        package: None,
        item: *local_id,
    };
    *local_id = local_id.successor();
    item_id
}

fn resolve(
    kind: NameKind,
    globals: &GlobalScope,
    locals: &[Scope],
    path: &ast::Path,
) -> Result<Res, Error> {
    let name = path.name.name.as_ref();
    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    let mut candidates = HashMap::new();
    let mut vars = true;

    for scope in locals.iter().rev() {
        if namespace.is_empty() {
            if let Some(res) = resolve_scope_locals(kind, globals, scope, vars, name) {
                // Local declarations shadow everything.
                return Ok(res);
            }
        }

        if let Some(namespaces) = scope.opens.get(namespace) {
            candidates = resolve_explicit_opens(kind, globals, namespaces, name);
            if !candidates.is_empty() {
                // Explicit opens shadow prelude and unopened globals.
                break;
            }
        }

        if scope.kind == ScopeKind::Callable {
            // Since local callables are not closures, hide local variables in parent scopes.
            vars = false;
        }
    }

    if candidates.is_empty() && namespace.is_empty() {
        // Prelude shadows unopened globals.
        let candidates = resolve_implicit_opens(kind, globals, PRELUDE, name);
        assert!(candidates.len() <= 1, "ambiguity in prelude resolution");
        if let Some(res) = single(candidates) {
            return Ok(res);
        }
    }

    if candidates.is_empty() {
        if let Some(&res) = globals.get(kind, namespace, name) {
            // An unopened global is the last resort.
            return Ok(res);
        }
    }

    if candidates.len() > 1 {
        let mut opens: Vec<_> = candidates.into_values().collect();
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
        single(candidates.into_keys()).ok_or_else(|| Error::NotFound(name.to_string(), path.span))
    }
}

fn resolve_scope_locals(
    kind: NameKind,
    globals: &GlobalScope,
    scope: &Scope,
    vars: bool,
    name: &str,
) -> Option<Res> {
    if vars {
        if let Some(&id) = scope.vars.get(name) {
            return Some(Res::Local(id));
        }
    }

    if let Some(&id) = scope.item(kind, name) {
        return Some(Res::Item(id));
    }

    if let ScopeKind::Namespace(namespace) = &scope.kind {
        if let Some(&res) = globals.get(kind, namespace, name) {
            return Some(res);
        }
    }

    None
}

fn resolve_implicit_opens(
    kind: NameKind,
    globals: &GlobalScope,
    namespaces: impl IntoIterator<Item = impl AsRef<str>>,
    name: &str,
) -> HashSet<Res> {
    let mut candidates = HashSet::new();
    for namespace in namespaces {
        let namespace = namespace.as_ref();
        if let Some(&res) = globals.get(kind, namespace, name) {
            candidates.insert(res);
        }
    }
    candidates
}

fn resolve_explicit_opens<'a>(
    kind: NameKind,
    globals: &GlobalScope,
    opens: impl IntoIterator<Item = &'a Open>,
    name: &str,
) -> HashMap<Res, &'a Open> {
    let mut candidates = HashMap::new();
    for open in opens {
        if let Some(&res) = globals.get(kind, &open.namespace, name) {
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
