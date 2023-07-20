// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::{hash_map::Entry, HashMap};

use qsc_hir::{
    assigner::Assigner,
    hir::{
        Block, CallableDecl, Expr, ExprKind, Ident, NodeId, Pat, QubitInit, Res, SpecDecl, Stmt,
    },
    mut_visit::{
        walk_block, walk_callable_decl, walk_expr, walk_ident, walk_pat, walk_qubit_init,
        walk_spec_decl, walk_stmt, MutVisitor,
    },
};

pub(crate) struct NodeIdRefresher<'a> {
    assigner: &'a mut Assigner,
    replacements: HashMap<NodeId, NodeId>,
}

impl<'a> NodeIdRefresher<'a> {
    pub(crate) fn new(assigner: &'a mut Assigner) -> Self {
        Self {
            assigner,
            replacements: HashMap::new(),
        }
    }

    fn freshen_id(&mut self, id: NodeId) -> NodeId {
        if id.is_default() {
            return self.assigner.next_node();
        }
        match self.replacements.entry(id) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let new_id = self.assigner.next_node();
                entry.insert(new_id);
                new_id
            }
        }
    }

    fn replace_id(&mut self, id: NodeId) -> NodeId {
        match self.replacements.get(&id) {
            Some(new_id) => *new_id,
            None => id,
        }
    }
}

impl MutVisitor for NodeIdRefresher<'_> {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        decl.id = self.freshen_id(decl.id);
        walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &mut SpecDecl) {
        decl.id = self.freshen_id(decl.id);
        walk_spec_decl(self, decl);
    }

    fn visit_block(&mut self, block: &mut Block) {
        block.id = self.freshen_id(block.id);
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        stmt.id = self.freshen_id(stmt.id);
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        expr.id = self.freshen_id(expr.id);

        match &mut expr.kind {
            ExprKind::Closure(captures, _) => {
                *captures = captures.iter_mut().map(|id| self.replace_id(*id)).collect();
            }
            ExprKind::Var(Res::Local(id), _) => {
                *id = self.replace_id(*id);
            }
            _ => {}
        }

        walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &mut Pat) {
        pat.id = self.freshen_id(pat.id);
        walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &mut QubitInit) {
        init.id = self.freshen_id(init.id);
        walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        ident.id = self.freshen_id(ident.id);
        walk_ident(self, ident);
    }
}
