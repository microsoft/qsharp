// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::panic;
use std::{mem::take, rc::Rc};

use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    hir::{
        BinOp, Block, Expr, ExprKind, Field, Ident, Lit, Mutability, NodeId, Pat, PatKind,
        PrimField, PrimTy, Res, Stmt, StmtKind, Ty, UnOp,
    },
    mut_visit::{walk_expr, MutVisitor},
};

use crate::Error;

#[cfg(test)]
mod tests;

pub fn loop_unification(unit: &mut CompileUnit) -> Vec<Error> {
    let mut pass = LoopUni {
        assigner: &mut unit.assigner,
    };
    pass.visit_package(&mut unit.package);
    vec![]
}

struct IdentTemplate {
    id: NodeId,
    span: Span,
    ty: Ty,
}

impl IdentTemplate {
    fn gen_local_ref(&self) -> Expr {
        Expr {
            id: NodeId::default(),
            span: self.span,
            ty: self.ty.clone(),
            kind: ExprKind::Var(Res::Local(self.id)),
        }
    }

    fn gen_pat(&self, label: &str) -> Pat {
        Pat {
            id: NodeId::default(),
            span: self.span,
            ty: self.ty.clone(),
            kind: PatKind::Bind(Ident {
                id: self.id,
                span: self.span,
                name: Rc::from(format!("{label}_{}", self.id)),
            }),
        }
    }
}

struct LoopUni<'a> {
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

        let continue_cond_id = self.gen_ident(Ty::Prim(PrimTy::Bool), cond_span);
        let continue_cond_init = gen_id_init(
            Mutability::Mutable,
            &continue_cond_id,
            "continue_cond",
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

        let array_id = self.gen_ident(iterable.ty.clone(), iterable_span);
        let array_capture = gen_id_init(Mutability::Immutable, &array_id, "array_id", *iterable);

        let len_id = self.gen_ident(Ty::Prim(PrimTy::Int), iterable_span);
        let len_capture = gen_id_init(
            Mutability::Immutable,
            &len_id,
            "len_id",
            gen_field_access(&array_id, PrimField::Length),
        );

        let index_id = self.gen_ident(Ty::Prim(PrimTy::Int), iterable_span);
        let index_init = gen_id_init(
            Mutability::Mutable,
            &index_id,
            "index_id",
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

        let range_id = self.gen_ident(Ty::Prim(PrimTy::Range), iterable_span);
        let range_capture = gen_id_init(Mutability::Immutable, &range_id, "range_id", *iterable);

        let index_id = self.gen_ident(Ty::Prim(PrimTy::Int), iterable_span);
        let index_init = gen_id_init(
            Mutability::Mutable,
            &index_id,
            "index_id",
            gen_field_access(&range_id, PrimField::Start),
        );

        let step_id = self.gen_ident(Ty::Prim(PrimTy::Int), iterable_span);
        let step_init = gen_id_init(
            Mutability::Immutable,
            &step_id,
            "step_id",
            gen_field_access(&range_id, PrimField::Step),
        );

        let end_id = self.gen_ident(Ty::Prim(PrimTy::Int), iterable_span);
        let end_init = gen_id_init(
            Mutability::Immutable,
            &end_id,
            "end_id",
            gen_field_access(&range_id, PrimField::End),
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

    fn gen_ident(&mut self, ty: Ty, span: Span) -> IdentTemplate {
        IdentTemplate {
            id: self.assigner.next_id(),
            span,
            ty,
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

fn gen_id_init(mutability: Mutability, ident: &IdentTemplate, label: &str, expr: Expr) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: ident.span,
        kind: StmtKind::Local(mutability, ident.gen_pat(label), expr),
    }
}

fn gen_field_access(container: &IdentTemplate, field: PrimField) -> Expr {
    Expr {
        id: NodeId::default(),
        span: container.span,
        ty: Ty::Prim(PrimTy::Int),
        kind: ExprKind::Field(Box::new(container.gen_local_ref()), Field::Prim(field)),
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
