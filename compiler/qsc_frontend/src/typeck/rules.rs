// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    convert,
    infer::{ArgTy, Class, Inferrer, TySource},
    Error, Table,
};
use crate::resolve::{self, Names, Res};
use qsc_ast::ast::{
    self, BinOp, Block, Expr, ExprKind, Functor, Lit, NodeId, Pat, PatKind, QubitInit,
    QubitInitKind, Spec, Stmt, StmtKind, StringComponent, TernOp, TyKind, UnOp,
};
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{self, ItemId},
    ty::{Arrow, FunctorSet, FunctorSetValue, GenericArg, Prim, Scheme, Ty},
};
use rustc_hash::FxHashMap;
use std::convert::identity;

/// An inferred partial term has a type, but may be the result of a diverging (non-terminating)
/// computation.
struct Partial<T> {
    ty: T,
    diverges: bool,
}

impl<T> Partial<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U) -> Partial<U> {
        Partial {
            ty: f(self.ty),
            diverges: self.diverges,
        }
    }
}

struct Context<'a> {
    names: &'a Names,
    globals: &'a FxHashMap<ItemId, Scheme>,
    table: &'a mut Table,
    return_ty: Option<Ty>,
    typed_holes: Vec<(NodeId, Span)>,
    new: Vec<NodeId>,
    inferrer: &'a mut Inferrer,
}

