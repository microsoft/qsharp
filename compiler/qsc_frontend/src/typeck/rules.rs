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

#[derive(Clone, Copy, Eq, PartialEq)]
enum Termination {
    Convergent,
    Divergent,
}

impl Termination {
    fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::Divergent, Self::Divergent) => Self::Divergent,
            _ => Self::Convergent,
        }
    }

    fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Divergent, _) | (_, Self::Divergent) => Self::Divergent,
            _ => Self::Convergent,
        }
    }

    fn with<T>(self, value: T) -> Fallible<T> {
        Fallible { term: self, value }
    }

    fn then<T>(&mut self, fallible: Fallible<T>) -> Fallible<T> {
        *self = self.or(fallible.term);
        fallible
    }
}

struct Fallible<T> {
    term: Termination,
    value: T,
}

impl<T> Fallible<T> {
    fn and<U>(self, other: Fallible<U>) -> Fallible<(T, U)> {
        Fallible {
            term: self.term.and(other.term),
            value: (self.value, other.value),
        }
    }
}

struct Context<'a> {
    resolutions: &'a Resolutions<NodeId>,
    globals: &'a HashMap<Res<NodeId>, Ty>,
    return_ty: Option<&'a Ty>,
    tys: &'a mut Tys<NodeId>,
    nodes: Vec<NodeId>,
    inferrer: Inferrer,
}

impl<'a> Context<'a> {
    fn new(
        resolutions: &'a Resolutions<NodeId>,
        globals: &'a HashMap<Res<NodeId>, Ty>,
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
        let block = self.infer_block(spec.block).value;
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

    fn infer_block(&mut self, block: &Block) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let mut last = None;
        for stmt in &block.stmts {
            last = Some(term.then(self.infer_stmt(stmt)));
        }

        let ty = self.diverge_or(term, last.unwrap_or(Termination::Convergent.with(Ty::UNIT)));
        self.record(block.id, ty.value.clone());
        ty
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let ty = match &stmt.kind {
            StmtKind::Empty => Termination::Convergent.with(Ty::UNIT),
            StmtKind::Expr(expr) => term.then(self.infer_expr(expr)),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = term.then(self.infer_expr(expr));
                self.inferrer.eq(pat.span, expr_ty.value, pat_ty);
                Termination::Convergent.with(Ty::UNIT)
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init_ty = term.then(self.infer_qubit_init(init));
                self.inferrer.eq(pat.span, init_ty.value, pat_ty);
                match block {
                    None => Termination::Convergent.with(Ty::UNIT),
                    Some(block) => term.then(self.infer_block(block)),
                }
            }
            StmtKind::Semi(expr) => {
                term.then(self.infer_expr(expr));
                Termination::Convergent.with(Ty::UNIT)
            }
        };

