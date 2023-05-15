// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::panic;
use std::{mem::take, rc::Rc};

use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    global::Table,
    hir::{
        BinOp, Block, Expr, ExprKind, Lit, Mutability, NodeId, Pat, PrimField, PrimTy, Stmt,
        StmtKind, Ty, UnOp,
    },
    mut_visit::{walk_expr, MutVisitor},
};

use crate::{
    common::{create_gen_core_ref, IdentTemplate},
    Error,
};

#[cfg(test)]
mod tests;

pub fn loop_unification(core: &Table, unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = LoopUni {
        core,
        assigner: &mut unit.assigner,
    };
    pass.visit_package(&mut unit.package);
    vec![]
}

struct LoopUni<'a> {
    core: &'a Table,
    assigner: &'a mut Assigner,
}

impl LoopUni<'_> {
    fn visit_repeat(
        &mut self,
        mut block: Block,
        cond: Box<Expr>,
        fixup: Option<Block>,
        span: Span,
    ) -> Expr {
        let cond_span = cond.span;

        let continue_cond_id = self.gen_ident("continue_cond", Ty::Prim(PrimTy::Bool), cond_span);
        let continue_cond_init = continue_cond_id.gen_id_init(
            Mutability::Mutable,
            Expr {
                id: NodeId::default(),
                span: cond_span,
                ty: Ty::Prim(PrimTy::Bool),
                kind: ExprKind::Lit(Lit::Bool(true)),
            },
        );

        let update = Stmt {
            id: NodeId::default(),
            span: cond_span,
            kind: StmtKind::Semi(Expr {
                id: NodeId::default(),
                span: cond_span,
                ty: Ty::UNIT,
                kind: ExprKind::Assign(
                    Box::new(continue_cond_id.gen_local_ref()),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: cond_span,
                        ty: Ty::Prim(PrimTy::Bool),
                        kind: ExprKind::UnOp(UnOp::NotL, cond),
                    }),
                ),
            }),
        };
        block.stmts.push(update);

        if let Some(fix_body) = fixup {
            let fix_if = Stmt {
                id: NodeId::default(),
                span: fix_body.span,
                kind: StmtKind::Expr(Expr {
                    id: NodeId::default(),
                    span: fix_body.span,
                    ty: Ty::UNIT,
                    kind: ExprKind::If(Box::new(continue_cond_id.gen_local_ref()), fix_body, None),
                }),
            };
            block.stmts.push(fix_if);
        }

        let new_block = Block {
            id: NodeId::default(),
            span,
            ty: Ty::UNIT,
            stmts: vec![
                continue_cond_init,
                Stmt {
                    id: NodeId::default(),
                    span,
                    kind: StmtKind::Expr(Expr {
                        id: NodeId::default(),
                        span,
                        ty: Ty::UNIT,
                        kind: ExprKind::While(Box::new(continue_cond_id.gen_local_ref()), block),
                    }),
                },
            ],
        };

        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::UNIT,
            kind: ExprKind::Block(new_block),
        }
    }

    fn visit_for_array(
        &mut self,
        iter: Pat,
        iterable: Box<Expr>,
        mut block: Block,
        span: Span,
    ) -> Expr {
        let iterable_span = iterable.span;

        let array_id = self.gen_ident("array_id", iterable.ty.clone(), iterable_span);
        let array_capture = array_id.gen_id_init(Mutability::Immutable, *iterable);

        let len_callee =
            create_gen_core_ref(self.core, "Microsoft.Quantum.Core", "Length", array_id.span);
        let len_id = self.gen_ident("len_id", Ty::Prim(PrimTy::Int), iterable_span);
        let len_capture = len_id.gen_id_init(
            Mutability::Immutable,
            Expr {
                id: NodeId::default(),
                span: array_id.span,
                ty: array_id.ty.clone(),
                kind: ExprKind::Call(Box::new(len_callee), Box::new(array_id.gen_local_ref())),
            },
        );

        let index_id = self.gen_ident("index_id", Ty::Prim(PrimTy::Int), iterable_span);
        let index_init = index_id.gen_id_init(
            Mutability::Mutable,
            Expr {
                id: NodeId::default(),
                span: iterable_span,
                ty: Ty::Prim(PrimTy::Int),
                kind: ExprKind::Lit(Lit::Int(0)),
            },
        );

        let pat_ty = iter.ty.clone();
        let pat_init = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                iter,
                Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    ty: pat_ty,
                    kind: ExprKind::Index(
                        Box::new(array_id.gen_local_ref()),
                        Box::new(index_id.gen_local_ref()),
                    ),
                },
            ),
        };

        let update_index = gen_id_add_update(
            &index_id,
            Expr {
                id: NodeId::default(),
                span: iterable_span,
                ty: Ty::Prim(PrimTy::Int),
                kind: ExprKind::Lit(Lit::Int(1)),
            },
        );

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = Expr {
            id: NodeId::default(),
            span: iterable_span,
            ty: Ty::Prim(PrimTy::Bool),
            kind: ExprKind::BinOp(
                BinOp::Lt,
                Box::new(index_id.gen_local_ref()),
                Box::new(len_id.gen_local_ref()),
            ),
        };

        let while_stmt = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Expr(Expr {
                id: NodeId::default(),
                span,
                ty: Ty::UNIT,
                kind: ExprKind::While(Box::new(cond), block),
            }),
        };

        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::UNIT,
            kind: ExprKind::Block(Block {
                id: NodeId::default(),
                span,
                ty: Ty::UNIT,
                stmts: vec![array_capture, len_capture, index_init, while_stmt],
            }),
        }
    }

    fn visit_for_range(
        &mut self,
        iter: Pat,
        iterable: Box<Expr>,
        mut block: Block,
        span: Span,
    ) -> Expr {
        let iterable_span = iterable.span;

        let range_id = self.gen_ident("range_id", Ty::Prim(PrimTy::Range), iterable_span);
        let range_capture = range_id.gen_id_init(Mutability::Immutable, *iterable);

        let index_id = self.gen_ident("index_id", Ty::Prim(PrimTy::Int), iterable_span);
        let index_init = index_id.gen_id_init(
            Mutability::Mutable,
            range_id.gen_field_access(PrimField::Start),
        );

        let step_id = self.gen_ident("step_id", Ty::Prim(PrimTy::Int), iterable_span);
        let step_init = step_id.gen_id_init(
            Mutability::Immutable,
            range_id.gen_field_access(PrimField::Step),
        );

        let end_id = self.gen_ident("end_id", Ty::Prim(PrimTy::Int), iterable_span);
        let end_init = end_id.gen_id_init(
            Mutability::Immutable,
            range_id.gen_field_access(PrimField::End),
        );

        let pat_init = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Local(Mutability::Immutable, iter, index_id.gen_local_ref()),
        };

        let update_index = gen_id_add_update(&index_id, step_id.gen_local_ref());

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = gen_range_cond(&index_id, &step_id, &end_id, iterable_span);

        let while_stmt = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Expr(Expr {
                id: NodeId::default(),
                span,
                ty: Ty::UNIT,
                kind: ExprKind::While(Box::new(cond), block),
            }),
        };

        Expr {
            id: NodeId::default(),
            span,
            ty: Ty::UNIT,
            kind: ExprKind::Block(Block {
                id: NodeId::default(),
                span,
                ty: Ty::UNIT,
                stmts: vec![range_capture, index_init, step_init, end_init, while_stmt],
            }),
        }
    }

    fn gen_ident(&mut self, label: &str, ty: Ty, span: Span) -> IdentTemplate {
        let id = self.assigner.next_id();
        IdentTemplate {
            id,
            span,
            ty,
            name: Rc::from(format!("{label}_{id}")),
        }
    }
}

