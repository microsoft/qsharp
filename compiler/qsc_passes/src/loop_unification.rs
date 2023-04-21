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
        let continue_cond_init = Stmt {
            id: NodeId::default(),
            span: cond_span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                Pat {
                    id: NodeId::default(),
                    span: cond_span,
                    kind: PatKind::Bind(continue_cond_id.clone(), None),
                },
                Expr {
                    id: NodeId::default(),
                    span: cond_span,
                    kind: ExprKind::Lit(Lit::Bool(true)),
                },
            ),
        };
        let update = Stmt {
            id: NodeId::default(),
            span: cond_span,
            kind: StmtKind::Semi(Expr {
                id: NodeId::default(),
                span: cond_span,
                kind: ExprKind::Assign(
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: cond_span,
                        kind: ExprKind::Path(Path {
                            id: NodeId::default(),
                            span: cond_span,
                            namespace: None,
                            name: continue_cond_id.clone(),
                        }),
                    }),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: cond_span,
                        kind: ExprKind::UnOp(UnOp::Neg, cond),
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
                    kind: ExprKind::If(
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: continue_cond_id.span,
                            kind: ExprKind::Path(Path {
                                id: NodeId::default(),
                                span: continue_cond_id.span,
                                namespace: None,
                                name: continue_cond_id.clone(),
                            }),
                        }),
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
                            Box::new(Expr {
                                id: NodeId::default(),
                                span,
                                kind: ExprKind::Path(Path {
                                    id: NodeId::default(),
                                    span: cond_span,
                                    namespace: None,
                                    name: continue_cond_id,
                                }),
                            }),
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
        let array_capture = Stmt {
            id: NodeId::default(),
            span: iterable_span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                Pat {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: PatKind::Bind(array_id.clone(), None),
                },
                *iterable,
            ),
        };
        let len_id = self.gen_ident(iterable_span);
        let len_capture = Stmt {
            id: NodeId::default(),
            span: iterable_span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                Pat {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: PatKind::Bind(len_id.clone(), None),
                },
                Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::Call(
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::Path(Path {
                                id: NodeId::default(),
                                span: iterable_span,
                                namespace: None,
                                name: Ident {
                                    id: NodeId::default(),
                                    span: iterable_span,
                                    name: "Length".to_owned(),
                                },
                            }),
                        }),
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::Paren(Box::new(Expr {
                                id: NodeId::default(),
                                span: iterable_span,
                                kind: ExprKind::Path(Path {
                                    id: NodeId::default(),
                                    span: iterable_span,
                                    namespace: None,
                                    name: len_id.clone(),
                                }),
                            })),
                        }),
                    ),
                },
            ),
        };

        let index_id = self.gen_ident(iterable_span);
        let index_init = Stmt {
            id: NodeId::default(),
            span: iterable_span,
            kind: StmtKind::Local(
                Mutability::Mutable,
                Pat {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: PatKind::Bind(index_id.clone(), None),
                },
                Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::Lit(Lit::Int(0)),
                },
            ),
        };

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
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::Path(Path {
                                id: NodeId::default(),
                                span: iterable_span,
                                namespace: None,
                                name: array_id,
                            }),
                        }),
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: iterable_span,
                            kind: ExprKind::Path(Path {
                                id: NodeId::default(),
                                span: iterable_span,
                                namespace: None,
                                name: index_id.clone(),
                            }),
                        }),
                    ),
                },
            ),
        };

        let update_index = Stmt {
            id: NodeId::default(),
            span: iterable_span,
            kind: StmtKind::Semi(Expr {
                id: NodeId::default(),
                span: iterable_span,
                kind: ExprKind::AssignOp(
                    BinOp::Add,
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: iterable_span,
                        kind: ExprKind::Path(Path {
                            id: NodeId::default(),
                            span: iterable_span,
                            namespace: None,
                            name: index_id.clone(),
                        }),
                    }),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: iterable_span,
                        kind: ExprKind::Lit(Lit::Int(1)),
                    }),
                ),
            }),
        };

        block.stmts.insert(0, pat_init);
        block.stmts.push(update_index);

        let cond = Expr {
            id: NodeId::default(),
            span: iterable_span,
            kind: ExprKind::BinOp(
                BinOp::Lt,
                Box::new(Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::Path(Path {
                        id: NodeId::default(),
                        span: iterable_span,
                        namespace: None,
                        name: index_id,
                    }),
                }),
                Box::new(Expr {
                    id: NodeId::default(),
                    span: iterable_span,
                    kind: ExprKind::Path(Path {
                        id: NodeId::default(),
                        span: iterable_span,
                        namespace: None,
                        name: len_id,
                    }),
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
        todo!();
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
