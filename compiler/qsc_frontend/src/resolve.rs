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

pub type Resolutions = HashMap<NodeId, Res>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Res {
    pub package: PackageRes,
    pub node: NodeId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackageRes {
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
    Unresolved(HashSet<Res>),
}

pub(super) struct Resolver<'a> {
    resolutions: Resolutions,
    global_tys: HashMap<&'a str, HashMap<&'a str, Res>>,
    global_terms: HashMap<&'a str, HashMap<&'a str, Res>>,
    opens: HashMap<&'a str, HashSet<&'a str>>,
    locals: Vec<HashMap<&'a str, Res>>,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub(super) fn into_resolutions(self) -> (Resolutions, Vec<Error>) {
        (self.resolutions, self.errors)
    }

    fn resolve_ty(&mut self, path: &Path) {
        match resolve(&self.global_tys, &self.opens, &[], path) {
            Ok(res) => {
                self.resolutions.insert(path.id, res);
            }
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &Path) {
        match resolve(&self.global_terms, &self.opens, &self.locals, path) {
            Ok(res) => {
                self.resolutions.insert(path.id, res);
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
        self.opens = HashMap::from([("", HashSet::from([namespace.name.name.as_str()]))]);
        for item in &namespace.items {
            if let ItemKind::Open(namespace, alias) = &item.kind {
                let alias = alias.as_ref().map_or("", |a| &a.name);
                self.opens.entry(alias).or_default().insert(&namespace.name);
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
    tys: HashMap<&'a str, HashMap<&'a str, Res>>,
    terms: HashMap<&'a str, HashMap<&'a str, Res>>,
    package: PackageRes,
    namespace: &'a str,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new() -> Self {
        Self {
            resolutions: Resolutions::new(),
            tys: HashMap::new(),
            terms: HashMap::new(),
            package: PackageRes::Local,
            namespace: "",
        }
    }

    pub(super) fn set_package(&mut self, package: PackageId) {
        self.package = PackageRes::Extern(package);
    }

    pub(super) fn into_resolver(self) -> Resolver<'a> {
        Resolver {
            resolutions: self.resolutions,
            global_tys: self.tys,
            global_terms: self.terms,
            opens: HashMap::new(),
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
        if self.package != PackageRes::Local && visibility == Some(VisibilityKind::Internal) {
            return;
        }

        match &item.kind {
            ItemKind::Ty(name, _) => {
                let res = Res {
                    package: self.package,
                    node: name.id,
                };
                if self.package == PackageRes::Local {
                    self.resolutions.insert(name.id, res);
                }
                self.tys
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, res);
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, res);
            }
            ItemKind::Callable(decl) => {
                let res = Res {
                    package: self.package,
                    node: decl.name.id,
                };
                if self.package == PackageRes::Local {
                    self.resolutions.insert(decl.name.id, res);
                }
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&decl.name.name, res);
            }
            ItemKind::Open(..) => {}
        }
    }
}

fn bind<'a>(resolutions: &mut Resolutions, env: &mut HashMap<&'a str, Res>, pat: &'a Pat) {
    match &pat.kind {
        PatKind::Bind(name, _) => {
            let res = Res {
                package: PackageRes::Local,
                node: name.id,
            };
            resolutions.insert(name.id, res);
            env.insert(name.name.as_str(), res);
        }
        PatKind::Discard(_) | PatKind::Elided => {}
        PatKind::Paren(pat) => bind(resolutions, env, pat),
        PatKind::Tuple(pats) => pats.iter().for_each(|p| bind(resolutions, env, p)),
    }
}

fn resolve(
    globals: &HashMap<&str, HashMap<&str, Res>>,
    opens: &HashMap<&str, HashSet<&str>>,
    locals: &[HashMap<&str, Res>],
    path: &Path,
) -> Result<Res, Error> {
    if path.namespace.is_none() {
        for env in locals.iter().rev() {
            if let Some(&id) = env.get(path.name.name.as_str()) {
                return Ok(id);
            }
        }
    }

    let namespace = path.namespace.as_ref().map_or("", |i| &i.name);
    let name = path.name.name.as_str();
    let mut candidates = HashSet::new();
    if let Some(&res) = globals.get(namespace).and_then(|n| n.get(name)) {
        candidates.insert(res);
    }

    if let Some(namespaces) = opens.get(namespace) {
        for namespace in namespaces {
            if let Some(&res) = globals.get(namespace).and_then(|n| n.get(name)) {
                candidates.insert(res);
            }
        }
    }

    if candidates.len() == 1 {
        Ok(candidates
            .into_iter()
            .next()
            .expect("Set should have exactly one item."))
    } else {
        Err(Error {
            span: path.span,
            kind: ErrorKind::Unresolved(candidates),
        })
    }
}
