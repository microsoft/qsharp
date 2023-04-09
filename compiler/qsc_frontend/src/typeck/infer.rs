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

struct Context<'a> {
    resolutions: &'a Resolutions,
    globals: &'a HashMap<DefId, Ty>,
    return_ty: Option<&'a Ty>,
    tys: Tys,
    solver: Solver,
}

impl<'a> Context<'a> {
    fn new(
        resolutions: &'a Resolutions,
        globals: &'a HashMap<DefId, Ty>,
        return_ty: Option<&'a Ty>,
    ) -> Self {
        Self {
            resolutions,
            globals,
            return_ty,
            tys: Tys::new(),
            solver: Solver::new(),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_ty = termination.update(self.infer_expr(first));
                    for item in rest {
                        let item_ty = termination.update(self.infer_expr(item));
                        self.solver.eq(item.span, first_ty.clone(), item_ty);
                    }

                    Ty::Array(Box::new(first_ty))
                }
                None => Ty::Array(Box::new(self.solver.fresh())),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item_ty = termination.update(self.infer_expr(item));
                let size_ty = termination.update(self.infer_expr(size));
                self.solver.eq(size.span, Ty::Prim(TyPrim::Int), size_ty);
                Ty::Array(Box::new(item_ty))
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_ty = self.infer_expr(lhs).unwrap();
                let rhs_ty = termination.update(self.infer_expr(rhs));
                self.solver.eq(lhs.span, lhs_ty, rhs_ty);
                Ty::UNIT
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                termination.update(self.infer_binop(expr.span, *op, lhs, rhs));
                Ty::UNIT
            }
            ExprKind::AssignUpdate(container, index, item) => {
                termination.update(self.infer_update(expr.span, container, index, item));
                Ty::UNIT
            }
            ExprKind::BinOp(op, lhs, rhs) => {
                termination.update(self.infer_binop(expr.span, *op, lhs, rhs))
            }
            ExprKind::Block(block) => termination.update(self.infer_block(block)),
            ExprKind::Call(callee, input) => {
                let callee_ty = termination.update(self.infer_expr(callee));
                let input_ty = termination.update(self.infer_expr(input));
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
                termination.update(self.infer_block(within));
                termination.update(self.infer_block(apply))
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).unwrap();
                self.solver
                    .eq(message.span, Ty::Prim(TyPrim::String), message_ty);
                termination = Termination::Diverges;
                Ty::Err
            }
            ExprKind::Field(record, name) => {
                let record_ty = termination.update(self.infer_expr(record));
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
                let container_ty = termination.update(self.infer_expr(container));
                self.solver.class(
                    container.span,
                    Class::Iterable {
                        container: container_ty,
                        item: item_ty,
                    },
                );

                termination.update(self.infer_block(body));
                Ty::UNIT
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_ty = termination.update(self.infer_expr(cond));
                self.solver.eq(cond.span, Ty::Prim(TyPrim::Bool), cond_ty);
                let true_ty = self.infer_block(if_true);
                let false_ty = if_false
                    .as_ref()
                    .map_or(Fallible::Convergent(Ty::UNIT), |e| self.infer_expr(e));
                let (true_ty, false_ty) = termination.update_and(true_ty, false_ty);
                self.solver.eq(expr.span, true_ty.clone(), false_ty);
                true_ty
            }
            ExprKind::Index(container, index) => {
                let container_ty = termination.update(self.infer_expr(container));
                let index_ty = termination.update(self.infer_expr(index));
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
                let body = termination.update(self.infer_expr(body));
                Ty::Arrow(*kind, Box::new(input), Box::new(body), HashSet::new())
            }
            ExprKind::Lit(Lit::BigInt(_)) => Ty::Prim(TyPrim::BigInt),
            ExprKind::Lit(Lit::Bool(_)) => Ty::Prim(TyPrim::Bool),
            ExprKind::Lit(Lit::Double(_)) => Ty::Prim(TyPrim::Double),
            ExprKind::Lit(Lit::Int(_)) => Ty::Prim(TyPrim::Int),
            ExprKind::Lit(Lit::Pauli(_)) => Ty::Prim(TyPrim::Pauli),
            ExprKind::Lit(Lit::Result(_)) => Ty::Prim(TyPrim::Result),
            ExprKind::Lit(Lit::String(_)) => Ty::Prim(TyPrim::String),
            ExprKind::Paren(expr) => termination.update(self.infer_expr(expr)),
            ExprKind::Path(path) => match self.resolutions.get(&path.id) {
                None => Ty::Err,
                Some(id) => {
                    if let Some(ty) = self.globals.get(id) {
                        self.solver.instantiate(ty)
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
                    let ty = termination.update(self.infer_expr(expr));
                    self.solver.eq(expr.span, Ty::Prim(TyPrim::Int), ty);
                }
                Ty::Prim(TyPrim::Range)
            }
            ExprKind::Repeat(body, until, fixup) => {
                termination.update(self.infer_block(body));
                let until_ty = termination.update(self.infer_expr(until));
                self.solver.eq(until.span, Ty::Prim(TyPrim::Bool), until_ty);
                if let Some(fixup) = fixup {
                    termination.update(self.infer_block(fixup));
                }
                Ty::UNIT
            }
            ExprKind::Return(expr) => {
                let ty = self.infer_expr(expr).unwrap();
                if let Some(return_ty) = &self.return_ty {
                    self.solver.eq(expr.span, (*return_ty).clone(), ty);
                }
                termination = Termination::Diverges;
                Ty::Err
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond_ty = termination.update(self.infer_expr(cond));
                self.solver.eq(cond.span, Ty::Prim(TyPrim::Bool), cond_ty);
                let true_ty = self.infer_expr(if_true);
                let false_ty = self.infer_expr(if_false);
                let (true_ty, false_ty) = termination.update_and(true_ty, false_ty);
                self.solver.eq(expr.span, true_ty.clone(), false_ty);
                true_ty
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                termination.update(self.infer_update(expr.span, container, index, item))
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = termination.update(self.infer_expr(item));
                    tys.push(ty);
                }
                Ty::Tuple(tys)
            }
            ExprKind::UnOp(op, expr) => termination.update(self.infer_unop(*op, expr)),
            ExprKind::While(cond, body) => {
                let cond_ty = termination.update(self.infer_expr(cond));
                self.solver.eq(cond.span, Ty::Prim(TyPrim::Bool), cond_ty);
                termination.update(self.infer_block(body));
                Ty::UNIT
            }
            ExprKind::Err | ExprKind::Hole => self.solver.fresh(),
        };

        let ty = if termination.diverges() {
            self.solver.fresh()
        } else {
            ty
        };
        self.tys.insert(expr.id, ty.clone());
        termination.wrap(ty)
    }

    fn infer_block(&mut self, block: &Block) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let mut last = None;
        for stmt in &block.stmts {
            let ty = termination.update(self.infer_stmt(stmt));
            last = Some(ty);
        }

        let ty = if termination.diverges() {
            self.solver.fresh()
        } else {
            last.unwrap_or(Ty::UNIT)
        };
        self.tys.insert(block.id, ty.clone());
        termination.wrap(ty)
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let ty = match &stmt.kind {
            StmtKind::Empty => Ty::UNIT,
            StmtKind::Expr(expr) => termination.update(self.infer_expr(expr)),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = termination.update(self.infer_expr(expr));
                self.solver.eq(pat.span, expr_ty, pat_ty);
                Ty::UNIT
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init_ty = termination.update(self.infer_qubit_init(init));
                self.solver.eq(pat.span, init_ty, pat_ty);
                match block {
                    None => Ty::UNIT,
                    Some(block) => termination.update(self.infer_block(block)),
                }
            }
            StmtKind::Semi(expr) => {
                termination.update(self.infer_expr(expr));
                Ty::UNIT
            }
        };

        let ty = if termination.diverges() {
            self.solver.fresh()
        } else {
            ty
        };
        self.tys.insert(stmt.id, ty.clone());
        termination.wrap(ty)
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        let ty = match &pat.kind {
            PatKind::Bind(name, None) => {
                let ty = self.solver.fresh();
                self.tys.insert(name.id, ty.clone());
                ty
            }
            PatKind::Bind(name, Some(ty)) => {
                let ty = self.convert_ty(ty);
                self.tys.insert(name.id, ty.clone());
                ty
            }
            PatKind::Discard(None) | PatKind::Elided => self.solver.fresh(),
            PatKind::Discard(Some(ty)) => self.convert_ty(ty),
            PatKind::Paren(inner) => self.infer_pat(inner),
            PatKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_pat(item)).collect())
            }
        };

        self.tys.insert(pat.id, ty.clone());
        ty
    }

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let length_ty = termination.update(self.infer_expr(length));
                self.solver
                    .eq(length.span, Ty::Prim(TyPrim::Int), length_ty);
                Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit)))
            }
            QubitInitKind::Paren(inner) => termination.update(self.infer_qubit_init(inner)),
            QubitInitKind::Single => Ty::Prim(TyPrim::Qubit),
            QubitInitKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = termination.update(self.infer_qubit_init(item));
                    tys.push(ty);
                }
                Ty::Tuple(tys)
            }
        };

        let ty = if termination.diverges() {
            self.solver.fresh()
        } else {
            ty
        };
        self.tys.insert(init.id, ty.clone());
        termination.wrap(ty)
    }

    fn infer_unop(&mut self, op: UnOp, expr: &Expr) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let operand_ty = termination.update(self.infer_expr(expr));
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
            UnOp::Unwrap => todo!("user-defined types not supported"),
        };

        termination.wrap(if termination.diverges() {
            self.solver.fresh()
        } else {
            ty
        })
    }

    #[allow(clippy::too_many_lines)]
    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let lhs_ty = termination.update(self.infer_expr(lhs));
        let rhs_ty = termination.update(self.infer_expr(rhs));

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

        termination.wrap(if termination.diverges() {
            self.solver.fresh()
        } else {
            ty
        })
    }

    fn infer_update(
        &mut self,
        span: Span,
        container: &Expr,
        index: &Expr,
        item: &Expr,
    ) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let container_ty = termination.update(self.infer_expr(container));
        let index_ty = termination.update(self.infer_expr(index));
        let item_ty = termination.update(self.infer_expr(item));
        self.solver.class(
            span,
            Class::HasIndex {
                container: container_ty.clone(),
                index: index_ty,
                item: item_ty,
            },
        );

        termination.wrap(if termination.diverges() {
            self.solver.fresh()
        } else {
            container_ty
        })
    }

    fn convert_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &ty.kind {
            TyKind::Array(item) => Ty::Array(Box::new(self.convert_ty(item))),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(
                *kind,
                Box::new(self.convert_ty(input)),
                Box::new(self.convert_ty(output)),
                functors
                    .as_ref()
                    .map_or(HashSet::new(), FunctorExpr::to_set),
            ),
            TyKind::Hole => self.solver.fresh(),
            TyKind::Paren(inner) => self.convert_ty(inner),
            TyKind::Path(path) => Ty::DefId(
                *self
                    .resolutions
                    .get(&path.id)
                    .expect("path should be resolved"),
            ),
            &TyKind::Prim(prim) => Ty::Prim(prim),
            TyKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.convert_ty(item)).collect())
            }
            TyKind::Var(name) => Ty::Param(name.name.clone()),
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

