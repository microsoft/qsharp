// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        Block, Expr, ExprKind, Ident, Mutability, NodeId, Pat, PatKind, Path, QubitInit,
        QubitInitKind, Stmt, StmtKind,
    },
    mut_visit::{walk_expr, walk_stmt, MutVisitor},
};
use std::mem::take;

fn remove_extra_parens(pat: Pat) -> Pat {
    match pat.kind {
        PatKind::Bind(_, _) | PatKind::Discard(_) | PatKind::Elided => pat,
        PatKind::Paren(p) => remove_extra_parens(*p),
        PatKind::Tuple(ps) => Pat {
            id: pat.id,
            span: pat.span,
            kind: PatKind::Tuple(ps.into_iter().map(remove_extra_parens).collect()),
        },
    }
}

struct QubitIdent {
    id: Ident,
    is_array: bool,
}

pub struct ReplaceQubitAllocation {
    qubits_curr_callable: Vec<Vec<QubitIdent>>,
    qubits_curr_block: Vec<QubitIdent>,
    prefix_qubits: Vec<QubitIdent>,
    gen_id_count: u32,
}

impl ReplaceQubitAllocation {
    #[must_use]
    pub fn new() -> ReplaceQubitAllocation {
        ReplaceQubitAllocation {
            qubits_curr_callable: Vec::new(),
            qubits_curr_block: Vec::new(),
            prefix_qubits: Vec::new(),
            gen_id_count: 0,
        }
    }

    fn process_qubit_stmt(
        &mut self,
        stmt_span: Span,
        pat: Pat,
        mut init: QubitInit,
        block: Option<Block>,
    ) -> Vec<Stmt> {
        fn is_non_tuple(init: &mut QubitInit) -> (bool, Option<Expr>) {
            match &mut init.kind {
                QubitInitKind::Array(e) => (true, Some(take(e))),
                QubitInitKind::Paren(_) => is_non_tuple(init),
                QubitInitKind::Single => (true, None),
                QubitInitKind::Tuple(_) => (false, None),
            }
        }

        let mut new_stmts: Vec<Stmt> = vec![];
        let mut new_ids: Vec<QubitIdent> = vec![];

        if let (true, opt) = is_non_tuple(&mut init) {
            if let PatKind::Bind(id, _) = remove_extra_parens(pat).kind {
                new_ids.push(QubitIdent {
                    id: id.clone(),
                    is_array: opt.is_some(),
                });
                new_stmts.push(match opt {
                    Some(mut size) => {
                        self.visit_expr(&mut size);
                        create_array_alloc_stmt(&id, size)
                    }
                    None => create_alloc_stmt(&id),
                });
            } else {
                panic!("Shape of identifier pattern doesn't match shape of initializer");
            }
        } else {
            let (assignment_expr, mut ids) = self.process_qubit_init(init);
            new_stmts = ids
                .iter_mut()
                .map(|(id, size)| match size {
                    Some(size) => {
                        self.visit_expr(size);
                        create_array_alloc_stmt(id, size.clone())
                    }
                    None => create_alloc_stmt(id),
                })
                .collect();
            new_ids = ids
                .into_iter()
                .map(|(id, expr)| QubitIdent {
                    id,
                    is_array: expr.is_some(),
                })
                .collect();
            new_stmts.push(Stmt {
                id: NodeId::default(),
                span: stmt_span,
                kind: StmtKind::Local(
                    Mutability::Immutable,
                    remove_extra_parens(pat),
                    assignment_expr,
                ),
            });
        }

        if let Some(mut block) = block {
            self.prefix_qubits = new_ids;
            block.stmts.splice(0..0, new_stmts);
            self.visit_block(&mut block);
            vec![Stmt {
                id: NodeId::default(),
                span: stmt_span,
                kind: StmtKind::Expr(Expr {
                    id: NodeId::default(),
                    span: stmt_span,
                    kind: ExprKind::Block(block),
                }),
            }]
        } else {
            self.qubits_curr_block.extend(new_ids);
            new_stmts
        }
    }

