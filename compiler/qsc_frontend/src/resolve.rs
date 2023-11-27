// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{self, CallableDecl, Ident, NodeId, TopLevelNode},
    visit::{self as ast_visit, walk_attr, Visitor as AstVisitor},
};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    assigner::Assigner,
    global,
    hir::{self, ItemId, ItemStatus, LocalItemId, PackageId},
    ty::{ParamId, Prim},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{collections::hash_map::Entry, rc::Rc, str::FromStr, vec};
use thiserror::Error;

use crate::compile::preprocess::TrackedName;

const PRELUDE: &[&str] = &[
    "Microsoft.Quantum.Canon",
    "Microsoft.Quantum.Core",
    "Microsoft.Quantum.Intrinsic",
];

// All AST Path nodes get mapped
// All AST Ident nodes get mapped, except those under AST Path nodes
pub(super) type Names = IndexMap<NodeId, Res>;

/// A resolution. This connects a usage of a name with the declaration of that name by uniquely
/// identifying the node that declared it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Res {
    /// A global item.
    Item(ItemId, ItemStatus),
    /// A local variable.
    Local(NodeId),
    /// A type/functor parameter in the generics section of the parent callable decl.
    Param(ParamId),
    /// A primitive type.
    PrimTy(Prim),
    /// The unit type.
    UnitTy,
}

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("`{name}` could refer to the item in `{first_open}` or `{second_open}`")]
    #[diagnostic(code("Qsc.Resolve.Ambiguous"))]
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

    #[error("`{name}` could refer to the item in `{candidate_a}` or an item in `{candidate_b}`")]
    #[diagnostic(help("both namespaces are implicitly opened by the prelude"))]
    #[diagnostic(code("Qsc.Resolve.AmbiguousPrelude"))]
    AmbiguousPrelude {
        name: String,
        candidate_a: String,
        candidate_b: String,
        #[label("ambiguous name")]
        span: Span,
    },

    #[error("duplicate declaration of `{0}` in namespace `{1}`")]
    #[diagnostic(code("Qsc.Resolve.Duplicate"))]
    Duplicate(String, String, #[label] Span),

    #[error("duplicate name `{0}` in pattern")]
    #[diagnostic(help("a name cannot shadow another name in the same pattern"))]
    #[diagnostic(code("Qsc.Resolve.DuplicateBinding"))]
    DuplicateBinding(String, #[label] Span),

    #[error("`{0}` not found")]
    #[diagnostic(code("Qsc.Resolve.NotFound"))]
    NotFound(String, #[label] Span),

    #[error("`{0}` not found")]
    #[diagnostic(help(
        "found a matching item `{1}` that is not available for the current compilation configuration"
    ))]
    #[diagnostic(code("Qsc.Resolve.NotFound"))]
    NotAvailable(String, String, #[label] Span),

    #[error("use of unimplemented item `{0}`")]
    #[diagnostic(help("this item is not implemented and cannot be used"))]
    #[diagnostic(code("Qsc.Resolve.Unimplemented"))]
    Unimplemented(String, #[label] Span),
}

struct Scope {
    kind: ScopeKind,
    opens: FxHashMap<Rc<str>, Vec<Open>>,
    tys: FxHashMap<Rc<str>, ItemId>,
    terms: FxHashMap<Rc<str>, ItemId>,
    vars: FxHashMap<Rc<str>, NodeId>,
    ty_vars: FxHashMap<Rc<str>, ParamId>,
}

impl Scope {
    fn new(kind: ScopeKind) -> Self {
        Self {
            kind,
            opens: FxHashMap::default(),
            tys: FxHashMap::default(),
            terms: FxHashMap::default(),
            vars: FxHashMap::default(),
            ty_vars: FxHashMap::default(),
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
    tys: FxHashMap<Rc<str>, FxHashMap<Rc<str>, Res>>,
    terms: FxHashMap<Rc<str>, FxHashMap<Rc<str>, Res>>,
    namespaces: FxHashSet<Rc<str>>,
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
    names: Names,
    dropped_names: Vec<TrackedName>,
    curr_params: Option<FxHashSet<Rc<str>>>,
    globals: GlobalScope,
    scopes: Vec<Scope>,
    errors: Vec<Error>,
}

impl Resolver {
    pub(super) fn new(globals: GlobalTable, dropped_names: Vec<TrackedName>) -> Self {
        Self {
            names: globals.names,
            dropped_names,
            curr_params: None,
            globals: globals.scope,
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub(super) fn with_persistent_local_scope(
        globals: GlobalTable,
        dropped_names: Vec<TrackedName>,
    ) -> Self {
        Self {
            names: globals.names,
            dropped_names,
            curr_params: None,
            globals: globals.scope,
            scopes: vec![Scope::new(ScopeKind::Block)],
            errors: Vec::new(),
        }
    }

    pub(super) fn names(&self) -> &Names {
        &self.names
    }

    pub(super) fn drain_errors(&mut self) -> vec::Drain<Error> {
        self.errors.drain(..)
    }

    pub(super) fn with<'a>(&'a mut self, assigner: &'a mut Assigner) -> With<'a> {
        With {
            resolver: self,
            assigner,
        }
    }

    pub(super) fn into_names(self) -> (Names, Vec<Error>) {
        (self.names, self.errors)
    }

    pub(super) fn extend_dropped_names(&mut self, dropped_names: Vec<TrackedName>) {
        self.dropped_names.extend(dropped_names);
    }

    pub(super) fn bind_fragments(&mut self, ast: &ast::Package, assigner: &mut Assigner) {
        for node in &mut ast.nodes.iter() {
            match node {
                ast::TopLevelNode::Namespace(namespace) => {
                    bind_global_items(
                        &mut self.names,
                        &mut self.globals,
                        namespace,
                        assigner,
                        &mut self.errors,
                    );
                }
                ast::TopLevelNode::Stmt(stmt) => {
                    if let ast::StmtKind::Item(item) = stmt.kind.as_ref() {
                        self.bind_local_item(assigner, item);
                    }
                }
            }
        }
    }

    fn check_item_status(&mut self, res: Res, name: String, span: Span) {
        if let Res::Item(_, ItemStatus::Unimplemented) = res {
            self.errors.push(Error::Unimplemented(name, span));
        }
    }

    fn resolve_ident(&mut self, kind: NameKind, name: &Ident) {
        let namespace = None;
        match resolve(kind, &self.globals, &self.scopes, name, &namespace) {
            Ok(res) => {
                self.check_item_status(res, name.name.to_string(), name.span);
                self.names.insert(name.id, res);
            }
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_path(&mut self, kind: NameKind, path: &ast::Path) {
        let name = &path.name;
        let namespace = &path.namespace;
        match resolve(kind, &self.globals, &self.scopes, name, namespace) {
            Ok(res) => {
                self.check_item_status(res, path.name.name.to_string(), path.span);
                self.names.insert(path.id, res);
            }
            Err(err) => {
                if let Error::NotFound(name, span) = err {
                    if let Some(dropped_name) =
                        self.dropped_names.iter().find(|n| n.name.as_ref() == name)
                    {
                        self.errors.push(Error::NotAvailable(
                            name,
                            format!("{}.{}", dropped_name.namespace, dropped_name.name),
                            span,
                        ));
                    } else {
                        self.errors.push(Error::NotFound(name, span));
                    }
                } else {
                    self.errors.push(err);
                }
            }
        }
    }

    fn bind_pat(&mut self, pat: &ast::Pat) {
        let mut bindings = FxHashSet::default();
        self.bind_pat_recursive(pat, &mut bindings);
    }

    fn bind_pat_recursive(&mut self, pat: &ast::Pat, bindings: &mut FxHashSet<Rc<str>>) {
        match &*pat.kind {
            ast::PatKind::Bind(name, _) => {
                if !bindings.insert(Rc::clone(&name.name)) {
                    self.errors
                        .push(Error::DuplicateBinding(name.name.to_string(), name.span));
                }
                let scope = self.scopes.last_mut().expect("binding should have scope");
                self.names.insert(name.id, Res::Local(name.id));
                scope.vars.insert(Rc::clone(&name.name), name.id);
            }
            ast::PatKind::Discard(_) | ast::PatKind::Elided | ast::PatKind::Err => {}
            ast::PatKind::Paren(pat) => self.bind_pat_recursive(pat, bindings),
            ast::PatKind::Tuple(pats) => pats
                .iter()
                .for_each(|p| self.bind_pat_recursive(p, bindings)),
        }
    }

    fn bind_open(&mut self, name: &ast::Ident, alias: &Option<Box<ast::Ident>>) {
        let alias = alias.as_ref().map_or("".into(), |a| Rc::clone(&a.name));
        let scope = self.scopes.last_mut().expect("open item should have scope");
        if self.globals.namespaces.contains(&name.name) {
            scope.opens.entry(alias).or_default().push(Open {
                namespace: Rc::clone(&name.name),
                span: name.span,
            });
        } else {
            self.errors
                .push(Error::NotFound(name.name.to_string(), name.span));
        }
    }

    pub(super) fn bind_local_item(&mut self, assigner: &mut Assigner, item: &ast::Item) {
        match &*item.kind {
            ast::ItemKind::Open(name, alias) => self.bind_open(name, alias),
            ast::ItemKind::Callable(decl) => {
                let id = intrapackage(assigner.next_item());
                self.names.insert(
                    decl.name.id,
                    Res::Item(
                        id,
                        ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(&item.attrs)),
                    ),
                );
                let scope = self.scopes.last_mut().expect("binding should have scope");
                scope.terms.insert(Rc::clone(&decl.name.name), id);
            }
            ast::ItemKind::Ty(name, _) => {
                let id = intrapackage(assigner.next_item());
                self.names.insert(
                    name.id,
                    Res::Item(
                        id,
                        ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(&item.attrs)),
                    ),
                );
                let scope = self.scopes.last_mut().expect("binding should have scope");
                scope.tys.insert(Rc::clone(&name.name), id);
                scope.terms.insert(Rc::clone(&name.name), id);
            }
            ast::ItemKind::Err => {}
        }
    }

    fn bind_type_parameters(&mut self, decl: &CallableDecl) {
        decl.generics.iter().enumerate().for_each(|(ix, ident)| {
            let scope = self
                .scopes
                .last_mut()
                .expect("type parameters should have scope");
            scope.ty_vars.insert(Rc::clone(&ident.name), ix.into());
            self.names.insert(ident.id, Res::Param(ix.into()));
        });
    }
}

pub(super) struct With<'a> {
    resolver: &'a mut Resolver,
    assigner: &'a mut Assigner,
}

impl With<'_> {
    fn with_scope(&mut self, kind: ScopeKind, f: impl FnOnce(&mut Self)) {
        self.resolver.scopes.push(Scope::new(kind));
        f(self);
        self.resolver
            .scopes
            .pop()
            .expect("pushed scope should be the last element on the stack");
    }

    fn with_pat(&mut self, kind: ScopeKind, pat: &ast::Pat, f: impl FnOnce(&mut Self)) {
        self.with_scope(kind, |visitor| {
            visitor.resolver.bind_pat(pat);
            f(visitor);
        });
    }

    fn with_spec_pat(&mut self, kind: ScopeKind, pat: &ast::Pat, f: impl FnOnce(&mut Self)) {
        let mut bindings = self
            .resolver
            .curr_params
            .as_ref()
            .map_or_else(FxHashSet::default, std::clone::Clone::clone);
        self.with_scope(kind, |visitor| {
            visitor.resolver.bind_pat_recursive(pat, &mut bindings);
            f(visitor);
        });
    }
}

impl AstVisitor<'_> for With<'_> {
    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        let kind = ScopeKind::Namespace(Rc::clone(&namespace.name.name));
        self.with_scope(kind, |visitor| {
            for item in &*namespace.items {
                if let ast::ItemKind::Open(name, alias) = &*item.kind {
                    visitor.resolver.bind_open(name, alias);
                }
            }

            ast_visit::walk_namespace(visitor, namespace);
        });
    }

    fn visit_attr(&mut self, attr: &ast::Attr) {
        // The Config attribute arguments do not go through name resolution.
        if hir::Attr::from_str(attr.name.name.as_ref()) != Ok(hir::Attr::Config) {
            walk_attr(self, attr);
        }
    }

    fn visit_callable_decl(&mut self, decl: &ast::CallableDecl) {
        fn collect_param_names(pat: &ast::Pat, names: &mut FxHashSet<Rc<str>>) {
            match &*pat.kind {
                ast::PatKind::Bind(name, _) => {
                    names.insert(Rc::clone(&name.name));
                }
                ast::PatKind::Discard(_) | ast::PatKind::Elided | ast::PatKind::Err => {}
                ast::PatKind::Paren(pat) => collect_param_names(pat, names),
                ast::PatKind::Tuple(pats) => {
                    pats.iter().for_each(|p| collect_param_names(p, names));
                }
            }
        }
        let mut param_names = FxHashSet::default();
        collect_param_names(&decl.input, &mut param_names);
        let prev_param_names = self.resolver.curr_params.replace(param_names);
        self.with_scope(ScopeKind::Callable, |visitor| {
            visitor.resolver.bind_type_parameters(decl);
            visitor.resolver.bind_pat(&decl.input);
            ast_visit::walk_callable_decl(visitor, decl);
        });
        self.resolver.curr_params = prev_param_names;
    }

    fn visit_spec_decl(&mut self, decl: &ast::SpecDecl) {
        if let ast::SpecBody::Impl(input, block) = &decl.body {
            self.with_spec_pat(ScopeKind::Block, input, |visitor| {
                visitor.visit_block(block);
            });
        } else {
            ast_visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_ty(&mut self, ty: &ast::Ty) {
        match &*ty.kind {
            ast::TyKind::Path(path) => {
                self.resolver.resolve_path(NameKind::Ty, path);
            }
            ast::TyKind::Param(ident) => {
                self.resolver.resolve_ident(NameKind::Ty, ident);
            }
            _ => ast_visit::walk_ty(self, ty),
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        self.with_scope(ScopeKind::Block, |visitor| {
            for stmt in &*block.stmts {
                if let ast::StmtKind::Item(item) = &*stmt.kind {
                    visitor.resolver.bind_local_item(visitor.assigner, item);
                }
            }

            ast_visit::walk_block(visitor, block);
        });
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match &*stmt.kind {
            ast::StmtKind::Item(item) => self.visit_item(item),
            ast::StmtKind::Local(_, pat, _) => {
                ast_visit::walk_stmt(self, stmt);
                self.resolver.bind_pat(pat);
            }
            ast::StmtKind::Qubit(_, pat, init, block) => {
                ast_visit::walk_qubit_init(self, init);
                self.resolver.bind_pat(pat);
                if let Some(block) = block {
                    self.visit_block(block);
                }
            }
            ast::StmtKind::Empty
            | ast::StmtKind::Expr(_)
            | ast::StmtKind::Semi(_)
            | ast::StmtKind::Err => {
                ast_visit::walk_stmt(self, stmt);
            }
        }
    }

    fn visit_expr(&mut self, expr: &ast::Expr) {
        match &*expr.kind {
            ast::ExprKind::For(pat, iter, block) => {
                self.visit_expr(iter);
                self.with_pat(ScopeKind::Block, pat, |visitor| visitor.visit_block(block));
            }
            ast::ExprKind::Lambda(_, input, output) => {
                self.with_pat(ScopeKind::Block, input, |visitor| {
                    visitor.visit_expr(output);
                });
            }
            ast::ExprKind::Path(path) => self.resolver.resolve_path(NameKind::Term, path),
            ast::ExprKind::TernOp(ast::TernOp::Update, container, index, replace)
            | ast::ExprKind::AssignUpdate(container, index, replace) => {
                self.visit_expr(container);
                if !is_field_update(&self.resolver.globals, &self.resolver.scopes, index) {
                    self.visit_expr(index);
                }
                self.visit_expr(replace);
            }
            _ => ast_visit::walk_expr(self, expr),
        }
    }
}

pub(super) struct GlobalTable {
    names: Names,
    scope: GlobalScope,
}

impl GlobalTable {
    pub(super) fn new() -> Self {
        let builtins: [(Rc<str>, Res); 10] = [
            ("BigInt".into(), Res::PrimTy(Prim::BigInt)),
            ("Bool".into(), Res::PrimTy(Prim::Bool)),
            ("Double".into(), Res::PrimTy(Prim::Double)),
            ("Int".into(), Res::PrimTy(Prim::Int)),
            ("Pauli".into(), Res::PrimTy(Prim::Pauli)),
            ("Qubit".into(), Res::PrimTy(Prim::Qubit)),
            ("Range".into(), Res::PrimTy(Prim::Range)),
            ("Result".into(), Res::PrimTy(Prim::Result)),
            ("String".into(), Res::PrimTy(Prim::String)),
            ("Unit".into(), Res::UnitTy),
        ];
        let mut core: FxHashMap<Rc<str>, Res> = FxHashMap::default();
        for (name, res) in builtins {
            core.insert(name, res);
        }
        let mut tys: FxHashMap<Rc<str>, FxHashMap<Rc<str>, Res>> = FxHashMap::default();
        tys.insert("Microsoft.Quantum.Core".into(), core);

        Self {
            names: IndexMap::new(),
            scope: GlobalScope {
                tys,
                terms: FxHashMap::default(),
                namespaces: FxHashSet::default(),
            },
        }
    }

    pub(super) fn add_local_package(
        &mut self,
        assigner: &mut Assigner,
        package: &ast::Package,
    ) -> Vec<Error> {
        let mut errors = Vec::new();
        for node in &*package.nodes {
            match node {
                TopLevelNode::Namespace(namespace) => {
                    bind_global_items(
                        &mut self.names,
                        &mut self.scope,
                        namespace,
                        assigner,
                        &mut errors,
                    );
                }
                TopLevelNode::Stmt(_) => {
                    unimplemented!("did not expect top-level statements in the ast")
                }
            }
        }
        errors
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
                        .insert(global.name, Res::Item(ty.id, global.status));
                }
                global::Kind::Term(term) => {
                    self.scope
                        .terms
                        .entry(global.namespace)
                        .or_default()
                        .insert(global.name, Res::Item(term.id, global.status));
                }
                global::Kind::Namespace => {
                    self.scope.namespaces.insert(global.name);
                }
            }
        }
    }
}

fn bind_global_items(
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
    namespace: &ast::Namespace,
    assigner: &mut Assigner,
    errors: &mut Vec<Error>,
) {
    names.insert(
        namespace.name.id,
        Res::Item(intrapackage(assigner.next_item()), ItemStatus::Available),
    );
    scope.namespaces.insert(Rc::clone(&namespace.name.name));

    for item in &*namespace.items {
        match bind_global_item(
            names,
            scope,
            &namespace.name.name,
            || intrapackage(assigner.next_item()),
            item,
        ) {
            Ok(()) => {}
            Err(error) => errors.push(error),
        }
    }
}

/// Tries to extract a field name from an expression in cases where it is syntactically ambiguous
/// whether the expression is a field name or a variable name. This applies to the index operand in
/// a ternary update operator.
pub(super) fn extract_field_name<'a>(names: &Names, expr: &'a ast::Expr) -> Option<&'a Rc<str>> {
    // Follow the same reasoning as `is_field_update`.
    match &*expr.kind {
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
    match &*index.kind {
        ast::ExprKind::Path(path) if path.namespace.is_none() => !matches!(
            {
                let name = &path.name;
                let namespace = &path.namespace;
                resolve(NameKind::Term, globals, scopes, name, namespace)
            },
            Ok(Res::Local(_))
        ),
        _ => false,
    }
}

fn ast_attrs_as_hir_attrs(attrs: &[Box<ast::Attr>]) -> Vec<hir::Attr> {
    attrs
        .iter()
        .filter_map(|attr| hir::Attr::from_str(attr.name.name.as_ref()).ok())
        .collect()
}

fn bind_global_item(
    names: &mut Names,
    scope: &mut GlobalScope,
    namespace: &Rc<str>,
    next_id: impl FnOnce() -> ItemId,
    item: &ast::Item,
) -> Result<(), Error> {
    match &*item.kind {
        ast::ItemKind::Callable(decl) => {
            let item_id = next_id();
            let status = ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(item.attrs.as_ref()));
            let res = Res::Item(item_id, status);
            names.insert(decl.name.id, res);
            match scope
                .terms
                .entry(Rc::clone(namespace))
                .or_default()
                .entry(Rc::clone(&decl.name.name))
            {
                Entry::Occupied(_) => Err(Error::Duplicate(
                    decl.name.name.to_string(),
                    namespace.to_string(),
                    decl.name.span,
                )),
                Entry::Vacant(entry) => {
                    entry.insert(res);
                    Ok(())
                }
            }
        }
        ast::ItemKind::Ty(name, _) => {
            let item_id = next_id();
            let status = ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(item.attrs.as_ref()));
            let res = Res::Item(item_id, status);
            names.insert(name.id, res);
            match (
                scope
                    .terms
                    .entry(Rc::clone(namespace))
                    .or_default()
                    .entry(Rc::clone(&name.name)),
                scope
                    .tys
                    .entry(Rc::clone(namespace))
                    .or_default()
                    .entry(Rc::clone(&name.name)),
            ) {
                (Entry::Occupied(_), _) | (_, Entry::Occupied(_)) => Err(Error::Duplicate(
                    name.name.to_string(),
                    namespace.to_string(),
                    name.span,
                )),
                (Entry::Vacant(term_entry), Entry::Vacant(ty_entry)) => {
                    term_entry.insert(res);
                    ty_entry.insert(res);
                    Ok(())
                }
            }
        }
        ast::ItemKind::Err | ast::ItemKind::Open(..) => Ok(()),
    }
}