        let ty = self.diverge_or(term, ty);
        self.record(stmt.id, ty.value.clone());
        ty
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_ty = term.then(self.infer_expr(first));
                    for item in rest {
                        let item_ty = term.then(self.infer_expr(item));
                        self.inferrer
                            .eq(item.span, first_ty.value.clone(), item_ty.value);
                    }
                    Termination::Convergent.with(Ty::Array(Box::new(first_ty.value)))
                }
                None => Termination::Convergent.with(Ty::Array(Box::new(self.inferrer.fresh()))),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item_ty = term.then(self.infer_expr(item));
                let size_ty = term.then(self.infer_expr(size));
                self.inferrer
                    .eq(size.span, Ty::Prim(Prim::Int), size_ty.value);
                Termination::Convergent.with(Ty::Array(Box::new(item_ty.value)))
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_ty = term.then(self.infer_expr(lhs));
                let rhs_ty = term.then(self.infer_expr(rhs));
                self.inferrer.eq(lhs.span, lhs_ty.value, rhs_ty.value);
                Termination::Convergent.with(Ty::UNIT)
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                term.then(self.infer_binop(expr.span, *op, lhs, rhs));
                Termination::Convergent.with(Ty::UNIT)
            }
            ExprKind::AssignUpdate(container, index, item) => {
                term.then(self.infer_update(expr.span, container, index, item));
                Termination::Convergent.with(Ty::UNIT)
            }
            ExprKind::BinOp(op, lhs, rhs) => term.then(self.infer_binop(expr.span, *op, lhs, rhs)),
            ExprKind::Block(block) => term.then(self.infer_block(block)),
            ExprKind::Call(callee, input) => {
                // TODO: Handle partial application. (It's probably easier to turn them into lambdas
                // before type inference.)
                // https://github.com/microsoft/qsharp/issues/151
                let callee_ty = term.then(self.infer_expr(callee));
                let input_ty = term.then(self.infer_expr(input));
                let output_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::Call {
                        callee: callee_ty.value,
                        input: input_ty.value,
                        output: output_ty.clone(),
                    },
                );
                Termination::Convergent.with(output_ty)
            }
            ExprKind::Conjugate(within, apply) => {
                term.then(self.infer_block(within));
                term.then(self.infer_block(apply))
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).value;
                self.inferrer
                    .eq(message.span, Ty::Prim(Prim::String), message_ty);
                term = Termination::Divergent;
                Termination::Divergent.with(self.inferrer.fresh())
            }
            ExprKind::Field(record, name) => {
                let record_ty = term.then(self.infer_expr(record));
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasField {
                        record: record_ty.value,
                        name: name.name.clone(),
                        item: item_ty.clone(),
                    },
                );
                Termination::Convergent.with(item_ty)
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let container_ty = term.then(self.infer_expr(container));
                self.inferrer.class(
                    container.span,
                    Class::Iterable {
                        container: container_ty.value,
                        item: item_ty,
                    },
                );
                term.then(self.infer_block(body));
                Termination::Convergent.with(Ty::UNIT)
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_ty = term.then(self.infer_expr(cond));
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), cond_ty.value);
                let true_ty = self.infer_block(if_true);
                let false_ty = if_false.as_ref().map_or_else(
                    || Termination::Convergent.with(Ty::UNIT),
                    |e| self.infer_expr(e),
                );
                let tys = term.then(true_ty.and(false_ty));
                self.inferrer
                    .eq(expr.span, tys.value.0.clone(), tys.value.1);
                tys.term.with(tys.value.0)
            }
            ExprKind::Index(container, index) => {
                let container_ty = term.then(self.infer_expr(container));
                let index_ty = term.then(self.infer_expr(index));
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasIndex {
                        container: container_ty.value,
                        index: index_ty.value,
                        item: item_ty.clone(),
                    },
                );
                Termination::Convergent.with(item_ty)
            }
            ExprKind::Lambda(kind, input, body) => {
                // TODO: Infer the supported functors or require that they are explicitly listed.
                // https://github.com/microsoft/qsharp/issues/151
                let input = self.infer_pat(input);
                let body = self.infer_expr(body).value;
                Termination::Convergent.with(Ty::Arrow(
                    kind.into(),
                    Box::new(input),
                    Box::new(body),
                    HashSet::new(),
                ))
            }
            ExprKind::Lit(Lit::BigInt(_)) => Termination::Convergent.with(Ty::Prim(Prim::BigInt)),
            ExprKind::Lit(Lit::Bool(_)) => Termination::Convergent.with(Ty::Prim(Prim::Bool)),
            ExprKind::Lit(Lit::Double(_)) => Termination::Convergent.with(Ty::Prim(Prim::Double)),
            ExprKind::Lit(Lit::Int(_)) => Termination::Convergent.with(Ty::Prim(Prim::Int)),
            ExprKind::Lit(Lit::Pauli(_)) => Termination::Convergent.with(Ty::Prim(Prim::Pauli)),
            ExprKind::Lit(Lit::Result(_)) => Termination::Convergent.with(Ty::Prim(Prim::Result)),
            ExprKind::Lit(Lit::String(_)) => Termination::Convergent.with(Ty::Prim(Prim::String)),
            ExprKind::Paren(expr) => term.then(self.infer_expr(expr)),
            ExprKind::Path(path) => match self.resolutions.get(path.id) {
                None => Termination::Convergent.with(Ty::Err),
                Some(res) => match self.globals.get(res) {
                    Some(ty) => {
                        let mut ty = ty.clone();
                        self.inferrer.freshen(&mut ty);
                        Termination::Convergent.with(ty)
                    }
                    None => match res {
                        &Res::Internal(id) => Termination::Convergent.with(
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
                for expr in start.iter().chain(step).chain(end) {
                    let ty = term.then(self.infer_expr(expr));
                    self.inferrer.eq(expr.span, Ty::Prim(Prim::Int), ty.value);
                }
                Termination::Convergent.with(Ty::Prim(Prim::Range))
            }
            ExprKind::Repeat(body, until, fixup) => {
                term.then(self.infer_block(body));
                let until_ty = term.then(self.infer_expr(until));
                self.inferrer
                    .eq(until.span, Ty::Prim(Prim::Bool), until_ty.value);
                if let Some(fixup) = fixup {
                    term.then(self.infer_block(fixup));
                }
                Termination::Convergent.with(Ty::UNIT)
            }
            ExprKind::Return(expr) => {
                let ty = self.infer_expr(expr).value;
                if let Some(return_ty) = &self.return_ty {
                    self.inferrer.eq(expr.span, (*return_ty).clone(), ty);
                }
                term = Termination::Divergent;
                Termination::Divergent.with(self.inferrer.fresh())
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond_ty = term.then(self.infer_expr(cond));
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), cond_ty.value);
                let true_ty = self.infer_expr(if_true);
                let false_ty = self.infer_expr(if_false);
                let tys = term.then(true_ty.and(false_ty));
                self.inferrer
                    .eq(expr.span, tys.value.0.clone(), tys.value.1);
                tys.term.with(tys.value.0)
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                term.then(self.infer_update(expr.span, container, index, item))
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = term.then(self.infer_expr(item));
                    tys.push(ty.value);
                }
                Termination::Convergent.with(Ty::Tuple(tys))
            }
            ExprKind::UnOp(op, expr) => term.then(self.infer_unop(*op, expr)),
            ExprKind::While(cond, body) => {
                let cond_ty = term.then(self.infer_expr(cond));
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), cond_ty.value);
                term.then(self.infer_block(body));
                Termination::Convergent.with(Ty::UNIT)
            }
            ExprKind::Err | ExprKind::Hole => Termination::Convergent.with(self.inferrer.fresh()),
        };

        let ty = self.diverge_or(term, ty);
        self.record(expr.id, ty.value.clone());
        ty
    }

    fn infer_unop(&mut self, op: UnOp, operand: &Expr) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let operand_ty = term.then(self.infer_expr(operand));

        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.inferrer
                    .class(operand.span, Class::Adj(operand_ty.value.clone()));
                operand_ty
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.inferrer.fresh();
                self.inferrer.class(
                    operand.span,
                    Class::Ctl {
                        op: operand_ty.value,
                        with_ctls: with_ctls.clone(),
                    },
                );
                Termination::Convergent.with(with_ctls)
            }
            UnOp::Neg | UnOp::NotB | UnOp::Pos => {
                self.inferrer
                    .class(operand.span, Class::Num(operand_ty.value.clone()));
                operand_ty
            }
            UnOp::NotL => {
                self.inferrer
                    .eq(operand.span, Ty::Prim(Prim::Bool), operand_ty.value.clone());
                operand_ty
            }
            UnOp::Unwrap => {
                let base = self.inferrer.fresh();
                self.inferrer.class(
                    operand.span,
                    Class::Unwrap {
                        wrapper: operand_ty.value,
                        base: base.clone(),
                    },
                );
                Termination::Convergent.with(base)
            }
        };

        self.diverge_or(term, ty)
    }

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let lhs_ty = term.then(self.infer_expr(lhs));
        let rhs_ty = term.then(self.infer_expr(rhs));

        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.inferrer.eq(span, lhs_ty.value.clone(), rhs_ty.value);
                self.inferrer
                    .eq(lhs.span, Ty::Prim(Prim::Bool), lhs_ty.value.clone());
                lhs_ty
            }
            BinOp::Eq | BinOp::Neq => {
                self.inferrer.eq(span, lhs_ty.value.clone(), rhs_ty.value);
                self.inferrer.class(lhs.span, Class::Eq(lhs_ty.value));
                Termination::Convergent.with(Ty::Prim(Prim::Bool))
            }
            BinOp::Add => {
                self.inferrer.eq(span, lhs_ty.value.clone(), rhs_ty.value);
                self.inferrer
                    .class(lhs.span, Class::Add(lhs_ty.value.clone()));
                lhs_ty
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.inferrer.eq(span, lhs_ty.value.clone(), rhs_ty.value);
                self.inferrer.class(lhs.span, Class::Num(lhs_ty.value));
                Termination::Convergent.with(Ty::Prim(Prim::Bool))
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Sub
            | BinOp::XorB => {
                self.inferrer.eq(span, lhs_ty.value.clone(), rhs_ty.value);
                self.inferrer
                    .class(lhs.span, Class::Num(lhs_ty.value.clone()));
                lhs_ty
            }
            BinOp::Exp => {
                self.inferrer.class(
                    span,
                    Class::Exp {
                        base: lhs_ty.value.clone(),
                        power: rhs_ty.value,
                    },
                );
                lhs_ty
            }
            BinOp::Shl | BinOp::Shr => {
                self.inferrer
                    .class(lhs.span, Class::Integral(lhs_ty.value.clone()));
                self.inferrer
                    .eq(rhs.span, Ty::Prim(Prim::Int), rhs_ty.value);
                lhs_ty
            }
        };

        self.diverge_or(term, ty)
    }

    fn infer_update(
        &mut self,
        span: Span,
        container: &Expr,
        index: &Expr,
        item: &Expr,
    ) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let container_ty = term.then(self.infer_expr(container));
        let index_ty = term.then(self.infer_expr(index));
        let item_ty = term.then(self.infer_expr(item));
        self.inferrer.class(
            span,
            Class::HasIndex {
                container: container_ty.value.clone(),
                index: index_ty.value,
                item: item_ty.value,
            },
        );
        self.diverge_or(term, container_ty)
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

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Fallible<Ty> {
        let mut term = Termination::Convergent;
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let length_ty = term.then(self.infer_expr(length));
                self.inferrer
                    .eq(length.span, Ty::Prim(Prim::Int), length_ty.value);
                Termination::Convergent.with(Ty::Array(Box::new(Ty::Prim(Prim::Qubit))))
            }
            QubitInitKind::Paren(inner) => term.then(self.infer_qubit_init(inner)),
            QubitInitKind::Single => Termination::Convergent.with(Ty::Prim(Prim::Qubit)),
            QubitInitKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    tys.push(term.then(self.infer_qubit_init(item)).value);
                }
                Termination::Convergent.with(Ty::Tuple(tys))
            }
        };

        let ty = self.diverge_or(term, ty);
        self.record(init.id, ty.value.clone());
        ty
    }

    fn diverge_or(&mut self, term: Termination, default: Fallible<Ty>) -> Fallible<Ty> {
        match (term, default.term) {
            (Termination::Convergent, _) | (_, Termination::Divergent) => default,
            (Termination::Divergent, Termination::Convergent) => {
                Termination::Divergent.with(self.inferrer.fresh())
            }
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
    resolutions: &Resolutions<NodeId>,
    globals: &HashMap<Res<NodeId>, Ty>,
    tys: &mut Tys<NodeId>,
    spec: SpecImpl,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_spec(spec);
    context.solve()
}

pub(super) fn entry_expr(
    resolutions: &Resolutions<NodeId>,
    globals: &HashMap<Res<NodeId>, Ty>,
    tys: &mut Tys<NodeId>,
    entry: &Expr,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_expr(entry);
    context.solve()
}
