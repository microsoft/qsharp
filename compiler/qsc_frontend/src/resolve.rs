// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::compile::PackageId;
use miette::Diagnostic;
use qsc_ast::{
    ast::{
        Block, CallableDecl, Expr, ExprKind, Item, ItemKind, Namespace, NodeId, Pat, PatKind, Path,
        Span, SpecBody, SpecDecl, Stmt, StmtKind, Ty, TyKind, VisibilityKind,
    },
    visit::{self, Visitor},
};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

const PRELUDE: &[&str] = &[
    "Microsoft.Quantum.Canon",
    "Microsoft.Quantum.Core",
    "Microsoft.Quantum.Intrinsic",
];

pub type Resolutions = HashMap<NodeId, DefId>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DefId {
    pub package: PackageSrc,
    pub node: NodeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackageSrc {
    Local,
    Extern(PackageId),
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
    locals: Vec<HashMap<&'a str, DefId>>,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub(super) fn into_resolutions(self) -> (Resolutions, Vec<Error>) {
        (self.resolutions, self.errors)
    }

    fn resolve_ty(&mut self, path: &Path) {
        match resolve(&self.tys, &self.opens, self.namespace, &[], path) {
            Ok(id) => {
                self.resolutions.insert(path.id, id);
            }
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &Path) {
        match resolve(&self.terms, &self.opens, self.namespace, &self.locals, path) {
            Ok(id) => {
                self.resolutions.insert(path.id, id);
            }
            Err(err) => self.errors.push(err),
        }
    }

    fn with_scope(&mut self, pat: Option<&'a Pat>, f: impl FnOnce(&mut Self)) {
        let mut env = HashMap::new();
        pat.into_iter()
            .for_each(|p| bind(&mut self.resolutions, &mut env, p));
        self.locals.push(env);
        f(self);
        self.locals.pop();
    }
}

impl<'a> Visitor<'a> for Resolver<'a> {
    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.opens = HashMap::new();
        self.namespace = &namespace.name.name;
        for item in &namespace.items {
            if let ItemKind::Open(name, alias) = &item.kind {
                let alias = alias.as_ref().map_or("", |a| &a.name);
                self.opens
                    .entry(alias)
                    .or_default()
                    .insert(&name.name, name.span);
            }
        }

        visit::walk_namespace(self, namespace);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        self.with_scope(Some(&decl.input), |resolver| {
            visit::walk_callable_decl(resolver, decl);
        });
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        if let SpecBody::Impl(input, block) = &decl.body {
            self.with_scope(Some(input), |resolver| resolver.visit_block(block));
        } else {
            visit::walk_spec_decl(self, decl);
        }
    }

    fn visit_ty(&mut self, ty: &'a Ty) {
        if let TyKind::Path(path) = &ty.kind {
            self.resolve_ty(path);
        } else {
            visit::walk_ty(self, ty);
        }
    }

    fn visit_block(&mut self, block: &'a Block) {
        self.with_scope(None, |resolver| visit::walk_block(resolver, block));
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        visit::walk_stmt(self, stmt);

        match &stmt.kind {
            StmtKind::Local(_, pat, _) | StmtKind::Qubit(_, pat, _, _) => {
                let env = self
                    .locals
                    .last_mut()
                    .expect("parent block of statement should have added environment");
                bind(&mut self.resolutions, env, pat);
            }
            StmtKind::Expr(..) | StmtKind::Semi(..) => {}
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::For(pat, iter, block) => {
                self.visit_expr(iter);
                self.with_scope(Some(pat), |resolver| resolver.visit_block(block));
            }
            ExprKind::Lambda(_, input, output) => {
                self.with_scope(Some(input), |resolver| resolver.visit_expr(output));
            }
            ExprKind::Path(path) => self.resolve_term(path),
            _ => visit::walk_expr(self, expr),
        }
    }
}

pub(super) struct GlobalTable<'a> {
    resolutions: Resolutions,
    tys: HashMap<&'a str, HashMap<&'a str, DefId>>,
    terms: HashMap<&'a str, HashMap<&'a str, DefId>>,
    package: PackageSrc,
    namespace: &'a str,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new() -> Self {
        Self {
            resolutions: Resolutions::new(),
            tys: HashMap::new(),
            terms: HashMap::new(),
            package: PackageSrc::Local,
            namespace: "",
        }
    }

    pub(super) fn set_package(&mut self, package: PackageId) {
        self.package = PackageSrc::Extern(package);
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

impl<'a> Visitor<'a> for GlobalTable<'a> {
    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.namespace = &namespace.name.name;
        visit::walk_namespace(self, namespace);
        self.namespace = "";
    }

    fn visit_item(&mut self, item: &'a Item) {
        let visibility = item.meta.visibility.map(|v| v.kind);
        if self.package != PackageSrc::Local && visibility == Some(VisibilityKind::Internal) {
            return;
        }

        match &item.kind {
            ItemKind::Ty(name, _) => {
                let id = DefId {
                    package: self.package,
                    node: name.id,
                };
                if self.package == PackageSrc::Local {
                    self.resolutions.insert(name.id, id);
                }
                self.tys
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, id);
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, id);
            }
            ItemKind::Callable(decl) => {
                let id = DefId {
                    package: self.package,
                    node: decl.name.id,
                };
                if self.package == PackageSrc::Local {
                    self.resolutions.insert(decl.name.id, id);
                }
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&decl.name.name, id);
            }
            ItemKind::Open(..) => {}
        }
    }
}

fn bind<'a>(resolutions: &mut Resolutions, env: &mut HashMap<&'a str, DefId>, pat: &'a Pat) {
    match &pat.kind {
        PatKind::Bind(name, _) => {
            let id = DefId {
                package: PackageSrc::Local,
                node: name.id,
            };
            resolutions.insert(name.id, id);
            env.insert(name.name.as_str(), id);
        }
        PatKind::Discard(_) | PatKind::Elided => {}
        PatKind::Paren(pat) => bind(resolutions, env, pat),
        PatKind::Tuple(pats) => pats.iter().for_each(|p| bind(resolutions, env, p)),
    }
}

fn resolve(
    globals: &HashMap<&str, HashMap<&str, DefId>>,
    opens: &HashMap<&str, HashMap<&str, Span>>,
    parent: &str,
    locals: &[HashMap<&str, DefId>],
    path: &Path,
) -> Result<DefId, Error> {
    let name = path.name.name.as_str();
    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    if namespace.is_empty() {
        if let Some(&id) = locals.iter().rev().find_map(|env| env.get(name)) {
            // Locals shadow everything.
            return Ok(id);
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
