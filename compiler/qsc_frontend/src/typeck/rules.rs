// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    infer::{self, Class, Inferrer},
    ty::{Prim, Ty},
    Error, Tys,
};
use crate::resolve::{Res, Resolutions};
use qsc_ast::ast::{
    self, BinOp, Block, Expr, ExprKind, Functor, FunctorExpr, Lit, NodeId, Pat, PatKind, QubitInit,
    QubitInitKind, Spec, Stmt, StmtKind, TernOp, TyKind, UnOp,
};
use qsc_data_structures::span::Span;
use std::{
    collections::{HashMap, HashSet},
    convert::Into,
};

struct Partial {
    ty: Ty,
    diverges: bool,
}

struct Context<'a> {
    resolutions: &'a Resolutions,
    globals: &'a HashMap<Res, Ty>,
    return_ty: Option<&'a Ty>,
    tys: &'a mut Tys<NodeId>,
    nodes: Vec<NodeId>,
    inferrer: Inferrer,
}

impl<'a> Context<'a> {
    fn new(
        resolutions: &'a Resolutions,
        globals: &'a HashMap<Res, Ty>,
        tys: &'a mut Tys<NodeId>,
    ) -> Self {
        Self {
            resolutions,
            globals,
            return_ty: None,
            tys,
            nodes: Vec::new(),
            inferrer: Inferrer::new(),
        }
    }

    fn infer_spec(&mut self, spec: SpecImpl<'a>) {
        let callable_input = self.infer_pat(spec.callable_input);
        if let Some(input) = spec.spec_input {
            let expected = match spec.spec {
                Spec::Body | Spec::Adj => callable_input,
                Spec::Ctl | Spec::CtlAdj => Ty::Tuple(vec![
                    Ty::Array(Box::new(Ty::Prim(Prim::Qubit))),
                    callable_input,
                ]),
            };
            let actual = self.infer_pat(input);
            self.inferrer.eq(input.span, expected, actual);
        }

        self.return_ty = Some(spec.output);
        let block = self.infer_block(spec.block).ty;
        if let Some(return_ty) = self.return_ty {
            self.inferrer.eq(spec.block.span, return_ty.clone(), block);
        }

        self.return_ty = None;
    }