    fn process_qubit_init(&mut self, init: QubitInit) -> (Expr, Vec<(Ident, Option<Expr>)>) {
        match init.kind {
            QubitInitKind::Array(size) => {
                let gen_id = self.gen_ident(init.span);
                let expr = Expr {
                    id: NodeId::default(),
                    span: init.span,
                    kind: ExprKind::Path(Path {
                        id: NodeId::default(),
                        span: init.span,
                        namespace: None,
                        name: gen_id.clone(),
                    }),
                };
                (expr, vec![(gen_id, Some(*size))])
            }
            QubitInitKind::Paren(i) => self.process_qubit_init(*i),
            QubitInitKind::Single => {
                let gen_id = self.gen_ident(init.span);
                let expr = Expr {
                    id: NodeId::default(),
                    span: init.span,
                    kind: ExprKind::Path(Path {
                        id: NodeId::default(),
                        span: init.span,
                        namespace: None,
                        name: gen_id.clone(),
                    }),
                };
                (expr, vec![(gen_id, None)])
            }
            QubitInitKind::Tuple(inits) => {
                let mut exprs: Vec<Expr> = vec![];
                let mut ids: Vec<(Ident, Option<Expr>)> = vec![];
                for i in inits {
                    let (sub_expr, sub_ids) = self.process_qubit_init(i);
                    exprs.push(sub_expr);
                    ids.extend(sub_ids);
                }
                let tuple_expr = Expr {
                    id: NodeId::default(),
                    span: init.span,
                    kind: ExprKind::Tuple(exprs),
                };
                (tuple_expr, ids)
            }
        }
    }

    fn gen_ident(&mut self, span: Span) -> Ident {
        let new_id = Ident {
            id: NodeId::default(),
            span,
            name: format!("__generated_ident_{}__", self.gen_id_count),
        };
        self.gen_id_count += 1;
        new_id
    }

    fn is_qubits_empty(&self) -> bool {
        self.qubits_curr_block.is_empty()
            && self
                .qubits_curr_callable
                .iter()
                .all(std::vec::Vec::is_empty)
    }

    fn get_dealloc_stmts(qubits: &[QubitIdent]) -> Vec<Stmt> {
        qubits
            .iter()
            .rev()
            .map(|qubit| {
                if qubit.is_array {
                    create_array_dealloc_stmt(&qubit.id)
                } else {
                    create_dealloc_stmt(&qubit.id)
                }
            })
            .collect()
    }

    fn get_dealloc_stmts_for_block(&self) -> Vec<Stmt> {
        ReplaceQubitAllocation::get_dealloc_stmts(&self.qubits_curr_block)
    }

    fn get_dealloc_stmts_for_callable(&self) -> Vec<Stmt> {
        let mut stmts = ReplaceQubitAllocation::get_dealloc_stmts(&self.qubits_curr_block);
        stmts.extend(
            self.qubits_curr_callable
                .iter()
                .rev()
                .flat_map(|q| ReplaceQubitAllocation::get_dealloc_stmts(q)),
        );
        stmts
    }
}

impl Default for ReplaceQubitAllocation {
    fn default() -> Self {
        Self::new()
    }
}

