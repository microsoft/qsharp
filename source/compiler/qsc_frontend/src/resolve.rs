// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod import_item;
#[cfg(test)]
mod tests;

use crate::compile::preprocess::TrackedName;
use crate::resolve::import_item::{ValidImportOrExportItem, iter_valid_items};
use miette::Diagnostic;
use qsc_ast::ast::{ImportOrExportDecl, ItemKind, Package};
use qsc_ast::{
    ast::{
        self, CallableBody, CallableDecl, ClassConstraints, Ident, Idents, ImportKind, Item,
        NodeId, PathKind, SpecBody, SpecGen, TopLevelNode, TypeParameter,
    },
    visit::{self as ast_visit, Visitor as AstVisitor, walk_attr},
};
use qsc_data_structures::{
    index_map::IndexMap,
    namespaces::{ClobberedNamespace, NamespaceId, NamespaceTreeRoot, PRELUDE},
    span::Span,
};
use qsc_hir::{
    assigner::Assigner,
    global,
    hir::{self, ItemId, ItemStatus, LocalItemId, PackageId},
    ty::{ParamId, Prim},
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{cmp::Ordering, ops::Deref};
use std::{collections::hash_map::Entry, rc::Rc, str::FromStr, vec};
use thiserror::Error;

// All AST Path nodes that are namespace paths get mapped
// All AST Ident nodes get mapped, except those under AST Path nodes
// The first Ident of an AST Path node that is a field accessor gets mapped instead of the Path node
pub(super) type Names = IndexMap<NodeId, Res>;

// If the path is a field accessor, returns the mapped node id of the first ident's declaration and the vec of part's idents.
// Otherwise, returns None.
// Field accessor paths have their leading segment mapped as a local variable, whereas namespace paths have their path id mapped.
#[must_use]
pub fn path_as_field_accessor<'a>(
    names: &Names,
    path: &'a impl Idents,
) -> Option<(NodeId, Vec<&'a ast::Ident>)> {
    let parts: Vec<&Ident> = path.iter().collect();
    let first = parts.first().expect("path should have at least one part");
    if let Some(&Res::Local(node_id)) = names.get(first.id) {
        return Some((node_id, parts));
    }
    // If any of the above conditions are not met, return None.
    None
}

/// A resolution. This connects a usage of a name with the declaration of that name by uniquely
/// identifying the node that declared it.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Res {
    /// A global or local item.
    Item(ItemId, ItemStatus),
    /// A local variable.
    Local(NodeId),
    /// A type/functor parameter in the generics section of the parent callable decl.
    Param {
        id: ParamId,
        bounds: ClassConstraints,
    },
    /// A primitive type.
    PrimTy(Prim),
    /// The unit type.
    UnitTy,
    /// Something that can be imported
    Importable(ImportableItemKind, ImportableVisibility),
}

impl Res {
    #[must_use]
    pub fn item_id(&self) -> Option<ItemId> {
        match self {
            Res::Item(id, ..)
            | Res::Importable(
                ImportableItemKind::Callable(id, _) | ImportableItemKind::Ty(id, _),
                ..,
            ) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ImportableItemKind {
    Callable(ItemId, ItemStatus), // Callable, or term?
    Ty(ItemId, ItemStatus),
    Namespace(NamespaceId, Option<ItemId>), // not all namespaces have item IDs
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ImportableVisibility {
    Unused,
}

#[derive(Clone, Debug, Diagnostic, Error, PartialEq)]
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

    #[error("duplicate declaration of `{0}`")]
    #[diagnostic(code("Qsc.Resolve.DuplicateName"))]
    DuplicateName(String, #[label] Span),

    #[error("duplicate name `{0}` in pattern")]
    #[diagnostic(help("a name cannot shadow another name in the same pattern"))]
    #[diagnostic(code("Qsc.Resolve.DuplicateBinding"))]
    DuplicateBinding(String, #[label] Span),

    #[error("duplicate intrinsic `{0}`")]
    #[diagnostic(help(
        "each callable declared as `body intrinsic` or `@SimulatableIntrinsic` must have a globally unique name"
    ))]
    #[diagnostic(code("Qsc.Resolve.DuplicateIntrinsic"))]
    DuplicateIntrinsic(String, #[label] Span),

    #[error("duplicate export of item `{name}`")]
    DuplicateExport {
        name: String,
        #[label]
        span: Span,
        #[label("item was previously exported here")]
        existing_span: Span,
    },

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

    #[error("export statements are not allowed in a local scope")]
    #[diagnostic(code("Qsc.Resolve.ExportFromLocalScope"))]
    ExportFromLocalScope(#[label] Span),

    #[error(
        "namespace export {namespace_name} overwrites (clobbers) an existing external namespace of the same name"
    )]
    ClobberedNamespace {
        namespace_name: String,
        #[label]
        span: Span,
    },

    #[error("reexporting a namespace from another package is not supported")]
    #[diagnostic(code("Qsc.Resolve.CrossPackageNamespaceReexport"))]
    CrossPackageNamespaceReexport(#[label] Span),
}

#[derive(Debug, Clone)]
pub struct Scope {
    /// The span that the scope applies to. For callables and namespaces, this includes
    /// the entire callable / namespace declaration. For blocks, this includes the braces.
    span: Span,
    kind: ScopeKind,
    /// Open statements. The key is the namespace name or alias.
    /// TODO: why would the key be a namespace name? isn't it a single ident?
    opens: FxHashMap<Vec<Rc<str>>, Vec<Open>>,
    /// Local newtype declarations.
    tys: FxHashMap<Rc<str>, ScopeItemEntry>,
    /// Local callable and newtype declarations.
    terms: FxHashMap<Rc<str>, ScopeItemEntry>,
    /// Importable items, such as callables, types, and namespaces.
    importables: FxHashMap<Rc<str>, Res>,
    /// Local variables, including callable parameters, for loop bindings, etc.
    /// The u32 is the `valid_at` offset - the lowest offset at which the variable name is available.
    /// It's used to determine which variables are visible at a specific offset in the scope.
    ///
    /// Bug: Because we keep track of only one `valid_at` offset per name,
    /// when a variable is later shadowed in the same scope,
    /// it is missed in the list. <a href=https://github.com/microsoft/qsharp/issues/897 />
    vars: FxHashMap<Rc<str>, (u32, NodeId)>,
    /// Type parameters.
    ty_vars: FxHashMap<Rc<str>, (ParamId, ClassConstraints)>,
}

#[derive(Debug, Clone)]
pub struct ScopeItemEntry {
    pub id: ItemId,
    pub source: ItemSource,
}

impl ScopeItemEntry {
    #[must_use]
    pub fn new(id: ItemId, source: ItemSource) -> Self {
        Self { id, source }
    }
}

#[derive(PartialEq, Debug, Clone, Default)]
pub enum ItemSource {
    // if the item was imported with an alias, the alias is stored here
    Imported(Option<Ident>),
    #[default]
    Declared,
}

impl Scope {
    fn new(kind: ScopeKind, span: Span) -> Self {
        Self {
            span,
            kind,
            opens: FxHashMap::default(),
            tys: FxHashMap::default(),
            terms: FxHashMap::default(),
            importables: FxHashMap::default(),
            vars: FxHashMap::default(),
            ty_vars: FxHashMap::default(),
        }
    }

    fn item(&self, kind: NameKind, name: &str) -> Option<Res> {
        match kind {
            NameKind::Term => self
                .terms
                .get(name)
                .map(|x| Res::Item(x.id, ItemStatus::Available)),
            NameKind::Ty => self
                .tys
                .get(name)
                .map(|x| Res::Item(x.id, ItemStatus::Available)),
            NameKind::Importable => self.importables.get(name).cloned(),
        }
    }

    /// A `ScopeKind, Span` pair uniquely identifies a scope.
    fn key(&self) -> (&ScopeKind, Span) {
        (&self.kind, self.span)
    }
}

type ScopeId = usize;

#[derive(Debug, Clone, Default)]
pub struct Locals {
    // order is ascending by span (outermost -> innermost)
    scopes: Vec<Scope>,
}

impl Locals {
    fn get_scopes<'a>(
        &'a self,
        scope_chain: &'a [ScopeId],
    ) -> impl Iterator<Item = &'a Scope> + 'a {
        // reverse to go from innermost -> outermost
        scope_chain.iter().rev().map(|id| {
            self.scopes
                .get(*id)
                .unwrap_or_else(|| panic!("scope with id {id:?} should exist"))
        })
    }

    fn push_scope(&mut self, kind: ScopeKind, span: Span) -> ScopeId {
        // First, check if this scope has already been created in a prior pass.
        for (id, existing_scope) in self.scopes.iter_mut().enumerate() {
            if existing_scope.key() == (&kind, span) {
                // If the scope already exists, return that.
                return id;
            }
        }

        // Add it to the list of known scopes.
        let id = self.scopes.len();
        self.scopes.insert(id, Scope::new(kind, span));
        id
    }

    fn get_scope_mut(&mut self, id: ScopeId) -> &mut Scope {
        self.scopes
            .get_mut(id)
            .unwrap_or_else(|| panic!("scope with id {id:?} should exist"))
    }

    #[must_use]
    pub fn get_all_at_offset(&self, offset: u32) -> Vec<Local> {
        let mut vars = true;
        let mut all_locals = Vec::new();
        self.for_each_scope_at_offset(offset, |scope| {
            // inner to outer
            all_locals.extend(get_scope_locals(scope, offset, vars));

            if scope.kind == ScopeKind::Callable {
                // Since local callables are not closures, hide local variables in parent scopes.
                vars = false;
            }
        });

        // deduping by name will effectively make locals in a child scope
        // shadow the locals in its parent scopes
        all_locals.dedup_by(|a, b| a.name == b.name);

        all_locals
    }

    fn for_each_scope_at_offset<F>(&self, offset: u32, mut f: F)
    where
        F: FnMut(&Scope),
    {
        // reverse to go from innermost -> outermost
        self.scopes.iter().rev().for_each(|scope| {
            // the block span includes the delimiters (e.g. the braces)
            if scope.span.lo < offset && scope.span.hi > offset {
                f(scope);
            }
        });
    }
}

#[derive(Debug)]
pub struct Local {
    pub name: Rc<str>,
    pub kind: LocalKind,
}

#[derive(Debug)]
pub enum LocalKind {
    /// A local callable or UDT.
    Item(ItemId),
    /// A type parameter.
    TyParam(ParamId),
    /// A local variable or parameter.
    Var(NodeId),
}

#[derive(Debug, Clone, Default)]
pub struct GlobalScope {
    tys: IndexMap<NamespaceId, FxHashMap<Rc<str>, Res>>,
    terms: IndexMap<NamespaceId, FxHashMap<Rc<str>, Res>>,
    // imports: IndexMap<NamespaceId, FxHashMap<Rc<str>, Res>>,
    // Technically the name could exist both as a namespace and an item
    // within the same namespace, as we don't explicitly disallow that.
    // We don't go out of our way to detect this condition, so the name
    // resolution will be somewhat undeterministic in this case.
    importables: IndexMap<NamespaceId, FxHashMap<Rc<str>, Res>>,
    namespace_items: IndexMap<NamespaceId, Res>, // TODO: do we need this? Maybe namespace->itemId mapping can live in the lowerer
    namespaces: NamespaceTreeRoot,
    intrinsics: FxHashSet<Rc<str>>,
    self_exported_item_ids: FxHashMap<ItemId, Span>,
}

impl GlobalScope {
    fn find_namespace<'a>(&self, ns: impl IntoIterator<Item = &'a str>) -> Option<NamespaceId> {
        self.namespaces.get_namespace_id(ns)
    }

    fn get(&self, kind: NameKind, namespace: NamespaceId, name: &str) -> Option<&Res> {
        let items = match kind {
            NameKind::Term => &self.terms,
            NameKind::Ty => &self.tys,
            NameKind::Importable => &self.importables,
        };
        // eprintln!("looking up name {name} in namespace {namespace:?}");
        items.get(namespace).and_then(|items| items.get(name))
    }

    /// Creates a namespace in the namespace mapping. Note that namespaces are tracked separately from their
    /// item contents. This returns a [`NamespaceId`] which you can use to add more tys and terms to the scope.
    fn insert_or_find_namespace(&mut self, name: impl IntoIterator<Item = Rc<str>>) -> NamespaceId {
        self.namespaces.insert_or_find_namespace(name)
    }

    /// Given a starting namespace, search from that namespace.
    fn insert_or_find_namespace_from_root(
        &mut self,
        ns: Vec<Rc<str>>,
        root: NamespaceId,
    ) -> NamespaceId {
        self.namespaces.insert_or_find_namespace_from_root(ns, root)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ScopeKind {
    Namespace(NamespaceId),
    Callable,
    Block,
}

#[derive(Clone, Copy, Debug)]
enum NameKind {
    Term,
    Ty,
    /// Something that we can import: a callable, a UDT,
    /// a namespace, or another import
    Importable,
}

#[derive(Debug, Clone)]
struct Open {
    namespace: NamespaceId,
    span: Span,
}

impl Eq for Open {}

impl PartialEq<Self> for Open {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace
    }
}

impl PartialOrd<Self> for Open {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Open {
    fn cmp(&self, other: &Self) -> Ordering {
        let a: usize = self.namespace.into();
        let b: usize = other.namespace.into();
        a.cmp(&b)
    }
}

pub(super) struct Resolver {
    names: Names,
    dropped_names: Vec<TrackedName>,
    curr_params: Option<FxHashSet<Rc<str>>>,
    curr_scope_chain: Vec<ScopeId>,
    globals: GlobalScope,
    locals: Locals,
    errors: Vec<Error>,
}

pub(super) struct ExportImportVisitor<'a> {
    pub(super) resolver: &'a mut Resolver,
    /// Some state change that could cause other imports to
    /// resolve differently, a new bound import, or a new open added
    new_imports_bound: bool,
    attempted_imports: FxHashMap<NodeId, Result<(), Error>>,
}

impl ExportImportVisitor<'_> {
    /// Given a function, apply that function to `Self` within a scope. In other words, this
    /// function automatically pushes a scope before `f` and pops it after.
    fn with_scope(&mut self, span: Span, kind: ScopeKind, f: impl FnOnce(&mut Self)) {
        self.resolver.push_scope(span, kind);
        f(self);
        self.resolver.pop_scope();
    }
}

impl AstVisitor<'_> for ExportImportVisitor<'_> {
    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        let ns = self
            .resolver
            .globals
            .find_namespace(namespace.name.str_iter())
            .expect("namespace should exist by this point");

