// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::mem::take;

use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        Block, Expr, ExprKind, Ident, Lit, Mutability, NodeId, Pat, PatKind, Path, Stmt, StmtKind,
        UnOp,
    },
    mut_visit::MutVisitor,
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
}

impl Default for LoopUni {
    fn default() -> Self {
        Self::new()
    }
}

impl MutVisitor for LoopUni {
    #[allow(clippy::too_many_lines)]
    fn visit_expr(&mut self, expr: &mut qsc_hir::hir::Expr) {
        if let ExprKind::Repeat(block, cond, fixup) = &mut expr.kind {
            let continue_cond_id = self.gen_ident(cond.span);
            let continue_cond_init = Stmt {
                id: NodeId::default(),
                span: cond.span,
                kind: StmtKind::Local(
                    Mutability::Immutable,
                    Pat {
                        id: NodeId::default(),
                        span: cond.span,
                        kind: PatKind::Bind(continue_cond_id.clone(), None),
                    },
                    Expr {
                        id: NodeId::default(),
                        span: cond.span,
                        kind: ExprKind::Lit(Lit::Bool(true)),
                    },
                ),
            };
            let update = Stmt {
                id: NodeId::default(),
                span: cond.span,
                kind: StmtKind::Semi(Expr {
                    id: NodeId::default(),
                    span: cond.span,
                    kind: ExprKind::Assign(
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: cond.span,
                            kind: ExprKind::Path(Path {
                                id: NodeId::default(),
                                span: cond.span,
                                namespace: None,
                                name: continue_cond_id.clone(),
                            }),
                        }),
                        Box::new(Expr {
                            id: NodeId::default(),
                            span: cond.span,
                            kind: ExprKind::UnOp(UnOp::Neg, take(cond)),
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
                            fix_body.clone(),
                            None,
                        ),
                    }),
                };
                block.stmts.push(fix_if);
            }

            let new_block = Block {
                id: NodeId::default(),
                span: expr.span,
                stmts: vec![
                    continue_cond_init,
                    Stmt {
                        id: NodeId::default(),
                        span: expr.span,
                        kind: StmtKind::Expr(Expr {
                            id: NodeId::default(),
                            span: expr.span,
                            kind: ExprKind::While(
                                Box::new(Expr {
                                    id: NodeId::default(),
                                    span: expr.span,
                                    kind: ExprKind::Path(Path {
                                        id: NodeId::default(),
                                        span: cond.span,
                                        namespace: None,
                                        name: continue_cond_id,
                                    }),
                                }),
                                block.clone(),
                            ),
                        }),
                    },
                ],
            };

            *expr = Expr {
                id: NodeId::default(),
                span: expr.span,
                kind: ExprKind::Block(new_block),
            };
        }
    }
}
