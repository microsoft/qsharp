// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use std::{collections::HashSet, mem::take};

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    hir::{Block, Expr, ExprKind, NodeId, Res, Stmt, StmtKind, Ty},
    mut_visit::{self, MutVisitor},
    visit::{self, Visitor},
};
use thiserror::Error;

use crate::{
    invert_block::adj_invert_block,
    spec_gen::adj_gen::{self, AdjDistrib},
};

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[diagnostic(transparent)]
    #[error(transparent)]
    AdjGen(adj_gen::Error),

    #[error("variable cannot be assigned in apply-block since it is used in within-block")]
    #[diagnostic(help("updating mutable variables in the apply-block that are used in the within-block can violate logic reversibility"))]
    ApplyAssign(#[label] Span),
}

/// Generates adjoint inverted blocks for within-blocks across all conjugate expressions,
/// eliminating the conjugate expression from the compilation unit.
pub fn invert_conjugate_exprs(unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = ConjugateElim {
        assigner: &mut unit.assigner,
        errors: Vec::new(),
    };
    pass.visit_package(&mut unit.package);

    pass.errors
}

struct ConjugateElim<'a> {
    assigner: &'a mut Assigner,
    errors: Vec<Error>,
}

impl<'a> MutVisitor for ConjugateElim<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        mut_visit::walk_expr(self, expr);

        match take(&mut expr.kind) {
            ExprKind::Conjugate(within, apply) => {
                let mut usage = Usage {
                    used: HashSet::new(),
                };
                usage.visit_block(&within);
                let mut assign_check = AssignmentCheck {
                    used: usage.used,
                    errors: Vec::new(),
                };
                assign_check.visit_block(&apply);
                self.errors.extend(assign_check.errors);

                let mut adj_within = within.clone();
                if let Err(invert_errors) = adj_invert_block(self.assigner, &mut adj_within) {
                    self.errors.extend(
                        invert_errors
                            .into_iter()
                            .map(adj_gen::Error::LogicSep)
                            .map(Error::AdjGen),
                    );
                    return;
                }
                let mut distrib = AdjDistrib { errors: Vec::new() };
                distrib.visit_block(&mut adj_within);
                self.errors
                    .extend(distrib.errors.into_iter().map(Error::AdjGen));

                let new_block = Block {
                    id: NodeId::default(),
                    span: Span::default(),
                    ty: Ty::UNIT,
                    stmts: vec![
                        block_as_stmt(within),
                        block_as_stmt(apply),
                        block_as_stmt(adj_within),
                    ],
                };
                *expr = block_as_expr(new_block);
            }
            kind => expr.kind = kind,
        }
    }
}

fn block_as_expr(block: Block) -> Expr {
    Expr {
        id: NodeId::default(),
        span: Span::default(),
        ty: Ty::UNIT,
        kind: ExprKind::Block(block),
    }
}

fn block_as_stmt(block: Block) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: Span::default(),
        kind: StmtKind::Expr(block_as_expr(block)),
    }
}

struct Usage {
    used: HashSet<NodeId>,
}

impl<'a> Visitor<'a> for Usage {
    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::Name(Res::Local(id)) => {
                self.used.insert(*id);
            }
            _ => visit::walk_expr(self, expr),
        }
    }
}

struct AssignmentCheck {
    used: HashSet<NodeId>,
    errors: Vec<Error>,
}

impl<'a> Visitor<'a> for AssignmentCheck {
    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::Assign(lhs, rhs) => {
                self.visit_expr(rhs);
                self.check_assign(lhs);
            }
            _ => visit::walk_expr(self, expr),
        }
    }
}

impl AssignmentCheck {
    fn check_assign(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Hole => {}
            ExprKind::Paren(expr) => self.check_assign(expr),
            ExprKind::Name(Res::Local(id)) => {
                if self.used.contains(id) {
                    self.errors.push(Error::ApplyAssign(expr.span));
                }
            }
            ExprKind::Tuple(var_tup) => {
                for expr in var_tup {
                    self.check_assign(expr);
                }
            }
            _ => panic!("unexpected expr type in assignment"),
        }
    }
}
