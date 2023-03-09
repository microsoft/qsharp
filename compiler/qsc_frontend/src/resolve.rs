// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::compile::PackageId;
use qsc_ast::{
    ast::{
        Block, CallableDecl, Expr, ExprKind, Item, ItemKind, Namespace, NodeId, Pat, PatKind, Path,
        Span, SpecBody, SpecDecl, Stmt, StmtKind, Ty, TyKind, VisibilityKind,
    },
    visit::{self, Visitor},
};
use std::collections::{HashMap, HashSet};

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

#[derive(Debug)]
pub(super) struct Error {
    pub(super) span: Span,
    pub(super) kind: ErrorKind,
}

#[derive(Debug)]
pub(super) enum ErrorKind {
    Unresolved(HashSet<DefId>),
}

pub(super) struct Resolver<'a> {
    resolutions: Resolutions,
    tys: HashMap<&'a str, HashMap<&'a str, DefId>>,
    terms: HashMap<&'a str, HashMap<&'a str, DefId>>,
    opens: HashMap<&'a str, HashSet<&'a str>>,
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
                self.opens.entry(alias).or_default().insert(&name.name);
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
                    .expect("Statement should have an environment.");
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
    opens: &HashMap<&str, HashSet<&str>>,
    parent: &str,
    locals: &[HashMap<&str, DefId>],
    path: &Path,
) -> Result<DefId, Error> {
    let name = path.name.name.as_str();
    if path.namespace.is_none() {
        if let Some(&id) = locals.iter().rev().find_map(|env| env.get(name)) {
            // Locals shadow everything.
            return Ok(id);
        } else if let Some(&id) = globals.get(parent).and_then(|ns| ns.get(name)) {
            // Items in the parent namespace shadow opens.
            return Ok(id);
        }
    }

    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    let mut candidates = HashSet::new();
    if let Some(namespaces) = opens.get(namespace) {
        for namespace in namespaces {
            if let Some(&id) = globals.get(namespace).and_then(|ns| ns.get(name)) {
                // Opens shadow unopened globals.
                candidates.insert(id);
            }
        }
    }

    if candidates.is_empty() {
        if let Some(&id) = globals.get(namespace).and_then(|ns| ns.get(name)) {
            // An unopened global is the last resort.
            return Ok(id);
        }
    }

    if candidates.len() == 1 {
        Ok(candidates
            .into_iter()
            .next()
            .expect("Candidates should not be empty."))
    } else {
        Err(Error {
            span: path.span,
            kind: ErrorKind::Unresolved(candidates),
        })
    }
}