        let root_id = self.resolver.globals.namespaces.root_id();
        self.with_scope(namespace.span, ScopeKind::Namespace(ns), |visitor| {
            let new_imports_bound = resolve_and_bind_namespace_imports_and_exports(
                namespace,
                ns,
                root_id,
                &mut visitor.attempted_imports,
                visitor.resolver,
            );
            visitor.new_imports_bound = new_imports_bound || visitor.new_imports_bound;
        });
    }

    // TODO: maybe make this a check in the parser
    // TODO: we're not actually hitting this in the case we expect!
    fn visit_item(&mut self, item: &Item) {
        item.attrs.iter().for_each(|a| self.visit_attr(a));
        match &*item.kind {
            ItemKind::ImportOrExport(decl) if decl.is_export() => self
                .resolver
                .errors
                .push(Error::ExportFromLocalScope(item.span)),
            ItemKind::Open(..) => (),
            _ => ast_visit::walk_item(self, item),
        }
    }
}

fn resolve_and_bind_namespace_imports_and_exports(
    namespace: &ast::Namespace,
    ns: NamespaceId,
    root_id: NamespaceId,
    attempted_imports: &mut FxHashMap<NodeId, Result<(), Error>>,
    resolver: &mut Resolver,
) -> bool {
    let mut new_imports_bound = false;
    // Start by adding the opens and glob imports in this scope

    // the below line ensures that this namespace opens itself, in case
    // we are re-declaring a namespace. This is important, as without this,
    // a re-declared namespace would only have knowledge of the items declared within this declaration.
    // TODO: test this claim
    resolver.add_open(&namespace.name, None, root_id);

    // Handle all opens first
    for item in &*namespace.items {
        if let ItemKind::Open(PathKind::Ok(path), alias) = &*item.kind {
            resolver.add_open(path.as_ref(), alias.as_deref(), ns);
        }
    }

    // Glob imports are treated as opens, handle them too
    for item in &namespace.items {
        if let ItemKind::ImportOrExport(decl) = &*item.kind {
            for item in iter_valid_items(decl) {
                if let ImportKind::Wildcard = item.kind {
                    resolver.add_open(item.path, None, ns);
                }
            }
        }
    }

    // now, attempt to resolve all imports (exports included), and bind the names
    // this may not resolve all imports, as import resolution is iterative
    for item in &namespace.items {
        if let ItemKind::ImportOrExport(decl) = &*item.kind {
            let bound = resolver.resolve_and_bind_import_or_export_decl(
                decl,
                Some(ns),
                Some(&namespace.name),
                attempted_imports,
            );
            new_imports_bound = bound || new_imports_bound;
        }
    }
    new_imports_bound
}

impl Resolver {
    pub(crate) fn resolve_and_bind_all_namespace_imports_and_exports(&mut self, package: &Package) {
        let mut visitor = ExportImportVisitor {
            resolver: self,
            new_imports_bound: false,
            attempted_imports: FxHashMap::default(),
        };
        for i in 1..=100 {
            visitor.new_imports_bound = false;
            visitor.visit_package(package);
            if !visitor.new_imports_bound {
                // If no imports were resolved in this pass, we are done.
                break;
            }

            // TODO: technically an error, not a panic
            assert!(
                i < 100,
                "could not finish resolving imports after 100 iterations"
            );
        }
        for (_, result) in visitor.attempted_imports.drain() {
            if let Err(err) = result {
                self.errors.push(err);
            }
        }
    }

