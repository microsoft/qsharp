// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_ast::{
    ast::{
        Block, CallableDecl, Expr, ExprKind, Item, ItemKind, Namespace, NodeId, Pat, PatKind, Path,
        Span, SpecBody, SpecDecl, Stmt, StmtKind, Ty, TyKind,
    },
    visit::{self, Visitor},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DefId {
    package: PackageId,
    node: NodeId,
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

#[derive(Debug)]
pub struct Table(HashMap<NodeId, DefId>);

impl Table {
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<DefId> {
        self.0.get(&node).copied()
    }

    pub fn resolves_to(&mut self, node: NodeId, def: DefId) {
        self.0.insert(node, def);
    }
}

pub(super) struct Resolver<'a> {
    table: Table,
    global_tys: HashMap<&'a str, HashMap<&'a str, DefId>>,
    global_terms: HashMap<&'a str, HashMap<&'a str, DefId>>,
    opens: HashMap<&'a str, HashSet<&'a str>>,
    locals: Vec<HashMap<&'a str, DefId>>,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub(super) fn into_table(self) -> (Table, Vec<Error>) {
        (self.table, self.errors)
    }

    fn resolve_ty(&mut self, path: &Path) {
        match resolve(&self.global_tys, &self.opens, &[], path) {
            Ok(def) => self.table.resolves_to(path.id, def),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &Path) {
        match resolve(&self.global_terms, &self.opens, &self.locals, path) {
            Ok(def) => self.table.resolves_to(path.id, def),
            Err(err) => self.errors.push(err),
        }
    }

    fn with_scope(&mut self, pat: Option<&'a Pat>, f: impl FnOnce(&mut Self)) {
        let mut env = HashMap::new();
        pat.into_iter().for_each(|p| bind(&mut env, p));
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
            StmtKind::Borrow(pat, _, _)
            | StmtKind::Let(pat, _)
            | StmtKind::Mutable(pat, _)
            | StmtKind::Use(pat, _, _) => {
                let env = self
                    .locals
                    .last_mut()
                    .expect("Statement should have an environment.");
                bind(env, pat);
            }
            StmtKind::Expr(_) | StmtKind::Semi(_) => {}
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
    symbols: Table,
    tys: HashMap<&'a str, HashMap<&'a str, DefId>>,
    terms: HashMap<&'a str, HashMap<&'a str, DefId>>,
    package: PackageId,
    namespace: &'a str,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new() -> Self {
        Self {
            symbols: Table(HashMap::new()),
            tys: HashMap::new(),
            terms: HashMap::new(),
            package: PackageId(0),
            namespace: "",
        }
    }

    pub(super) fn into_resolver(self) -> Resolver<'a> {
        Resolver {
            table: self.symbols,
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
        let def = DefId {
            package: self.package,
            node: item.id,
        };

        match &item.kind {
            ItemKind::Ty(name, _) => {
                self.tys
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, def);

                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&name.name, def);
            }
            ItemKind::Callable(decl) => {
                self.terms
                    .entry(self.namespace)
                    .or_default()
                    .insert(&decl.name.name, def);
            }
            ItemKind::Open(..) => {}
        }
    }
}

fn bind<'a>(env: &mut HashMap<&'a str, DefId>, pat: &'a Pat) {
    match &pat.kind {
        PatKind::Bind(name, _) => {
            let def = DefId {
                package: PackageId(0),
                node: name.id,
            };
            env.insert(name.name.as_str(), def);
        }
        PatKind::Discard(_) | PatKind::Elided => {}
        PatKind::Paren(pat) => bind(env, pat),
        PatKind::Tuple(pats) => pats.iter().for_each(|p| bind(env, p)),
    }
}

fn resolve(
    globals: &HashMap<&str, HashMap<&str, DefId>>,
    opens: &HashMap<&str, HashSet<&str>>,
    locals: &[HashMap<&str, DefId>],
    path: &Path,
) -> Result<DefId, Error> {
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
    if let Some(&def) = globals.get(namespace).and_then(|n| n.get(name)) {
        candidates.insert(def);
    }

    if let Some(namespaces) = opens.get(namespace) {
        for namespace in namespaces {
            if let Some(&def) = globals.get(namespace).and_then(|n| n.get(name)) {
                candidates.insert(def);
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
