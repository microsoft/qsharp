// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//#[cfg(test)]
//mod tests;

use std::mem::{replace, take};

//use miette::Diagnostic;
use qsc_ast::{
    ast::{
        Block, Expr, ExprKind, Ident, Mutability, NodeId, Pat, PatKind, Path, QubitInit,
        QubitInitKind, Span, Stmt, StmtKind,
    },
    mut_visit::{walk_stmt, MutVisitor},
};

pub struct ReplaceQubitAllocation {
    qubits_curr_block: Vec<Ident>,
    prefix_qubits: Vec<Ident>,
    gen_id_count: u32,
}

impl ReplaceQubitAllocation {
    pub fn new() -> ReplaceQubitAllocation {
        ReplaceQubitAllocation {
            qubits_curr_block: Vec::new(),
            prefix_qubits: Vec::new(),
            gen_id_count: 0,
        }
    }
}

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

// fn remove_extra_parens_mut(pat: &mut Pat) {
//     match &mut pat.kind {
//         PatKind::Bind(_, _) | PatKind::Discard(_) | PatKind::Elided => {}
//         PatKind::Paren(p) => {
//             remove_extra_parens_mut(p);
//             *pat = replace(
//                 p,
//                 Pat {
//                     id: NodeId::default(),
//                     span: Span::default(),
//                     kind: PatKind::Elided,
//                 },
//             );
//         }
//         PatKind::Tuple(ps) => ps.iter_mut().for_each(remove_extra_parens_mut),
//     }
// }

impl ReplaceQubitAllocation {
    fn process_qubit_stmt(
        &mut self,
        stmt_span: Span,
        pat: Pat,
        init: QubitInit,
        block: Option<Block>,
    ) -> Vec<Stmt> {
        let (assignment_expr, ids) = self.process_qubit_init(init);
        let mut new_stmts: Vec<Stmt> = ids.iter().map(create_alloc_stmt).collect();
        new_stmts.push(Stmt {
            id: NodeId::default(),
            span: stmt_span,
            kind: StmtKind::Local(
                Mutability::Immutable,
                remove_extra_parens(pat),
                assignment_expr,
            ),
        });

        if let Some(mut block) = block {
            self.prefix_qubits = ids;
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
            self.qubits_curr_block.extend(ids);
            new_stmts
        }
    }

    fn process_qubit_init(&mut self, init: QubitInit) -> (Expr, Vec<Ident>) {
        match init.kind {
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
                (expr, vec![gen_id])
            }
            QubitInitKind::Tuple(inits) => {
                let mut exprs: Vec<Expr> = vec![];
                let mut ids: Vec<Ident> = vec![];
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
            QubitInitKind::Array(_) => todo!(),
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
}

impl MutVisitor for ReplaceQubitAllocation {
    fn visit_block(&mut self, block: &mut Block) {
        let qubits_super_block = take(&mut self.qubits_curr_block);
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

        while let Some(qubit) = &self.qubits_curr_block.pop() {
            block.stmts.push(create_dealloc_stmt(qubit));
        }
        self.qubits_curr_block = qubits_super_block;
    }
}

fn create_alloc_stmt(qubit: &Ident) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: qubit.span,
        kind: StmtKind::Local(
            Mutability::Immutable,
            Pat {
                id: NodeId::default(),
                span: qubit.span,
                kind: PatKind::Bind(qubit.clone(), None),
            },
            Expr {
                id: NodeId::default(),
                span: qubit.span,
                kind: ExprKind::Call(
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: qubit.span,
                        kind: ExprKind::Path(Path {
                            id: NodeId::default(),
                            span: qubit.span,
                            namespace: Some(Ident {
                                id: NodeId::default(),
                                span: qubit.span,
                                name: "QIR.Runtime".to_owned(),
                            }),
                            name: Ident {
                                id: NodeId::default(),
                                span: qubit.span,
                                name: "__quantum__rt__qubit_allocate".to_owned(),
                            },
                        }),
                    }),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: qubit.span,
                        kind: ExprKind::Tuple(vec![]),
                    }),
                ),
            },
        ),
    }
}

fn create_dealloc_stmt(qubit: &Ident) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: qubit.span,
        kind: StmtKind::Semi(Expr {
            id: NodeId::default(),
            span: qubit.span,
            kind: ExprKind::Call(
                Box::new(Expr {
                    id: NodeId::default(),
                    span: qubit.span,
                    kind: ExprKind::Path(Path {
                        id: NodeId::default(),
                        span: qubit.span,
                        namespace: Some(Ident {
                            id: NodeId::default(),
                            span: qubit.span,
                            name: "QIR.Runtime".to_owned(),
                        }),
                        name: Ident {
                            id: NodeId::default(),
                            span: qubit.span,
                            name: "__quantum__rt__qubit_release".to_owned(),
                        },
                    }),
                }),
                Box::new(Expr {
                    id: NodeId::default(),
                    span: qubit.span,
                    kind: ExprKind::Tuple(vec![Expr {
                        id: NodeId::default(),
                        span: qubit.span,
                        kind: ExprKind::Path(Path {
                            id: NodeId::default(),
                            span: qubit.span,
                            namespace: None,
                            name: qubit.clone(),
                        }),
                    }]),
                }),
            ),
        }),
    }
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