    pub(super) fn new(globals: GlobalTable, dropped_names: Vec<TrackedName>) -> Self {
        Self {
            names: globals.names,
            dropped_names,
            curr_params: None,
            globals: globals.scope,
            locals: Locals::default(),
            curr_scope_chain: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub(super) fn with_persistent_local_scope(
        globals: GlobalTable,
        dropped_names: Vec<TrackedName>,
    ) -> Self {
        let mut locals = Locals::default();
        let scope_id = locals.push_scope(
            ScopeKind::Block,
            Span {
                lo: 0,
                hi: u32::MAX,
            },
        );
        Self {
            names: globals.names,
            dropped_names,
            curr_params: None,
            globals: globals.scope,
            locals,
            curr_scope_chain: vec![scope_id],
            errors: Vec::new(),
        }
    }

    pub(super) fn names(&self) -> &Names {
        &self.names
    }

    pub(super) fn globals(&self) -> &GlobalScope {
        &self.globals
    }

    pub(super) fn locals(&self) -> &Locals {
        &self.locals
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

    pub(super) fn into_result(self) -> (Names, GlobalScope, Locals, Vec<Error>) {
        (self.names, self.globals, self.locals, self.errors)
    }

    pub(super) fn extend_dropped_names(&mut self, dropped_names: Vec<TrackedName>) {
        self.dropped_names.extend(dropped_names);
    }

    pub(super) fn bind_fragments(&mut self, ast: &ast::Package, assigner: &mut Assigner) {
        for node in &mut ast.nodes.iter() {
            match node {
                TopLevelNode::Namespace(namespace) => {
                    bind_global_items(
                        &mut self.names,
                        &mut self.globals,
                        namespace,
                        assigner,
                        &mut self.errors,
                    );
                }
                TopLevelNode::Stmt(stmt) => {
                    if let ast::StmtKind::Item(item) = stmt.kind.as_ref() {
                        self.bind_local_item(assigner, item, None);
                    }
                }
            }
        }

        let stmts = ast
            .nodes
            .iter()
            .filter_map(|node| {
                if let TopLevelNode::Stmt(stmt) = node {
                    Some(stmt.as_ref())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.resolve_and_bind_local_imports_and_exports(&stmts);
    }

    fn check_item_status(&mut self, res: &Res, name: String, span: Span) {
        if let Res::Item(_, ItemStatus::Unimplemented) = res {
            self.errors.push(Error::Unimplemented(name, span));
        }
    }

    fn resolve_ident(&mut self, kind: NameKind, name: &Ident) {
        match resolve(
            kind,
            &self.globals,
            self.locals.get_scopes(&self.curr_scope_chain),
            name,
            None,
        ) {
            Ok(res) => {
                self.check_item_status(&res, name.name.to_string(), name.span);
                self.names.insert(name.id, res);
            }
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_path_kind(&mut self, kind: NameKind, path: &ast::PathKind) -> Result<(), Error> {
        match path {
            PathKind::Ok(path) => self.resolve_path(kind, path).map(|_| ()),
            PathKind::Err(incomplete_path) => {
                // First we check if the the path can be resolved as a field accessor.
                // We do this by checking if the first part of the path is a local variable.
                if let (NameKind::Term, Some(incomplete_path)) = (kind, incomplete_path) {
                    let first = incomplete_path
                        .segments
                        .first()
                        .expect("path `segments` should have at least one element");
                    match resolve(
                        kind,
                        &self.globals,
                        self.locals.get_scopes(&self.curr_scope_chain),
                        first,
                        None,
                    ) {
                        Ok(res) if matches!(res, Res::Local(_)) => {
                            // The path is a field accessor.
                            self.names.insert(first.id, res.clone());
                            return Ok(());
                        }
                        Err(err) if !matches!(err, Error::NotFound(_, _)) => return Err(err), // Local was found but has issues.
                        _ => return Ok(()), // The path is assumed to not be a field accessor, so move on.
                    }
                }

                Ok(())
            }
        }
    }

    fn resolve_path(&mut self, kind: NameKind, path: &ast::Path) -> Result<Res, Error> {
        let name = &path.name;
        let segments = &path.segments;

        // First we check if the the path can be resolved as a field accessor.
        // We do this by checking if the first part of the path is a local variable.
        if let (NameKind::Term, Some(segments)) = (kind, segments) {
            let first = segments
                .first()
                .expect("path `segments` should have at least one element");
            match resolve(
                kind,
                &self.globals,
                self.locals.get_scopes(&self.curr_scope_chain),
                first,
                None,
            ) {
                Ok(res) if matches!(res, Res::Local(_)) => {
                    // The path is a field accessor.
                    self.names.insert(first.id, res.clone());
                    return Ok(res);
                }
                Err(err) if !matches!(err, Error::NotFound(_, _)) => return Err(err), // Local was found but has issues.
                _ => {} // The path is assumed to not be a field accessor, so move on to process it as a namespace path.
            }
        }

        // If the path is not a field accessor, we resolve it as a namespace path.
        // This is done by passing in the last part of the path as the name to resolve,
        // with the rest of the parts as the namespace segments.
        match resolve(
            kind,
            &self.globals,
            self.locals.get_scopes(&self.curr_scope_chain),
            name,
            segments.as_deref(),
        ) {
            Ok(res) => {
                self.check_item_status(&res, path.name.name.to_string(), path.span);
                // eprintln!(
                //     "resolved path {} id {} to {:?}",
                //     path.full_name(),
                //     path.id,
                //     res
                // );
                self.names.insert(path.id, res.clone());
                Ok(res)
            }
            Err(err) => {
                if let Error::NotFound(name, span) = err {
                    if let Some(dropped_name) =
                        self.dropped_names.iter().find(|n| n.name.as_ref() == name)
                    {
                        Err(Error::NotAvailable(
                            name,
                            format!("{}.{}", dropped_name.namespace, dropped_name.name),
                            span,
                        ))
                    } else {
                        Err(Error::NotFound(name, span))
                    }
                } else {
                    Err(err)
                }
            }
        }
    }

    /// # Arguments
    ///
    /// * `pat` - The pattern to bind.
    /// * `valid_at` - The offset at which the name becomes defined. This is used to determine
    ///   whether a name is in scope at a given offset.
    ///   e.g. For a local variable, this would be immediately after the declaration statement.
    ///   For input parameters to a callable, this would be the start of the body block.
    fn bind_pat(&mut self, pat: &ast::Pat, valid_at: u32) {
        let mut bindings = FxHashSet::default();
        self.bind_pat_recursive(pat, valid_at, &mut bindings);
    }

    fn bind_pat_recursive(
        &mut self,
        pat: &ast::Pat,
        valid_at: u32,
        bindings: &mut FxHashSet<Rc<str>>,
    ) {
        match &*pat.kind {
            ast::PatKind::Bind(name, _) => {
                if !bindings.insert(Rc::clone(&name.name)) {
                    self.errors
                        .push(Error::DuplicateBinding(name.name.to_string(), name.span));
                }
                self.names.insert(name.id, Res::Local(name.id));
                self.current_scope_mut()
                    .vars
                    .insert(Rc::clone(&name.name), (valid_at, name.id));
            }
            ast::PatKind::Discard(_) | ast::PatKind::Elided | ast::PatKind::Err => {}
            ast::PatKind::Paren(pat) => self.bind_pat_recursive(pat, valid_at, bindings),
            ast::PatKind::Tuple(pats) => pats
                .iter()
                .for_each(|p| self.bind_pat_recursive(p, valid_at, bindings)),
        }
    }

    fn resolve_namespace(
        &mut self,
        ns: &impl Idents,
        current_namespace: NamespaceId,
    ) -> Option<NamespaceId> {
        let (_current_ns_name, current_namespace) = self
            .globals
            .namespaces
            .find_namespace_by_id(&current_namespace);
        // try scoping from the current namespace, and then use the absolute namespace as the backup
        let id = if let Some(id) = (*current_namespace)
            .borrow()
            .get_namespace_id(ns.str_iter())
        {
            id
        } else if let Some(id) = self.globals.namespaces.get_namespace_id(ns.str_iter()) {
            id
        } else {
            return None;
        };
        Some(id)
    }

    /// Adds this namespace to the current scope's opens. Namespace path is resolved
    /// relative to the current namespace.
    fn add_open(
        &mut self,
        path: &impl Idents,
        alias: Option<&Ident>,
        current_namespace: NamespaceId,
    ) {
        // eprintln!(
        //     "adding open for path {}, alias {:?}",
        //     path.full_name(),
        //     alias
        // );
        let ns_id = self.resolve_namespace(path, current_namespace);
        let Some(id) = ns_id else {
            self.errors.push(Error::NotFound(
                path.full_name().to_string(),
                path.full_span(),
            ));
            return;
        };

        let alias = alias.as_ref().map_or(vec![], |a| vec![Rc::clone(&a.name)]);

        let current_opens = self
            .current_scope_mut()
            .opens
            .entry(alias.clone())
            .or_default();

        let open = Open {
            namespace: id,
            span: path.full_span(),
        };
        if !current_opens.contains(&open) {
            current_opens.push(open);
        }
    }

    pub(super) fn bind_local_item(
        &mut self,
        assigner: &mut Assigner,
        item: &ast::Item,
        _namespace: Option<NamespaceId>,
    ) {
        match &*item.kind {
            ast::ItemKind::Open(..) => {}
            ast::ItemKind::Callable(decl) => {
                let id = intrapackage(assigner.next_item());
                let item_status = ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(&item.attrs));
                self.names.insert(decl.name.id, Res::Item(id, item_status));
                let scope = self.current_scope_mut();
                scope.terms.insert(
                    Rc::clone(&decl.name.name),
                    ScopeItemEntry::new(id, ItemSource::Declared),
                );
                scope.importables.insert(
                    Rc::clone(&decl.name.name),
                    Res::Importable(
                        ImportableItemKind::Callable(id, item_status),
                        ImportableVisibility::Unused,
                    ),
                );
            }
            ast::ItemKind::Ty(name, _) => {
                let id = intrapackage(assigner.next_item());
                let item_status = ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(&item.attrs));
                self.names.insert(name.id, Res::Item(id, item_status));
                let scope = self.current_scope_mut();
                scope.tys.insert(
                    Rc::clone(&name.name),
                    ScopeItemEntry::new(id, ItemSource::Declared),
                );
                scope.terms.insert(
                    Rc::clone(&name.name),
                    ScopeItemEntry::new(id, ItemSource::Declared),
                );
                scope.importables.insert(
                    Rc::clone(&name.name),
                    Res::Importable(
                        ImportableItemKind::Ty(id, item_status),
                        ImportableVisibility::Unused,
                    ),
                );
            }
            ast::ItemKind::Struct(decl) => {
                let id = intrapackage(assigner.next_item());
                let item_status = ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(&item.attrs));
                self.names.insert(decl.name.id, Res::Item(id, item_status));
                let scope = self.current_scope_mut();
                scope.tys.insert(
                    Rc::clone(&decl.name.name),
                    ScopeItemEntry::new(id, ItemSource::Declared),
                );
                scope.terms.insert(
                    Rc::clone(&decl.name.name),
                    ScopeItemEntry::new(id, ItemSource::Declared),
                );
                scope.importables.insert(
                    Rc::clone(&decl.name.name),
                    Res::Importable(
                        ImportableItemKind::Ty(id, item_status),
                        ImportableVisibility::Unused,
                    ),
                );
            }
            ast::ItemKind::ImportOrExport(decl) => {
                assign_item_id(&mut self.names, || intrapackage(assigner.next_item()), decl);
            }
            ast::ItemKind::Err => (),
        }
    }

    /// True if this call caused new names to become available in the scope.
    /// This is used to decide whether we should keep iterating.
    fn resolve_and_bind_import_or_export_decl(
        &mut self,
        decl: &ImportOrExportDecl,
        current_namespace: Option<NamespaceId>,
        current_namespace_name: Option<&[Ident]>,
        attempted_imports_to_resolution_results: &mut FxHashMap<NodeId, Result<(), Error>>,
    ) -> bool {
        let decl_items = filter_dropped_names(
            iter_valid_items(decl),
            current_namespace_name,
            &self.dropped_names,
        );

        if decl.is_export() && current_namespace.is_none() {
            self.errors.push(Error::ExportFromLocalScope(decl.span));
            return false;
        }

        let mut any_imports_were_bound = false;
        for valid_item in decl_items {
            let import_was_bound = match &valid_item.kind {
                ImportKind::Wildcard => {
                    // we should have handled these already
                    false
                }
                ImportKind::Direct { .. } => {
                    if attempted_imports_to_resolution_results
                        .get(&valid_item.name().id)
                        .is_some_and(Result::is_ok)
                    {
                        // If this item has already been bound, skip it.
                        continue;
                    }

                    let (resolution_result, import_was_bound) =
                        self.try_resolve_and_bind_import_or_export(current_namespace, &valid_item);

                    attempted_imports_to_resolution_results
                        .insert(valid_item.name().id, resolution_result);

                    import_was_bound
                }
            };
            any_imports_were_bound = any_imports_were_bound || import_was_bound;
        }
        any_imports_were_bound
    }

    fn try_resolve_and_bind_import_or_export(
        &mut self,
        current_namespace: Option<NamespaceId>,
        valid_item: &ValidImportOrExportItem<'_>,
    ) -> (Result<(), Error>, bool) {
        match self.resolve_path(NameKind::Importable, valid_item.path) {
            Ok(Res::Importable(imported_item_kind, _visibility)) => {
                let import_item_id = match self.names.get(valid_item.name().id) {
                    Some(Res::Item(item_id, ..)) => *item_id,
                    _ => panic!("unexpected resolution for import name"),
                };
                // TODO: check visibility? Not sure how I'm using this yet. Probably not at all.
                //
                // Successfully resolved. Bind this import's name as term/ty/importable
                // or any combination thereof so that it can be resolved from
                // expressions, other declarations, imports, etc

                // If the import item has the same namespace and name as the item
                // we just imported, we can skip binding it.

                let bind_result = if valid_item.is_export {
                    // If this an export, we bind in the global scope.
                    self.bind_export(
                        valid_item.name(),
                        import_item_id,
                        &imported_item_kind,
                        current_namespace.expect("current namespace should be set for exports"),
                    )
                } else {
                    // If this is an import, we bind in the current scope (block or namespace)
                    self.bind_import_in_current_scope(
                        current_namespace,
                        &valid_item,
                        import_item_id,
                        &imported_item_kind,
                    )
                };

                let new_import_was_bound = bind_result.is_ok();

                if let Err(err) = bind_result {
                    self.errors.push(err);
                }

                (Ok(()), new_import_was_bound)
            }
            Err(err) => (Err(err), false),
            Ok(res) => {
                unreachable!(
                    "we should never resolve an importable item to a non-importable res: {res:?}"
                );
            }
        }
    }

    fn bind_import_in_current_scope(
        &mut self,
        current_namespace: Option<NamespaceId>,
        valid_item: &ValidImportOrExportItem<'_>,
        import_item_id: ItemId,
        imported_item_kind: &ImportableItemKind,
    ) -> Result<(), Error> {
        match imported_item_kind {
            ImportableItemKind::Callable(imported_item_id, status) => self
                .bind_imported_callable_name_in_current_scope(
                    valid_item.name(),
                    current_namespace,
                    import_item_id,
                    *imported_item_id,
                    *status,
                ),
            ImportableItemKind::Ty(imported_item_id, status) => self
                .bind_imported_ty_name_in_current_scope(
                    valid_item.name(),
                    current_namespace,
                    import_item_id,
                    *imported_item_id,
                    *status,
                ),
            ImportableItemKind::Namespace(namespace_id, _) => {
                // A direct import of a namespace is the same as an open with an alias
                // TODO: add this namespace as an importable into the scope as well?

                let current_opens = self
                    .current_scope_mut()
                    .opens
                    .entry(vec![valid_item.name().name.clone()])
                    .or_default();

                let open = Open {
                    namespace: *namespace_id,
                    span: valid_item.path.full_span(),
                };
                if !current_opens.contains(&open) {
                    current_opens.push(open);
                }
                Ok(())
            }
        }
    }

    fn bind_export(
        &mut self,
        name: &Ident,
        export_item_id: ItemId,
        imported_item_kind: &ImportableItemKind,
        namespace: NamespaceId,
    ) -> Result<(), Error> {
        match *imported_item_kind {
            ImportableItemKind::Callable(imported_item_id, imported_item_status) => {
                bind_callable_export(
                    name,
                    namespace,
                    export_item_id,
                    imported_item_id,
                    imported_item_status,
                    &mut self.names,
                    &mut self.globals,
                )
            }
            ImportableItemKind::Ty(imported_item_id, imported_item_status) => bind_ty_export(
                name,
                namespace,
                export_item_id,
                imported_item_id,
                imported_item_status,
                &mut self.names,
                &mut self.globals,
            ),
            ImportableItemKind::Namespace(original_namespace_id, namespace_item_id) => {
                // Inserting this name as an alias for namespace_id
                let res = self
                    .globals
                    .namespaces
                    .insert_or_find_namespace_from_root_with_id(
                        vec![name.name.clone()],
                        namespace,
                        original_namespace_id,
                    );

                match res {
                    Ok(()) => bind_namespace_export(
                        name,
                        namespace,
                        export_item_id,
                        namespace_item_id,
                        original_namespace_id,
                        &mut self.names,
                        &mut self.globals,
                    ),
                    Err(_) => Err(Error::ClobberedNamespace {
                        namespace_name: name.name.to_string(),
                        span: name.span,
                    }),
                }
            }
        }
    }

    fn resolve_and_bind_local_imports_and_exports<T>(&mut self, stmts: &[T])
    where
        T: Deref<Target = ast::Stmt>,
    {
        // Now, resolve and bind all imports, do this
        // iteratively until all valid imports are resolved.
        let mut attempted_imports: FxHashMap<NodeId, Result<(), Error>> = FxHashMap::default();
        let mut new_imports_bound;

        for i in 1..=100 {
            new_imports_bound = false;

            let mut import_and_export_items = Vec::new();

            for stmt in stmts {
                let stmt = &**stmt;
                if let ast::StmtKind::Item(item) = &*stmt.kind {
                    if let ItemKind::ImportOrExport(decl) = &*item.kind {
                        for item in iter_valid_items(decl) {
                            import_and_export_items.push(item);
                        }
                    }
                }
            }

            // Handle all opens first
            for stmt in stmts {
                let stmt = &**stmt;
                if let ast::StmtKind::Item(item) = &*stmt.kind {
                    if let ItemKind::Open(PathKind::Ok(path), alias) = &*item.kind {
                        self.add_open(path.as_ref(), alias.as_deref(), self.current_namespace());
                    }
                }
            }

            // Glob imports are treated as opens, handle them too
            for item in &import_and_export_items {
                if let ImportKind::Wildcard = item.kind {
                    self.add_open(item.path, None, self.current_namespace());
                }
            }

            // now, attempt to resolve all imports, and bind the names
            // this may not resolve all imports, as import resolution is iterative

            for stmt in stmts {
                let stmt = &**stmt;
                if let ast::StmtKind::Item(item) = &*stmt.kind {
                    if let ItemKind::ImportOrExport(decl) = &*item.kind {
                        let bound = self.resolve_and_bind_import_or_export_decl(
                            decl,
                            None,
                            None,
                            &mut attempted_imports,
                        );
                        new_imports_bound = new_imports_bound || bound;
                    }
                }
            }

            if !new_imports_bound {
                // If no new imports were bound, we can stop.
                break;
            }
            // TODO: technically an error, not a panic
            assert!(
                i < 100,
                "could not finish resolving local imports after 100 iterations"
            );
        }
        for (_, result) in attempted_imports.drain() {
            if let Err(err) = result {
                self.errors.push(err);
            }
        }
    }

    /// For a given callable declaration, bind the names of the type parameters
    /// into the current scope. Tracks the constraints defined on the type parameters
    /// as well, for later use in type checking.
    fn bind_type_parameters(&mut self, decl: &CallableDecl) {
        decl.generics
            .iter()
            .enumerate()
            .for_each(|(ix, type_parameter)| {
                self.current_scope_mut().ty_vars.insert(
                    Rc::clone(&type_parameter.ty.name),
                    (ix.into(), type_parameter.constraints.clone()),
                );
                self.names.insert(
                    type_parameter.ty.id,
                    Res::Param {
                        id: ix.into(),
                        bounds: type_parameter.constraints.clone(),
                    },
                );
            });
    }

    fn bind_imported_callable_name_in_current_scope(
        &mut self,
        name: &Ident,
        namespace_id: Option<NamespaceId>,
        item_id: ItemId,
        original_item_id: ItemId,
        status: ItemStatus,
    ) -> Result<(), Error> {
        self.names.insert(name.id, Res::Item(item_id, status));

        if let Some(namespace_id) = namespace_id {
            // Check for collisions in the global scope - these will be item declarations or exports
            if let Some(res) = self
                .globals
                .importables
                .get_mut_or_default(namespace_id)
                .get(&name.name)
            {
                if res.item_id() == Some(original_item_id) {
                    // This is the item we're importing, not a collision,
                    // but we also don't need to bind the name again.
                    return Ok(());
                }
                return Err(Error::Duplicate(
                    name.name.to_string(),
                    self.globals
                        .namespaces
                        .find_namespace_by_id(&namespace_id)
                        .0
                        .join("."),
                    name.span,
                ));
            }
        }

        let scope = self.current_scope_mut();
        // allow shadowing in non-namespace scopes
        let allow_shadowing = matches!(scope.kind, ScopeKind::Block | ScopeKind::Callable);

        match scope.importables.entry(name.name.clone()) {
            Entry::Occupied(mut entry) if allow_shadowing => {
                entry.insert(Res::Importable(
                    ImportableItemKind::Callable(original_item_id, status),
                    ImportableVisibility::Unused,
                ));
                scope.terms.insert(
                    name.name.clone(),
                    ScopeItemEntry::new(original_item_id, ItemSource::Declared),
                );
            }
            Entry::Occupied(_) => {
                return Err(Error::DuplicateName(name.name.to_string(), name.span));
            }
            Entry::Vacant(entry) => {
                entry.insert(Res::Importable(
                    ImportableItemKind::Callable(original_item_id, status),
                    ImportableVisibility::Unused,
                ));
                scope.terms.insert(
                    name.name.clone(),
                    ScopeItemEntry::new(original_item_id, ItemSource::Declared),
                );
            }
        }
        Ok(())
    }

    /// Checks for collisions, which is different from local scope
    fn bind_imported_ty_name_in_current_scope(
        &mut self,
        name: &Ident,
        namespace_id: Option<NamespaceId>,
        item_id: ItemId,
        original_item_id: ItemId,
        status: ItemStatus,
    ) -> Result<(), Error> {
        self.names.insert(name.id, Res::Item(item_id, status));

        if let Some(namespace_id) = namespace_id {
            // Check for collisions in the global scope - these will be item declarations or exports
            if let Some(res) = self
                .globals
                .importables
                .get_mut_or_default(namespace_id)
                .get(&name.name)
            {
                if res.item_id() == Some(original_item_id) {
                    // This is the item we're importing, not a collision,
                    // but we also don't need to bind the name again.
                    return Ok(());
                }
                return Err(Error::Duplicate(
                    name.name.to_string(),
                    self.globals
                        .namespaces
                        .find_namespace_by_id(&namespace_id)
                        .0
                        .join("."),
                    name.span,
                ));
            }
        }

        let scope = self.current_scope_mut();

        // allow shadowing in non-namespace scopes
        let allow_shadowing = matches!(scope.kind, ScopeKind::Block | ScopeKind::Callable);

        match scope.importables.entry(name.name.clone()) {
            Entry::Occupied(mut entry) if allow_shadowing => {
                entry.insert(Res::Importable(
                    ImportableItemKind::Callable(original_item_id, status),
                    ImportableVisibility::Unused,
                ));
            }
            Entry::Occupied(_) => {
                return Err(Error::DuplicateName(name.name.to_string(), name.span));
            }
            Entry::Vacant(entry) => {
                entry.insert(Res::Importable(
                    ImportableItemKind::Callable(original_item_id, status),
                    ImportableVisibility::Unused,
                ));
            }
        }
        // eprintln!("inserting {} as ty in scope", name.name);
        scope.terms.insert(
            name.name.clone(),
            ScopeItemEntry::new(original_item_id, ItemSource::Declared),
        );
        scope.tys.insert(
            name.name.clone(),
            ScopeItemEntry::new(original_item_id, ItemSource::Declared),
        );
        Ok(())
    }

    fn push_scope(&mut self, span: Span, kind: ScopeKind) {
        let scope_id = self.locals.push_scope(kind, span);
        self.curr_scope_chain.push(scope_id);
    }

    fn pop_scope(&mut self) {
        self.curr_scope_chain
            .pop()
            .expect("pushed scope should be the last element on the stack");
    }

    /// Returns the innermost scope in the current scope chain.
    fn current_scope_mut(&mut self) -> &mut Scope {
        let scope_id = *self
            .curr_scope_chain
            .last()
            .expect("there should be at least one scope at location");

        self.locals.get_scope_mut(scope_id)
    }

    fn current_namespace(&self) -> NamespaceId {
        self.locals
            .get_scopes(&self.curr_scope_chain)
            .find_map(|scope| {
                if let ScopeKind::Namespace(id) = scope.kind {
                    Some(id)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| self.globals.namespaces.root_id())
    }

    pub(crate) fn with_errors(self, errors: Vec<Error>) -> Resolver {
        Resolver {
            names: self.names,
            dropped_names: self.dropped_names,
            curr_params: self.curr_params,
            globals: self.globals,
            locals: self.locals,
            curr_scope_chain: self.curr_scope_chain,
            errors,
        }
    }
}

fn filter_dropped_names<'a>(
    decl_items: impl Iterator<Item = ValidImportOrExportItem<'a>>,
    current_namespace_name: Option<&'a [Ident]>,
    dropped_names: &[TrackedName],
) -> Vec<ValidImportOrExportItem<'a>> {
    if let Some(current_namespace_name) = current_namespace_name {
        decl_items
            // filter out any dropped names
            // this is so you can still export an item that has been conditionally removed from compilation
            // without a resolution error in the export statement itself
            // This is not a perfect solution, re-exporting an aliased name from another namespace that has been
            // conditionally compiled out will still fail. However, this is the only way to solve this
            // problem without upleveling the preprocessor into the resolver, so it can do resolution-aware
            // dropped_names population.
            .filter(|item| {
                let item_as_tracked_name =
                    path_as_tracked_name(item.path, &current_namespace_name.full_name());
                !dropped_names.contains(&item_as_tracked_name)
            })
            .collect::<Vec<_>>()
    } else {
        decl_items.collect::<Vec<_>>()
    }
}

fn path_as_tracked_name(path: &ast::Path, current_namespace_name: &Rc<str>) -> TrackedName {
    TrackedName {
        name: path.name.name.clone(),
        namespace: current_namespace_name.clone(),
    }
}

/// Constructed from a [Resolver] and an [Assigner], this structure implements `Visitor`
/// on the package AST where it resolves identifiers, and assigns IDs to them.
pub(super) struct With<'a> {
    resolver: &'a mut Resolver,
    assigner: &'a mut Assigner,
}

impl With<'_> {
    /// Given a function, apply that function to `Self` within a scope. In other words, this
    /// function automatically pushes a scope before `f` and pops it after.
    fn with_scope(&mut self, span: Span, kind: ScopeKind, f: impl FnOnce(&mut Self)) {
        self.resolver.push_scope(span, kind);
        f(self);
        self.resolver.pop_scope();
    }

    /// Apply `f` to self while a pattern's constituent identifiers are in scope. Removes those
    /// identifiers from the scope after `f`.
    fn with_pat(&mut self, span: Span, kind: ScopeKind, pat: &ast::Pat, f: impl FnOnce(&mut Self)) {
        self.visit_pat(pat);
        self.with_scope(span, kind, |visitor| {
            // The bindings are valid from the beginning of the scope
            visitor.resolver.bind_pat(pat, span.lo);
            f(visitor);
        });
    }

    fn with_spec_pat(
        &mut self,
        span: Span,
        kind: ScopeKind,
        pat: &ast::Pat,
        f: impl FnOnce(&mut Self),
    ) {
        let mut bindings = self
            .resolver
            .curr_params
            .as_ref()
            .map_or_else(FxHashSet::default, std::clone::Clone::clone);
        self.with_scope(span, kind, |visitor| {
            visitor
                .resolver
                .bind_pat_recursive(pat, span.lo, &mut bindings);
            f(visitor);
        });
    }
}

impl AstVisitor<'_> for With<'_> {
    fn visit_namespace(&mut self, namespace: &ast::Namespace) {
        let ns = self
            .resolver
            .globals
            .find_namespace(namespace.name.str_iter())
            .expect("namespace should exist by this point");

        let kind = ScopeKind::Namespace(ns);
        self.with_scope(namespace.span, kind, |visitor| {
            for item in &namespace.items {
                match &*item.kind {
                    ItemKind::ImportOrExport(..) | ItemKind::Open(..) => {
                        // Global imports and exports should have been handled
                        // at this point.
                    }
                    _ => ast_visit::walk_item(visitor, item),
                }
            }
        });
    }

    fn visit_item(&mut self, item: &ast::Item) {
        match &*item.kind {
            ItemKind::ImportOrExport(decl) if decl.is_import() => {}
            ItemKind::Open(PathKind::Ok(_), _) => {}
            _ => ast_visit::walk_item(self, item),
        }
    }

    fn visit_attr(&mut self, attr: &ast::Attr) {
        // The Config attribute arguments do not go through name resolution.
        if hir::Attr::from_str(attr.name.name.as_ref()) != Ok(hir::Attr::Config) {
            walk_attr(self, attr);
        }
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
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
        self.with_scope(decl.span, ScopeKind::Callable, |visitor| {
            visitor.resolver.bind_type_parameters(decl);
            // The parameter bindings are valid after the end of the input pattern.
            // (More accurately, in the callable body, but we don't have a start offset for that).
            visitor.resolver.bind_pat(&decl.input, decl.input.span.hi);
            ast_visit::walk_callable_decl(visitor, decl);
        });
        self.resolver.curr_params = prev_param_names;
    }

    fn visit_spec_decl(&mut self, decl: &ast::SpecDecl) {
        if let SpecBody::Impl(input, block) = &decl.body {
            self.with_spec_pat(block.span, ScopeKind::Block, input, |visitor| {
                visitor.visit_block(block);
            });
        } else {
            ast_visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_ty(&mut self, ty: &ast::Ty) {
        match &*ty.kind {
            ast::TyKind::Path(PathKind::Ok(path)) => {
                if let Err(e) = self.resolver.resolve_path(NameKind::Ty, path) {
                    self.resolver.errors.push(e);
                }
            }
            ast::TyKind::Param(TypeParameter { ty, .. }) => {
                self.resolver.resolve_ident(NameKind::Ty, ty);
            }
            _ => ast_visit::walk_ty(self, ty),
        }
    }

    fn visit_block(&mut self, block: &ast::Block) {
        // eprintln!("visiting block {}", block.id);
        self.with_scope(block.span, ScopeKind::Block, |visitor| {
            // First, bind all declarations besides imports
            for stmt in &block.stmts {
                if let ast::StmtKind::Item(item) = &*stmt.kind {
                    visitor
                        .resolver
                        .bind_local_item(visitor.assigner, item, None);
                }
            }

            visitor
                .resolver
                .resolve_and_bind_local_imports_and_exports(block.stmts.as_ref());

            ast_visit::walk_block(visitor, block);
        });
    }

    fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match &*stmt.kind {
            ast::StmtKind::Item(item) => self.visit_item(item),
            ast::StmtKind::Local(_, pat, _) => {
                ast_visit::walk_stmt(self, stmt);
                // The binding is valid after end of the statement.
                self.resolver.bind_pat(pat, stmt.span.hi);
            }
            ast::StmtKind::Qubit(_, pat, init, block) => {
                ast_visit::walk_qubit_init(self, init);
                if let Some(block) = block {
                    self.with_pat(block.span, ScopeKind::Block, pat, |visitor| {
                        visitor.visit_block(block);
                    });
                } else {
                    // The binding is valid after end of the statement.
                    self.resolver.bind_pat(pat, stmt.span.hi);
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
                self.with_pat(block.span, ScopeKind::Block, pat, |visitor| {
                    visitor.visit_block(block);
                });
            }
            ast::ExprKind::Lambda(_, input, output) => {
                self.with_pat(output.span, ScopeKind::Block, input, |visitor| {
                    visitor.visit_expr(output);
                });
            }
            ast::ExprKind::Path(path) => {
                if let Err(e) = self.resolver.resolve_path_kind(NameKind::Term, path) {
                    self.resolver.errors.push(e);
                }
            }
            ast::ExprKind::TernOp(ast::TernOp::Update, container, index, replace)
            | ast::ExprKind::AssignUpdate(container, index, replace) => {
                self.visit_expr(container);
                if !is_field_update(
                    &self.resolver.globals,
                    self.resolver
                        .locals
                        .get_scopes(&self.resolver.curr_scope_chain),
                    index,
                ) {
                    self.visit_expr(index);
                }
                self.visit_expr(replace);
            }
            ast::ExprKind::Struct(PathKind::Ok(path), copy, fields) => {
                if let Err(e) = self.resolver.resolve_path(NameKind::Ty, path) {
                    self.resolver.errors.push(e);
                }
                copy.iter().for_each(|c| self.visit_expr(c));
                fields.iter().for_each(|f| self.visit_field_assign(f));
            }
            _ => ast_visit::walk_expr(self, expr),
        }
    }
}

pub(super) struct GlobalTable {
    names: Names,
    scope: GlobalScope,
}

impl std::fmt::Debug for GlobalTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let _ = writeln!(f, "  names:");
        // for (id, res) in &self.names {
        //     let _ = writeln!(f, "    {id}: {:?}", res.item_id());
        // }

        writeln!(f, "  global scope namespace items:")?;
        for (id, res) in &self.scope.namespace_items {
            let (name, _) = self.scope.namespaces.find_namespace_by_id(&id);
            writeln!(f, "    {id}: {}: {:?}", name.join("."), res.item_id())?;
        }
        Ok(())
    }
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

        let mut scope = GlobalScope::default();
        let ns = scope.insert_or_find_namespace(vec![Rc::from("Std"), Rc::from("Core")]);

        let mut tys = IndexMap::default();
        tys.insert(ns, core);

        Self {
            names: IndexMap::new(),
            scope: GlobalScope {
                tys,
                terms: IndexMap::default(),
                importables: IndexMap::default(),
                namespace_items: IndexMap::default(),
                namespaces: scope.namespaces,
                intrinsics: FxHashSet::default(),
                self_exported_item_ids: FxHashMap::default(),
            },
        }
    }

    pub(super) fn add_local_package(
        &mut self,
        assigner: &mut Assigner,
        package: &ast::Package,
    ) -> Vec<Error> {
        let mut errors = Vec::new();
        for node in &package.nodes {
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

    #[allow(clippy::too_many_lines)]
    pub(super) fn add_external_package(
        &mut self,
        id: PackageId,
        package: &hir::Package,
        store: &crate::compile::PackageStore,
        alias: Option<&str>,
    ) -> Result<(), Vec<Error>> {
        // eprintln!("add external package");
        // if there is a package-level alias defined, use that for the root namespace.
        let root = match alias {
            Some(alias) => self.scope.insert_or_find_namespace(vec![Rc::from(alias)]),
            // otherwise, these namespaces will be inserted into the root of the local package
            // without any alias.
            None => self.scope.namespaces.root_id(),
        };

        let mut errs = Vec::new();

        for global in global::iter_package(Some(id), package).filter(|global| {
            global.visibility == hir::Visibility::Public
                || matches!(&global.kind, global::Kind::Callable(t) if t.intrinsic)
        }) {
            // If the namespace is `Main` and we have an alias, we treat it as the root of the package, so there's no
            // namespace prefix between the dependency alias and the defined items.
            let global_namespace = if global.namespace.len() == 1
                && &*global.namespace[0] == "Main"
                && alias.is_some()
            {
                vec![]
            } else {
                global.namespace.clone()
            };

            let namespace = self
                .scope
                .insert_or_find_namespace_from_root(global_namespace.clone(), root);

            match (global.kind, global.visibility) {
                (global::Kind::Ty(ty), hir::Visibility::Public) => {
                    // eprintln!(
                    //     "got a ty {} : {} , namespace: {}",
                    //     ty.id,
                    //     global.name,
                    //     global_namespace.join(".")
                    // );
                    self.scope
                        .terms
                        .get_mut_or_default(namespace)
                        .insert(global.name.clone(), Res::Item(ty.id, global.status));
                    self.scope
                        .tys
                        .get_mut_or_default(namespace)
                        .insert(global.name.clone(), Res::Item(ty.id, global.status));
                    self.scope.importables.get_mut_or_default(namespace).insert(
                        global.name,
                        Res::Importable(
                            ImportableItemKind::Ty(ty.id, global.status),
                            ImportableVisibility::Unused,
                        ),
                    );
                }
                (global::Kind::Callable(term), visibility) => {
                    if visibility == hir::Visibility::Public {
                        self.scope
                            .terms
                            .get_mut_or_default(namespace)
                            .insert(global.name.clone(), Res::Item(term.id, global.status));

                        self.scope.importables.get_mut_or_default(namespace).insert(
                            global.name.clone(),
                            Res::Importable(
                                ImportableItemKind::Callable(term.id, global.status),
                                ImportableVisibility::Unused,
                            ),
                        );
                    }
                    if term.intrinsic {
                        self.scope.intrinsics.insert(global.name);
                    }
                }
                (global::Kind::Namespace(id), hir::Visibility::Public) => {
                    let ns_id = self
                        .scope
                        .insert_or_find_namespace_from_root(global_namespace.clone(), root);

                    let full_name = self.scope.namespaces.find_namespace_by_id(&ns_id).0;

                    let (name, parent_namespace) = full_name
                        .split_last()
                        .expect("namespace should not be empty");

                    // eprintln!("parent_namespace = {}", parent_namespace.join("."));

                    let parent_id = if parent_namespace.is_empty() {
                        self.scope.namespaces.root_id()
                    } else {
                        self.scope
                            .namespaces
                            .insert_or_find_namespace(parent_namespace.iter().cloned())
                    };

                    self.scope
                        .namespace_items
                        .insert(ns_id, Res::Item(id, ItemStatus::Available));

                    // eprintln!(
                    //     "added namespace with full_name {} id = {ns_id} parent_id = {parent_id}",
                    //     full_name.join(".")
                    // );

                    self.scope.importables.get_mut_or_default(parent_id).insert(
                        name.clone(),
                        Res::Importable(
                            ImportableItemKind::Namespace(ns_id, Some(id)),
                            ImportableVisibility::Unused,
                        ),
                    );
                }
                (global::Kind::Export(item_id), _) => {
                    // eprintln!("  export item: {} to item id {item_id}", global.name);
                    let Some(item) = find_item(store, item_id, id) else {
                        // eprintln!(
                        //     "could not resolve export item to {item_id} name: {} in {}",
                        //     global.name,
                        //     global_namespace.join(".")
                        // );
                        continue;
                    };
                    // eprintln!("    export item: found item {}", item.id);
                    match item.kind {
                        hir::ItemKind::Callable(..) => {
                            self.scope
                                .terms
                                .get_mut_or_default(namespace)
                                .insert(global.name.clone(), Res::Item(item_id, global.status));
                            self.scope.importables.get_mut_or_default(namespace).insert(
                                global.name,
                                Res::Importable(
                                    ImportableItemKind::Callable(item_id, global.status),
                                    ImportableVisibility::Unused,
                                ),
                            );
                        }
                        hir::ItemKind::Namespace(orig, _) => {
                            let orig_id = self
                                .scope
                                .insert_or_find_namespace_from_root(orig.clone().into(), root);

                            if let Err(ClobberedNamespace) = self.scope.namespaces.insert_with_id(
                                Some(namespace),
                                orig_id,
                                &global.name,
                            ) {
                                errs.push(Error::ClobberedNamespace {
                                    namespace_name: global.name.to_string(),
                                    span: Span::default(),
                                });
                            }

                            // TODO: should this be an importable? Probably not
                        }
                        hir::ItemKind::Ty(..) => {
                            self.scope.tys.get_mut_or_default(namespace).insert(
                                global.name.clone(),
                                Res::Item(item_id, ItemStatus::Available),
                            );
                            self.scope.terms.get_mut_or_default(namespace).insert(
                                global.name.clone(),
                                Res::Item(item_id, ItemStatus::Available),
                            );
                            self.scope.importables.get_mut_or_default(namespace).insert(
                                global.name,
                                Res::Importable(
                                    ImportableItemKind::Ty(item_id, global.status),
                                    ImportableVisibility::Unused,
                                ),
                            );
                        }
                        hir::ItemKind::Export(_, _) => {
                            unreachable!("find_item will never return an Export")
                        }
                    }
                }
                (_, hir::Visibility::Internal) => {}
            }
        }
        if !errs.is_empty() {
            return Err(errs);
        }
        // eprintln!("done adding external package!");
        Ok(())
    }
}

fn find_item(
    store: &crate::compile::PackageStore,
    item: ItemId,
    this_package: PackageId,
) -> Option<hir::Item> {
    let package_id = item.package.unwrap_or(this_package);
    let package = store.get(package_id)?;
    let item = package.package.items.get(item.item)?;
    match &item.kind {
        hir::ItemKind::Callable(_) | hir::ItemKind::Namespace(_, _) | hir::ItemKind::Ty(_, _) => {
            Some(item.clone())
        }
        hir::ItemKind::Export(_alias, hir::Res::Item(item)) => find_item(store, *item, package_id),
        hir::ItemKind::Export(_, _) => None,
    }
}

/// Given some namespace `namespace`, add all the globals declared within it to the global scope.
fn bind_global_items(
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
    namespace: &ast::Namespace,
    assigner: &mut Assigner,
    errors: &mut Vec<Error>,
) {
    let namespace_id = declare_namespace_id(namespace, assigner, names, scope);

    for item in &namespace.items {
        match bind_global_item(
            names,
            scope,
            namespace_id,
            || intrapackage(assigner.next_item()),
            item,
        ) {
            Ok(()) => {}
            Err(e) => errors.extend(e),
        }
    }
}

fn declare_namespace_id(
    namespace: &ast::Namespace,
    assigner: &mut Assigner,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> NamespaceId {
    let mut parent_id = scope.namespaces.root_id();
    // eprintln!("root id is {parent_id}");
    for part in namespace.name.rc_str_iter() {
        // eprintln!("part: {part}");
        // Bind every part of a dotted namespace declaration as
        // an importable, so import/export items can reference it.
        let ns_id = scope
            .namespaces
            .insert_or_find_namespace_from_root(vec![part.clone()], parent_id);

        if let Entry::Vacant(vacant_entry) = scope
            .importables
            .get_mut_or_default(parent_id)
            .entry(part.clone())
        {
            // eprintln!("binding namespace {part} as importable in {parent_id}");
            vacant_entry.insert(Res::Importable(
                ImportableItemKind::Namespace(ns_id, None),
                ImportableVisibility::Unused,
            ));
        }

        parent_id = ns_id;
    }

    let namespace_name = namespace.name.rc_str_iter().cloned().collect::<Vec<_>>();
    let node_id = namespace.id;
    let (name, parent_namespace) = namespace_name
        .split_last()
        .expect("namespace should not be empty");
    // TODO: do we need a fallback for root here?
    let parent_id = if parent_namespace.is_empty() {
        scope.namespaces.root_id()
    } else {
        scope.insert_or_find_namespace(parent_namespace.to_vec())
    };

    let namespace_id = scope.insert_or_find_namespace_from_root(vec![name.clone()], parent_id);
    // If the item id for the namespace already exists, reuse it.
    // Otherwise, insert the default value and return it
    // TODO: do we still need this behavior? I don't even know anymore
    if let Some(res) = scope.namespace_items.get(namespace_id) {
        names.insert(node_id, res.clone());
    } else {
        let item_id = intrapackage(assigner.next_item());

        let res = Res::Item(item_id, ItemStatus::Available);
        scope.namespace_items.insert(namespace_id, res);
        bind_namespace_name(
            name,
            node_id,
            parent_id,
            item_id,
            namespace_id,
            names,
            scope,
        );
    }
    namespace_id
}

fn bind_namespace_name(
    name: &Rc<str>,
    node_id: NodeId,
    parent_id: NamespaceId,
    item_id: ItemId,
    namespace_id: NamespaceId,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) {
    let res = Res::Item(item_id, ItemStatus::Available);
    scope.importables.get_mut_or_default(parent_id).insert(
        name.clone(),
        Res::Importable(
            ImportableItemKind::Namespace(namespace_id, Some(item_id)),
            ImportableVisibility::Unused,
        ),
    );
    names.insert(node_id, res);
}

fn bind_namespace_export(
    name: &Ident,
    parent_id: NamespaceId,
    import_item_id: ItemId,
    original_namespace_item_id: Option<ItemId>,
    namespace_id: NamespaceId,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> Result<(), Error> {
    if original_namespace_item_id.is_some_and(|item_id| item_id.package.is_some()) {
        // This is a namespace from an external package, and reexporting is disallowed
        // since it has no effect.

        return Err(Error::CrossPackageNamespaceReexport(name.span));
    }
    scope.importables.get_mut_or_default(parent_id).insert(
        name.name.clone(),
        Res::Importable(
            ImportableItemKind::Namespace(namespace_id, original_namespace_item_id),
            ImportableVisibility::Unused,
        ),
    );
    names.insert(name.id, Res::Item(import_item_id, ItemStatus::Available));
    Ok(())
}

/// Tries to extract a field name from an expression in cases where it is syntactically ambiguous
/// whether the expression is a field name or a variable name. This applies to the index operand in
/// a ternary update operator.
pub(super) fn extract_field_name<'a>(names: &Names, expr: &'a ast::Expr) -> Option<&'a Rc<str>> {
    // Follow the same reasoning as `is_field_update`.
    match &*expr.kind {
        ast::ExprKind::Path(PathKind::Ok(path))
            if path.segments.is_none() && !matches!(names.get(path.id), Some(Res::Local(_))) =>
        {
            Some(&path.name.name)
        }
        _ => None,
    }
}

fn is_field_update<'a>(
    globals: &GlobalScope,
    scopes: impl Iterator<Item = &'a Scope>,
    index: &ast::Expr,
) -> bool {
    // Disambiguate the update operator by looking at the index expression. If it's an
    // unqualified path that doesn't resolve to a local, assume that it's meant to be a field name.
    match &*index.kind {
        ast::ExprKind::Path(PathKind::Ok(path)) if path.segments.is_none() => !matches!(
            {
                let name = &path.name;
                let namespace = &path.segments;
                resolve(NameKind::Term, globals, scopes, name, namespace.as_deref())
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
    namespace: NamespaceId,
    mut next_id: impl FnMut() -> ItemId,
    item: &ast::Item,
) -> Result<(), Vec<Error>> {
    match &*item.kind {
        ast::ItemKind::Callable(decl) => {
            bind_callable(decl, namespace, next_id, item, names, scope)
        }
        ast::ItemKind::Ty(name, _) => bind_ty(name, namespace, next_id(), item, names, scope),
        ast::ItemKind::Struct(decl) => {
            bind_ty(&decl.name, namespace, next_id(), item, names, scope)
        }
        ast::ItemKind::ImportOrExport(decl) => {
            assign_item_id(names, next_id, decl);
            Ok(())
        }
        ast::ItemKind::Err | ast::ItemKind::Open(..) => Ok(()),
    }
}

fn assign_item_id(
    names: &mut IndexMap<NodeId, Res>,
    mut next_id: impl FnMut() -> ItemId,
    decl: &ImportOrExportDecl,
) {
    for item in iter_valid_items(decl) {
        if let ImportKind::Wildcard = item.kind {
            // don't bind any names for glob imports
            continue;
        }
        // Generate a new Item ID we can assign to this import, but don't bind the name yet
        names.insert(item.name().id, Res::Item(next_id(), ItemStatus::Available));
    }
}

fn bind_callable(
    decl: &CallableDecl,
    namespace: NamespaceId,
    next_id: impl FnOnce() -> ItemId,
    item: &ast::Item,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> Result<(), Vec<Error>> {
    let attrs = ast_attrs_as_hir_attrs(item.attrs.as_ref());
    let is_intrinsic = decl_is_intrinsic(decl, &attrs);
    let status = ItemStatus::from_attrs(&attrs);
    let item_id = next_id();

    let mut errs = vec![];
    if let Err(err) = bind_callable_name(&decl.name, namespace, item_id, status, names, scope) {
        errs.push(err);
    }

    if is_intrinsic && !scope.intrinsics.insert(Rc::clone(&decl.name.name)) {
        errs.push(Error::DuplicateIntrinsic(
            decl.name.name.to_string(),
            decl.name.span,
        ));
    }

    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

fn bind_callable_name(
    name: &Ident,
    namespace: NamespaceId,
    item_id: ItemId,
    status: ItemStatus,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> Result<(), Error> {
    let res = Res::Item(item_id, status);
    names.insert(name.id, res.clone());

    match (
        scope
            .terms
            .get_mut_or_default(namespace)
            .entry(Rc::clone(&name.name)),
        scope
            .importables
            .get_mut_or_default(namespace)
            .entry(Rc::clone(&name.name)),
    ) {
        (Entry::Occupied(_), _) | (_, Entry::Occupied(_)) => {
            let namespace_name = scope
                .namespaces
                .find_namespace_by_id(&namespace)
                .0
                .join(".");
            Err(Error::Duplicate(
                name.name.to_string(),
                namespace_name.to_string(),
                name.span,
            ))
        }
        (Entry::Vacant(term_entry), Entry::Vacant(importable_entry)) => {
            term_entry.insert(res);
            importable_entry.insert(Res::Importable(
                ImportableItemKind::Callable(item_id, status),
                ImportableVisibility::Unused,
            ));
            Ok(())
        }
    }
}

fn bind_callable_export(
    name: &Ident,
    namespace: NamespaceId,
    export_item_id: ItemId,
    original_item_id: ItemId,
    status: ItemStatus,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> Result<(), Error> {
    names.insert(name.id, Res::Item(export_item_id, status));

    match scope
        .importables
        .get_mut_or_default(namespace)
        .entry(Rc::clone(&name.name))
    {
        Entry::Occupied(occupied_entry) => {
            if occupied_entry.get().item_id() == Some(original_item_id) {
                if let Some(existing_span) = scope
                    .self_exported_item_ids
                    .insert(original_item_id, name.span)
                {
                    Err(Error::DuplicateExport {
                        name: name.name.to_string(),
                        span: name.span,
                        existing_span,
                    })
                } else {
                    // This is just an `import` or `export` of the item with the same name
                    // from the same namespace, no need to bind a new name here.
                    Ok(())
                }
            } else {
                Err(Error::Duplicate(
                    name.name.to_string(),
                    scope
                        .namespaces
                        .find_namespace_by_id(&namespace)
                        .0
                        .join(".")
                        .to_string(),
                    name.span,
                ))
            }
        }
        Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(Res::Importable(
                ImportableItemKind::Callable(original_item_id, status),
                ImportableVisibility::Unused,
            ));
            scope
                .terms
                .get_mut_or_default(namespace)
                .insert(Rc::clone(&name.name), Res::Item(original_item_id, status));
            Ok(())
        }
    }
}

fn bind_ty(
    name: &Ident,
    namespace: NamespaceId,
    item_id: ItemId,
    item: &ast::Item,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> Result<(), Vec<Error>> {
    let status = ItemStatus::from_attrs(&ast_attrs_as_hir_attrs(item.attrs.as_ref()));
    let res_term = Res::Item(item_id, status);
    let res_ty = Res::Item(item_id, status);
    names.insert(name.id, res_ty.clone());
    match (
        scope
            .terms
            .get_mut_or_default(namespace)
            .entry(Rc::clone(&name.name)),
        scope
            .tys
            .get_mut_or_default(namespace)
            .entry(Rc::clone(&name.name)),
        scope
            .importables
            .get_mut_or_default(namespace)
            .entry(Rc::clone(&name.name)),
    ) {
        (Entry::Occupied(_), _, _) | (_, Entry::Occupied(_), _) | (_, _, Entry::Occupied(_)) => {
            let namespace_name = scope
                .namespaces
                .find_namespace_by_id(&namespace)
                .0
                .join(".");
            Err(vec![Error::Duplicate(
                name.name.to_string(),
                namespace_name,
                name.span,
            )])
        }
        (Entry::Vacant(term_entry), Entry::Vacant(ty_entry), Entry::Vacant(importable_entry)) => {
            term_entry.insert(res_term);
            ty_entry.insert(res_ty);
            importable_entry.insert(Res::Importable(
                ImportableItemKind::Ty(item_id, status),
                ImportableVisibility::Unused,
            ));
            Ok(())
        }
    }
}

fn bind_ty_export(
    name: &Ident,
    namespace: NamespaceId,
    import_item_id: ItemId,
    original_item_id: ItemId,
    status: ItemStatus,
    names: &mut IndexMap<NodeId, Res>,
    scope: &mut GlobalScope,
) -> Result<(), Error> {
    names.insert(name.id, Res::Item(import_item_id, status));

    match scope
        .importables
        .get_mut_or_default(namespace)
        .entry(Rc::clone(&name.name))
    {
        Entry::Occupied(occupied_entry) => {
            if occupied_entry.get().item_id() == Some(original_item_id) {
                // Found the item we're importing, e.g.:
                //
                // struct Foo {}
                // export Foo;
                //
                if let Some(existing_span) = scope
                    .self_exported_item_ids
                    .insert(original_item_id, name.span)
                {
                    Err(Error::DuplicateExport {
                        name: name.name.to_string(),
                        span: name.span,
                        existing_span,
                    })
                } else {
                    // This is not a collision. We also don't need to
                    // bind this name, since it's already bound to the original
                    // declaration.
                    Ok(())
                }
            } else {
                Err(Error::Duplicate(
                    name.name.to_string(),
                    scope
                        .namespaces
                        .find_namespace_by_id(&namespace)
                        .0
                        .join(".")
                        .to_string(),
                    name.span,
                ))
            }
        }
        Entry::Vacant(vacant_entry) => {
            vacant_entry.insert(Res::Importable(
                ImportableItemKind::Ty(original_item_id, status),
                ImportableVisibility::Unused,
            ));

            scope
                .terms
                .get_mut_or_default(namespace)
                .insert(Rc::clone(&name.name), Res::Item(original_item_id, status));
            scope
                .tys
                .get_mut_or_default(namespace)
                .insert(Rc::clone(&name.name), Res::Item(original_item_id, status));
            Ok(())
        }
    }
}

fn decl_is_intrinsic(decl: &CallableDecl, attrs: &[hir::Attr]) -> bool {
    if attrs
        .iter()
        .any(|attr| matches!(attr, hir::Attr::SimulatableIntrinsic))
    {
        return true;
    }
    if let CallableBody::Specs(specs) = decl.body.as_ref() {
        specs
            .iter()
            .any(|spec| matches!(spec.body, SpecBody::Gen(SpecGen::Intrinsic)))
    } else {
        false
    }
}

/// Resolves a given symbol and namespace name, according to the Q# shadowing rules.
/// Shadowing rules are as follows:
/// - Local variables shadow everything. They are the first priority.
/// - Next, we check open statements for a non-prelude open.
/// - Then, we check the prelude.
/// - Lastly, we check the global namespace.
///
/// In the example `Foo.Bar.Baz()` -- the `provided_namespace_name` would be
///`Foo.Bar` and the `provided_symbol_name` would be `Baz`.
///
/// In the example `Foo()` -- the `provided_namespace_name` would be `None` and the
/// `provided_symbol_name` would be `Foo`.
/// returns the resolution if successful, or an error if not.
fn resolve<'a>(
    kind: NameKind,
    globals: &GlobalScope,
    scopes: impl Iterator<Item = &'a Scope>,
    provided_symbol_name: &Ident,
    provided_namespace_name: Option<&[Ident]>,
) -> Result<Res, Error> {
    // eprintln!(
    //     "resolving name {} with namekind {:?}",
    //     provided_symbol_name.name, kind
    // );
    if let Some(value) = check_all_scopes(
        kind,
        globals,
        provided_symbol_name,
        provided_namespace_name,
        scopes,
    ) {
        // eprintln!("found in a scope");
        return value;
    }

    // check the prelude
    if provided_namespace_name.is_none() {
        let prelude_candidates = find_symbol_in_namespaces(
            kind,
            globals,
            provided_namespace_name,
            provided_symbol_name,
            prelude_namespaces(globals).into_iter(),
            // prelude is opened by default
            &(std::iter::once((vec![], prelude_namespaces(globals))).collect()),
        );

        if prelude_candidates.len() > 1 {
            // If there are multiple candidates, sort them by namespace and return an error.
            let candidates: Vec<_> = prelude_candidates.into_iter().collect();
            let mut candidates = candidates
                .into_iter()
                .map(|(_candidate, ns_name)| ns_name)
                .collect::<Vec<_>>();
            candidates.sort();

            let mut candidates = candidates.into_iter();
            let candidate_a = candidates
                .next()
                .expect("infallible as per length check above");
            let candidate_b = candidates
                .next()
                .expect("infallible as per length check above");
            return Err(Error::AmbiguousPrelude {
                span: provided_symbol_name.span,
                name: provided_symbol_name.name.to_string(),
                candidate_a,
                candidate_b,
            });
        }
        // if there is a candidate, return it
        if let Some((res, _)) = single(prelude_candidates) {
            return Ok(res);
        }
    }

    // Lastly, check unopened globals. This is anything declared in the root namespace, which is
    // therefore globally available to the package
    let global_candidates = find_symbol_in_namespaces(
        kind,
        globals,
        provided_namespace_name,
        provided_symbol_name,
        std::iter::once((globals.namespaces.root_id(), ())),
        // there are no aliases in globals
        &FxHashMap::default(),
    );
    // eprintln!("  found {} global candidates", global_candidates.len());

    // we don't have to throw an error if there are extra candidates here, as we are only looking at the root,
    // and that's only one namespace. individual namespaces cannot have duplicate declarations.
    if let Some(res) = single(global_candidates.into_keys()) {
        return Ok(res);
    }

    Err(match provided_namespace_name {
        Some(ns) => {
            let full_name = (ns, provided_symbol_name);
            Error::NotFound(full_name.full_name().to_string(), full_name.full_span())
        }
        None => Error::NotFound(
            provided_symbol_name.name.to_string(),
            provided_symbol_name.span,
        ),
    })
}
/// Checks all given scopes, in the correct order, for a resolution.
/// Calls `check_scoped_resolutions` on each scope, and tracks if we should allow local variables in closures in parent scopes
/// using the `vars` parameter.
fn check_all_scopes<'a>(
    kind: NameKind,
    globals: &GlobalScope,
    provided_symbol_name: &Ident,
    provided_namespace_name: Option<&[Ident]>,
    scopes: impl Iterator<Item = &'a Scope>,
) -> Option<Result<Res, Error>> {
    let mut vars = true;

    for scope in scopes {
        if let Some(value) = check_scoped_resolutions(
            kind,
            globals,
            provided_symbol_name,
            provided_namespace_name,
            &mut vars,
            scope,
        ) {
            // eprintln!("found in scope: {:?}", scope);
            return Some(value);
        }
    }
    None
}

/// This function checks scopes for a given symbol and namespace name.
/// In a given [Scope], check:
/// 1. if any locally declared symbols match `provided_symbol_name`
/// 2. if any aliases in this scope match the provided namespace, and if they contain `provided_symbol_name`
/// 3. if any opens in this scope contain the `provided_symbol_name`
///
/// It follows the Q# shadowing rules:
/// - Local variables shadow everything. They are the first priority.
/// - Next, we check open statements for an explicit open.
/// - Then, we check the prelude.
/// - Lastly, we check the global namespace.
///
/// # Parameters
///
/// * `kind` - The [`NameKind`] of the name
/// * `globals` - The global scope to resolve the name against.
/// * `provided_symbol_name` - The symbol name to resolve.
/// * `provided_namespace_name` - The namespace of the symbol, if any.
/// * `vars` - A mutable reference to a boolean indicating whether to allow local variables in closures in parent scopes.
/// * `scope` - The scope to check for resolutions.
fn check_scoped_resolutions(
    kind: NameKind,
    globals: &GlobalScope,
    provided_symbol_name: &Ident,
    provided_namespace_name: Option<&[Ident]>,
    vars: &mut bool,
    scope: &Scope,
) -> Option<Result<Res, Error>> {
    if provided_namespace_name.is_none() {
        if let Some(res) =
            resolve_scope_locals(kind, globals, scope, *vars, &provided_symbol_name.name)
        {
            // eprintln!("found as a scope local");
            // Local declarations shadow everything.
            return Some(Ok(res));
        }
    }

    let aliases = scope
        .opens
        .iter()
        .map(|(alias, opens)| {
            (
                alias.clone(),
                opens.iter().map(|x| (x.namespace, x)).collect(),
            )
        })
        .collect::<FxHashMap<_, _>>();

    let namespaces_to_search = scope
        .opens
        .iter()
        .flat_map(|(_, open)| open)
        .map(|open @ Open { namespace, .. }| (*namespace, open));

    // eprintln!(
    //     "aliases: {:?}, namespaces_to_search: {:?}",
    //     aliases, namespaces_to_search
    // );
    let explicit_open_candidates = find_symbol_in_namespaces(
        kind,
        globals,
        provided_namespace_name,
        provided_symbol_name,
        namespaces_to_search,
        &aliases,
    );

    match explicit_open_candidates.len() {
        1 => {
            // eprintln!("found in explicit open");
            return Some(Ok(single(explicit_open_candidates.into_keys())
                .expect("we asserted on the length, so this is infallible")));
        }
        len if len > 1 => {
            return Some(Err(ambiguous_symbol_error(
                globals,
                provided_symbol_name,
                explicit_open_candidates.into_values().collect(),
            )));
        }
        _ => (),
    }
    if scope.kind == ScopeKind::Callable {
        // Since local callables are not closures, hide local variables in parent scopes.
        *vars = false;
    }
    None
}

/// This function returns type `Error::Ambiguous` and contains
/// the name of the ambiguous symbol and the namespaces that contain the conflicting entities.
/// # Arguments
///
/// * `globals` - The global scope to resolve the name against.
/// * `provided_symbol_name` - The symbol name that is ambiguous.
/// * `candidates` - A map of possible resolutions for the symbol, each associated with the `Open`
///   statement that brought it into scope. Note that only the first two opens in
///   the candidates are actually used in the error message.
fn ambiguous_symbol_error(
    globals: &GlobalScope,
    provided_symbol_name: &Ident,
    mut opens: Vec<&Open>,
) -> Error {
    opens.sort_unstable_by_key(|open| open.span);
    let (first_open_ns, _) = globals.namespaces.find_namespace_by_id(&opens[0].namespace);
    let (second_open_ns, _) = globals.namespaces.find_namespace_by_id(&opens[1].namespace);
    Error::Ambiguous {
        name: provided_symbol_name.name.to_string(),
        first_open: first_open_ns.join("."),
        second_open: second_open_ns.join("."),
        name_span: provided_symbol_name.span,
        first_open_span: opens[0].span,
        second_open_span: opens[1].span,
    }
}

fn find_symbol_in_namespaces<T, O>(
    kind: NameKind,
    globals: &GlobalScope,
    provided_namespace_name: Option<&[Ident]>,
    provided_symbol_name: &Ident,
    namespaces_to_search: T,
    aliases: &FxHashMap<Vec<Rc<str>>, Vec<(NamespaceId, O)>>,
) -> FxHashMap<Res, O>
where
    T: Iterator<Item = (NamespaceId, O)>,
    O: Clone + std::fmt::Debug,
{
    let opens = match provided_namespace_name {
        None => aliases.get(&Vec::new()),
        Some(namespace_name) => aliases.get(
            &namespace_name
                .iter()
                .next()
                .map(|x| vec![x.name.clone()])
                .unwrap_or_default(),
        ),
    };
    // eprintln!("provided namespace name: {provided_namespace_name:?}");
    // eprintln!("opens: {opens:?}");

    let mut candidates = FxHashMap::default();
    if let Some(opens) = opens {
        for open in opens {
            find_symbol_in_namespace(
                kind,
                globals,
                provided_namespace_name.as_ref().map(|x| &x[1..]),
                provided_symbol_name,
                &mut candidates,
                open.0,
                open.1.clone(),
            );
        }
        // check aliases to see if the provided namespace is actually an alias
        if provided_namespace_name.is_none() {
            candidates.extend(&mut opens.iter().filter_map(|(ns_id, open)| {
                globals
                    .get(kind, *ns_id, &provided_symbol_name.name)
                    .map(|res| (res.clone(), open.clone()))
            }));
        }
    }

    for (candidate_namespace_id, open) in namespaces_to_search {
        // eprintln!("searching in namespace {candidate_namespace_id} with open {open:?}");
        // let nnn = globals
        //     .namespaces
        //     .find_namespace_by_id(&candidate_namespace_id)
        //     .0
        //     .join(".");
        // eprintln!("  searching in namespace {nnn}");
        find_symbol_in_namespace(
            kind,
            globals,
            provided_namespace_name,
            provided_symbol_name,
            &mut candidates,
            candidate_namespace_id,
            open,
        );
    }

    if candidates.len() > 1 {
        // If there are multiple candidates, remove unimplemented items. This allows resolution to
        // succeed in cases where both an older, unimplemented API and newer, implemented API with the
        // same name are both in scope without forcing the user to fully qualify the name.
        candidates.retain(|res, _| !matches!(res, &Res::Item(_, ItemStatus::Unimplemented)));
    }
    candidates
}

/// returns `true` if the namespace should be skipped/is incorrect, so the caller can
/// iterate to the next namespace.
fn find_symbol_in_namespace<O>(
    kind: NameKind,
    globals: &GlobalScope,
    provided_namespace_name: Option<&[Ident]>,
    provided_symbol_name: &Ident,
    candidates: &mut FxHashMap<Res, O>,
    candidate_namespace_id: NamespaceId,
    open: O,
) where
    O: Clone + std::fmt::Debug,
{
    // eprintln!(
    //     "  attempting to find under {candidate_namespace_id} ({open:?}) : {}.{}",
    //     provided_namespace_name.map_or_else(Default::default, |ns| ast::Idents::full_name(&ns)),
    //     provided_symbol_name.name
    // );
    // Retrieve the namespace associated with the candidate_namespace_id from the global namespaces
    let (_, candidate_namespace) = globals
        .namespaces
        .find_namespace_by_id(&candidate_namespace_id);

    // Attempt to find a namespace within the candidate_namespace that matches the provided_namespace_name
    let namespace = provided_namespace_name.as_ref().and_then(|name| {
        candidate_namespace
            .borrow()
            .get_namespace_id(name.str_iter())
    });
    // eprintln!("    provided namespace was found to have id {namespace:?}");

    // if a namespace was provided, but not found, then this is not the correct namespace.
    // for example, if the query is `Foo.Bar.Baz`, we know there must exist a `Foo.Bar` somewhere.
    // If we didn't find it above, then even if we find `Baz` here, it is not the correct location.
    if provided_namespace_name.is_some() && namespace.is_none() {
        return;
    }

    // Attempt to get the symbol from the global scope.
    let namespace = namespace.unwrap_or(globals.namespaces.root_id());
    let res = globals.get(kind, namespace, &provided_symbol_name.name);

    // If a symbol was found, insert it into the candidates map
    if let Some(res) = res {
        candidates.insert(res.clone(), open);
    }
}

/// Fetch the name and namespace ID of all prelude namespaces.
pub fn prelude_namespaces(globals: &GlobalScope) -> Vec<(NamespaceId, String)> {
    let mut prelude = Vec::with_capacity(PRELUDE.len());

    // add prelude to the list of candidate namespaces last, as they are the final fallback for a symbol
    for prelude_namespace in PRELUDE {
        // when evaluating the prelude namespaces themselves, they won't have been created yet. So we only
        // include the ids that have been created.
        if let Some(id) = globals
            .namespaces
            .get_namespace_id(prelude_namespace.to_vec())
        {
            prelude.push((id, prelude_namespace.join(".")));
        }
    }
    prelude
}
/// Implements shadowing rules within a single scope.
/// A local variable always wins out against an item with the same name, even if they're declared in
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
                if let Some(&(_, id)) = scope.vars.get(name) {
                    return Some(Res::Local(id));
                }
            }
            NameKind::Ty => {
                if let Some((id, bounds)) = scope.ty_vars.get(name) {
                    return Some(Res::Param {
                        id: *id,
                        bounds: bounds.clone(),
                    });
                }
            }
            NameKind::Importable => {}
        }
    }

    if let Some(res) = scope.item(kind, name) {
        return Some(res);
    }

    if let ScopeKind::Namespace(namespace) = &scope.kind {
        // TODO: Is this really needed? Don't we normally already look at globals?
        if let Some(res) = globals.get(kind, *namespace, name) {
            return Some(res.clone());
        }
    }

    None
}

fn get_scope_locals(scope: &Scope, offset: u32, vars: bool) -> Vec<Local> {
    let mut names = Vec::new();

    // variables
    if vars {
        names.extend(scope.vars.iter().filter_map(|(name, (valid_at, id))| {
            // Bug: Because we keep track of only one `valid_at` offset per name,
            // when a variable is later shadowed in the same scope,
            // it is missed in the list. https://github.com/microsoft/qsharp/issues/897
            if offset >= *valid_at {
                Some(Local {
                    name: name.clone(),
                    kind: LocalKind::Var(*id),
                })
            } else {
                None
            }
        }));

        names.extend(
            scope
                .ty_vars
                .iter()
                .map(|(name, (id, _constraints))| Local {
                    name: name.clone(),
                    kind: LocalKind::TyParam(*id),
                }),
        );
    }

    // items
    // skip adding newtypes since they're already in the terms map
    names.extend(scope.terms.iter().filter_map(|(name, entry)| {
        if entry.source == ItemSource::Declared {
            Some(Local {
                name: name.clone(),
                kind: LocalKind::Item(entry.id),
            })
        } else {
            // Exclude imports as they are not local items
            None
        }
    }));

    names
}

/// Creates an [`ItemId`] for an item that is local to this package (internal to it).
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
