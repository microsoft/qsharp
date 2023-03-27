// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//#[cfg(test)]
//mod tests;

use std::mem::{swap, take};

//use miette::Diagnostic;
use qsc_ast::{
    ast::{
        Block, Expr, ExprKind, Ident, Mutability, NodeId, Pat, PatKind, Path, QubitInit,
        QubitInitKind, Span, Stmt, StmtKind,
    },
    mut_visit::{walk_block, walk_stmt, MutVisitor},
};
//use thiserror::Error;

// #[derive(Clone, Debug, Diagnostic, Error)]
// pub(super) enum Error {
//     #[error("adjointable/controllable operation `{0}` must return Unit")]
//     NonUnitReturn(String, #[label("must return Unit")] Span),

//     #[error("callable parameter `{0}` must be type annotated")]
//     ParameterNotTyped(String, #[label("missing type annotation")] Span),

//     #[error("{0} are not currently supported")]
//     NotCurrentlySupported(&'static str, #[label("not currently supported")] Span),
// }

// pub(super) fn validate(package: &Package) -> Vec<Error> {
//     let mut validator = Validator {
//         validation_errors: Vec::new(),
//     };
//     validator.visit_package(package);
//     validator.validation_errors
// }

pub struct ReplaceQubitAllocation {
    //qubits_per_blocks_stack: Vec<Option<Vec<Ident>>>,
    qubits_curr_block: Vec<Ident>,
    prefix_qubits: Vec<Ident>,
}

impl ReplaceQubitAllocation {
    pub fn new() -> ReplaceQubitAllocation {
        ReplaceQubitAllocation {
            //qubits_per_blocks_stack: Vec::new(),
            qubits_curr_block: Vec::new(),
            prefix_qubits: Vec::new(),
        }
    }
}

impl ReplaceQubitAllocation {
    fn process_qubit_stmt(&mut self, pat: Pat, init: QubitInit, block: Option<Block>) -> Vec<Stmt> {
        let qs_w_ty = assign_qubit_type(&pat, &init);

        // assume single qubit ids
        match block {
            Some(mut block) => {
                for q in &qs_w_ty {
                    self.prefix_qubits.push(q.0.clone());
                }
                self.visit_block(&mut block);
                vec![Stmt {
                    id: NodeId::zero(),
                    span: Span::default(),
                    kind: StmtKind::Expr(Expr {
                        id: NodeId::zero(),
                        span: Span::default(),
                        kind: ExprKind::Block(block),
                    }),
                }]
            }
            None => qs_w_ty
                .iter()
                .map(|q| {
                    self.qubits_curr_block.push(q.0.clone());
                    create_alloc_stmt(q.0)
                })
                .collect(),
        }

        //walk_stmt(self, stmt);
    }
}

