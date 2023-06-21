// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    hir::{Block, CallableDecl, Expr, Ident, NodeId, Pat, QubitInit, SpecDecl, Stmt},
    visit::{self, Visitor},
};
use qsc_data_structures::index_map::IndexMap;

#[derive(Default)]
pub struct Validator {
    ids: IndexMap<NodeId, ()>,
}

impl Validator {
    fn check(&mut self, id: NodeId) {
        if id.is_default() {
            panic!("default node ID should be replaced")
        } else if self.ids.contains_key(id) {
            panic!("duplicate node ID");
        } else {
            self.ids.insert(id, ());
        }
    }
}

impl Visitor<'_> for Validator {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        self.check(decl.id);
        visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &SpecDecl) {
        self.check(decl.id);
        visit::walk_spec_decl(self, decl);
    }

    fn visit_block(&mut self, block: &Block) {
        self.check(block.id);
        visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        self.check(stmt.id);
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        self.check(expr.id);
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &Pat) {
        self.check(pat.id);
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &QubitInit) {
        self.check(init.id);
        visit::walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &Ident) {
        self.check(ident.id);
    }
}
