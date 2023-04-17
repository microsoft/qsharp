// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashSet;

use qsc_hir::{
    hir::{Block, Expr, ExprKind, NodeId},
    mut_visit::{walk_expr, MutVisitor},
};

use crate::logic_sep::{check_block_separatable, Error};

pub(crate) fn adj_invert_block(block: &mut Block) -> Result<(), Vec<Error>> {
    let op_call_stmts = check_block_separatable(block)?;
    let mut pass = BlockInverter { op_call_stmts };
    pass.visit_block(block);
    Ok(())
}

struct BlockInverter {
    op_call_stmts: HashSet<NodeId>,
}

impl MutVisitor for BlockInverter {
    fn visit_block(&mut self, block: &mut Block) {
        let mut determ = Vec::new();
        let mut nondeterm = Vec::new();
        for mut stmt in block.stmts.drain(..) {
            self.visit_stmt(&mut stmt);
            if self.op_call_stmts.contains(&stmt.id) {
                nondeterm.push(stmt);
            } else {
                determ.push(stmt);
            }
        }
        nondeterm.reverse();
        block.stmts.append(&mut determ);
        block.stmts.append(&mut nondeterm);
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        // TODO: This should also handle the inversion of for-loop expressions.
        match &mut expr.kind {
            ExprKind::Conjugate(_, apply) => {
                // Only invert the apply block, within block inversion handled by a different pass.
                self.visit_block(apply);
            }
            _ => walk_expr(self, expr),
        }
    }
}
