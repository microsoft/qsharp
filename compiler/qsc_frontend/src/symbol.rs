// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::{
    ast::{
        Block, CallableDecl, Expr, ExprKind, ItemKind, Namespace, NodeId, Pat, PatKind, SpecBody,
        SpecDecl, Stmt, StmtKind,
    },
    visit::{self, Visitor},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub(super) struct Id(u32);

impl Id {
    fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

pub(super) struct Table {
    nodes: HashMap<NodeId, Id>,
    next_id: Id,
}

impl Table {
    fn declare_symbol(&mut self, node: NodeId) -> Id {
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
    global_tys: HashMap<&'a str, HashMap<&'a str, Id>>,
    global_terms: HashMap<&'a str, HashMap<&'a str, Id>>,
    opens: HashMap<&'a str, HashSet<&'a str>>,
    locals: Vec<HashMap<&'a str, Id>>,
}

impl<'a> Resolver<'a> {
    pub(super) fn into_table(self) -> Table {
        self.symbols
    }

    fn insert_bindings(&mut self, env: &mut HashMap<&'a str, Id>, pat: &'a Pat) {
        match &pat.kind {
            PatKind::Bind(name, _) => {
                let id = self.symbols.declare_symbol(name.id);
                env.insert(name.name.as_str(), id);
            }
            PatKind::Discard(_) | PatKind::Elided => {}
            PatKind::Paren(pat) => self.insert_bindings(env, pat),
            PatKind::Tuple(pats) => pats.iter().for_each(|p| self.insert_bindings(env, p)),
        }
    }
}

impl<'a> From<GlobalTable<'a>> for Resolver<'a> {
    fn from(value: GlobalTable<'a>) -> Self {
        Self {
            symbols: value.symbols,
            global_tys: value.tys,
            global_terms: value.terms,
            opens: HashMap::new(),
            locals: Vec::new(),
        }
    }
}

impl<'a> Visitor<'a> for Resolver<'a> {
    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.opens = HashMap::new();
        for item in &namespace.items {
            if let ItemKind::Open(namespace, alias) = &item.kind {
                let alias = alias.as_ref().map_or("", |a| &a.name);
                self.opens.entry(alias).or_default().insert(&namespace.name);
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

    fn visit_block(&mut self, block: &'a Block) {
        self.locals.push(HashMap::new());
        visit::walk_block(self, block);
        self.locals.pop();
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
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

        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let ExprKind::Path(path) = &expr.kind {
            todo!()
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

impl<'a> Visitor<'a> for GlobalTable<'a> {
    fn visit_namespace(&mut self, namespace: &'a Namespace) {
        self.namespace = &namespace.name.name;
        visit::walk_namespace(self, namespace);
    }

    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        let id = self.symbols.declare_symbol(decl.name.id);
        self.terms
            .entry(self.namespace)
            .or_default()
            .insert(&decl.name.name, id);
    }
}