enum Fallible<T> {
    Convergent(T),
    Divergent(T),
}

impl<T> Fallible<T> {
    fn unwrap(self) -> T {
        match self {
            Fallible::Convergent(value) | Fallible::Divergent(value) => value,
        }
    }
}

enum Termination {
    Converges,
    Diverges,
}

impl Termination {
    fn diverges(&self) -> bool {
        matches!(self, Self::Diverges)
    }

    fn wrap<T>(&self, value: T) -> Fallible<T> {
        match self {
            Self::Converges => Fallible::Convergent(value),
            Self::Diverges => Fallible::Divergent(value),
        }
    }
}

impl Termination {
    fn update<T>(&mut self, fallible: Fallible<T>) -> T {
        match fallible {
            Fallible::Convergent(value) => value,
            Fallible::Divergent(value) => {
                *self = Termination::Diverges;
                value
            }
        }
    }

    fn update_and<T>(&mut self, f1: Fallible<T>, f2: Fallible<T>) -> (T, T) {
        match (f1, f2) {
            (Fallible::Divergent(v1), Fallible::Divergent(v2)) => {
                *self = Termination::Diverges;
                (v1, v2)
            }
            (f1, f2) => (f1.unwrap(), f2.unwrap()),
        }
    }
}

pub(super) struct SpecImpl<'a> {
    pub(super) kind: Spec,
    pub(super) input: Option<&'a Pat>,
    pub(super) callable_input: &'a Pat,
    pub(super) output: &'a Ty,
    pub(super) output_span: Span,
    pub(super) functors: &'a HashSet<Functor>,
    pub(super) block: &'a Block,
}

pub(super) fn entry_expr(
    resolutions: &Resolutions,
    globals: &HashMap<DefId, Ty>,
    entry: &Expr,
) -> (Tys, Vec<Error>) {
    let mut context = Context::new(resolutions, globals, None);
    context.infer_expr(entry);
    context.solve()
}

pub(super) fn spec(
    resolutions: &Resolutions,
    globals: &HashMap<DefId, Ty>,
    spec: SpecImpl,
) -> (Tys, Vec<Error>) {
    let mut context = Context::new(resolutions, globals, Some(spec.output));
    let callable_input = context.infer_pat(spec.callable_input);
    if let Some(input) = spec.input {
        let expected = match spec.kind {
            Spec::Body | Spec::Adj => callable_input,
            Spec::Ctl | Spec::CtlAdj => Ty::Tuple(vec![
                Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit))),
                callable_input,
            ]),
        };
        let actual = context.infer_pat(input);
        context.solver.eq(input.span, expected, actual);
    }

    if !spec.functors.is_empty() {
        context
            .solver
            .eq(spec.output_span, Ty::UNIT, spec.output.clone());
    }

    let block = context.infer_block(spec.block).unwrap();
    context
        .solver
        .eq(spec.block.span, spec.output.clone(), block);
    context.solve()
}
