// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use core::panic;
use std::mem::take;

use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        BinOp, Block, Expr, ExprKind, Ident, Lit, Mutability, NodeId, Pat, PatKind, Path, Stmt,
        StmtKind, UnOp,
    },
    mut_visit::{walk_expr, MutVisitor},
};

pub struct LoopUni {
    gen_id_count: u32,
}

impl LoopUni {
    #[must_use]
    pub fn new() -> LoopUni {
        LoopUni { gen_id_count: 0 }
    }

    fn gen_ident(&mut self, span: Span) -> Ident {
        let new_id = Ident {
            id: NodeId::default(),
            span,
            name: format!("__continue_cond_{}__", self.gen_id_count),
        };
        self.gen_id_count += 1;
        new_id
    }

    #[allow(clippy::too_many_lines)]
    fn visit_repeat(
        &mut self,
        mut block: Block,
        cond: Box<Expr>,
        fixup: Option<Block>,
        span: Span,
    ) -> Expr {
        let cond_span = cond.span;

        let continue_cond_id = self.gen_ident(cond_span);
        let continue_cond_init = LoopUni::gen_id_init(
            Mutability::Mutable,
            continue_cond_id.clone(),
            Expr {
                id: NodeId::default(),
                span: cond_span,
                kind: ExprKind::Lit(Lit::Bool(true)),
            },
        );

        let update = LoopUni::gen_id_update(
            continue_cond_id.clone(),
            Expr {
                id: NodeId::default(),
                span: cond_span,
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
                    kind: ExprKind::If(
                        Box::new(LoopUni::gen_path(None, continue_cond_id.clone())),
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
            stmts: vec![
                continue_cond_init,
                Stmt {
                    id: NodeId::default(),
                    span,
                    kind: StmtKind::Expr(Expr {
                        id: NodeId::default(),
                        span,
                        kind: ExprKind::While(
                            Box::new(LoopUni::gen_path(None, continue_cond_id)),
                            block,
                        ),
                    }),
                },
            ],
        };

        Expr {
            id: NodeId::default(),
            span,
            kind: ExprKind::Block(new_block),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn visit_for_array(
        &mut self,
        iter: Pat,
        iterable: Box<Expr>,
        mut block: Block,
        span: Span,
    ) -> Expr {
        let iterable_span = iterable.span;

        let array_id = self.gen_ident(iterable_span);
        let array_capture =
            LoopUni::gen_id_init(Mutability::Immutable, array_id.clone(), *iterable);

        let len_id = self.gen_ident(iterable_span);
        let len_capture = LoopUni::gen_id_init(
            Mutability::Immutable,
            len_id.clone(),
            LoopUni::gen_call(
                "Microsoft.Quantum.Core".to_owned(),
                "Length".to_owned(),
                array_id.clone(),
            ),
        );

        let index_id = self.gen_ident(iterable_span);
        let index_init = LoopUni::gen_id_init(
            Mutability::Mutable,
            index_id.clone(),
            Expr {
                id: NodeId::default(),
                span: iterable_span,
                kind: ExprKind::Lit(Lit::Int(0)),
            },
        );

        let pat_init = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                iter,
                Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::Index(
                        Box::new(LoopUni::gen_path(None, array_id)),
                        Box::new(LoopUni::gen_path(None, index_id.clone())),
                    ),
                },
            ),
        };

        let update_index = LoopUni::gen_id_update(
            index_id.clone(),
            Expr {
                id: NodeId::default(),
                span: iterable_span,
                kind: ExprKind::Lit(Lit::Int(1)),
            },
        );

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = Expr {
            id: NodeId::default(),
            span: iterable_span,
            kind: ExprKind::BinOp(
                BinOp::Lt,
                Box::new(LoopUni::gen_path(None, index_id)),
                Box::new(LoopUni::gen_path(None, len_id)),
            ),
        };

        let while_stmt = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Expr(Expr {
                id: NodeId::default(),
                span,
                kind: ExprKind::While(Box::new(cond), block),
            }),
        };

        Expr {
            id: NodeId::default(),
            span,
            kind: ExprKind::Block(Block {
                id: NodeId::default(),
                span,
                stmts: vec![array_capture, len_capture, index_init, while_stmt],
            }),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn visit_for_range(
        &mut self,
        iter: Pat,
        iterable: Box<Expr>,
        mut block: Block,
        span: Span,
    ) -> Expr {
        let iterable_span = iterable.span;

        let range_id = self.gen_ident(iterable_span);
        let range_capture =
            LoopUni::gen_id_init(Mutability::Immutable, range_id.clone(), *iterable);

        let index_id = self.gen_ident(iterable_span);
        let index_init = LoopUni::gen_id_init(
            Mutability::Mutable,
            index_id.clone(),
            LoopUni::gen_call(
                "Microsoft.Quantum.Core".to_owned(),
                "RangeStart".to_owned(),
                range_id.clone(),
            ),
        );

        let step_id = self.gen_ident(iterable_span);
        let step_init = LoopUni::gen_id_init(
            Mutability::Immutable,
            step_id.clone(),
            LoopUni::gen_call(
                "Microsoft.Quantum.Core".to_owned(),
                "RangeStep".to_owned(),
                range_id.clone(),
            ),
        );

        let end_id = self.gen_ident(iterable_span);
        let end_init = LoopUni::gen_id_init(
            Mutability::Immutable,
            end_id.clone(),
            LoopUni::gen_call(
                "Microsoft.Quantum.Core".to_owned(),
                "RangeEnd".to_owned(),
                range_id,
            ),
        );

        let pat_init = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                iter,
                LoopUni::gen_path(None, index_id.clone()),
            ),
        };

        let update_index =
            LoopUni::gen_id_update(index_id.clone(), LoopUni::gen_path(None, step_id.clone()));

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = Expr {
            id: NodeId::default(),
            span: iterable_span,
            kind: ExprKind::BinOp(
                BinOp::OrL,
                Box::new(Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::BinOp(
                        BinOp::AndL,
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::BinOp(
                                BinOp::Gt,
                                Box::new(LoopUni::gen_path(None, step_id.clone())),
                                Box::new(Expr {
                                    id: NodeId::default(),
                                    span: iterable_span,
                                    kind: ExprKind::Lit(Lit::Int(0)),
                                }),
                            ),
                        }),
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::BinOp(
                                BinOp::Lte,
                                Box::new(LoopUni::gen_path(None, index_id.clone())),
                                Box::new(LoopUni::gen_path(None, end_id.clone())),
                            ),
                        }),
                    ),
                }),
                Box::new(Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::BinOp(
                        BinOp::AndL,
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::BinOp(
                                BinOp::Lt,
                                Box::new(LoopUni::gen_path(None, step_id)),
                                Box::new(Expr {
                                    id: NodeId::default(),
                                    span: iterable_span,
                                    kind: ExprKind::Lit(Lit::Int(0)),
                                }),
                            ),
                        }),
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::BinOp(
                                BinOp::Gte,
                                Box::new(LoopUni::gen_path(None, index_id)),
                                Box::new(LoopUni::gen_path(None, end_id)),
                            ),
                        }),
                    ),
                }),
            ),
        };

        let while_stmt = Stmt {
            id: NodeId::default(),
            span,
            kind: StmtKind::Expr(Expr {
                id: NodeId::default(),
                span,
                kind: ExprKind::While(Box::new(cond), block),
            }),
        };

        Expr {
            id: NodeId::default(),
            span,
            kind: ExprKind::Block(Block {
                id: NodeId::default(),
                span,
                stmts: vec![range_capture, index_init, step_init, end_init, while_stmt],
            }),
        }
    }

    fn gen_path(namespace: Option<Ident>, name: Ident) -> Expr {
        Expr {
            id: NodeId::default(),
            span: name.span,
            kind: ExprKind::Path(Path {
                id: NodeId::default(),
                span: name.span,
                namespace,
                name,
            }),
        }
    }

    fn gen_id_init(mutability: Mutability, ident: Ident, expr: Expr) -> Stmt {
        Stmt {
            id: NodeId::default(),
            span: ident.span,
            kind: StmtKind::Local(
                mutability,
                Pat {
                    id: NodeId::default(),
                    span: ident.span,
                    kind: PatKind::Bind(ident, None),
                },
                expr,
            ),
        }
    }

    fn gen_call(call_namespace: String, call_name: String, arg: Ident) -> Expr {
        Expr {
            id: NodeId::default(),
            span: arg.span,
            kind: ExprKind::Call(
                Box::new(LoopUni::gen_path(
                    Some(Ident {
                        id: NodeId::default(),
                        span: arg.span,
                        name: call_namespace,
                    }),
                    Ident {
                        id: NodeId::default(),
                        span: arg.span,
                        name: call_name,
                    },
                )),
                Box::new(LoopUni::gen_path(None, arg)),
            ),
        }
    }

    fn gen_id_update(ident: Ident, expr: Expr) -> Stmt {
        Stmt {
            id: NodeId::default(),
            span: ident.span,
            kind: StmtKind::Semi(Expr {
                id: NodeId::default(),
                span: ident.span,
                kind: ExprKind::AssignOp(
                    BinOp::Add,
                    Box::new(LoopUni::gen_path(None, ident)),
                    Box::new(expr),
                ),
            }),
        }
    }
}

impl Default for LoopUni {
    fn default() -> Self {
        Self::new()
    }
}

impl MutVisitor for LoopUni {
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
