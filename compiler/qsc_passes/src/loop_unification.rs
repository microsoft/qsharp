// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::panic;
use std::{mem::take, rc::Rc};

use qsc_data_structures::span::Span;
use qsc_frontend::compile::{CompileUnit, Context};
use qsc_hir::{
    hir::{
        BinOp, Block, Expr, ExprKind, Ident, Lit, Mutability, NodeId, Pat, PatKind, PrimTy, Res,
        Stmt, StmtKind, Ty, UnOp,
    },
    mut_visit::{walk_expr, MutVisitor},
};

use crate::Error;

pub fn loop_unification(unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = LoopUni {
        context: &mut unit.context,
        gen_id_count: 0,
    };
    pass.visit_package(&mut unit.package);
    vec![]
}

pub struct LoopUni<'a> {
    context: &'a mut Context,
    gen_id_count: u32,
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

        let continue_cond_id = self.gen_ident("continue_cond", cond_span);
        let continue_cond_init = LoopUni::gen_id_init(
            Mutability::Mutable,
            continue_cond_id.clone(),
            Ty::Prim(PrimTy::Bool),
            Expr {
                id: NodeId::default(),
                span: cond_span,
                ty: Ty::Prim(PrimTy::Bool),
                kind: ExprKind::Lit(Lit::Bool(true)),
            },
        );

        let update = LoopUni::gen_id_update(
            &continue_cond_id,
            Expr {
                id: NodeId::default(),
                span: cond_span,
                ty: Ty::Prim(PrimTy::Bool),
                kind: ExprKind::UnOp(UnOp::Neg, cond),
            },
        );
        block.stmts.push(update);

        if let Some(fix_body) = fixup {
            let fix_if = Stmt {
                id: NodeId::default(),
                span: fix_body.span,
                kind: StmtKind::Expr(Expr {
                    id: NodeId::default(),
                    span: fix_body.span,
                    ty: Ty::UNIT,
                    kind: ExprKind::If(
                        Box::new(LoopUni::gen_local_ref(
                            &continue_cond_id,
                            Ty::Prim(PrimTy::Bool),
                        )),
                        fix_body,
                        None,
                    ),
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
                        kind: ExprKind::While(
                            Box::new(LoopUni::gen_local_ref(
                                &continue_cond_id,
                                Ty::Prim(PrimTy::Bool),
                            )),
                            block,
                        ),
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
        let iterable_ty = iterable.ty.clone();

        let array_id = self.gen_ident("array_id", iterable_span);
        let array_capture = LoopUni::gen_id_init(
            Mutability::Immutable,
            array_id.clone(),
            iterable_ty.clone(),
            *iterable,
        );

        let len_id = self.gen_ident("len_id", iterable_span);
        let len_capture = LoopUni::gen_id_init(
            Mutability::Immutable,
            len_id.clone(),
            Ty::Prim(PrimTy::Int),
            LoopUni::gen_field_access(
                "Length".to_owned(),
                Ty::Prim(PrimTy::Int),
                &array_id,
                iterable_ty.clone(),
            ),
        );

        let index_id = self.gen_ident("index_id", iterable_span);
        let index_init = LoopUni::gen_id_init(
            Mutability::Mutable,
            index_id.clone(),
            Ty::Prim(PrimTy::Int),
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
                        Box::new(LoopUni::gen_local_ref(&array_id, iterable_ty)),
                        Box::new(LoopUni::gen_local_ref(&index_id, Ty::Prim(PrimTy::Int))),
                    ),
                },
            ),
        };

        let update_index = LoopUni::gen_id_update(
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
                Box::new(LoopUni::gen_local_ref(&index_id, Ty::Prim(PrimTy::Int))),
                Box::new(LoopUni::gen_local_ref(&len_id, Ty::Prim(PrimTy::Int))),
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

        let range_id = self.gen_ident("range_id", iterable_span);
        let range_capture = LoopUni::gen_id_init(
            Mutability::Immutable,
            range_id.clone(),
            Ty::Prim(PrimTy::Range),
            *iterable,
        );

        let index_id = self.gen_ident("index_id", iterable_span);
        let index_init = LoopUni::gen_id_init(
            Mutability::Mutable,
            index_id.clone(),
            Ty::Prim(PrimTy::Int),
            LoopUni::gen_field_access(
                "Start".to_owned(),
                Ty::Prim(PrimTy::Int),
                &range_id,
                Ty::Prim(PrimTy::Range),
            ),
        );

        let step_id = self.gen_ident("step_id", iterable_span);
        let step_init = LoopUni::gen_id_init(
            Mutability::Immutable,
            step_id.clone(),
            Ty::Prim(PrimTy::Int),
            LoopUni::gen_field_access(
                "Step".to_owned(),
                Ty::Prim(PrimTy::Int),
                &range_id,
                Ty::Prim(PrimTy::Range),
            ),
        );

        let end_id = self.gen_ident("end_id", iterable_span);
        let end_init = LoopUni::gen_id_init(
            Mutability::Immutable,
            end_id.clone(),
            Ty::Prim(PrimTy::Int),
            LoopUni::gen_field_access(
                "End".to_owned(),
                Ty::Prim(PrimTy::Int),
                &range_id,
                Ty::Prim(PrimTy::Range),
            ),
        );

        let pat_init = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                iter,
                LoopUni::gen_local_ref(&index_id, Ty::Prim(PrimTy::Int)),
            ),
        };

        let update_index = LoopUni::gen_id_update(
            &index_id,
            LoopUni::gen_local_ref(&step_id, Ty::Prim(PrimTy::Int)),
        );

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = LoopUni::gen_range_cond(&index_id, &step_id, &end_id, iterable_span);

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

    fn gen_ident(&mut self, label: &str, span: Span) -> Ident {
        let new_id = Ident {
            id: self.context.assigner_mut().next_id(),
            span,
            name: Rc::from(format!("__{}_{}__", label, self.gen_id_count)),
        };
        self.gen_id_count += 1;
        new_id
    }

    fn gen_range_cond(index: &Ident, step: &Ident, end: &Ident, span: Span) -> Expr {
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
                                Box::new(LoopUni::gen_local_ref(step, Ty::Prim(PrimTy::Int))),
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
                                Box::new(LoopUni::gen_local_ref(index, Ty::Prim(PrimTy::Int))),
                                Box::new(LoopUni::gen_local_ref(end, Ty::Prim(PrimTy::Int))),
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
                                Box::new(LoopUni::gen_local_ref(step, Ty::Prim(PrimTy::Int))),
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
                                Box::new(LoopUni::gen_local_ref(index, Ty::Prim(PrimTy::Int))),
                                Box::new(LoopUni::gen_local_ref(end, Ty::Prim(PrimTy::Int))),
                            ),
                        }),
                    ),
                }),
            ),
        }
    }

    fn gen_local_ref(name: &Ident, ty: Ty) -> Expr {
        Expr {
            id: NodeId::default(),
            span: name.span,
            ty,
            kind: ExprKind::Name(Res::Local(name.id)),
        }
    }

    fn gen_id_init(mutability: Mutability, ident: Ident, ty: Ty, expr: Expr) -> Stmt {
        Stmt {
            id: NodeId::default(),
            span: ident.span,
            kind: StmtKind::Local(
                mutability,
                Pat {
                    id: NodeId::default(),
                    span: ident.span,
                    ty,
                    kind: PatKind::Bind(ident),
                },
                expr,
            ),
        }
    }

    fn gen_field_access(
        field_name: String,
        field_ty: Ty,
        container: &Ident,
        container_ty: Ty,
    ) -> Expr {
        Expr {
            id: NodeId::default(),
            span: container.span,
            ty: field_ty,
            kind: ExprKind::Field(
                Box::new(LoopUni::gen_local_ref(container, container_ty)),
                Ident {
                    id: NodeId::default(),
                    span: container.span,
                    name: Rc::from(field_name),
                },
            ),
        }
    }

    fn gen_id_update(ident: &Ident, expr: Expr) -> Stmt {
        Stmt {
            id: NodeId::default(),
            span: ident.span,
            kind: StmtKind::Semi(Expr {
                id: NodeId::default(),
                span: ident.span,
                ty: Ty::UNIT,
                kind: ExprKind::AssignOp(
                    BinOp::Add,
                    Box::new(LoopUni::gen_local_ref(ident, expr.ty.clone())),
                    Box::new(expr),
                ),
            }),
        }
    }
}

impl MutVisitor for LoopUni<'_> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        let new_expr = take(expr);
        match new_expr.kind {
            ExprKind::Repeat(block, cond, fixup) => {
                *expr = self.visit_repeat(block, cond, fixup, expr.span);
            }
            ExprKind::For(iter, iterable, block) => {
                let is_array = true; // ToDo: check the type of the iterable expression
                if is_array {
                    *expr = self.visit_for_array(iter, iterable, block, expr.span);
                } else if !is_array {
                    *expr = self.visit_for_range(iter, iterable, block, expr.span);
                } else {
                    // This scenario should have been caught by type-checking earlier
                    panic!("The type of the iterable must be either array or range.");
                }
            }
            _ => *expr = new_expr,
        }

        walk_expr(self, expr);
    }
}
