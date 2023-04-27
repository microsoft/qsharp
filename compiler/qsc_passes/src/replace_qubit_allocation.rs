// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        Block, Expr, ExprKind, Ident, Mutability, NodeId, Pat, PatKind, PrimTy, QubitInit,
        QubitInitKind, Res, Stmt, StmtKind, Ty,
    },
    mut_visit::{walk_expr, walk_stmt, MutVisitor},
};
use std::{mem::take, rc::Rc};

fn remove_extra_parens(pat: Pat) -> Pat {
    match pat.kind {
        PatKind::Bind(_) | PatKind::Discard | PatKind::Elided => pat,
        PatKind::Paren(p) => remove_extra_parens(*p),
        PatKind::Tuple(ps) => {
            let new_ps: Vec<Pat> = ps.into_iter().map(remove_extra_parens).collect();
            Pat {
                id: pat.id,
                span: pat.span,
                ty: Ty::Tuple(new_ps.iter().map(|p| p.ty.clone()).collect()),
                kind: PatKind::Tuple(new_ps),
            }
        }
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
            if let PatKind::Bind(id) = remove_extra_parens(pat).kind {
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
                    ty: block.ty.clone(),
                    kind: ExprKind::Block(block),
                }),
            }]
        } else {
            self.qubits_curr_block.extend(new_ids);
            new_stmts
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

    fn process_qubit_init(&mut self, init: QubitInit) -> (Expr, Vec<(Ident, Option<Expr>)>) {
        match init.kind {
            QubitInitKind::Array(size) => {
                let gen_id = self.gen_ident(init.span);
                let expr = ReplaceQubitAllocation::gen_local_ref(
                    &gen_id,
                    Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
                );
                (expr, vec![(gen_id, Some(*size))])
            }
            QubitInitKind::Paren(i) => self.process_qubit_init(*i),
            QubitInitKind::Single => {
                let gen_id = self.gen_ident(init.span);
                let expr = ReplaceQubitAllocation::gen_local_ref(&gen_id, Ty::Prim(PrimTy::Qubit));
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
                    ty: Ty::Tuple(exprs.iter().map(|e| e.ty.clone()).collect()),
                    kind: ExprKind::Tuple(exprs),
                };
                (tuple_expr, ids)
            }
        }
    }

    fn gen_ident(&mut self, span: Span) -> Ident {
        let new_id = Ident {
            id: todo!(),
            span,
            name: Rc::from(format!("__generated_ident_{}__", self.gen_id_count)),
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
                        let ty = end.ty.clone();
                        *s = Stmt {
                            id: NodeId::default(),
                            span: s.span,
                            kind: StmtKind::Local(
                                Mutability::Immutable,
                                Pat {
                                    id: NodeId::default(),
                                    span: end.span,
                                    ty: ty.clone(),
                                    kind: PatKind::Bind(end_capture.clone()),
                                },
                                take(end),
                            ),
                        };
                        Some(Stmt {
                            id: NodeId::default(),
                            span: s.span,
                            kind: StmtKind::Expr(ReplaceQubitAllocation::gen_local_ref(
                                &end_capture,
                                ty,
                            )),
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
                    let ty = e.ty.clone();
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
                                ty: ty.clone(),
                                kind: PatKind::Bind(rtrn_capture.clone()),
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
                            ty: Ty::UNIT, //ToDo: double-check that return expression have this type
                            kind: ExprKind::Return(Box::new(
                                ReplaceQubitAllocation::gen_local_ref(&rtrn_capture, ty),
                            )),
                        }),
                    });
                    let new_expr = Expr {
                        id: NodeId::default(),
                        span: expr.span,
                        ty: Ty::UNIT, //ToDo: double-check type of blocks with `return` in them.
                        kind: ExprKind::Block(Block {
                            id: NodeId::default(),
                            span: expr.span,
                            ty: Ty::UNIT, //ToDo: double-check type of blocks with `return` in them.
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
                ty: todo!(),
                kind: PatKind::Bind(ident.clone()),
            },
            Expr {
                id: NodeId::default(),
                span: ident.span,
                ty: todo!(),
                kind: ExprKind::Call(
                    // Box::new(Expr {
                    //     id: NodeId::default(),
                    //     span: ident.span,
                    //     ty: todo!(),
                    //     kind: ExprKind::Path(Path {
                    //         id: NodeId::default(),
                    //         span: ident.span,
                    //         namespace: Some(Ident {
                    //             id: NodeId::default(),
                    //             span: ident.span,
                    //             name: "QIR.Runtime".to_owned(),
                    //         }),
                    //         name: Ident {
                    //             id: NodeId::default(),
                    //             span: ident.span,
                    //             name: func_name,
                    //         },
                    //     }),
                    // }),
                    todo!(),
                    Box::new(Expr {
                        id: NodeId::default(),
                        span: ident.span,
                        ty: todo!(),
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
        "__quantum__rt__qubit_allocate_array".to_owned(),
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
            ty: todo!(),
            kind: ExprKind::Call(
                // Box::new(Expr {
                //     id: NodeId::default(),
                //     span: ident.span,
                //     kind: ExprKind::Path(Path {
                //         id: NodeId::default(),
                //         span: ident.span,
                //         namespace: Some(Ident {
                //             id: NodeId::default(),
                //             span: ident.span,
                //             name: "QIR.Runtime".to_owned(),
                //         }),
                //         name: Ident {
                //             id: NodeId::default(),
                //             span: ident.span,
                //             name: func_name,
                //         },
                //     }),
                // }),
                todo!(),
                Box::new(Expr {
                    id: NodeId::default(),
                    span: ident.span,
                    ty: todo!(),
                    kind: ExprKind::Tuple(vec![ReplaceQubitAllocation::gen_local_ref(
                        &ident,
                        todo!(),
                    )]),
                }),
            ),
        }),
    }
}

fn create_array_dealloc_stmt(ident: &Ident) -> Stmt {
    create_general_dealloc_stmt("__quantum__rt__qubit_release_array".to_owned(), ident)
}

fn create_dealloc_stmt(ident: &Ident) -> Stmt {
    create_general_dealloc_stmt("__quantum__rt__qubit_release".to_owned(), ident)
}
