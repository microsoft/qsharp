// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{Block, CallableKind, Expr, ExprKind, NodeId, StmtKind, Ty},
    visit::{walk_expr, Visitor},
};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("cannot generate adjoint with this expression")]
    #[diagnostic(help("assignments, repeat-loops, while-loops, and returns cannot be used in blocks that require generated adjoint"))]
    ExprForbidden(#[label] Span),

    #[error("cannot generate adjoint of block with {0} type")]
    #[diagnostic(help("adjoint generation can only be performed with blocks of type Unit"))]
    NonUnitBlock(Ty, #[label] Span),

    #[error("cannot generate adjoint with operation call in this position")]
    #[diagnostic(help("in blocks that require generated adjoint, operation calls can only appear as top-level statements or in a qubit allocation block, conjugate block, for-loop block, or conditional block"))]
    OpCallForbidden(#[label] Span),
}

/// Checks that the given block is separatable, meaning classical statements and quantum statements
/// across the block and any nested expressions/blocks can be logically separated. On success, returns a `HashSet` of
/// all quantum statement node ids, based on whether any operation calls are present in that statement.
pub(crate) fn find_quantum_stmts(block: &Block) -> Result<HashSet<NodeId>, Vec<Error>> {
    let mut pass = SepCheck {
        errors: Vec::new(),
        op_call_present: HashSet::new(),
        op_call_allowed: true,
        has_op_call: false,
    };
    pass.visit_block(block);
    if pass.errors.is_empty() {
        Ok(pass.op_call_present)
    } else {
        Err(pass.errors)
    }
}

struct SepCheck {
    errors: Vec<Error>,
    op_call_present: HashSet<NodeId>,
    op_call_allowed: bool,
    has_op_call: bool,
}

impl<'a> Visitor<'a> for SepCheck {
    fn visit_block(&mut self, block: &'a Block) {
        match &block.ty {
            Ty::Tuple(tup) if tup.is_empty() => {}
            ty if self.op_call_allowed => {
                self.errors
                    .push(Error::NonUnitBlock(ty.clone(), block.span));
                self.op_call_allowed = false;
            }
            _ => {}
        }

        let prior = self.op_call_allowed;
        let mut has_inner_op_call = false;
        for stmt in &block.stmts {
            match &stmt.kind {
                StmtKind::Empty => {}

                StmtKind::Local(..) | StmtKind::Qubit(_, _, _, None) => {
                    self.op_call_allowed = false;
                    self.visit_stmt(stmt);
                    self.op_call_allowed = prior;
                }

                StmtKind::Qubit(_, _, init, Some(qubit_block)) => {
                    self.op_call_allowed = false;
                    self.visit_qubit_init(init);
                    self.op_call_allowed = prior;
                    self.handle_block(qubit_block, stmt.id);
                }

                StmtKind::Expr(expr) | StmtKind::Semi(expr) => match &expr.kind {
                    ExprKind::Block(block) => self.handle_block(block, stmt.id),
                    ExprKind::Call(callee, args) => {
                        has_inner_op_call = self.handle_call(expr, stmt.id, callee, args, prior)
                            || has_inner_op_call;
                    }
                    ExprKind::Conjugate(within, apply) => {
                        self.handle_block(within, stmt.id);
                        self.handle_block(apply, stmt.id);
                    }
                    ExprKind::For(_, iter, loop_block) => {
                        self.op_call_allowed = false;
                        self.visit_expr(iter);
                        self.op_call_allowed = prior;
                        self.handle_block(loop_block, stmt.id);
                    }
                    ExprKind::If(cond, then_block, else_expr) => {
                        self.handle_if_expr(prior, cond, then_block, else_expr);
                        if self.has_op_call {
                            self.has_op_call = false;
                            self.op_call_present.insert(stmt.id);
                        }
                    }

                    ExprKind::Array(_)
                    | ExprKind::ArrayRepeat(..)
                    | ExprKind::BinOp(..)
                    | ExprKind::Err
                    | ExprKind::Fail(..)
                    | ExprKind::Field(..)
                    | ExprKind::Hole
                    | ExprKind::Index(..)
                    | ExprKind::Lambda(..)
                    | ExprKind::Lit(..)
                    | ExprKind::Paren(..)
                    | ExprKind::Name(..)
                    | ExprKind::Range(..)
                    | ExprKind::TernOp(..)
                    | ExprKind::Tuple(..)
                    | ExprKind::UnOp(..) => {
                        self.op_call_allowed = false;
                        self.visit_expr(expr);
                        self.op_call_allowed = prior;
                    }

                    ExprKind::Assign(..)
                    | ExprKind::AssignOp(..)
                    | ExprKind::AssignUpdate(..)
                    | ExprKind::Repeat(..)
                    | ExprKind::Return(..)
                    | ExprKind::While(..) => {
                        self.errors.push(Error::ExprForbidden(expr.span));
                    }
                },
            }
        }
        self.has_op_call = self.has_op_call || has_inner_op_call;
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let ExprKind::Call(callee, _) = &expr.kind {
            if matches!(callee.ty, Ty::Arrow(CallableKind::Operation, _, _, _)) {
                self.errors.push(Error::OpCallForbidden(expr.span));
            }
        }
        walk_expr(self, expr);
    }
}

impl SepCheck {
    fn handle_if_expr(
        &mut self,
        prior: bool,
        cond: &Expr,
        then_block: &Block,
        else_expr: &Option<Box<Expr>>,
    ) {
        self.op_call_allowed = false;
        self.visit_expr(cond);
        self.op_call_allowed = prior;

        self.visit_block(then_block);

        if let Some(else_expr) = else_expr {
            match &else_expr.kind {
                ExprKind::Block(else_block) => {
                    self.visit_block(else_block);
                }
                ExprKind::If(inner_cond, inner_then, inner_else) => {
                    self.handle_if_expr(prior, inner_cond, inner_then, inner_else);
                }
                _ => panic!("else expr should be if-expr or block-expr, got: {else_expr}"),
            }
        }
    }

    fn handle_block(&mut self, block: &Block, id: NodeId) {
        self.visit_block(block);
        if self.has_op_call {
            self.has_op_call = false;
            self.op_call_present.insert(id);
        }
    }

    fn handle_call(
        &mut self,
        expr: &Expr,
        id: NodeId,
        callee: &Expr,
        args: &Expr,
        prior: bool,
    ) -> bool {
        let is_op_call = matches!(callee.ty, Ty::Arrow(CallableKind::Operation, _, _, _));
        self.op_call_allowed = false;
        self.visit_expr(callee);
        self.visit_expr(args);
        self.op_call_allowed = prior;
        let mut has_inner_op_call = false;
        if is_op_call {
            self.op_call_present.insert(id);
            if self.op_call_allowed {
                has_inner_op_call = true;
            } else {
                self.errors.push(Error::OpCallForbidden(expr.span));
            }
        }
        has_inner_op_call
    }
}
