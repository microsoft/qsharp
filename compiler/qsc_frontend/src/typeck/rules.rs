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
enum Divergence {
    Convergent,
    Divergent,
}

impl Divergence {
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
}

struct Fallible<T> {
    ty: T,
    diverges: Divergence,
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

    fn infer_block(&mut self, block: &Block) -> Fallible<Ty> {
        let mut diverges = Divergence::Convergent;
        let mut last = None;
        for stmt in &block.stmts {
            let ty = self.infer_stmt(stmt);
            diverges = diverges.or(ty.diverges);
            last = Some(ty);
        }

        let ty = self.diverge_if(diverges, last.unwrap_or(converge(Ty::UNIT)));
        self.record(block.id, ty.ty.clone());
        ty
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Fallible<Ty> {
        let ty = match &stmt.kind {
            StmtKind::Empty => converge(Ty::UNIT),
            StmtKind::Expr(expr) => self.infer_expr(expr),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = self.infer_expr(expr);
                self.inferrer.eq(pat.span, expr_ty.ty, pat_ty);
                self.diverge_if(expr_ty.diverges, converge(Ty::UNIT))
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init_ty = self.infer_qubit_init(init);
                self.inferrer.eq(pat.span, init_ty.ty, pat_ty);
                match block {
                    None => self.diverge_if(init_ty.diverges, converge(Ty::UNIT)),
                    Some(block) => {
                        let block_ty = self.infer_block(block);
                        self.diverge_if(init_ty.diverges, block_ty)
                    }
                }
            }
            StmtKind::Semi(expr) => {
                let ty = self.infer_expr(expr);
                self.diverge_if(ty.diverges, converge(Ty::UNIT))
            }
        };

        self.record(stmt.id, ty.ty.clone());
        ty
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Fallible<Ty> {
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_ty = self.infer_expr(first);
                    let mut diverges = first_ty.diverges;
                    for item in rest {
                        let item_ty = self.infer_expr(item);
                        diverges = diverges.or(item_ty.diverges);
                        self.inferrer.eq(item.span, first_ty.ty.clone(), item_ty.ty);
                    }
                    self.diverge_if(diverges, converge(Ty::Array(Box::new(first_ty.ty))))
                }
                None => converge(Ty::Array(Box::new(self.inferrer.fresh()))),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item_ty = self.infer_expr(item);
                let size_ty = self.infer_expr(size);
                self.inferrer.eq(size.span, Ty::Prim(Prim::Int), size_ty.ty);
                self.diverge_if(
                    item_ty.diverges.or(size_ty.diverges),
                    converge(Ty::Array(Box::new(item_ty.ty))),
                )
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_ty = self.infer_expr(lhs);
                let rhs_ty = self.infer_expr(rhs);
                self.inferrer.eq(lhs.span, lhs_ty.ty, rhs_ty.ty);
                self.diverge_if(lhs_ty.diverges.or(rhs_ty.diverges), converge(Ty::UNIT))
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let ty = self.infer_binop(expr.span, *op, lhs, rhs);
                self.diverge_if(ty.diverges, converge(Ty::UNIT))
            }
            ExprKind::AssignUpdate(container, index, item) => {
                let ty = self.infer_update(expr.span, container, index, item);
                self.diverge_if(ty.diverges, converge(Ty::UNIT))
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(expr.span, *op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                // TODO: Handle partial application. (It's probably easier to turn them into lambdas
                // before type inference.)
                // https://github.com/microsoft/qsharp/issues/151
                let callee_ty = self.infer_expr(callee);
                let input_ty = self.infer_expr(input);
                let output_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::Call {
                        callee: callee_ty.ty,
                        input: input_ty.ty,
                        output: output_ty.clone(),
                    },
                );
                self.diverge_if(
                    callee_ty.diverges.or(input_ty.diverges),
                    converge(output_ty),
                )
            }
            ExprKind::Conjugate(within, apply) => {
                let within_ty = self.infer_block(within);
                let apply_ty = self.infer_block(apply);
                self.diverge_if(within_ty.diverges, apply_ty)
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).ty;
                self.inferrer
                    .eq(message.span, Ty::Prim(Prim::String), message_ty);
                self.diverge()
            }
            ExprKind::Field(record, name) => {
                let record_ty = self.infer_expr(record);
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasField {
                        record: record_ty.ty,
                        name: name.name.clone(),
                        item: item_ty.clone(),
                    },
                );
                self.diverge_if(record_ty.diverges, converge(item_ty))
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let container_ty = self.infer_expr(container);
                self.inferrer.class(
                    container.span,
                    Class::Iterable {
                        container: container_ty.ty,
                        item: item_ty,
                    },
                );
                let body_ty = self.infer_block(body);
                self.diverge_if(
                    container_ty.diverges.or(body_ty.diverges),
                    converge(Ty::UNIT),
                )
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_ty = self.infer_expr(cond);
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), cond_ty.ty);
                let true_ty = self.infer_block(if_true);
                let false_ty = if_false
                    .as_ref()
                    .map_or(converge(Ty::UNIT), |e| self.infer_expr(e));
                self.inferrer.eq(expr.span, true_ty.ty.clone(), false_ty.ty);
                let diverges = cond_ty.diverges.or(true_ty.diverges.and(false_ty.diverges));
                self.diverge_if(
                    diverges,
                    Fallible {
                        ty: true_ty.ty,
                        diverges,
                    },
                )
            }
            ExprKind::Index(container, index) => {
                let container_ty = self.infer_expr(container);
                let index_ty = self.infer_expr(index);
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasIndex {
                        container: container_ty.ty,
                        index: index_ty.ty,
                        item: item_ty.clone(),
                    },
                );
                self.diverge_if(
                    container_ty.diverges.or(index_ty.diverges),
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
                let mut diverges = Divergence::Convergent;
                for expr in start.iter().chain(step).chain(end) {
                    let ty = self.infer_expr(expr);
                    diverges = diverges.or(ty.diverges);
                    self.inferrer.eq(expr.span, Ty::Prim(Prim::Int), ty.ty);
                }
                self.diverge_if(diverges, converge(Ty::Prim(Prim::Range)))
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body_ty = self.infer_block(body);
                let until_ty = self.infer_expr(until);
                self.inferrer
                    .eq(until.span, Ty::Prim(Prim::Bool), until_ty.ty);
                let fixup_diverges = fixup
                    .as_ref()
                    .map_or(Divergence::Convergent, |f| self.infer_block(f).diverges);
                self.diverge_if(
                    body_ty.diverges.or(until_ty.diverges).or(fixup_diverges),
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
                let cond_ty = self.infer_expr(cond);
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), cond_ty.ty);
                let true_ty = self.infer_expr(if_true);
                let false_ty = self.infer_expr(if_false);
                self.inferrer.eq(expr.span, true_ty.ty.clone(), false_ty.ty);
                let diverges = cond_ty.diverges.or(true_ty.diverges.and(false_ty.diverges));
                self.diverge_if(
                    diverges,
                    Fallible {
                        ty: true_ty.ty,
                        diverges,
                    },
                )
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                self.infer_update(expr.span, container, index, item)
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = Divergence::Convergent;
                for item in items {
                    let ty = self.infer_expr(item);
                    diverges = diverges.or(ty.diverges);
                    tys.push(ty.ty);
                }
                self.diverge_if(diverges, converge(Ty::Tuple(tys)))
            }
            ExprKind::UnOp(op, expr) => self.infer_unop(*op, expr),
            ExprKind::While(cond, body) => {
                let cond_ty = self.infer_expr(cond);
                self.inferrer
                    .eq(cond.span, Ty::Prim(Prim::Bool), cond_ty.ty);
                let body_ty = self.infer_block(body);
                self.diverge_if(cond_ty.diverges.or(body_ty.diverges), converge(Ty::UNIT))
            }
            ExprKind::Err | ExprKind::Hole => converge(self.inferrer.fresh()),
        };

        self.record(expr.id, ty.ty.clone());
        ty
    }

    fn infer_unop(&mut self, op: UnOp, operand: &Expr) -> Fallible<Ty> {
        let operand_ty = self.infer_expr(operand);
        let diverges = operand_ty.diverges;
        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.inferrer
                    .class(operand.span, Class::Adj(operand_ty.ty.clone()));
                operand_ty
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.inferrer.fresh();
                self.inferrer.class(
                    operand.span,
                    Class::Ctl {
                        op: operand_ty.ty,
                        with_ctls: with_ctls.clone(),
                    },
                );
                converge(with_ctls)
            }
            UnOp::Neg | UnOp::NotB | UnOp::Pos => {
                self.inferrer
                    .class(operand.span, Class::Num(operand_ty.ty.clone()));
                operand_ty
            }
            UnOp::NotL => {
                self.inferrer
                    .eq(operand.span, Ty::Prim(Prim::Bool), operand_ty.ty.clone());
                operand_ty
            }
            UnOp::Unwrap => {
                let base = self.inferrer.fresh();
                self.inferrer.class(
                    operand.span,
                    Class::Unwrap {
                        wrapper: operand_ty.ty,
                        base: base.clone(),
                    },
                );
                converge(base)
            }
        };

        self.diverge_if(diverges, ty)
    }

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Fallible<Ty> {
        let lhs_ty = self.infer_expr(lhs);
        let rhs_ty = self.infer_expr(rhs);
        let diverges = lhs_ty.diverges.or(rhs_ty.diverges);
        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.inferrer.eq(span, lhs_ty.ty.clone(), rhs_ty.ty);
                self.inferrer
                    .eq(lhs.span, Ty::Prim(Prim::Bool), lhs_ty.ty.clone());
                lhs_ty
            }
            BinOp::Eq | BinOp::Neq => {
                self.inferrer.eq(span, lhs_ty.ty.clone(), rhs_ty.ty);
                self.inferrer.class(lhs.span, Class::Eq(lhs_ty.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::Add => {
                self.inferrer.eq(span, lhs_ty.ty.clone(), rhs_ty.ty);
                self.inferrer.class(lhs.span, Class::Add(lhs_ty.ty.clone()));
                lhs_ty
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.inferrer.eq(span, lhs_ty.ty.clone(), rhs_ty.ty);
                self.inferrer.class(lhs.span, Class::Num(lhs_ty.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Sub
            | BinOp::XorB => {
                self.inferrer.eq(span, lhs_ty.ty.clone(), rhs_ty.ty);
                self.inferrer.class(lhs.span, Class::Num(lhs_ty.ty.clone()));
                lhs_ty
            }
            BinOp::Exp => {
                self.inferrer.class(
                    span,
                    Class::Exp {
                        base: lhs_ty.ty.clone(),
                        power: rhs_ty.ty,
                    },
                );
                lhs_ty
            }
            BinOp::Shl | BinOp::Shr => {
                self.inferrer
                    .class(lhs.span, Class::Integral(lhs_ty.ty.clone()));
                self.inferrer.eq(rhs.span, Ty::Prim(Prim::Int), rhs_ty.ty);
                lhs_ty
            }
        };

        self.diverge_if(diverges, ty)
    }

    fn infer_update(
        &mut self,
        span: Span,
        container: &Expr,
        index: &Expr,
        item: &Expr,
    ) -> Fallible<Ty> {
        let container_ty = self.infer_expr(container);
        let index_ty = self.infer_expr(index);
        let item_ty = self.infer_expr(item);
        self.inferrer.class(
            span,
            Class::HasIndex {
                container: container_ty.ty.clone(),
                index: index_ty.ty,
                item: item_ty.ty,
            },
        );
        self.diverge_if(index_ty.diverges.or(item_ty.diverges), container_ty)
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
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let length_ty = self.infer_expr(length);
                self.inferrer
                    .eq(length.span, Ty::Prim(Prim::Int), length_ty.ty);
                self.diverge_if(
                    length_ty.diverges,
                    converge(Ty::Array(Box::new(Ty::Prim(Prim::Qubit)))),
                )
            }
            QubitInitKind::Paren(inner) => self.infer_qubit_init(inner),
            QubitInitKind::Single => converge(Ty::Prim(Prim::Qubit)),
            QubitInitKind::Tuple(items) => {
                let mut diverges = Divergence::Convergent;
                let mut tys = Vec::new();
                for item in items {
                    let ty = self.infer_qubit_init(item);
                    diverges = diverges.or(ty.diverges);
                    tys.push(self.infer_qubit_init(item).ty);
                }
                self.diverge_if(diverges, converge(Ty::Tuple(tys)))
            }
        };

        self.record(init.id, ty.ty.clone());
        ty
    }

    fn diverge(&mut self) -> Fallible<Ty> {
        Fallible {
            ty: self.inferrer.fresh(),
            diverges: Divergence::Divergent,
        }
    }

    fn diverge_if(&mut self, divergence: Divergence, ty: Fallible<Ty>) -> Fallible<Ty> {
        match divergence {
            Divergence::Convergent => ty,
            Divergence::Divergent if matches!(ty.diverges, Divergence::Divergent) => ty,
            Divergence::Divergent => self.diverge(),
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

fn converge<T>(ty: T) -> Fallible<T> {
    Fallible {
        ty,
        diverges: Divergence::Convergent,
    }
}
