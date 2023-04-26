// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{CallableKind, Expr, ExprKind, Functor, NodeId, PrimTy, Res, Ty, UnOp},
    mut_visit::{walk_expr, MutVisitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("operation does not support the controlled functor")]
    #[diagnostic(help("each operation called inside an operation with compiler-generated controlled specializations must support the controlled functor"))]
    MissingCtlFunctor(#[label] Span),
}

pub(super) struct CtlDistrib {
    pub(super) ctls: Res,
    pub(super) errors: Vec<Error>,
}

impl MutVisitor for CtlDistrib {
    fn visit_expr(&mut self, expr: &mut Expr) {
        if let ExprKind::Call(op, args) = &mut expr.kind {
            match &op.ty {
                Ty::Arrow(CallableKind::Operation, input, output, functors)
                    if functors.contains(&Functor::Ctl) =>
                {
                    op.kind = ExprKind::UnOp(UnOp::Functor(Functor::Ctl), op.clone());
                    op.ty = Ty::Arrow(
                        CallableKind::Operation,
                        Box::new(Ty::Tuple(vec![
                            Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
                            Ty::clone(input),
                        ])),
                        output.clone(),
                        functors.clone(),
                    );

                    args.kind = ExprKind::Tuple(vec![
                        Expr {
                            id: NodeId::default(),
                            span: args.span,
                            ty: Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
                            kind: ExprKind::Name(self.ctls),
                        },
                        Expr::clone(args),
                    ]);
                    args.ty = Ty::Tuple(vec![
                        Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
                        Ty::clone(&args.ty),
                    ]);
                }
                Ty::Arrow(CallableKind::Operation, _, _, _) => {
                    self.errors.push(Error::MissingCtlFunctor(op.span));
                }
                _ => {}
            }
        }

        match &mut expr.kind {
            ExprKind::Conjugate(_, apply) => {
                // Only transform the apply block, the within block can remain as-is.
                self.visit_block(apply);
            }
            _ => walk_expr(self, expr),
        }
    }
}