fn resolve(
    kind: NameKind,
    globals: &GlobalScope,
    locals: &[Scope],
    name: &Ident,
    namespace: &Option<Box<Ident>>,
) -> Result<Res, Error> {
    let mut candidates = FxHashMap::default();
    let mut vars = true;
    let name_str = &(*name.name);
    let namespace = namespace.as_ref().map_or("", |i| &i.name);
    for scope in locals.iter().rev() {
        if namespace.is_empty() {
            if let Some(res) = resolve_scope_locals(kind, globals, scope, vars, name_str) {
                // Local declarations shadow everything.
                return Ok(res);
            }
        }

        if let Some(namespaces) = scope.opens.get(namespace) {
            candidates = resolve_explicit_opens(kind, globals, namespaces, name_str);
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
        let candidates = resolve_implicit_opens(kind, globals, PRELUDE, name_str);
        if candidates.len() > 1 {
            let mut candidates: Vec<_> = candidates.into_iter().collect();
            candidates.sort_by_key(|x| x.1);
            let mut candidates = candidates
                .into_iter()
                .map(|candidate| candidate.1.to_string());
            let candidate_a = candidates
                .next()
                .expect("infallible as per length check above");
            let candidate_b = candidates
                .next()
                .expect("infallible as per length check above");
            return Err(Error::AmbiguousPrelude {
                span: name.span,
                name: name.name.to_string(),
                candidate_a,
                candidate_b,
            });
        }
        if let Some((res, _)) = single(candidates) {
            return Ok(res);
        }
    }

    if candidates.is_empty() {
        if let Some(&res) = globals.get(kind, namespace, name_str) {
            // An unopened global is the last resort.
            return Ok(res);
        }
    }

    if candidates.len() > 1 {
        // If there are multiple candidates, remove unimplemented items. This allows resolution to
        // succeed in cases where both an older, unimplemented API and newer, implemented API with the
        // same name are both in scope without forcing the user to fully qualify the name.
        let mut removals = Vec::new();
        for res in candidates.keys() {
            if let Res::Item(_, ItemStatus::Unimplemented) = res {
                removals.push(*res);
            }
        }
        for res in removals {
            candidates.remove(&res);
        }
    }

    if candidates.len() > 1 {
        let mut opens: Vec<_> = candidates.into_values().collect();
        opens.sort_unstable_by_key(|open| open.span);
        Err(Error::Ambiguous {
            name: name_str.to_string(),
            first_open: opens[0].namespace.to_string(),
            second_open: opens[1].namespace.to_string(),
            name_span: name.span,
            first_open_span: opens[0].span,
            second_open_span: opens[1].span,
        })
    } else {
        single(candidates.into_keys())
            .ok_or_else(|| Error::NotFound(name_str.to_string(), name.span))
    }
}

/// Implements shadowing rules within a single scope.
/// A local variable always wins out against an item with the same name, if they're declared in
/// the same scope. It is implemented in a way that resembles Rust:
/// ```rust
/// let foo = || 1;
/// fn foo() -> i32 { 2 }
/// dbg!(foo()); // 1, not 2
/// ```
fn resolve_scope_locals(
    kind: NameKind,
    globals: &GlobalScope,
    scope: &Scope,
    vars: bool,
    name: &str,
) -> Option<Res> {
    if vars {
        match kind {
            NameKind::Term => {
                if let Some(&id) = scope.vars.get(name) {
                    return Some(Res::Local(id));
                }
            }
            NameKind::Ty => {
                if let Some(&id) = scope.ty_vars.get(name) {
                    return Some(Res::Param(id));
                }
            }
        }
    }

    if let Some(&id) = scope.item(kind, name) {
        return Some(Res::Item(id, ItemStatus::Available));
    }

    if let ScopeKind::Namespace(namespace) = &scope.kind {
        if let Some(&res) = globals.get(kind, namespace, name) {
            return Some(res);
        }
    }

    None
}
/// The return type represents the resolution of implicit opens, but also
/// retains the namespace that the resolution comes from.
/// This retained namespace string is used for error reporting.
fn resolve_implicit_opens<'a, 'b>(
    kind: NameKind,
    globals: &'b GlobalScope,
    namespaces: impl IntoIterator<Item = &'a &'a str>,
    name: &'b str,
) -> FxHashMap<Res, &'a str> {
    let mut candidates = FxHashMap::default();
    for namespace in namespaces {
        if let Some(&res) = globals.get(kind, namespace, name) {
            candidates.insert(res, *namespace);
        }
    }
    candidates
}

fn resolve_explicit_opens<'a>(
    kind: NameKind,
    globals: &GlobalScope,
    opens: impl IntoIterator<Item = &'a Open>,
    name: &str,
) -> FxHashMap<Res, &'a Open> {
    let mut candidates = FxHashMap::default();
    for open in opens {
        if let Some(&res) = globals.get(kind, &open.namespace, name) {
            candidates.insert(res, open);
        }
    }
    candidates
}

fn intrapackage(item: LocalItemId) -> ItemId {
    ItemId {
        package: None,
        item,
    }
}

fn single<T>(xs: impl IntoIterator<Item = T>) -> Option<T> {
    let mut xs = xs.into_iter();
    let x = xs.next();
    match xs.next() {
        None => x,
        Some(_) => None,
    }
}