impl MutVisitor for ReplaceQubitAllocation {
    fn visit_block(&mut self, block: &mut Block) {
        let qubits_super_block = take(&mut self.qubits_curr_block);
        self.qubits_curr_callable.push(qubits_super_block);
        self.qubits_curr_block = take(&mut self.prefix_qubits);

        // walk block
        let old_stmts = take(&mut block.stmts);
        for mut stmt in old_stmts {
            if let StmtKind::Qubit(_, pat, init, qubit_scope) = stmt.kind {
                block
                    .stmts
                    .extend(self.process_qubit_stmt(stmt.span, pat, init, qubit_scope));
            } else {
                walk_stmt(self, &mut stmt);
                block.stmts.push(stmt);
            }
        }

        if !self.qubits_curr_block.is_empty() {
            let new_end_stmt: Option<Stmt> = match block.stmts.last_mut() {
                Some(s) => {
                    if let StmtKind::Expr(end) = &mut s.kind {
                        let end_capture = self.gen_ident(end.span);
                        *s = Stmt {
                            id: NodeId::default(),
                            span: s.span,
                            kind: StmtKind::Local(
                                Mutability::Immutable,
                                Pat {
                                    id: NodeId::default(),
                                    span: end.span,
                                    kind: PatKind::Bind(end_capture.clone(), None),
                                },
                                take(end),
                            ),
                        };
                        Some(Stmt {
                            id: NodeId::default(),
                            span: s.span,
                            kind: StmtKind::Expr(Expr {
                                id: NodeId::default(),
                                span: s.span,
                                kind: ExprKind::Path(Path {
                                    id: NodeId::default(),
                                    span: end_capture.span,
                                    namespace: None,
                                    name: end_capture,
                                }),
                            }),
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            };

            block.stmts.extend(self.get_dealloc_stmts_for_block());
            if let Some(end) = new_end_stmt {
                block.stmts.push(end);
            }
        }

        self.qubits_curr_block = self
            .qubits_curr_callable
            .pop()
            .expect("missing expected vector of qubits identifiers");
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        match &mut expr.kind {
            ExprKind::Return(e) => {
                if self.is_qubits_empty() {
                    self.visit_expr(e);
                } else {
                    let rtrn_capture = self.gen_ident(e.span);
                    self.visit_expr(e);
                    let mut stmts: Vec<Stmt> = vec![];
                    stmts.push(Stmt {
                        id: NodeId::default(),
                        span: e.span,
                        kind: StmtKind::Local(
                            Mutability::Immutable,
                            Pat {
                                id: NodeId::default(),
                                span: e.span,
                                kind: PatKind::Bind(rtrn_capture.clone(), None),
                            },
                            take(e),
                        ),
                    });
                    stmts.extend(self.get_dealloc_stmts_for_callable());
                    stmts.push(Stmt {
                        id: NodeId::default(),
                        span: expr.span,
                        kind: StmtKind::Semi(Expr {
                            id: NodeId::default(),
                            span: expr.span,
                            kind: ExprKind::Return(Box::new(Expr {
                                id: NodeId::default(),
                                span: rtrn_capture.span,
                                kind: ExprKind::Path(Path {
                                    id: NodeId::default(),
                                    span: rtrn_capture.span,
                                    namespace: None,
                                    name: rtrn_capture,
                                }),
                            })),
                        }),
                    });
                    let new_expr = Expr {
                        id: NodeId::default(),
                        span: expr.span,
                        kind: ExprKind::Block(Block {
                            id: NodeId::default(),
                            span: expr.span,
                            stmts,
                        }),
                    };
                    *expr = new_expr;
                }
            }
            ExprKind::Lambda(_, _, e) => {
                let super_block_qubits = take(&mut self.qubits_curr_block);
                let super_callable_qubits = take(&mut self.qubits_curr_callable);
                self.visit_expr(e);
                self.qubits_curr_callable = super_callable_qubits;
                self.qubits_curr_block = super_block_qubits;
            }
            _ => walk_expr(self, expr),
        }
    }
}

fn create_general_alloc_stmt(func_name: String, ident: &Ident, array_size: Option<Expr>) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: ident.span,
        kind: StmtKind::Local(
            Mutability::Immutable,
            Pat {
                id: NodeId::default(),
                span: ident.span,
                kind: PatKind::Bind(ident.clone(), None),
            },
            Expr {
                id: NodeId::default(),
                span: ident.span,
                kind: ExprKind::Call(
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: ident.span,
                        kind: ExprKind::Path(Path {
                            id: NodeId::default(),
                            span: ident.span,
                            namespace: Some(Ident {
                                id: NodeId::default(),
                                span: ident.span,
                                name: "QIR.Runtime".to_owned(),
                            }),
                            name: Ident {
                                id: NodeId::default(),
                                span: ident.span,
                                name: func_name,
                            },
                        }),
                    }),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: ident.span,
                        kind: ExprKind::Tuple(match array_size {
                            Some(size) => vec![size],
                            None => vec![],
                        }),
                    }),
                ),
            },
        ),
    }
}