impl MutVisitor for LoopUni<'_> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);
        match take(&mut expr.kind) {
            ExprKind::Repeat(block, cond, fixup) => {
                *expr = self.visit_repeat(block, cond, fixup, expr.span);
            }
            ExprKind::For(iter, iterable, block) => {
                match iterable.ty {
                    Ty::Array(_) => *expr = self.visit_for_array(iter, iterable, block, expr.span),
                    Ty::Prim(PrimTy::Range) => {
                        *expr = self.visit_for_range(iter, iterable, block, expr.span);
                    }
                    _ => {
                        // This scenario should have been caught by type-checking earlier
                        panic!("The type of the iterable must be either array or range.")
                    }
                }
            }
            kind => expr.kind = kind,
        };
    }
}

fn gen_range_cond(
    index: &IdentTemplate,
    step: &IdentTemplate,
    end: &IdentTemplate,
    span: Span,
) -> Expr {
    Expr {
        id: NodeId::default(),
        span,
        ty: Ty::Prim(PrimTy::Bool),
        kind: ExprKind::BinOp(
            BinOp::OrL,
            Box::new(Expr {
                id: NodeId::default(),
                span,
                ty: Ty::Prim(PrimTy::Bool),
                kind: ExprKind::BinOp(
                    BinOp::AndL,
                    Box::new(Expr {
                        id: NodeId::default(),
                        span,
                        ty: Ty::Prim(PrimTy::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Gt,
                            Box::new(step.gen_local_ref()),
                            Box::new(Expr {
                                id: NodeId::default(),
                                span,
                                ty: Ty::Prim(PrimTy::Int),
                                kind: ExprKind::Lit(Lit::Int(0)),
                            }),
                        ),
                    }),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span,
                        ty: Ty::Prim(PrimTy::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Lte,
                            Box::new(index.gen_local_ref()),
                            Box::new(end.gen_local_ref()),
                        ),
                    }),
                ),
            }),
            Box::new(Expr {
                id: NodeId::default(),
                span,
                ty: Ty::Prim(PrimTy::Bool),
                kind: ExprKind::BinOp(
                    BinOp::AndL,
                    Box::new(Expr {
                        id: NodeId::default(),
                        span,
                        ty: Ty::Prim(PrimTy::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Lt,
                            Box::new(step.gen_local_ref()),
                            Box::new(Expr {
                                id: NodeId::default(),
                                span,
                                ty: Ty::Prim(PrimTy::Int),
                                kind: ExprKind::Lit(Lit::Int(0)),
                            }),
                        ),
                    }),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span,
                        ty: Ty::Prim(PrimTy::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Gte,
                            Box::new(index.gen_local_ref()),
                            Box::new(end.gen_local_ref()),
                        ),
                    }),
                ),
            }),
        ),
    }
}

fn gen_id_add_update(ident: &IdentTemplate, expr: Expr) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: ident.span,
        kind: StmtKind::Semi(Expr {
            id: NodeId::default(),
            span: ident.span,
            ty: Ty::UNIT,
            kind: ExprKind::AssignOp(BinOp::Add, Box::new(ident.gen_local_ref()), Box::new(expr)),
        }),
    }
}
