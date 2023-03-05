// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{
        Block, CallableDecl, Expr, ExprKind, Item, ItemKind, Namespace, NodeId, Pat, PatKind, Path,
        Span, SpecBody, SpecDecl, Stmt, StmtKind, Ty, TyKind,
    },
    visit::{self, Visitor},
};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u32);

impl Id {
    fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
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

#[derive(Debug)]
pub struct Table {
    nodes: HashMap<NodeId, Id>,
    next_id: Id,
}

impl Table {
    #[must_use]
    pub fn get(&self, node: NodeId) -> Option<Id> {
        self.nodes.get(&node).copied()
    }

    pub fn declare(&mut self, node: NodeId) -> Id {
        let id = self.next_id;
        self.next_id = self.next_id.successor();
        self.nodes.insert(node, id);
        id
    }

    pub fn use_symbol(&mut self, node: NodeId, symbol: Id) {
        self.nodes.insert(node, symbol);
    }
}

pub(super) struct Resolver<'a> {
    table: Table,
    tys: HashMap<&'a str, HashMap<&'a str, Id>>,
    terms: HashMap<&'a str, HashMap<&'a str, Id>>,
    namespace: &'a str,
    opens: HashMap<&'a str, HashMap<&'a str, Span>>,
    locals: Vec<HashMap<&'a str, Id>>,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub(super) fn into_table(self) -> (Table, Vec<Error>) {
        (self.table, self.errors)
    }

    fn resolve_ty(&mut self, path: &Path) {
        match resolve(&self.tys, self.namespace, &self.opens, &[], path) {
            Ok(symbol) => self.table.use_symbol(path.id, symbol),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &Path) {
        match resolve(&self.terms, self.namespace, &self.opens, &self.locals, path) {
            Ok(symbol) => self.table.use_symbol(path.id, symbol),
            Err(err) => self.errors.push(err),
        }
    }

    fn with_scope(&mut self, pat: Option<&'a Pat>, f: impl FnOnce(&mut Self)) {
        let mut env = HashMap::new();
        pat.into_iter()
            .for_each(|p| bind(&mut self.table, &mut env, p));
        self.locals.push(env);
        f(self);
        self.locals.pop();
    }
}

impl<'a> Visitor<'a> for Resolver<'a> {
    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.namespace = &namespace.name.name;
        self.opens = HashMap::new();
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
            StmtKind::Borrow(pat, _, _)
            | StmtKind::Let(pat, _)
            | StmtKind::Mutable(pat, _)
            | StmtKind::Use(pat, _, _) => {
                let env = self
                    .locals
                    .last_mut()
                    .expect("Statement should have an environment.");
                bind(&mut self.table, env, pat);
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
    tys: HashMap<&'a str, HashMap<&'a str, Id>>,
    terms: HashMap<&'a str, HashMap<&'a str, Id>>,
    namespace: &'a str,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new() -> Self {
        Self {
            symbols: Table {
                nodes: HashMap::new(),
                next_id: Id(0),
            },
            tys: HashMap::new(),
            terms: HashMap::new(),
            namespace: "",
        }
    }

    pub(super) fn into_resolver(self) -> Resolver<'a> {
        Resolver {
            table: self.symbols,
            tys: self.tys,
            terms: self.terms,
            namespace: "",
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
        if let ItemKind::Ty(name, _) = &item.kind {
            let id = self.symbols.declare(name.id);
            self.tys
                .entry(self.namespace)
                .or_default()
                .insert(&name.name, id);
            self.terms
                .entry(self.namespace)
                .or_default()
                .insert(&name.name, id);
        } else {
            visit::walk_item(self, item);
        }
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        let id = self.symbols.declare(decl.name.id);
        self.terms
            .entry(self.namespace)
            .or_default()
            .insert(&decl.name.name, id);
    }
}

fn bind<'a>(table: &mut Table, env: &mut HashMap<&'a str, Id>, pat: &'a Pat) {
    match &pat.kind {
        PatKind::Bind(name, _) => {
            let id = table.declare(name.id);
            env.insert(name.name.as_str(), id);
        }
        PatKind::Discard(_) | PatKind::Elided => {}
        PatKind::Paren(pat) => bind(table, env, pat),
        PatKind::Tuple(pats) => pats.iter().for_each(|p| bind(table, env, p)),
    }
}

fn resolve(
    globals: &HashMap<&str, HashMap<&str, Id>>,
    parent: &str,
    opens: &HashMap<&str, HashMap<&str, Span>>,
    locals: &[HashMap<&str, Id>],
    path: &Path,
) -> Result<Id, Error> {
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
    let mut candidates = HashMap::new();
    if let Some(namespaces) = opens.get(namespace) {
        for (&namespace, &span) in namespaces {
            if let Some(&id) = globals.get(namespace).and_then(|ns| ns.get(name)) {
                // Opens shadow unopened globals.
                candidates.insert(id, span);
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
            .into_keys()
            .next()
            .expect("Candidates should not be empty."))
    } else if candidates.is_empty() {
        Err(Error::NotFound(name.to_string(), path.span))
    } else {
        let mut spans: Vec<_> = candidates.into_values().collect();
        spans.sort();
        Err(Error::Ambiguous(
            name.to_string(),
            path.span,
            spans[0],
            spans[1],
        ))
    }
}
