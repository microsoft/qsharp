// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::{
    assigner::Assigner,
    global::Table,
    hir::{
        Block, Expr, ExprKind, Mutability, NodeId, Pat, PatKind, PrimTy, QubitInit, QubitInitKind,
        Stmt, StmtKind, Ty,
    },
    mut_visit::{walk_expr, walk_stmt, MutVisitor},
};
use std::{mem::take, rc::Rc};

use crate::common::{create_gen_core_ref, IdentTemplate};

pub fn replace_qubit_allocation(unit: &mut CompileUnit, core_table: &Table) {
    let mut pass = ReplaceQubitAllocation {
        assigner: &mut unit.assigner,
        core_table,
        qubits_curr_callable: Vec::new(),
        qubits_curr_block: Vec::new(),
        prefix_qubits: Vec::new(),
    };
    pass.visit_package(&mut unit.package);
}

struct QubitIdent {
    id: IdentTemplate,
    is_array: bool,
}

struct ReplaceQubitAllocation<'a> {
    assigner: &'a mut Assigner,
    core_table: &'a Table,
    qubits_curr_callable: Vec<Vec<QubitIdent>>,
    qubits_curr_block: Vec<QubitIdent>,
    prefix_qubits: Vec<QubitIdent>,
}

impl ReplaceQubitAllocation<'_> {
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
                QubitInitKind::Single => (true, None),
                QubitInitKind::Tuple(_) => (false, None),
            }
        }

        let mut new_stmts: Vec<Stmt> = vec![];
        let mut new_ids: Vec<QubitIdent> = vec![];

        if let (true, opt) = is_non_tuple(&mut init) {
            if let PatKind::Bind(id) = pat.kind {
                let id = IdentTemplate {
                    id: id.id,
                    span: id.span,
                    name: id.name,
                    ty: pat.ty,
                };
                let is_array = opt.is_some();
                new_stmts.push(match opt {
                    Some(mut size) => {
                        self.visit_expr(&mut size);
                        self.create_array_alloc_stmt(&id, size)
                    }
                    None => self.create_alloc_stmt(&id),
                });
                new_ids.push(QubitIdent { id, is_array });
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
                        self.create_array_alloc_stmt(id, size.clone())
                    }
                    None => self.create_alloc_stmt(id),
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
                kind: StmtKind::Local(Mutability::Immutable, pat, assignment_expr),
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

    fn process_qubit_init(
        &mut self,
        init: QubitInit,
    ) -> (Expr, Vec<(IdentTemplate, Option<Expr>)>) {
        match init.kind {
            QubitInitKind::Array(size) => {
                let gen_id =
                    self.gen_ident(Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))), init.span);
                let expr = gen_id.gen_local_ref();
                (expr, vec![(gen_id, Some(*size))])
            }
            QubitInitKind::Single => {
                let gen_id = self.gen_ident(Ty::Prim(PrimTy::Qubit), init.span);
                let expr = gen_id.gen_local_ref();
                (expr, vec![(gen_id, None)])
            }
            QubitInitKind::Tuple(inits) => {
                let mut exprs: Vec<Expr> = vec![];
                let mut ids: Vec<(IdentTemplate, Option<Expr>)> = vec![];
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

    fn gen_ident(&mut self, ty: Ty, span: Span) -> IdentTemplate {
        let id = self.assigner.next_id();
        IdentTemplate {
            id,
            span,
            name: Rc::from(format!("generated_ident_{id}")),
            ty,
        }
    }

    fn is_qubits_empty(&self) -> bool {
        self.qubits_curr_block.is_empty()
            && self
                .qubits_curr_callable
                .iter()
                .all(std::vec::Vec::is_empty)
    }

    fn get_dealloc_stmts(&self, qubits: &[QubitIdent]) -> Vec<Stmt> {
        qubits
            .iter()
            .rev()
            .map(|qubit| {
                if qubit.is_array {
                    self.create_array_dealloc_stmt(&qubit.id)
                } else {
                    self.create_dealloc_stmt(&qubit.id)
                }
            })
            .collect()
    }

    fn get_dealloc_stmts_for_block(&self) -> Vec<Stmt> {
        self.get_dealloc_stmts(&self.qubits_curr_block)
    }

    fn get_dealloc_stmts_for_callable(&self) -> Vec<Stmt> {
        let mut stmts = self.get_dealloc_stmts(&self.qubits_curr_block);
        stmts.extend(
            self.qubits_curr_callable
                .iter()
                .rev()
                .flat_map(|q| self.get_dealloc_stmts(q)),
        );
        stmts
    }

    fn create_alloc_stmt(&self, ident: &IdentTemplate) -> Stmt {
        create_general_alloc_stmt(
            ident,
            create_gen_core_ref(
                self.core_table,
                "QIR.Runtime",
                "__quantum__rt__qubit_allocate",
                ident.span,
            ),
            None,
        )
    }

    fn create_array_alloc_stmt(&self, ident: &IdentTemplate, array_size: Expr) -> Stmt {
        create_general_alloc_stmt(
            ident,
            create_gen_core_ref(
                self.core_table,
                "QIR.Runtime",
                "__quantum__rt__qubit_allocate_array",
                ident.span,
            ),
            Some(array_size),
        )
    }

    fn create_dealloc_stmt(&self, ident: &IdentTemplate) -> Stmt {
        create_general_dealloc_stmt(
            create_gen_core_ref(
                self.core_table,
                "QIR.Runtime",
                "__quantum__rt__qubit_release",
                ident.span,
            ),
            ident,
        )
    }

    fn create_array_dealloc_stmt(&self, ident: &IdentTemplate) -> Stmt {
        create_general_dealloc_stmt(
            create_gen_core_ref(
                self.core_table,
                "QIR.Runtime",
                "__quantum__rt__qubit_release_array",
                ident.span,
            ),
            ident,
        )
    }
}