impl MutVisitor for ReplaceQubitAllocation {
    fn visit_block(&mut self, block: &mut Block) {
        let qubits_super_block = take(&mut self.qubits_curr_block);

        let mut prefix_stmts = Vec::new();
        for qubit in &self.prefix_qubits {
            prefix_stmts.push(create_alloc_stmt(qubit));
        }
        self.qubits_curr_block.extend(take(&mut self.prefix_qubits));
        if !prefix_stmts.is_empty() {
            block.stmts.splice(0..0, prefix_stmts);
        }

        // walk block
        let old_stmts = take(&mut block.stmts);
        for mut stmt in old_stmts {
            if let StmtKind::Qubit(_, pat, init, qubit_scope) = stmt.kind {
                block
                    .stmts
                    .extend(self.process_qubit_stmt(pat, init, qubit_scope));
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

    // fn visit_stmt(&mut self, stmt: &mut Stmt) {
    //     if let StmtKind::Qubit(_, pat, init, block) = &mut stmt.kind {
    //         let qs_w_ty = assign_qubit_type(pat, init);

    //         // assume single qubit ids
    //         match block {
    //             Some(block) => {
    //                 for q in &qs_w_ty {
    //                     self.prefix_qubits.push(q.0.clone());
    //                 }
    //                 self.visit_block(block);
    //                 *stmt = Stmt {
    //                     id: stmt.id,
    //                     span: stmt.span,
    //                     kind: StmtKind::Expr(Expr {
    //                         id: NodeId::zero(),
    //                         span: stmt.span,
    //                         kind: ExprKind::Block(*block),
    //                     }),
    //                 };
    //             }
    //             None => {
    //                 for q in &qs_w_ty {
    //                     self.qubits_curr_block.push(q.0.clone());
    //                     *stmt = create_alloc_stmt(q.0);
    //                 }
    //             }
    //         }
    //     }

    //     walk_stmt(self, stmt);
    // }
}

fn create_alloc_stmt(qubit: &Ident) -> Stmt {
    Stmt {
        id: NodeId::zero(),
        span: Span::default(),
        kind: StmtKind::Local(
            Mutability::Immutable,
            Pat {
                id: NodeId::zero(),
                span: Span::default(),
                kind: PatKind::Bind(qubit.clone(), None),
            },
            Expr {
                id: NodeId::zero(),
                span: Span::default(),
                kind: ExprKind::Call(
                    Box::new(Expr {
                        id: NodeId::zero(),
                        span: Span::default(),
                        kind: ExprKind::Path(Path {
                            id: NodeId::zero(),
                            span: Span::default(),
                            namespace: Some(Ident {
                                id: NodeId::zero(),
                                span: Span::default(),
                                name: "QIR.Runtime".to_owned(),
                            }),
                            name: Ident {
                                id: NodeId::zero(),
                                span: Span::default(),
                                name: "__quantum__rt__qubit_allocate".to_owned(),
                            },
                        }),
                    }),
                    Box::new(Expr {
                        id: NodeId::zero(),
                        span: Span::default(),
                        kind: ExprKind::Tuple(vec![]),
                    }),
                ),
            },
        ),
    }
}

fn create_dealloc_stmt(qubit: &Ident) -> Stmt {
    Stmt {
        id: NodeId::zero(),
        span: Span::default(),
        kind: StmtKind::Semi(Expr {
            id: NodeId::zero(),
            span: Span::default(),
            kind: ExprKind::Call(
                Box::new(Expr {
                    id: NodeId::zero(),
                    span: Span::default(),
                    kind: ExprKind::Path(Path {
                        id: NodeId::zero(),
                        span: Span::default(),
                        namespace: Some(Ident {
                            id: NodeId::zero(),
                            span: Span::default(),
                            name: "QIR.Runtime".to_owned(),
                        }),
                        name: Ident {
                            id: NodeId::zero(),
                            span: Span::default(),
                            name: "__quantum__rt__qubit_release".to_owned(),
                        },
                    }),
                }),
                Box::new(Expr {
                    id: NodeId::zero(),
                    span: Span::default(),
                    kind: ExprKind::Tuple(vec![Expr {
                        id: NodeId::zero(),
                        span: Span::default(),
                        kind: ExprKind::Path(Path {
                            id: NodeId::zero(),
                            span: Span::default(),
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
fn assign_qubit_type<'a>(pat: &'a Pat, init: &'a QubitInit) -> Vec<(&'a Ident, &'a QubitInit)> {
    let init_no_parens = remove_extra_init_parens(init);

    match &pat.kind {
        PatKind::Bind(name, _) => vec![(name, init_no_parens)],
        PatKind::Discard(_) => vec![],
        PatKind::Elided => todo!("error state for `use` statements"),
        PatKind::Paren(pat) => assign_qubit_type(pat, init_no_parens),
        PatKind::Tuple(tup) => {
            if let QubitInitKind::Tuple(init_tup) = &init_no_parens.kind {
                assert!(
                    tup.len() == init_tup.len(),
                    "qubit tuple initializer length doesn't match identifier tuple length"
                );
                tup.iter()
                    .zip(init_tup.iter())
                    .flat_map(|(pat, init)| assign_qubit_type(pat, init))
                    .collect()
            } else {
                panic!("cannot initialize an identifier tuple with non-tuple qubit initializer");
            }
        }
    }
}

fn remove_extra_init_parens(init: &QubitInit) -> &QubitInit {
    if let QubitInitKind::Paren(p) = &init.kind {
        return remove_extra_init_parens(p);
    }
    init
}
