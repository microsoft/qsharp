// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    convert,
    infer::{self, Class, Inferrer},
    Error, Tys,
};
use crate::resolve::{Res, Resolutions};
use qsc_ast::ast::{
    self, BinOp, Block, Expr, ExprKind, Functor, FunctorExpr, Lit, NodeId, Pat, PatKind, QubitInit,
    QubitInitKind, Spec, Stmt, StmtKind, TernOp, TyKind, UnOp,
};
use qsc_data_structures::span::Span;
use qsc_hir::hir::{ItemId, PrimTy, Ty};
use std::collections::{HashMap, HashSet};

/// An inferred partial term has a type, but may be the result of a diverging (non-terminating)
/// computation.
struct Partial {
    ty: Ty,
    diverges: bool,
}

struct Context<'a> {
    resolutions: &'a Resolutions,
    globals: &'a HashMap<ItemId, Ty>,
    return_ty: Option<&'a Ty>,
    tys: &'a mut Tys,
    nodes: Vec<NodeId>,
    inferrer: Inferrer,
}

impl<'a> Context<'a> {
    fn new(
        resolutions: &'a Resolutions,
        globals: &'a HashMap<ItemId, Ty>,
        tys: &'a mut Tys,
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
                    Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit))),
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
                convert::callable_kind_from_ast(*kind),
                Box::new(self.infer_ty(input)),
                Box::new(self.infer_ty(output)),
                functors
                    .as_ref()
                    .map_or(HashSet::new(), FunctorExpr::to_set)
                    .into_iter()
                    .map(convert::functor_from_ast)
                    .collect(),
            ),
            TyKind::Hole => self.inferrer.fresh(),
            TyKind::Paren(inner) => self.infer_ty(inner),
            TyKind::Path(_) => Ty::Err, // TODO: Resolve user-defined types.
            TyKind::Param(name) => Ty::Param(name.name.clone()),
            &TyKind::Prim(prim) => Ty::Prim(convert::prim_from_ast(prim)),
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_ty(item)).collect())
            }
        }
    }

    fn infer_block(&mut self, block: &Block) -> Partial {
        let mut diverges = false;
        let mut last = None;
        for stmt in &block.stmts {
            let stmt = self.infer_stmt(stmt);
            diverges = diverges || stmt.diverges;
            last = Some(stmt);
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
                let expr = self.infer_expr(expr);
                self.inferrer.eq(pat.span, expr.ty, pat_ty);
                self.diverge_if(expr.diverges, converge(Ty::UNIT))
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
        };

        self.record(stmt.id, ty.ty.clone());
        ty
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Partial {
        let ty = match &expr.kind {
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
                None => converge(Ty::Array(Box::new(self.inferrer.fresh()))),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item = self.infer_expr(item);
                let size_span = size.span;
                let size = self.infer_expr(size);
                self.inferrer.eq(size_span, Ty::Prim(PrimTy::Int), size.ty);
                self.diverge_if(
                    item.diverges || size.diverges,
                    converge(Ty::Array(Box::new(item.ty))),
                )
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_span = lhs.span;
                let lhs = self.infer_expr(lhs);
                let rhs = self.infer_expr(rhs);
                self.inferrer.eq(lhs_span, lhs.ty, rhs.ty);
                self.diverge_if(lhs.diverges || rhs.diverges, converge(Ty::UNIT))
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let binop = self.infer_binop(expr.span, *op, lhs, rhs);
                self.diverge_if(binop.diverges, converge(Ty::UNIT))
            }
            ExprKind::AssignUpdate(container, index, item) => {
                let update = self.infer_update(expr.span, container, index, item);
                self.diverge_if(update.diverges, converge(Ty::UNIT))
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(expr.span, *op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                // TODO: Handle partial application. (It's probably easier to turn them into lambdas
                // before type inference.)
                // https://github.com/microsoft/qsharp/issues/151
                let callee = self.infer_expr(callee);
                let input = self.infer_expr(input);
                let output_ty = self.inferrer.fresh();
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
                    .eq(message.span, Ty::Prim(PrimTy::String), message_ty);
                self.diverge()
            }
            ExprKind::Field(record, name) => {
                let record = self.infer_expr(record);
                let item_ty = self.inferrer.fresh();
                self.inferrer.class(
                    expr.span,
                    Class::HasField {
                        record: record.ty,
                        name: name.name.clone(),
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
                self.inferrer.eq(cond_span, Ty::Prim(PrimTy::Bool), cond.ty);
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
                let item_ty = self.inferrer.fresh();
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
            ExprKind::Lambda(kind, input, body) => {
                // TODO: Infer the supported functors or require that they are explicitly listed.
                // https://github.com/microsoft/qsharp/issues/151
                let input = self.infer_pat(input);
                let body = self.infer_expr(body).ty;
                converge(Ty::Arrow(
                    convert::callable_kind_from_ast(*kind),
                    Box::new(input),
                    Box::new(body),
                    HashSet::new(),
                ))
            }
            ExprKind::Lit(Lit::BigInt(_)) => converge(Ty::Prim(PrimTy::BigInt)),
            ExprKind::Lit(Lit::Bool(_)) => converge(Ty::Prim(PrimTy::Bool)),
            ExprKind::Lit(Lit::Double(_)) => converge(Ty::Prim(PrimTy::Double)),
            ExprKind::Lit(Lit::Int(_)) => converge(Ty::Prim(PrimTy::Int)),
            ExprKind::Lit(Lit::Pauli(_)) => converge(Ty::Prim(PrimTy::Pauli)),
            ExprKind::Lit(Lit::Result(_)) => converge(Ty::Prim(PrimTy::Result)),
            ExprKind::Lit(Lit::String(_)) => converge(Ty::Prim(PrimTy::String)),
            ExprKind::Paren(expr) => self.infer_expr(expr),
            ExprKind::Path(path) => match self.resolutions.get(path.id) {
                None => converge(Ty::Err),
                Some(Res::Item(item)) => {
                    let mut ty = self
                        .globals
                        .get(item)
                        .expect("global item should have type")
                        .clone();
                    self.inferrer.freshen(&mut ty);
                    converge(ty)
                }
                Some(&Res::Local(node)) => converge(
                    self.tys
                        .get(node)
                        .expect("local variable should have inferred type")
                        .clone(),
                ),
            },
            ExprKind::Range(start, step, end) => {
                let mut diverges = false;
                for expr in start.iter().chain(step).chain(end) {
                    let span = expr.span;
                    let expr = self.infer_expr(expr);
                    diverges = diverges || expr.diverges;
                    self.inferrer.eq(span, Ty::Prim(PrimTy::Int), expr.ty);
                }
                self.diverge_if(diverges, converge(Ty::Prim(PrimTy::Range)))
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body = self.infer_block(body);
                let until_span = until.span;
                let until = self.infer_expr(until);
                self.inferrer
                    .eq(until_span, Ty::Prim(PrimTy::Bool), until.ty);
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
                self.inferrer.eq(cond_span, Ty::Prim(PrimTy::Bool), cond.ty);
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
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                self.infer_update(expr.span, container, index, item)
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut diverges = false;
                for item in items {
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
                self.inferrer.eq(cond_span, Ty::Prim(PrimTy::Bool), cond.ty);
                let body = self.infer_block(body);
                self.diverge_if(cond.diverges || body.diverges, converge(Ty::UNIT))
            }
            ExprKind::Err | ExprKind::Hole => converge(self.inferrer.fresh()),
        };

        self.record(expr.id, ty.ty.clone());
        ty
    }

    fn infer_unop(&mut self, op: UnOp, operand: &Expr) -> Partial {
        let span = operand.span;
        let operand = self.infer_expr(operand);
        let diverges = operand.diverges;
        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.inferrer.class(span, Class::Adj(operand.ty.clone()));
                operand
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.inferrer.fresh();
                self.inferrer.class(
                    span,
                    Class::Ctl {
                        op: operand.ty,
                        with_ctls: with_ctls.clone(),
                    },
                );
                converge(with_ctls)
            }
            UnOp::Neg | UnOp::NotB | UnOp::Pos => {
                self.inferrer.class(span, Class::Num(operand.ty.clone()));
                operand
            }
            UnOp::NotL => {
                self.inferrer
                    .eq(span, Ty::Prim(PrimTy::Bool), operand.ty.clone());
                operand
            }
            UnOp::Unwrap => {
                let base = self.inferrer.fresh();
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

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Partial {
        let lhs_span = lhs.span;
        let lhs = self.infer_expr(lhs);
        let rhs_span = rhs.span;
        let rhs = self.infer_expr(rhs);
        let diverges = lhs.diverges || rhs.diverges;

        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer
                    .eq(lhs_span, Ty::Prim(PrimTy::Bool), lhs.ty.clone());
                lhs
            }
            BinOp::Eq | BinOp::Neq => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Eq(lhs.ty));
                converge(Ty::Prim(PrimTy::Bool))
            }
            BinOp::Add => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Add(lhs.ty.clone()));
                lhs
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.inferrer.eq(span, lhs.ty.clone(), rhs.ty);
                self.inferrer.class(lhs_span, Class::Num(lhs.ty));
                converge(Ty::Prim(PrimTy::Bool))
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Sub
            | BinOp::XorB => {
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
                self.inferrer.eq(rhs_span, Ty::Prim(PrimTy::Int), rhs.ty);
                lhs
            }
        };

        self.diverge_if(diverges, ty)
    }

    fn infer_update(&mut self, span: Span, container: &Expr, index: &Expr, item: &Expr) -> Partial {
        let container = self.infer_expr(container);
        let index = self.infer_expr(index);
        let item = self.infer_expr(item);
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
                let length_span = length.span;
                let length = self.infer_expr(length);
                self.inferrer
                    .eq(length_span, Ty::Prim(PrimTy::Int), length.ty);
                self.diverge_if(
                    length.diverges,
                    converge(Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit)))),
                )
            }
            QubitInitKind::Paren(inner) => self.infer_qubit_init(inner),
            QubitInitKind::Single => converge(Ty::Prim(PrimTy::Qubit)),
            QubitInitKind::Tuple(items) => {
                let mut diverges = false;
                let mut tys = Vec::new();
                for item in items {
                    let item = self.infer_qubit_init(item);
                    diverges = diverges || item.diverges;
                    tys.push(item.ty);
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
    globals: &HashMap<ItemId, Ty>,
    tys: &mut Tys,
    spec: SpecImpl,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_spec(spec);
    context.solve()
}

pub(super) fn expr(
    resolutions: &Resolutions,
    globals: &HashMap<ItemId, Ty>,
    tys: &mut Tys,
    expr: &Expr,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_expr(expr);
    context.solve()
}

pub(super) fn stmt(
    resolutions: &Resolutions,
    globals: &HashMap<ItemId, Ty>,
    tys: &mut Tys,
    stmt: &Stmt,
) -> Vec<Error> {
    let mut context = Context::new(resolutions, globals, tys);
    context.infer_stmt(stmt);
    context.solve()
}

fn converge(ty: Ty) -> Partial {
    Partial {
        ty,
        diverges: false,
    }
}