impl MutVisitor for ReplaceQubitAllocation<'_> {
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
                        let end_capture = self.gen_ident(end.ty.clone(), end.span);
                        *s = end_capture.gen_id_init(Mutability::Immutable, take(end));
                        Some(Stmt {
                            id: NodeId::default(),
                            span: s.span,
                            kind: StmtKind::Expr(end_capture.gen_local_ref()),
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
                    let rtrn_capture = self.gen_ident(e.ty.clone(), e.span);
                    self.visit_expr(e);
                    let mut stmts: Vec<Stmt> = vec![];
                    stmts.push(rtrn_capture.gen_id_init(Mutability::Immutable, take(e)));
                    stmts.extend(self.get_dealloc_stmts_for_callable());
                    stmts.push(Stmt {
                        id: NodeId::default(),
                        span: expr.span,
                        kind: StmtKind::Semi(Expr {
                            id: NodeId::default(),
                            span: expr.span,
                            ty: expr.ty.clone(),
                            kind: ExprKind::Return(Box::new(rtrn_capture.gen_local_ref())),
                        }),
                    });
                    let new_expr = Expr {
                        id: NodeId::default(),
                        span: expr.span,
                        ty: expr.ty.clone(),
                        kind: ExprKind::Block(Block {
                            id: NodeId::default(),
                            span: expr.span,
                            ty: expr.ty.clone(),
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

fn create_general_alloc_stmt(
    ident: &IdentTemplate,
    call_expr: Expr,
    array_size: Option<Expr>,
) -> Stmt {
    ident.gen_id_init(
        Mutability::Immutable,
        Expr {
            id: NodeId::default(),
            span: ident.span,
            ty: Ty::Prim(PrimTy::Qubit),
            kind: ExprKind::Call(
                Box::new(call_expr),
                Box::new(array_size.unwrap_or(Expr {
                    id: NodeId::default(),
                    span: ident.span,
                    ty: Ty::UNIT,
                    kind: ExprKind::Tuple(vec![]),
                })),
            ),
        },
    )
}

fn create_general_dealloc_stmt(call_expr: Expr, ident: &IdentTemplate) -> Stmt {
    Stmt {
        id: NodeId::default(),
        span: ident.span,
        kind: StmtKind::Semi(Expr {
            id: NodeId::default(),
            span: ident.span,
            ty: Ty::UNIT,
            kind: ExprKind::Call(Box::new(call_expr), Box::new(ident.gen_local_ref())),
        }),
    }
}