    fn infer_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &ty.kind {
            TyKind::Array(item) => Ty::Array(Box::new(self.infer_ty(item))),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(
                kind.into(),
                Box::new(self.infer_ty(input)),
                Box::new(self.infer_ty(output)),
                functors
                    .as_ref()
                    .map_or(HashSet::new(), FunctorExpr::to_set)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            ),
            TyKind::Hole => self.inferrer.fresh(),
            TyKind::Paren(inner) => self.infer_ty(inner),
            TyKind::Path(_) => Ty::Err, // TODO: Resolve user-defined types.
            &TyKind::Prim(prim) => Ty::Prim(prim.into()),
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_ty(item)).collect())
            }
            TyKind::Var(name) => Ty::Param(name.name.clone()),
        }
    }

    fn infer_block(&mut self, block: &Block) -> Partial {
        let mut diverges = false;
        let mut last = None;
        for stmt in &block.stmts {
            let partial = self.infer_stmt(stmt);
            diverges |= partial.diverges;
            last = Some(partial);
        }

        let ty = self.diverge_if(diverges, last.unwrap_or(converge(Ty::UNIT)));
        self.record(block.id, ty.ty.clone());
        ty
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Partial {
        let ty = match &stmt.kind {
            StmtKind::Empty => converge(Ty::UNIT),
            StmtKind::Expr(expr) => self.infer_expr(expr),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let partial_expr = self.infer_expr(expr);
                self.inferrer.eq(pat.span, partial_expr.ty, pat_ty);
                self.diverge_if(partial_expr.diverges, converge(Ty::UNIT))
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let partial_init = self.infer_qubit_init(init);
                self.inferrer.eq(pat.span, partial_init.ty, pat_ty);
                match block {
                    None => self.diverge_if(partial_init.diverges, converge(Ty::UNIT)),
                    Some(block) => {
                        let block_ty = self.infer_block(block);
                        self.diverge_if(partial_init.diverges, block_ty)
                    }
                }
            }
            StmtKind::Semi(expr) => {
                let partial = self.infer_expr(expr);
                self.diverge_if(partial.diverges, converge(Ty::UNIT))
            }
        };

        self.record(stmt.id, ty.ty.clone());
        ty
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Partial {
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_partial = self.infer_expr(first);
                    let mut diverges = first_partial.diverges;
                    for item in rest {
                        let partial_item = self.infer_expr(item);
                        diverges = diverges || partial_item.diverges;
                        self.inferrer
                            .eq(item.span, first_partial.ty.clone(), partial_item.ty);
                    }
                    self.diverge_if(diverges, converge(Ty::Array(Box::new(first_partial.ty))))
                }
                None => converge(Ty::Array(Box::new(self.inferrer.fresh()))),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let partial_item = self.infer_expr(item);
                let partial_size = self.infer_expr(size);
                self.inferrer
                    .eq(size.span, Ty::Prim(Prim::Int), partial_size.ty);
                self.diverge_if(
                    partial_item.diverges || partial_size.diverges,
                    converge(Ty::Array(Box::new(partial_item.ty))),
                )
            }
            ExprKind::Assign(lhs, rhs) => {
                let partial_lhs = self.infer_expr(lhs);
                let partial_rhs = self.infer_expr(rhs);
                self.inferrer.eq(lhs.span, partial_lhs.ty, partial_rhs.ty);
                self.diverge_if(
                    partial_lhs.diverges || partial_rhs.diverges,
                    converge(Ty::UNIT),
                )
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let partial = self.infer_binop(expr.span, *op, lhs, rhs);
                self.diverge_if(partial.diverges, converge(Ty::UNIT))
            }
            ExprKind::AssignUpdate(container, index, item) => {
                let partial = self.infer_update(expr.span, container, index, item);
                self.diverge_if(partial.diverges, converge(Ty::UNIT))
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(expr.span, *op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                // TODO: Handle partial application. (It's probably easier to turn them into lambdas
                // before type inference.)
                // https://github.com/microsoft/qsharp/issues/151
                let partial_callee = self.infer_expr(callee);
                let partial_input = self.infer_expr(input);
                let output_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::Call {
                        callee: partial_callee.ty,
                        input: partial_input.ty,
                        output: output_ty.clone(),
                    },
                );
                self.diverge_if(
                    partial_callee.diverges || partial_input.diverges,
                    converge(output_ty),
                )
            }
            ExprKind::Conjugate(within, apply) => {
                let partial_within = self.infer_block(within);
                let partial_apply = self.infer_block(apply);
                self.diverge_if(partial_within.diverges, partial_apply)
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).ty;
                self.inferrer
                    .eq(message.span, Ty::Prim(Prim::String), message_ty);
                self.diverge()
            }
            ExprKind::Field(record, name) => {
                let partial_record = self.infer_expr(record);
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasField {
                        record: partial_record.ty,
                        name: name.name.clone(),
                        item: item_ty.clone(),
                    },
                );
                self.diverge_if(partial_record.diverges, converge(item_ty))
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let partial_container = self.infer_expr(container);
                self.inferrer.class(
                    container.span,
                    Class::Iterable {
                        container: partial_container.ty,
                        item: item_ty,
                    },
                );
                let partial_body = self.infer_block(body);
                self.diverge_if(
                    partial_container.diverges || partial_body.diverges,
                    converge(Ty::UNIT),
                )
            }
            ExprKind::If(cond, if_true, if_false) => {
                let partial_cond = self.infer_expr(cond);
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), partial_cond.ty);
                let partial_true = self.infer_block(if_true);
                let partial_false = if_false
                    .as_ref()
                    .map_or(converge(Ty::UNIT), |e| self.infer_expr(e));
                self.inferrer
                    .eq(expr.span, partial_true.ty.clone(), partial_false.ty);
                self.diverge_if(
                    partial_cond.diverges,
                    Partial {
                        diverges: partial_true.diverges && partial_false.diverges,
                        ..partial_true
                    },
                )
            }
            ExprKind::Index(container, index) => {
                let partial_container = self.infer_expr(container);
                let partial_index = self.infer_expr(index);
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasIndex {
                        container: partial_container.ty,
                        index: partial_index.ty,
                        item: item_ty.clone(),
                    },
                );
                self.diverge_if(
                    partial_container.diverges || partial_index.diverges,
                    converge(item_ty),
                )
            }
            ExprKind::Lambda(kind, input, body) => {
                // TODO: Infer the supported functors or require that they are explicitly listed.
                // https://github.com/microsoft/qsharp/issues/151
                let input = self.infer_pat(input);
                let body = self.infer_expr(body).ty;
                converge(Ty::Arrow(
                    kind.into(),
                    Box::new(input),
                    Box::new(body),
                    HashSet::new(),
                ))
            }
            ExprKind::Lit(Lit::BigInt(_)) => converge(Ty::Prim(Prim::BigInt)),
            ExprKind::Lit(Lit::Bool(_)) => converge(Ty::Prim(Prim::Bool)),
            ExprKind::Lit(Lit::Double(_)) => converge(Ty::Prim(Prim::Double)),
            ExprKind::Lit(Lit::Int(_)) => converge(Ty::Prim(Prim::Int)),
            ExprKind::Lit(Lit::Pauli(_)) => converge(Ty::Prim(Prim::Pauli)),
            ExprKind::Lit(Lit::Result(_)) => converge(Ty::Prim(Prim::Result)),
            ExprKind::Lit(Lit::String(_)) => converge(Ty::Prim(Prim::String)),
            ExprKind::Paren(expr) => self.infer_expr(expr),
            ExprKind::Path(path) => match self.resolutions.get(path.id) {
                None => converge(Ty::Err),
                Some(res) => match self.globals.get(res) {
                    Some(ty) => {
                        let mut ty = ty.clone();
                        self.inferrer.freshen(&mut ty);
                        converge(ty)
                    }
                    None => match res {
                        &Res::Internal(id) => converge(
                            self.tys
                                .get(id)
                                .expect("local variable should have inferred type")
                                .clone(),
                        ),
                        Res::External(..) => {
                            panic!("path resolves to external package but definition not found")
                        }
                    },
                },
            },
            ExprKind::Range(start, step, end) => {
                let mut diverges = false;
                for expr in start.iter().chain(step).chain(end) {
                    let partial = self.infer_expr(expr);
                    diverges = diverges || partial.diverges;
                    self.inferrer.eq(expr.span, Ty::Prim(Prim::Int), partial.ty);
                }
                self.diverge_if(diverges, converge(Ty::Prim(Prim::Range)))
            }
            ExprKind::Repeat(body, until, fixup) => {
                let partial_body = self.infer_block(body);
                let partial_until = self.infer_expr(until);
                self.inferrer
                    .eq(until.span, Ty::Prim(Prim::Bool), partial_until.ty);
                let fixup_diverges = fixup
                    .as_ref()
                    .map_or(false, |f| self.infer_block(f).diverges);
                self.diverge_if(
                    partial_body.diverges || partial_until.diverges || fixup_diverges,
                    converge(Ty::UNIT),
                )
            }
            ExprKind::Return(expr) => {
                let ty = self.infer_expr(expr).ty;
                if let Some(return_ty) = &self.return_ty {
                    self.inferrer.eq(expr.span, (*return_ty).clone(), ty);
                }
                self.diverge()
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let partial_cond = self.infer_expr(cond);
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), partial_cond.ty);
                let partial_true = self.infer_expr(if_true);
                let partial_false = self.infer_expr(if_false);
                self.inferrer
                    .eq(expr.span, partial_true.ty.clone(), partial_false.ty);
                self.diverge_if(
                    partial_cond.diverges,
                    Partial {
                        diverges: partial_true.diverges && partial_false.diverges,
                        ..partial_true
                    },
                )
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                self.infer_update(expr.span, container, index, item)
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items {
                    let partial = self.infer_expr(item);
                    diverges = diverges || partial.diverges;
                    tys.push(partial.ty);
                }
                self.diverge_if(diverges, converge(Ty::Tuple(tys)))
            }
            ExprKind::UnOp(op, expr) => self.infer_unop(*op, expr),
            ExprKind::While(cond, body) => {
                let partial_cond = self.infer_expr(cond);
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), partial_cond.ty);
                let partial_body = self.infer_block(body);
                self.diverge_if(
                    partial_cond.diverges || partial_body.diverges,
                    converge(Ty::UNIT),
                )
            }
            ExprKind::Err | ExprKind::Hole => converge(self.inferrer.fresh()),
        };

        self.record(expr.id, ty.ty.clone());
        ty
    }

    fn infer_unop(&mut self, op: UnOp, operand: &Expr) -> Partial {
        let partial_operand = self.infer_expr(operand);
        let diverges = partial_operand.diverges;
        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.inferrer
                    .class(operand.span, Class::Adj(partial_operand.ty.clone()));
                partial_operand
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.inferrer.fresh();
                self.inferrer.class(
                    operand.span,
                    Class::Ctl {
                        op: partial_operand.ty,
                        with_ctls: with_ctls.clone(),
                    },
                );
                converge(with_ctls)
            }
            UnOp::Neg | UnOp::NotB | UnOp::Pos => {
                self.inferrer
                    .class(operand.span, Class::Num(partial_operand.ty.clone()));
                partial_operand
            }
            UnOp::NotL => {
                self.inferrer.eq(
                    operand.span,
                    Ty::Prim(Prim::Bool),
                    partial_operand.ty.clone(),
                );
                partial_operand
            }
            UnOp::Unwrap => {
                let base = self.inferrer.fresh();
                self.inferrer.class(
                    operand.span,
                    Class::Unwrap {
                        wrapper: partial_operand.ty,
                        base: base.clone(),
                    },
                );
                converge(base)
            }
        };

        self.diverge_if(diverges, ty)
    }

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Partial {
        let partial_lhs = self.infer_expr(lhs);
        let partial_rhs = self.infer_expr(rhs);
        let diverges = partial_lhs.diverges || partial_rhs.diverges;
        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.inferrer
                    .eq(span, partial_lhs.ty.clone(), partial_rhs.ty);
                self.inferrer
                    .eq(lhs.span, Ty::Prim(Prim::Bool), partial_lhs.ty.clone());
                partial_lhs
            }
            BinOp::Eq | BinOp::Neq => {
                self.inferrer
                    .eq(span, partial_lhs.ty.clone(), partial_rhs.ty);
                self.inferrer.class(lhs.span, Class::Eq(partial_lhs.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::Add => {
                self.inferrer
                    .eq(span, partial_lhs.ty.clone(), partial_rhs.ty);
                self.inferrer
                    .class(lhs.span, Class::Add(partial_lhs.ty.clone()));
                partial_lhs
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.inferrer
                    .eq(span, partial_lhs.ty.clone(), partial_rhs.ty);
                self.inferrer.class(lhs.span, Class::Num(partial_lhs.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Sub
            | BinOp::XorB => {
                self.inferrer
                    .eq(span, partial_lhs.ty.clone(), partial_rhs.ty);
                self.inferrer
                    .class(lhs.span, Class::Num(partial_lhs.ty.clone()));
                partial_lhs
            }
            BinOp::Exp => {
                self.inferrer.class(
                    span,
                    Class::Exp {
                        base: partial_lhs.ty.clone(),
                        power: partial_rhs.ty,
                    },
                );
                partial_lhs
            }
            BinOp::Shl | BinOp::Shr => {
                self.inferrer
                    .class(lhs.span, Class::Integral(partial_lhs.ty.clone()));
                self.inferrer
                    .eq(rhs.span, Ty::Prim(Prim::Int), partial_rhs.ty);
                partial_lhs
            }
        };

        self.diverge_if(diverges, ty)
    }

    fn infer_update(&mut self, span: Span, container: &Expr, index: &Expr, item: &Expr) -> Partial {
        let partial_container = self.infer_expr(container);
        let partial_index = self.infer_expr(index);
        let partial_item = self.infer_expr(item);
        self.inferrer.class(
            span,
            Class::HasIndex {
                container: partial_container.ty.clone(),
                index: partial_index.ty,
                item: partial_item.ty,
            },
        );
        self.diverge_if(
            partial_index.diverges || partial_item.diverges,
            partial_container,
        )
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        let ty = match &pat.kind {
            PatKind::Bind(name, None) => {
                let ty = self.inferrer.fresh();
                self.record(name.id, ty.clone());
                ty
            }
            PatKind::Bind(name, Some(ty)) => {
                let ty = self.infer_ty(ty);
                self.record(name.id, ty.clone());
                ty
            }
            PatKind::Discard(None) | PatKind::Elided => self.inferrer.fresh(),
            PatKind::Discard(Some(ty)) => self.infer_ty(ty),
            PatKind::Paren(inner) => self.infer_pat(inner),
            PatKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_pat(item)).collect())
            }
        };

        self.record(pat.id, ty.clone());
        ty
    }

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Partial {
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let partial_length = self.infer_expr(length);
                self.inferrer
                    .eq(length.span, Ty::Prim(Prim::Int), partial_length.ty);
                self.diverge_if(
                    partial_length.diverges,
                    converge(Ty::Array(Box::new(Ty::Prim(Prim::Qubit)))),
                )
            }
            QubitInitKind::Paren(inner) => self.infer_qubit_init(inner),
            QubitInitKind::Single => converge(Ty::Prim(Prim::Qubit)),
            QubitInitKind::Tuple(items) => {
                let mut diverges = false;
                let mut tys = Vec::new();
                for item in items {
                    let partial = self.infer_qubit_init(item);
                    diverges = diverges || partial.diverges;
                    tys.push(self.infer_qubit_init(item).ty);
                }
                self.diverge_if(diverges, converge(Ty::Tuple(tys)))
            }
        };

        self.record(init.id, ty.ty.clone());
        ty
    }

    fn diverge(&mut self) -> Partial {
        Partial {
            ty: self.inferrer.fresh(),
            diverges: true,
        }
    }

    fn diverge_if(&mut self, diverges: bool, partial: Partial) -> Partial {
        if !diverges || partial.diverges {
            partial
        } else {
            self.diverge()
        }
    }

    fn record(&mut self, id: NodeId, ty: Ty) {
        self.nodes.push(id);
        self.tys.insert(id, ty);
    }

    fn solve(self) -> Vec<Error> {
        let (substs, errors) = self.inferrer.solve();
        for id in self.nodes {
            let ty = self.tys.get_mut(id).expect("node should have type");
            infer::substitute(&substs, ty);
        }
        errors
    }
}

#[derive(Clone, Copy)]
pub(super) struct SpecImpl<'a> {
    pub(super) spec: Spec,
    pub(super) callable_input: &'a Pat,
    pub(super) spec_input: Option<&'a Pat>,
    pub(super) output: &'a Ty,
    pub(super) block: &'a Block,
}

pub(super) fn spec(
    resolutions: &Resolutions,
    globals: &HashMap<Res, Ty>,
    tys: &mut Tys<NodeId>,
    spec: SpecImpl,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_spec(spec);
    context.solve()
}

pub(super) fn entry_expr(
    resolutions: &Resolutions,
    globals: &HashMap<Res, Ty>,
    tys: &mut Tys<NodeId>,
    entry: &Expr,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_expr(entry);
    context.solve()
}

fn converge(ty: Ty) -> Partial {
    Partial {
        ty,
        diverges: false,
    }
}
