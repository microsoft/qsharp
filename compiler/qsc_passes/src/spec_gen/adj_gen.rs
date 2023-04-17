// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::logic_sep;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{CallableKind, Expr, ExprKind, Functor, NodeId, Ty, UnOp},
    mut_visit::{walk_expr, MutVisitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("operation does not support the adjoint functor")]
    #[diagnostic(help("each operation called inside an operation with compiler-generated adjoint specializations must support the adjoint functor"))]
    MissingAdjFunctor(#[label] Span),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LogicSep(logic_sep::Error),
}

pub(super) struct AdjDistrib {
    pub(super) errors: Vec<Error>,
}

impl MutVisitor for AdjDistrib {
    fn visit_expr(&mut self, expr: &mut Expr) {
        if let ExprKind::Call(op, _) = &mut expr.kind {
            match &op.ty {
                Ty::Arrow(CallableKind::Operation, _, _, functor)
                    if functor.contains(&Functor::Adj) =>
                {
                    *op = Box::new(Expr {
                        id: NodeId::default(),
                        span: op.span,
                        ty: op.ty.clone(),
                        kind: ExprKind::UnOp(UnOp::Functor(Functor::Adj), op.clone()),
                    });
                }
                Ty::Arrow(CallableKind::Operation, _, _, _) => {
                    self.errors.push(Error::MissingAdjFunctor(op.span));
                }
                _ => {}
            }
        }

        if let ExprKind::Conjugate(_, apply) = &mut expr.kind {
            // Only transform the apply block, the within block can remain as-is.
            self.visit_block(apply);
        } else {
            walk_expr(self, expr);
        }
    }
}
