// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_frontend::{
    compile::Context,
    typeck::ty::{CallableKind, Functor, Prim, Ty},
};
use qsc_hir::{
    hir::{self, Expr, ExprKind, Path, UnOp},
    mut_visit::{walk_expr, MutVisitor},
};

pub(super) struct CtlDistrib<'a> {
    pub(super) ctls: &'a Path,
    pub(super) context: &'a mut Context,
}

impl<'a> MutVisitor for CtlDistrib<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        if let ExprKind::Call(op, args) = &mut expr.kind {
            let ty = self
                .context
                .tys()
                .get(op.id)
                .expect("type should be present in tys")
                .clone();
            match &ty {
                Ty::Arrow(CallableKind::Operation, args_ty, _, functor)
                    if functor.contains(&Functor::Ctl) =>
                {
                    let new_op_id = self.context.assigner_mut().next_id();
                    self.context.tys_mut().insert(new_op_id, ty.clone());
                    *op = Box::new(Expr {
                        id: new_op_id,
                        span: op.span,
                        kind: ExprKind::UnOp(UnOp::Functor(hir::Functor::Ctl), op.clone()),
                    });

                    let new_args_id = self.context.assigner_mut().next_id();
                    self.context.tys_mut().insert(
                        new_args_id,
                        Ty::Tuple(vec![
                            Ty::Array(Box::new(Ty::Prim(Prim::Qubit))),
                            *args_ty.clone(),
                        ]),
                    );
                    let new_ctls_path_id = self.context.assigner_mut().next_id();
                    self.context
                        .tys_mut()
                        .insert(new_ctls_path_id, Ty::Array(Box::new(Ty::Prim(Prim::Qubit))));
                    *args = Box::new(Expr {
                        id: new_args_id,
                        span: args.span,
                        kind: ExprKind::Tuple(vec![
                            Expr {
                                id: new_ctls_path_id,
                                span: args.span,
                                kind: ExprKind::Path(self.ctls.clone()),
                            },
                            *args.clone(),
                        ]),
                    });
                }
                Ty::Arrow(CallableKind::Operation, _, _, _) => todo!("missing functor"),
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
