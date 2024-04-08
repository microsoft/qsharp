// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::fir::{BlockId, ExprId, LocalVarId, NodeId, PatId, StmtId};

#[derive(Debug)]
pub struct Assigner {
    next_node: NodeId,
    next_block: BlockId,
    next_expr: ExprId,
    next_pat: PatId,
    next_stmt: StmtId,
    next_local: LocalVarId,
    stashed_local: LocalVarId,
}

impl Assigner {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_node: NodeId::FIRST,
            next_block: BlockId::default(),
            next_expr: ExprId::default(),
            next_pat: PatId::default(),
            next_stmt: StmtId::default(),
            next_local: LocalVarId::default(),
            stashed_local: LocalVarId::default(),
        }
    }

    pub fn next_node(&mut self) -> NodeId {
        let id = self.next_node;
        self.next_node = id.successor();
        id
    }

    pub fn next_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block = id.successor();
        id
    }

    pub fn next_expr(&mut self) -> ExprId {
        let id = self.next_expr;
        self.next_expr = id.successor();
        id
    }

    pub fn next_pat(&mut self) -> PatId {
        let id = self.next_pat;
        self.next_pat = id.successor();
        id
    }

    pub fn next_stmt(&mut self) -> StmtId {
        let id = self.next_stmt;
        self.next_stmt = id.successor();
        id
    }

    pub fn next_local(&mut self) -> LocalVarId {
        let id = self.next_local;
        self.next_local = id.successor();
        id
    }

    pub fn stash_local(&mut self) {
        self.stashed_local = self.next_local;
        self.next_local = LocalVarId::default();
    }

    pub fn reset_local(&mut self) {
        self.next_local = self.stashed_local;
        self.stashed_local = LocalVarId::default();
    }
}

impl Default for Assigner {
    fn default() -> Self {
        Self::new()
    }
}