fn create_array_alloc_stmt(ident: &Ident, array_size: Expr) -> Stmt {
    create_general_alloc_stmt(
        "__quantum__rt__qubit_array_allocate".to_owned(),
        ident,
        Some(array_size),
    )
}

fn create_alloc_stmt(ident: &Ident) -> Stmt {
    create_general_alloc_stmt("__quantum__rt__qubit_allocate".to_owned(), ident, None)
}

fn create_general_dealloc_stmt(func_name: String, ident: &Ident) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: ident.span,
        kind: StmtKind::Semi(Expr {
            id: NodeId::default(),
            span: ident.span,
            kind: ExprKind::Call(
                Box::new(Expr {
                    id: NodeId::default(),
                    span: ident.span,
                    kind: ExprKind::Path(Path {
                        id: NodeId::default(),
                        span: ident.span,
                        namespace: Some(Ident {
                            id: NodeId::default(),
                            span: ident.span,
                            name: "QIR.Runtime".to_owned(),
                        }),
                        name: Ident {
                            id: NodeId::default(),
                            span: ident.span,
                            name: func_name,
                        },
                    }),
                }),
                Box::new(Expr {
                    id: NodeId::default(),
                    span: ident.span,
                    kind: ExprKind::Tuple(vec![Expr {
                        id: NodeId::default(),
                        span: ident.span,
                        kind: ExprKind::Path(Path {
                            id: NodeId::default(),
                            span: ident.span,
                            namespace: None,
                            name: ident.clone(),
                        }),
                    }]),
                }),
            ),
        }),
    }
}

fn create_array_dealloc_stmt(ident: &Ident) -> Stmt {
    create_general_dealloc_stmt("__quantum__rt__qubit_array_release".to_owned(), ident)
}

fn create_dealloc_stmt(ident: &Ident) -> Stmt {
    create_general_dealloc_stmt("__quantum__rt__qubit_release".to_owned(), ident)
}

// This function is a temporary workaround until we have full type information on identifiers
// fn assign_qubit_type<'a>(pat: &'a Pat, init: &'a QubitInit) -> Vec<(&'a Ident, &'a QubitInit)> {
//     let init_no_parens = remove_extra_init_parens(init);

//     match &pat.kind {
//         PatKind::Bind(name, _) => vec![(name, init_no_parens)],
//         PatKind::Discard(_) => vec![],
//         PatKind::Elided => todo!("error state for `use` statements"),
//         PatKind::Paren(pat) => assign_qubit_type(pat, init_no_parens),
//         PatKind::Tuple(tup) => {
//             if let QubitInitKind::Tuple(init_tup) = &init_no_parens.kind {
//                 assert!(
//                     tup.len() == init_tup.len(),
//                     "qubit tuple initializer length doesn't match identifier tuple length"
//                 );
//                 tup.iter()
//                     .zip(init_tup.iter())
//                     .flat_map(|(pat, init)| assign_qubit_type(pat, init))
//                     .collect()
//             } else {
//                 panic!("cannot initialize an identifier tuple with non-tuple qubit initializer");
//             }
//         }
//     }
// }

// fn remove_extra_init_parens(init: &QubitInit) -> &QubitInit {
//     match &init.kind {
//         QubitInitKind::Paren(p) => remove_extra_init_parens(p),
//         _ => init,
//     }
// }
