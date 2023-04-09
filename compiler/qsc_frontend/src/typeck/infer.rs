// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    solve::{self, Class, Solver, Ty},
    Error, Tys,
};
use crate::resolve::{DefId, PackageSrc, Resolutions};
use qsc_ast::ast::{
    self, BinOp, Block, Expr, ExprKind, Functor, FunctorExpr, Lit, Pat, PatKind, QubitInit,
    QubitInitKind, Span, Spec, Stmt, StmtKind, TernOp, TyKind, TyPrim, UnOp,
};
use std::collections::{HashMap, HashSet};

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

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum Termination {
    #[default]
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

    fn then<T>(&mut self, fallible: Fallible<T>) -> T {
        *self = self.or(fallible.term);
        fallible.value
    }
}

struct Context<'a> {
    resolutions: &'a Resolutions,
    globals: &'a HashMap<DefId, Ty>,
    return_ty: Option<&'a Ty>,
    tys: Tys,
    solver: Solver,
}

impl<'a> Context<'a> {
    fn new(resolutions: &'a Resolutions, globals: &'a HashMap<DefId, Ty>) -> Self {
        Self {
            resolutions,
            globals,
            return_ty: None,
            tys: Tys::new(),
            solver: Solver::new(),
        }
    }

    fn infer_spec(&mut self, spec: SpecImpl<'a>) {
        let callable_input = self.infer_pat(spec.callable_input);
        if let Some(input) = spec.spec_input {
            let expected = match spec.spec {
                Spec::Body | Spec::Adj => callable_input,
                Spec::Ctl | Spec::CtlAdj => Ty::Tuple(vec![
                    Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit))),
                    callable_input,
                ]),
            };
            let actual = self.infer_pat(input);
            self.solver.eq(input.span, expected, actual);
        }

        self.return_ty = Some(spec.output);
        let block = self.infer_block(spec.block).value;
        if let Some(return_ty) = self.return_ty {
            self.solver.eq(spec.block.span, return_ty.clone(), block);
        }

        self.return_ty = None;
    }

    fn infer_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &ty.kind {
            TyKind::Array(item) => Ty::Array(Box::new(self.infer_ty(item))),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(
                *kind,
                Box::new(self.infer_ty(input)),
                Box::new(self.infer_ty(output)),
                functors
                    .as_ref()
                    .map_or(HashSet::new(), FunctorExpr::to_set),
            ),
            TyKind::Hole => self.solver.fresh(),
            TyKind::Paren(inner) => self.infer_ty(inner),
            TyKind::Path(path) => Ty::DefId(
                *self
                    .resolutions
                    .get(&path.id)
                    .expect("path should be resolved"),
            ),
            &TyKind::Prim(prim) => Ty::Prim(prim),
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_ty(item)).collect())
            }
            TyKind::Var(name) => Ty::Param(name.name.clone()),
        }
    }

    fn infer_block(&mut self, block: &Block) -> Fallible<Ty> {
        let mut term = Termination::default();
        let mut last = None;
        for stmt in &block.stmts {
            last = Some(term.then(self.infer_stmt(stmt)));
        }

        let ty = self.diverge_or(term, last.unwrap_or(Ty::UNIT));
        self.tys.insert(block.id, ty.clone());
        term.with(ty)
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Fallible<Ty> {
        let mut term = Termination::default();
        let ty = match &stmt.kind {
            StmtKind::Empty => Ty::UNIT,
            StmtKind::Expr(expr) => term.then(self.infer_expr(expr)),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = term.then(self.infer_expr(expr));
                self.solver.eq(pat.span, expr_ty, pat_ty);
                Ty::UNIT
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init_ty = term.then(self.infer_qubit_init(init));
                self.solver.eq(pat.span, init_ty, pat_ty);
                match block {
                    None => Ty::UNIT,
                    Some(block) => term.then(self.infer_block(block)),
                }
            }
            StmtKind::Semi(expr) => {
                term.then(self.infer_expr(expr));
                Ty::UNIT
            }
        };

        let ty = self.diverge_or(term, ty);
        self.tys.insert(stmt.id, ty.clone());
        term.with(ty)
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Fallible<Ty> {
        let mut term = Termination::default();
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_ty = term.then(self.infer_expr(first));
                    for item in rest {
                        let item_ty = term.then(self.infer_expr(item));
                        self.solver.eq(item.span, first_ty.clone(), item_ty);
                    }

                    Ty::Array(Box::new(first_ty))
                }
                None => Ty::Array(Box::new(self.solver.fresh())),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item_ty = term.then(self.infer_expr(item));
                let size_ty = term.then(self.infer_expr(size));
                self.solver.eq(size.span, Ty::Prim(TyPrim::Int), size_ty);
                Ty::Array(Box::new(item_ty))
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_ty = term.then(self.infer_expr(lhs));
                let rhs_ty = term.then(self.infer_expr(rhs));
                self.solver.eq(lhs.span, lhs_ty, rhs_ty);
                Ty::UNIT
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                term.then(self.infer_binop(expr.span, *op, lhs, rhs));
                Ty::UNIT
            }
            ExprKind::AssignUpdate(container, index, item) => {
                term.then(self.infer_update(expr.span, container, index, item));
                Ty::UNIT
            }
            ExprKind::BinOp(op, lhs, rhs) => term.then(self.infer_binop(expr.span, *op, lhs, rhs)),
            ExprKind::Block(block) => term.then(self.infer_block(block)),
            ExprKind::Call(callee, input) => {
                let callee_ty = term.then(self.infer_expr(callee));
                let input_ty = term.then(self.infer_expr(input));
                let output_ty = self.solver.fresh();
                self.solver.class(
                    expr.span,
                    Class::Call {
                        callee: callee_ty,
                        input: input_ty,
                        output: output_ty.clone(),
                    },
                );
                output_ty
            }
            ExprKind::Conjugate(within, apply) => {
                term.then(self.infer_block(within));
                term.then(self.infer_block(apply))
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).value;
                self.solver
                    .eq(message.span, Ty::Prim(TyPrim::String), message_ty);
                term = Termination::Divergent;
                Ty::Err
            }
            ExprKind::Field(record, name) => {
                let record_ty = term.then(self.infer_expr(record));
                let item_ty = self.solver.fresh();
                self.solver.class(
                    expr.span,
                    Class::HasField {
                        record: record_ty,
                        name: name.name.clone(),
                        item: item_ty.clone(),
                    },
                );
                item_ty
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let container_ty = term.then(self.infer_expr(container));
                self.solver.class(
                    container.span,
                    Class::Iterable {
                        container: container_ty,
                        item: item_ty,
                    },
                );

                term.then(self.infer_block(body));
                Ty::UNIT
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_ty = term.then(self.infer_expr(cond));
                self.solver.eq(cond.span, Ty::Prim(TyPrim::Bool), cond_ty);
                let true_ty = self.infer_block(if_true);
                let false_ty = if_false
                    .as_ref()
                    .map_or(Termination::default().with(Ty::UNIT), |e| {
                        self.infer_expr(e)
                    });
                let (true_ty, false_ty) = term.then(true_ty.and(false_ty));
                self.solver.eq(expr.span, true_ty.clone(), false_ty);
                true_ty
            }
            ExprKind::Index(container, index) => {
                let container_ty = term.then(self.infer_expr(container));
                let index_ty = term.then(self.infer_expr(index));
                let item_ty = self.solver.fresh();
                self.solver.class(
                    expr.span,
                    Class::HasIndex {
                        container: container_ty,
                        index: index_ty,
                        item: item_ty.clone(),
                    },
                );
                item_ty
            }
            ExprKind::Lambda(kind, input, body) => {
                let input = self.infer_pat(input);
                let body = term.then(self.infer_expr(body));
                Ty::Arrow(*kind, Box::new(input), Box::new(body), HashSet::new())
            }
            ExprKind::Lit(Lit::BigInt(_)) => Ty::Prim(TyPrim::BigInt),
            ExprKind::Lit(Lit::Bool(_)) => Ty::Prim(TyPrim::Bool),
            ExprKind::Lit(Lit::Double(_)) => Ty::Prim(TyPrim::Double),
            ExprKind::Lit(Lit::Int(_)) => Ty::Prim(TyPrim::Int),
            ExprKind::Lit(Lit::Pauli(_)) => Ty::Prim(TyPrim::Pauli),
            ExprKind::Lit(Lit::Result(_)) => Ty::Prim(TyPrim::Result),
            ExprKind::Lit(Lit::String(_)) => Ty::Prim(TyPrim::String),
            ExprKind::Paren(expr) => term.then(self.infer_expr(expr)),
            ExprKind::Path(path) => match self.resolutions.get(&path.id) {
                None => Ty::Err,
                Some(id) => {
                    if let Some(ty) = self.globals.get(id) {
                        self.solver.freshen(ty)
                    } else if id.package == PackageSrc::Local {
                        self.tys
                            .get(&id.node)
                            .expect("local variable should have inferred type")
                            .clone()
                    } else {
                        panic!("path resolves to external package, but definition not found in globals")
                    }
                }
            },
            ExprKind::Range(start, step, end) => {
                for expr in start.iter().chain(step).chain(end) {
                    let ty = term.then(self.infer_expr(expr));
                    self.solver.eq(expr.span, Ty::Prim(TyPrim::Int), ty);
                }
                Ty::Prim(TyPrim::Range)
            }
            ExprKind::Repeat(body, until, fixup) => {
                term.then(self.infer_block(body));
                let until_ty = term.then(self.infer_expr(until));
                self.solver.eq(until.span, Ty::Prim(TyPrim::Bool), until_ty);
                if let Some(fixup) = fixup {
                    term.then(self.infer_block(fixup));
                }
                Ty::UNIT
            }
            ExprKind::Return(expr) => {
                let ty = self.infer_expr(expr).value;
                if let Some(return_ty) = &self.return_ty {
                    self.solver.eq(expr.span, (*return_ty).clone(), ty);
                }
                term = Termination::Divergent;
                Ty::Err
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond_ty = term.then(self.infer_expr(cond));
                self.solver.eq(cond.span, Ty::Prim(TyPrim::Bool), cond_ty);
                let true_ty = self.infer_expr(if_true);
                let false_ty = self.infer_expr(if_false);
                let (true_ty, false_ty) = term.then(true_ty.and(false_ty));
                self.solver.eq(expr.span, true_ty.clone(), false_ty);
                true_ty
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                term.then(self.infer_update(expr.span, container, index, item))
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = term.then(self.infer_expr(item));
                    tys.push(ty);
                }
                Ty::Tuple(tys)
            }
            ExprKind::UnOp(op, expr) => term.then(self.infer_unop(*op, expr)),
            ExprKind::While(cond, body) => {
                let cond_ty = term.then(self.infer_expr(cond));
                self.solver.eq(cond.span, Ty::Prim(TyPrim::Bool), cond_ty);
                term.then(self.infer_block(body));
                Ty::UNIT
            }
            ExprKind::Err | ExprKind::Hole => self.solver.fresh(),
        };

        let ty = self.diverge_or(term, ty);
        self.tys.insert(expr.id, ty.clone());
        term.with(ty)
    }

    fn infer_unop(&mut self, op: UnOp, expr: &Expr) -> Fallible<Ty> {
        let Fallible {
            term,
            value: operand_ty,
        } = self.infer_expr(expr);

        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.solver.class(expr.span, Class::Adj(operand_ty.clone()));
                operand_ty
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.solver.fresh();
                self.solver.class(
                    expr.span,
                    Class::Ctl {
                        op: operand_ty,
                        with_ctls: with_ctls.clone(),
                    },
                );
                with_ctls
            }
            UnOp::Neg | UnOp::NotB | UnOp::Pos => {
                self.solver.class(expr.span, Class::Num(operand_ty.clone()));
                operand_ty
            }
            UnOp::NotL => {
                self.solver
                    .eq(expr.span, Ty::Prim(TyPrim::Bool), operand_ty.clone());
                operand_ty
            }
            UnOp::Unwrap => {
                let base = self.solver.fresh();
                self.solver.class(
                    expr.span,
                    Class::Unwrap {
                        wrapper: operand_ty,
                        base: base.clone(),
                    },
                );
                base
            }
        };

        term.with(self.diverge_or(term, ty))
    }

    #[allow(clippy::too_many_lines)]
    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Fallible<Ty> {
        let mut term = Termination::default();
        let lhs_ty = term.then(self.infer_expr(lhs));
        let rhs_ty = term.then(self.infer_expr(rhs));

        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.solver.eq(span, lhs_ty.clone(), rhs_ty);
                self.solver
                    .eq(lhs.span, Ty::Prim(TyPrim::Bool), lhs_ty.clone());
                lhs_ty
            }
            BinOp::Eq | BinOp::Neq => {
                self.solver.eq(span, lhs_ty.clone(), rhs_ty);
                self.solver.class(lhs.span, Class::Eq(lhs_ty));
                Ty::Prim(TyPrim::Bool)
            }
            BinOp::Add => {
                self.solver.eq(span, lhs_ty.clone(), rhs_ty);
                self.solver.class(lhs.span, Class::Add(lhs_ty.clone()));
                lhs_ty
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.solver.eq(span, lhs_ty.clone(), rhs_ty);
                self.solver.class(lhs.span, Class::Num(lhs_ty));
                Ty::Prim(TyPrim::Bool)
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Sub
            | BinOp::XorB => {
                self.solver.eq(span, lhs_ty.clone(), rhs_ty);
                self.solver.class(lhs.span, Class::Num(lhs_ty.clone()));
                lhs_ty
            }
            BinOp::Exp => {
                self.solver.class(
                    span,
                    Class::Exp {
                        base: lhs_ty.clone(),
                        power: rhs_ty,
                    },
                );
                lhs_ty
            }
            BinOp::Shl | BinOp::Shr => {
                self.solver.class(lhs.span, Class::Integral(lhs_ty.clone()));
                self.solver.eq(rhs.span, Ty::Prim(TyPrim::Int), rhs_ty);
                lhs_ty
            }
        };

        term.with(self.diverge_or(term, ty))
    }

    fn infer_update(
        &mut self,
        span: Span,
        container: &Expr,
        index: &Expr,
        item: &Expr,
    ) -> Fallible<Ty> {
        let mut term = Termination::default();
        let container_ty = term.then(self.infer_expr(container));
        let index_ty = term.then(self.infer_expr(index));
        let item_ty = term.then(self.infer_expr(item));
        self.solver.class(
            span,
            Class::HasIndex {
                container: container_ty.clone(),
                index: index_ty,
                item: item_ty,
            },
        );

        term.with(self.diverge_or(term, container_ty))
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        let ty = match &pat.kind {
            PatKind::Bind(name, None) => {
                let ty = self.solver.fresh();
                self.tys.insert(name.id, ty.clone());
                ty
            }
            PatKind::Bind(name, Some(ty)) => {
                let ty = self.infer_ty(ty);
                self.tys.insert(name.id, ty.clone());
                ty
            }
            PatKind::Discard(None) | PatKind::Elided => self.solver.fresh(),
            PatKind::Discard(Some(ty)) => self.infer_ty(ty),
            PatKind::Paren(inner) => self.infer_pat(inner),
            PatKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_pat(item)).collect())
            }
        };

        self.tys.insert(pat.id, ty.clone());
        ty
    }

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Fallible<Ty> {
        let mut term = Termination::default();
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let length_ty = term.then(self.infer_expr(length));
                self.solver
                    .eq(length.span, Ty::Prim(TyPrim::Int), length_ty);
                Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit)))
            }
            QubitInitKind::Paren(inner) => term.then(self.infer_qubit_init(inner)),
            QubitInitKind::Single => Ty::Prim(TyPrim::Qubit),
            QubitInitKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = term.then(self.infer_qubit_init(item));
                    tys.push(ty);
                }
                Ty::Tuple(tys)
            }
        };

        let ty = self.diverge_or(term, ty);
        self.tys.insert(init.id, ty.clone());
        term.with(ty)
    }

    fn diverge_or(&mut self, term: Termination, default: Ty) -> Ty {
        match term {
            Termination::Convergent => default,
            Termination::Divergent => self.solver.fresh(),
        }
    }

    fn solve(self) -> (Tys, Vec<Error>) {
        let (substs, errors) = self.solver.solve();
        let tys = self
            .tys
            .into_iter()
            .map(|(id, ty)| (id, solve::substitute(&substs, ty)))
            .collect();
        (tys, errors)
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

pub(super) fn entry(
    resolutions: &Resolutions,
    globals: &HashMap<DefId, Ty>,
    entry: &Expr,
) -> (Tys, Vec<Error>) {
    let mut context = Context::new(resolutions, globals);
    context.infer_expr(entry);
    context.solve()
}

pub(super) fn spec(
    resolutions: &Resolutions,
    globals: &HashMap<DefId, Ty>,
    spec: SpecImpl,
) -> (Tys, Vec<Error>) {
    let mut context = Context::new(resolutions, globals);
    context.infer_spec(spec);
    context.solve()
}