impl<'a> Context<'a> {
    fn new(
        names: &'a Names,
        globals: &'a FxHashMap<ItemId, Scheme>,
        table: &'a mut Table,
        inferrer: &'a mut Inferrer,
        new: Vec<NodeId>,
    ) -> Self {
        Self {
            names,
            globals,
            table,
            return_ty: None,
            typed_holes: Vec::new(),
            new,
            inferrer,
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

        self.return_ty = Some(spec.output.clone());
        let block = self.infer_block(spec.block);
        if let Some(return_ty) = self.return_ty.take() {
            let span = spec.block.stmts.last().map_or(spec.block.span, |s| s.span);
            self.inferrer.eq(span, return_ty, block.ty);
        }
    }

    fn infer_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &*ty.kind {
            TyKind::Array(item) => Ty::Array(Box::new(self.infer_ty(item))),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(Box::new(Arrow {
                kind: convert::callable_kind_from_ast(*kind),
                input: Box::new(self.infer_ty(input)),
                output: Box::new(self.infer_ty(output)),
                functors: FunctorSet::Value(
                    functors.as_ref().map_or(FunctorSetValue::Empty, |f| {
                        convert::eval_functor_expr(f.as_ref())
                    }),
                ),
            })),
            TyKind::Hole => self.inferrer.fresh_ty(TySource::not_divergent(ty.span)),
            TyKind::Paren(inner) => self.infer_ty(inner),
            TyKind::Path(path) => match self.names.get(path.id) {
                Some(&Res::Item(item, _)) => Ty::Udt(hir::Res::Item(item)),
                Some(&Res::PrimTy(prim)) => Ty::Prim(prim),
                Some(Res::UnitTy) => Ty::Tuple(Vec::new()),
                None => Ty::Err,
                // a path should never resolve to a parameter,
                // as there is a syntactic difference between
                // paths and parameters.
                // So realistically, by construction, `Param` here is unreachable.
                Some(resolve::Res::Local(_) | resolve::Res::Param(_)) => unreachable!(
                    "A path should never resolve \
                    to a local or a parameter, as there is syntactic differentiation."
                ),
            },
            TyKind::Param(name) => match self.names.get(name.id) {
                Some(Res::Param(id)) => Ty::Param(*id),
                None => Ty::Err,
                Some(_) => unreachable!(
                    "A parameter should never resolve to a non-parameter type, as there \
                    is syntactic differentiation"
                ),
            },
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_ty(item)).collect())
            }
            TyKind::Err => Ty::Err,
        }
    }

    fn infer_block(&mut self, block: &Block) -> Partial<Ty> {
        let mut diverges = false;
        let mut last = None;
        for stmt in &*block.stmts {
            let stmt = self.infer_stmt(stmt);
            diverges = diverges || stmt.diverges;
            last = Some(stmt);
        }

        let ty = self.diverge_if(diverges, last.unwrap_or(converge(Ty::UNIT)));
        self.record(block.id, ty.ty.clone());
        ty
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Partial<Ty> {
        let ty = match &*stmt.kind {
            StmtKind::Empty | StmtKind::Item(_) => converge(Ty::UNIT),
            StmtKind::Expr(expr) => self.infer_expr(expr),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = self.infer_expr(expr);
                self.inferrer.eq(expr.span, pat_ty, expr_ty.ty);
                self.diverge_if(expr_ty.diverges, converge(Ty::UNIT))
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init = self.infer_qubit_init(init);
                self.inferrer.eq(pat.span, init.ty, pat_ty);
                match block {
                    None => self.diverge_if(init.diverges, converge(Ty::UNIT)),
                    Some(block) => {
                        let block_ty = self.infer_block(block);
                        self.diverge_if(init.diverges, block_ty)
                    }
                }
            }
            StmtKind::Semi(expr) => {
                let expr = self.infer_expr(expr);
                self.diverge_if(expr.diverges, converge(Ty::UNIT))
            }
            StmtKind::Err => converge(Ty::Err),
        };

        self.record(stmt.id, ty.ty.clone());
        ty
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Partial<Ty> {
        let ty = match &*expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first = self.infer_expr(first);
                    let mut diverges = first.diverges;
                    for item in rest {
                        let span = item.span;
                        let item = self.infer_expr(item);
                        diverges = diverges || item.diverges;
                        self.inferrer.eq(span, first.ty.clone(), item.ty);
                    }
                    self.diverge_if(diverges, converge(Ty::Array(Box::new(first.ty))))
                }
                None => converge(Ty::Array(Box::new(
                    self.inferrer.fresh_ty(TySource::not_divergent(expr.span)),
                ))),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item = self.infer_expr(item);
                let size_span = size.span;
                let size = self.infer_expr(size);
                self.inferrer.eq(size_span, Ty::Prim(Prim::Int), size.ty);
                self.diverge_if(
                    item.diverges || size.diverges,
                    converge(Ty::Array(Box::new(item.ty))),
                )
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_span = lhs.span;
                let lhs = self.infer_hole_tuple(identity, identity, Ty::Tuple, Ty::clone, lhs);
                let rhs = self.infer_expr(rhs);
                self.inferrer.eq(lhs_span, lhs.ty, rhs.ty);
                self.diverge_if(lhs.diverges || rhs.diverges, converge(Ty::UNIT))
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let binop = self.infer_binop(expr.span, *op, lhs, rhs);
                self.diverge_if(binop.diverges, converge(Ty::UNIT))
            }
            ExprKind::AssignUpdate(container, index, replace) => {
                let update = self.infer_update(expr.span, container, index, replace);
                self.diverge_if(update.diverges, converge(Ty::UNIT))
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(expr.span, *op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                let callee = self.infer_expr(callee);
                let input = self.infer_hole_tuple(
                    ArgTy::Hole,
                    ArgTy::Given,
                    ArgTy::Tuple,
                    ArgTy::to_ty,
                    input,
                );
                let output_ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.inferrer.class(
                    expr.span,
                    Class::Call {
                        callee: callee.ty,
                        input: input.ty,
                        output: output_ty.clone(),
                    },
                );
                self.diverge_if(callee.diverges || input.diverges, converge(output_ty))
            }
            ExprKind::Conjugate(within, apply) => {
                let within = self.infer_block(within);
                let apply = self.infer_block(apply);
                self.diverge_if(within.diverges, apply)
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).ty;
                self.inferrer
                    .eq(message.span, Ty::Prim(Prim::String), message_ty);
                self.diverge()
            }
            ExprKind::Field(record, name) => {
                let record = self.infer_expr(record);
                let item_ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.inferrer.class(
                    expr.span,
                    Class::HasField {
                        record: record.ty,
                        name: name.name.to_string(),
                        item: item_ty.clone(),
                    },
                );
                self.diverge_if(record.diverges, converge(item_ty))
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let container_span = container.span;
                let container = self.infer_expr(container);
                self.inferrer.class(
                    container_span,
                    Class::Iterable {
                        container: container.ty,
                        item: item_ty,
                    },
                );
                let body = self.infer_block(body);
                self.diverge_if(container.diverges || body.diverges, converge(Ty::UNIT))
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_span = cond.span;
                let cond = self.infer_expr(cond);
                self.inferrer.eq(cond_span, Ty::Prim(Prim::Bool), cond.ty);
                let if_true = self.infer_block(if_true);
                let if_false = if_false
                    .as_ref()
                    .map_or(converge(Ty::UNIT), |e| self.infer_expr(e));
                self.inferrer.eq(expr.span, if_true.ty.clone(), if_false.ty);
                self.diverge_if(
                    cond.diverges,
                    Partial {
                        diverges: if_true.diverges && if_false.diverges,
                        ..if_true
                    },
                )
            }
            ExprKind::Index(container, index) => {
                let container = self.infer_expr(container);
                let index = self.infer_expr(index);
                let item_ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.inferrer.class(
                    expr.span,
                    Class::HasIndex {
                        container: container.ty,
                        index: index.ty,
                        item: item_ty.clone(),
                    },
                );
                self.diverge_if(container.diverges || index.diverges, converge(item_ty))
            }
            ExprKind::Interpolate(components) => {
                let mut diverges = false;
                for component in components.iter() {
                    match component {
                        StringComponent::Expr(expr) => {
                            let span = expr.span;
                            let expr = self.infer_expr(expr.as_ref());
                            self.inferrer.class(span, Class::Show(expr.ty));
                            diverges = diverges || expr.diverges;
                        }
                        StringComponent::Lit(_) => {}
                    }
                }

                self.diverge_if(diverges, converge(Ty::Prim(Prim::String)))
            }
            ExprKind::Lambda(kind, input, body) => {
                let input = self.infer_pat(input);
                let prev_ret_ty = self.return_ty.take();
                let output_ty = self.inferrer.fresh_ty(TySource::not_divergent(body.span));
                self.return_ty = Some(output_ty);
                let body_partial = self.infer_expr(body);
                let output_ty = self
                    .return_ty
                    .take()
                    .expect("return type should be present");
                self.return_ty = prev_ret_ty;
                if !body_partial.diverges {
                    // Only when the type of the body converges do we need to unify with the inferred output type.
                    // Otherwise we'd get spurious errors from lambdas that use explicit return-expr rather than implicit.
                    self.inferrer
                        .eq(body.span, output_ty.clone(), body_partial.ty);
                }
                converge(Ty::Arrow(Box::new(Arrow {
                    kind: convert::callable_kind_from_ast(*kind),
                    input: Box::new(input),
                    output: Box::new(output_ty),
                    functors: self.inferrer.fresh_functor(),
                })))
            }
            ExprKind::Lit(lit) => match lit.as_ref() {
                Lit::BigInt(_) => converge(Ty::Prim(Prim::BigInt)),
                Lit::Bool(_) => converge(Ty::Prim(Prim::Bool)),
                Lit::Double(_) => converge(Ty::Prim(Prim::Double)),
                Lit::Int(_) => converge(Ty::Prim(Prim::Int)),
                Lit::Pauli(_) => converge(Ty::Prim(Prim::Pauli)),
                Lit::Result(_) => converge(Ty::Prim(Prim::Result)),
                Lit::String(_) => converge(Ty::Prim(Prim::String)),
            },
            ExprKind::Paren(expr) => self.infer_expr(expr),
            ExprKind::Path(path) => match self.names.get(path.id) {
                None => converge(Ty::Err),
                Some(Res::Item(item, _)) => {
                    let scheme = self.globals.get(item).expect("item should have scheme");
                    let (ty, args) = self.inferrer.instantiate(scheme, expr.span);
                    self.table.generics.insert(expr.id, args);
                    converge(Ty::Arrow(Box::new(ty)))
                }
                Some(&Res::Local(node)) => converge(
                    self.table
                        .terms
                        .get(node)
                        .expect("local should have type")
                        .clone(),
                ),
                Some(Res::PrimTy(_) | Res::UnitTy | Res::Param(_)) => {
                    panic!("expression resolves to type")
                }
            },
            ExprKind::Range(start, step, end) => {
                let mut diverges = false;
                for expr in start.iter().chain(step).chain(end) {
                    let span = expr.span;
                    let expr = self.infer_expr(expr);
                    diverges = diverges || expr.diverges;
                    self.inferrer.eq(span, Ty::Prim(Prim::Int), expr.ty);
                }

                let ty = if start.is_none() && end.is_none() {
                    Prim::RangeFull
                } else if start.is_none() {
                    Prim::RangeTo
                } else if end.is_none() {
                    Prim::RangeFrom
                } else {
                    Prim::Range
                };

                self.diverge_if(diverges, converge(Ty::Prim(ty)))
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body = self.infer_block(body);
                let until_span = until.span;
                let until = self.infer_expr(until);
                self.inferrer.eq(until_span, Ty::Prim(Prim::Bool), until.ty);
                let fixup_diverges = fixup
                    .as_ref()
                    .map_or(false, |f| self.infer_block(f).diverges);
                self.diverge_if(
                    body.diverges || until.diverges || fixup_diverges,
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
                let cond_span = cond.span;
                let cond = self.infer_expr(cond);
                self.inferrer.eq(cond_span, Ty::Prim(Prim::Bool), cond.ty);
                let if_true = self.infer_expr(if_true);
                let if_false = self.infer_expr(if_false);
                self.inferrer.eq(expr.span, if_true.ty.clone(), if_false.ty);
                self.diverge_if(
                    cond.diverges,
                    Partial {
                        diverges: if_true.diverges && if_false.diverges,
                        ..if_true
                    },
                )
            }
            ExprKind::TernOp(TernOp::Update, container, index, replace) => {
                self.infer_update(expr.span, container, index, replace)
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items.iter() {
                    let item = self.infer_expr(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                self.diverge_if(diverges, converge(Ty::Tuple(tys)))
            }
            ExprKind::UnOp(op, expr) => self.infer_unop(*op, expr),
            ExprKind::While(cond, body) => {
                let cond_span = cond.span;
                let cond = self.infer_expr(cond);
                self.inferrer.eq(cond_span, Ty::Prim(Prim::Bool), cond.ty);
                let body = self.infer_block(body);
                self.diverge_if(cond.diverges || body.diverges, converge(Ty::UNIT))
            }
            ExprKind::Hole => {
                self.typed_holes.push((expr.id, expr.span));
                converge(self.inferrer.fresh_ty(TySource::not_divergent(expr.span)))
            }
            ExprKind::Err => converge(Ty::Err),
        };

        self.record(expr.id, ty.ty.clone());
        ty
    }

    fn infer_hole_tuple<T>(
        &mut self,
        hole: fn(Ty) -> T,
        given: fn(Ty) -> T,
        tuple: fn(Vec<T>) -> T,
        to_ty: fn(&T) -> Ty,
        expr: &Expr,
    ) -> Partial<T> {
        match expr.kind.as_ref() {
            ExprKind::Hole => {
                let ty = self.inferrer.fresh_ty(TySource::not_divergent(expr.span));
                self.record(expr.id, ty.clone());
                converge(hole(ty))
            }
            ExprKind::Paren(inner) => {
                let inner = self.infer_hole_tuple(hole, given, tuple, to_ty, inner);
                self.record(expr.id, to_ty(&inner.ty));
                inner
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items.iter() {
                    let item = self.infer_hole_tuple(hole, given, tuple, to_ty, item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                self.record(expr.id, Ty::Tuple(tys.iter().map(to_ty).collect()));
                self.diverge_if_map(given, diverges, converge(tuple(tys)))
            }
            _ => self.infer_expr(expr).map(given),
        }
    }

    fn infer_unop(&mut self, op: UnOp, operand: &Expr) -> Partial<Ty> {
        let span = operand.span;
        let operand = self.infer_expr(operand);
        let diverges = operand.diverges;
        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.inferrer.class(span, Class::Adj(operand.ty.clone()));
                operand
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.inferrer.fresh_ty(TySource::not_divergent(span));
                self.inferrer.class(
                    span,
                    Class::Ctl {
                        op: operand.ty,
                        with_ctls: with_ctls.clone(),
                    },
                );
                converge(with_ctls)
            }
            UnOp::Neg | UnOp::Pos => {
                self.inferrer.class(span, Class::Num(operand.ty.clone()));
                operand
            }
            UnOp::NotB => {
                self.inferrer
                    .class(span, Class::Integral(operand.ty.clone()));
                operand
            }
            UnOp::NotL => {
                self.inferrer
                    .eq(span, Ty::Prim(Prim::Bool), operand.ty.clone());
                operand
            }
            UnOp::Unwrap => {
                let base = self.inferrer.fresh_ty(TySource::not_divergent(span));
                self.inferrer.class(
                    span,
                    Class::Unwrap {
                        wrapper: operand.ty,
                        base: base.clone(),
                    },
                );
                converge(base)
            }
        };

        self.diverge_if(diverges, ty)
    }

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Partial<Ty> {
        let lhs_span = lhs.span;
        let lhs = self.infer_expr(lhs);
        let rhs_span = rhs.span;
        let rhs = self.infer_expr(rhs);
        let diverges = lhs.diverges || rhs.diverges;

        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer
                    .eq(lhs_span, Ty::Prim(Prim::Bool), lhs.ty.clone());
                lhs
            }
            BinOp::Eq | BinOp::Neq => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Eq(lhs.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::Add => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Add(lhs.ty.clone()));
                lhs
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Num(lhs.ty));
                converge(Ty::Prim(Prim::Bool))
            }
            BinOp::AndB | BinOp::OrB | BinOp::XorB => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer
                    .class(lhs_span, Class::Integral(lhs.ty.clone()));
                lhs
            }
            BinOp::Div | BinOp::Mod | BinOp::Mul | BinOp::Sub => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Num(lhs.ty.clone()));
                lhs
            }
            BinOp::Exp => {
                self.inferrer.class(
                    span,
                    Class::Exp {
                        base: lhs.ty.clone(),
                        power: rhs.ty,
                    },
                );
                lhs
            }
            BinOp::Shl | BinOp::Shr => {
                self.inferrer
                    .class(lhs_span, Class::Integral(lhs.ty.clone()));
                self.inferrer.eq(rhs_span, Ty::Prim(Prim::Int), rhs.ty);
                lhs
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
    ) -> Partial<Ty> {
        let container = self.infer_expr(container);
        let item = self.infer_expr(item);
        if let Some(field) = resolve::extract_field_name(self.names, index) {
            self.inferrer.class(
                span,
                Class::HasField {
                    record: container.ty.clone(),
                    name: field.to_string(),
                    item: item.ty.clone(),
                },
            );
            self.diverge_if(item.diverges, container)
        } else {
            let index = self.infer_expr(index);
            self.inferrer.class(
                span,
                Class::HasIndex {
                    container: container.ty.clone(),
                    index: index.ty,
                    item: item.ty,
                },
            );
            self.diverge_if(index.diverges || item.diverges, container)
        }
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        let ty = match &*pat.kind {
            PatKind::Bind(name, None) => {
                let ty = self.inferrer.fresh_ty(TySource::not_divergent(pat.span));
                self.record(name.id, ty.clone());
                ty
            }
            PatKind::Bind(name, Some(ty)) => {
                let ty = self.infer_ty(ty);
                self.record(name.id, ty.clone());
                ty
            }
            PatKind::Discard(None) | PatKind::Elided => {
                self.inferrer.fresh_ty(TySource::not_divergent(pat.span))
            }
            PatKind::Discard(Some(ty)) => self.infer_ty(ty),
            PatKind::Paren(inner) => self.infer_pat(inner),
            PatKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_pat(item)).collect())
            }
            PatKind::Err => Ty::Err,
        };

        self.record(pat.id, ty.clone());
        ty
    }

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Partial<Ty> {
        let ty = match &*init.kind {
            QubitInitKind::Array(length) => {
                let length_span = length.span;
                let length = self.infer_expr(length);
                self.inferrer
                    .eq(length_span, Ty::Prim(Prim::Int), length.ty);
                self.diverge_if(
                    length.diverges,
                    converge(Ty::Array(Box::new(Ty::Prim(Prim::Qubit)))),
                )
            }
            QubitInitKind::Paren(inner) => self.infer_qubit_init(inner),
            QubitInitKind::Single => converge(Ty::Prim(Prim::Qubit)),
            QubitInitKind::Tuple(items) => {
                let mut diverges = false;
                let mut tys = Vec::new();
                for item in items.iter() {
                    let item = self.infer_qubit_init(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
                }
                self.diverge_if(diverges, converge(Ty::Tuple(tys)))
            }
            QubitInitKind::Err => converge(Ty::Err),
        };

        self.record(init.id, ty.ty.clone());
        ty
    }

    fn diverge(&mut self) -> Partial<Ty> {
        Partial {
            ty: self.inferrer.fresh_ty(TySource::divergent()),
            diverges: true,
        }
    }

    fn diverge_if(&mut self, diverges: bool, partial: Partial<Ty>) -> Partial<Ty> {
        self.diverge_if_map(identity, diverges, partial)
    }

    fn diverge_if_map<T>(
        &mut self,
        f: impl FnOnce(Ty) -> T,
        diverges: bool,
        partial: Partial<T>,
    ) -> Partial<T> {
        if !diverges || partial.diverges {
            partial
        } else {
            self.diverge().map(f)
        }
    }

    fn record(&mut self, id: NodeId, ty: Ty) {
        self.new.push(id);
        self.table.terms.insert(id, ty);
    }

    pub(crate) fn solve(self) -> Vec<Error> {
        let mut errs = self.inferrer.solve(&self.table.udts);

        for id in self.new {
            let ty = self.table.terms.get_mut(id).expect("node should have type");
            self.inferrer.substitute_ty(ty);

            if let Some(args) = self.table.generics.get_mut(id) {
                for arg in args {
                    match arg {
                        GenericArg::Ty(ty) => self.inferrer.substitute_ty(ty),
                        GenericArg::Functor(functors) => {
                            self.inferrer.substitute_functor(functors);
                        }
                    }
                }
            }
        }

        for (id, span) in self.typed_holes {
            let ty = self.table.terms.get_mut(id).expect("node should have type");
            errs.push(Error(super::ErrorKind::TyHole(ty.clone(), span)));
        }

        errs
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
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    spec: SpecImpl,
) -> Vec<Error> {
    let mut inferrer = Inferrer::new();
    let mut context = Context::new(names, globals, table, &mut inferrer, Vec::new());
    context.infer_spec(spec);
    context.solve()
}

pub(super) fn expr(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    expr: &Expr,
) -> Vec<Error> {
    let mut inferrer = Inferrer::new();
    let mut context = Context::new(names, globals, table, &mut inferrer, Vec::new());
    context.infer_expr(expr);
    context.solve()
}

pub(super) fn stmt(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    inferrer: &mut Inferrer,
    stmt: &Stmt,
) -> Vec<NodeId> {
    let mut context = Context::new(names, globals, table, inferrer, Vec::new());
    context.infer_stmt(stmt);
    context.new
}

pub(super) fn solve(
    names: &Names,
    globals: &FxHashMap<ItemId, Scheme>,
    table: &mut Table,
    inferrer: &mut Inferrer,
    new_nodes: Vec<NodeId>,
) -> Vec<Error> {
    let context = Context::new(names, globals, table, inferrer, new_nodes);
    context.solve()
}

fn converge<T>(ty: T) -> Partial<T> {
    Partial {
        ty,
        diverges: false,
    }
}
