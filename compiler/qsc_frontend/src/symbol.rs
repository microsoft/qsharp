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
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Id(u32);

impl Id {
    fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Debug)]
pub(super) struct Error {
    pub(super) span: Span,
    pub(super) kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    NotFound(String),
    Ambiguous(String, Span, Span),
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

    fn declare(&mut self, node: NodeId) -> Id {
        let id = self.next_id;
        self.next_id = self.next_id.successor();
        self.nodes.insert(node, id);
        id
    }

    fn use_symbol(&mut self, node: NodeId, symbol: Id) {
        self.nodes.insert(node, symbol);
    }
}

pub(super) struct Resolver<'a> {
    symbols: Table,
    tys: HashMap<&'a str, HashMap<&'a str, Id>>,
    terms: HashMap<&'a str, HashMap<&'a str, Id>>,
    namespace: &'a str,
    opens: HashMap<&'a str, HashMap<&'a str, Span>>,
    locals: Vec<HashMap<&'a str, Id>>,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub(super) fn into_table(self) -> (Table, Vec<Error>) {
        (self.symbols, self.errors)
    }

    fn insert_bindings(&mut self, env: &mut HashMap<&'a str, Id>, pat: &'a Pat) {
        match &pat.kind {
            PatKind::Bind(name, _) => {
                let id = self.symbols.declare(name.id);
                env.insert(name.name.as_str(), id);
            }
            PatKind::Discard(_) | PatKind::Elided => {}
            PatKind::Paren(pat) => self.insert_bindings(env, pat),
            PatKind::Tuple(pats) => pats.iter().for_each(|p| self.insert_bindings(env, p)),
        }
    }

    fn resolve_ty(&mut self, path: &Path) {
        match resolve(&self.tys, self.namespace, &self.opens, &[], path) {
            Ok(symbol) => self.symbols.use_symbol(path.id, symbol),
            Err(err) => self.errors.push(err),
        }
    }

    fn resolve_term(&mut self, path: &Path) {
        match resolve(&self.terms, self.namespace, &self.opens, &self.locals, path) {
            Ok(symbol) => self.symbols.use_symbol(path.id, symbol),
            Err(err) => self.errors.push(err),
        }
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
        let mut params = HashMap::new();
        self.insert_bindings(&mut params, &decl.input);
        self.locals.push(params);
        visit::walk_callable_decl(self, decl);
        self.locals.pop();
    }

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        if let SpecBody::Impl(input, block) = &decl.body {
            let mut params = HashMap::new();
            self.insert_bindings(&mut params, input);
            self.locals.push(params);
            self.visit_block(block);
            self.locals.pop();
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
        self.locals.push(HashMap::new());
        visit::walk_block(self, block);
        self.locals.pop();
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        visit::walk_stmt(self, stmt);

        match &stmt.kind {
            StmtKind::Borrow(pat, _, _)
            | StmtKind::Let(pat, _)
            | StmtKind::Mutable(pat, _)
            | StmtKind::Use(pat, _, _) => {
                let mut env = self
                    .locals
                    .pop()
                    .expect("Statement should always have an environment.");
                self.insert_bindings(&mut env, pat);
                self.locals.push(env);
            }
            StmtKind::Expr(_) | StmtKind::Semi(_) => {}
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let ExprKind::Path(path) = &expr.kind {
            self.resolve_term(path);
        } else {
            visit::walk_expr(self, expr);
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
            symbols: self.symbols,
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
        Err(Error {
            span: path.span,
            kind: ErrorKind::NotFound(name.to_string()),
        })
    } else {
        let mut spans: Vec<_> = candidates.into_values().collect();
        spans.sort();
        Err(Error {
            span: path.span,
            kind: ErrorKind::Ambiguous(name.to_string(), spans[0], spans[1]),
        })
    }
}
