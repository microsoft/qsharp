// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::panic;
use std::mem::take;

use qsc_data_structures::span::Span;
use qsc_hir::{
    assigner::Assigner,
    global::Table,
    hir::{BinOp, Block, Expr, ExprKind, Lit, Mutability, Pat, PrimField, Stmt, StmtKind, UnOp},
    mut_visit::{walk_expr, MutVisitor},
    ty::{GenericArg, Prim, Ty},
};

use crate::common::{create_gen_core_ref, generated_name, IdentTemplate};

#[cfg(test)]
mod tests;

pub(crate) struct LoopUni<'a> {
    pub(crate) core: &'a Table,
    pub(crate) assigner: &'a mut Assigner,
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

        let continue_cond_id = self.gen_ident("continue_cond", Ty::Prim(Prim::Bool), cond_span);
        let continue_cond_init = continue_cond_id.gen_id_init(
            Mutability::Mutable,
            Expr {
                id: self.assigner.next_node(),
                span: cond_span,
                ty: Ty::Prim(Prim::Bool),
                kind: ExprKind::Lit(Lit::Bool(true)),
            },
            self.assigner,
        );

        let update = Stmt {
            id: self.assigner.next_node(),
            span: cond_span,
            kind: StmtKind::Semi(Expr {
                id: self.assigner.next_node(),
                span: cond_span,
                ty: Ty::UNIT,
                kind: ExprKind::Assign(
                    Box::new(continue_cond_id.gen_local_ref(self.assigner)),
                    Box::new(Expr {
                        id: self.assigner.next_node(),
                        span: cond_span,
                        ty: Ty::Prim(Prim::Bool),
                        kind: ExprKind::UnOp(UnOp::NotL, cond),
                    }),
                ),
            }),
        };
        block.stmts.push(update);

        if let Some(fix_body) = fixup {
            let fix_if = Stmt {
                id: self.assigner.next_node(),
                span: fix_body.span,
                kind: StmtKind::Expr(Expr {
                    id: self.assigner.next_node(),
                    span: fix_body.span,
                    ty: Ty::UNIT,
                    kind: ExprKind::If(
                        Box::new(continue_cond_id.gen_local_ref(self.assigner)),
                        Box::new(Expr {
                            id: self.assigner.next_node(),
                            span: fix_body.span,
                            ty: Ty::UNIT,
                            kind: ExprKind::Block(fix_body),
                        }),
                        None,
                    ),
                }),
            };
            block.stmts.push(fix_if);
        }

        let new_block = Block {
            id: self.assigner.next_node(),
            span,
            ty: Ty::UNIT,
            stmts: vec![
                continue_cond_init,
                Stmt {
                    id: self.assigner.next_node(),
                    span: Span::default(),
                    kind: StmtKind::Expr(Expr {
                        id: self.assigner.next_node(),
                        span,
                        ty: Ty::UNIT,
                        kind: ExprKind::While(
                            Box::new(continue_cond_id.gen_local_ref(self.assigner)),
                            block,
                        ),
                    }),
                },
            ],
        };

        Expr {
            id: self.assigner.next_node(),
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
        let array_capture = array_id.gen_id_init(Mutability::Immutable, *iterable, self.assigner);

        let Ty::Array(item_ty) = &array_id.ty else { panic!("iterator should have array type"); };
        let mut len_callee = create_gen_core_ref(
            self.core,
            "Microsoft.Quantum.Core",
            "Length",
            vec![GenericArg::Ty((**item_ty).clone())],
            array_id.span,
        );
        len_callee.id = self.assigner.next_node();
        let len_id = self.gen_ident("len_id", Ty::Prim(Prim::Int), iterable_span);
        let len_capture = len_id.gen_id_init(
            Mutability::Immutable,
            Expr {
                id: self.assigner.next_node(),
                span: array_id.span,
                ty: array_id.ty.clone(),
                kind: ExprKind::Call(
                    Box::new(len_callee),
                    Box::new(array_id.gen_local_ref(self.assigner)),
                ),
            },
            self.assigner,
        );

        let index_id = self.gen_ident("index_id", Ty::Prim(Prim::Int), iterable_span);
        let index_init = index_id.gen_steppable_id_init(
            Mutability::Mutable,
            Expr {
                id: self.assigner.next_node(),
                span: iterable_span,
                ty: Ty::Prim(Prim::Int),
                kind: ExprKind::Lit(Lit::Int(0)),
            },
            self.assigner,
        );

        let pat_ty = iter.ty.clone();
        let pat_init = Stmt {
            id: self.assigner.next_node(),
            span: iter.span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                iter,
                Expr {
                    id: self.assigner.next_node(),
                    span: iterable_span,
                    ty: pat_ty,
                    kind: ExprKind::Index(
                        Box::new(array_id.gen_local_ref(self.assigner)),
                        Box::new(index_id.gen_local_ref(self.assigner)),
                    ),
                },
            ),
        };

        let update_expr = Expr {
            id: self.assigner.next_node(),
            span: iterable_span,
            ty: Ty::Prim(Prim::Int),
            kind: ExprKind::Lit(Lit::Int(1)),
        };
        let update_index = gen_id_add_update(self.assigner, &index_id, update_expr);

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = Expr {
            id: self.assigner.next_node(),
            span: iterable_span,
            ty: Ty::Prim(Prim::Bool),
            kind: ExprKind::BinOp(
                BinOp::Lt,
                Box::new(index_id.gen_local_ref(self.assigner)),
                Box::new(len_id.gen_local_ref(self.assigner)),
            ),
        };

        let while_stmt = Stmt {
            id: self.assigner.next_node(),
            span: Span::default(),
            kind: StmtKind::Expr(Expr {
                id: self.assigner.next_node(),
                span,
                ty: Ty::UNIT,
                kind: ExprKind::While(Box::new(cond), block),
            }),
        };

        Expr {
            id: self.assigner.next_node(),
            span,
            ty: Ty::UNIT,
            kind: ExprKind::Block(Block {
                id: self.assigner.next_node(),
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

        let range_id = self.gen_ident("range_id", Ty::Prim(Prim::Range), iterable_span);
        let range_capture = range_id.gen_id_init(Mutability::Immutable, *iterable, self.assigner);

        let index_id = self.gen_ident("index_id", Ty::Prim(Prim::Int), iterable_span);
        let index_init = index_id.gen_steppable_id_init(
            Mutability::Mutable,
            range_id.gen_field_access(PrimField::Start, self.assigner),
            self.assigner,
        );

        let step_id = self.gen_ident("step_id", Ty::Prim(Prim::Int), iterable_span);
        let step_init = step_id.gen_id_init(
            Mutability::Immutable,
            range_id.gen_field_access(PrimField::Step, self.assigner),
            self.assigner,
        );

        let end_id = self.gen_ident("end_id", Ty::Prim(Prim::Int), iterable_span);
        let end_init = end_id.gen_id_init(
            Mutability::Immutable,
            range_id.gen_field_access(PrimField::End, self.assigner),
            self.assigner,
        );

        let pat_init = Stmt {
            id: self.assigner.next_node(),
            span: iter.span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                iter,
                index_id.gen_local_ref(self.assigner),
            ),
        };

        let update_expr = step_id.gen_local_ref(self.assigner);
        let update_index = gen_id_add_update(self.assigner, &index_id, update_expr);

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = gen_range_cond(self.assigner, &index_id, &step_id, &end_id, iterable_span);

        let while_stmt = Stmt {
            id: self.assigner.next_node(),
            span: Span::default(),
            kind: StmtKind::Expr(Expr {
                id: self.assigner.next_node(),
                span,
                ty: Ty::UNIT,
                kind: ExprKind::While(Box::new(cond), block),
            }),
        };

        Expr {
            id: self.assigner.next_node(),
            span,
            ty: Ty::UNIT,
            kind: ExprKind::Block(Block {
                id: self.assigner.next_node(),
                span,
                ty: Ty::UNIT,
                stmts: vec![range_capture, index_init, step_init, end_init, while_stmt],
            }),
        }
    }

    fn gen_ident(&mut self, label: &str, ty: Ty, span: Span) -> IdentTemplate {
        let id = self.assigner.next_node();
        IdentTemplate {
            id,
            span,
            ty,
            name: generated_name(&format!("{label}_{id}")),
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
                    Ty::Prim(Prim::Range) => {
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
    assigner: &mut Assigner,
    index: &IdentTemplate,
    step: &IdentTemplate,
    end: &IdentTemplate,
    span: Span,
) -> Expr {
    Expr {
        id: assigner.next_node(),
        span,
        ty: Ty::Prim(Prim::Bool),
        kind: ExprKind::BinOp(
            BinOp::OrL,
            Box::new(Expr {
                id: assigner.next_node(),
                span,
                ty: Ty::Prim(Prim::Bool),
                kind: ExprKind::BinOp(
                    BinOp::AndL,
                    Box::new(Expr {
                        id: assigner.next_node(),
                        span,
                        ty: Ty::Prim(Prim::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Gt,
                            Box::new(step.gen_local_ref(assigner)),
                            Box::new(Expr {
                                id: assigner.next_node(),
                                span,
                                ty: Ty::Prim(Prim::Int),
                                kind: ExprKind::Lit(Lit::Int(0)),
                            }),
                        ),
                    }),
                    Box::new(Expr {
                        id: assigner.next_node(),
                        span,
                        ty: Ty::Prim(Prim::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Lte,
                            Box::new(index.gen_local_ref(assigner)),
                            Box::new(end.gen_local_ref(assigner)),
                        ),
                    }),
                ),
            }),
            Box::new(Expr {
                id: assigner.next_node(),
                span,
                ty: Ty::Prim(Prim::Bool),
                kind: ExprKind::BinOp(
                    BinOp::AndL,
                    Box::new(Expr {
                        id: assigner.next_node(),
                        span,
                        ty: Ty::Prim(Prim::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Lt,
                            Box::new(step.gen_local_ref(assigner)),
                            Box::new(Expr {
                                id: assigner.next_node(),
                                span,
                                ty: Ty::Prim(Prim::Int),
                                kind: ExprKind::Lit(Lit::Int(0)),
                            }),
                        ),
                    }),
                    Box::new(Expr {
                        id: assigner.next_node(),
                        span,
                        ty: Ty::Prim(Prim::Bool),
                        kind: ExprKind::BinOp(
                            BinOp::Gte,
                            Box::new(index.gen_local_ref(assigner)),
                            Box::new(end.gen_local_ref(assigner)),
                        ),
                    }),
                ),
            }),
        ),
    }
}

fn gen_id_add_update(assigner: &mut Assigner, ident: &IdentTemplate, expr: Expr) -> Stmt {
    Stmt {
        id: assigner.next_node(),
        span: ident.span,
        kind: StmtKind::Semi(Expr {
            id: assigner.next_node(),
            span: ident.span,
            ty: Ty::UNIT,
            kind: ExprKind::AssignOp(
                BinOp::Add,
                Box::new(ident.gen_local_ref(assigner)),
                Box::new(expr),
            ),
        }),
    }
}
