// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    hir::{Block, CallableDecl, Expr, Ident, LocalItemId, NodeId, Pat, QubitInit, SpecDecl, Stmt},
    mut_visit::{self, MutVisitor},
};

/// The [Assigner] tracks the current state of IDs being handed out within a pass of the resolver.
/// It is used when visiting a package to assign IDs to all elements. Identifiers are resolved and
/// replaced with canonical IDs in this process. The AST gets all IDs resolved after the symbol resolution
/// run.
#[derive(Debug)]
pub struct Assigner {
    next_node: NodeId,
    next_item: LocalItemId,
}

impl Assigner {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_node: NodeId::FIRST,
            next_item: LocalItemId::default(),
        }
    }

    pub fn next_node(&mut self) -> NodeId {
        let id = self.next_node;
        self.next_node = id.successor();
        id
    }

    pub fn next_item(&mut self) -> LocalItemId {
        let id = self.next_item;
        self.next_item = id.successor();
        id
    }

    fn assign(&mut self, id: &mut NodeId) {
        if id.is_default() {
            *id = self.next_node();
        }
    }
}

impl Default for Assigner {
    fn default() -> Self {
        Self::new()
    }
}

impl MutVisitor for Assigner {
    fn visit_callable_decl(&mut self, decl: &mut CallableDecl) {
        self.assign(&mut decl.id);
        mut_visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &mut SpecDecl) {
        self.assign(&mut decl.id);
        mut_visit::walk_spec_decl(self, decl);
    }

    fn visit_block(&mut self, block: &mut Block) {
        self.assign(&mut block.id);
        mut_visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        self.assign(&mut stmt.id);
        mut_visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        self.assign(&mut expr.id);
        mut_visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &mut Pat) {
        self.assign(&mut pat.id);
        mut_visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &mut QubitInit) {
        self.assign(&mut init.id);
        mut_visit::walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &mut Ident) {
        self.assign(&mut ident.id);
    }
}
