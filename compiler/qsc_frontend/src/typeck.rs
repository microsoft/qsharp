// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::resolve::{DefId, PackageSrc, Resolutions};
use qsc_ast::ast::{
    BinOp, Block, CallableKind, Expr, ExprKind, Functor, FunctorExpr, Lit, NodeId, Pat, TernOp,
    TyPrim, UnOp,
};
use std::collections::{HashMap, HashSet};

enum Constraint {
    Eq(Ty, Ty),
    Class(Class),
}

#[derive(Clone)]
enum Ty {
    App(Box<Ty>, Vec<Ty>),
    Arrow(CallableKind, Box<Ty>, Box<Ty>, Option<FunctorExpr>),
    DefId(DefId),
    Prim(TyPrim),
    Tuple(Vec<Ty>),
    Var(u32),
    Void,
}

enum Class {
    Add(Ty),
    Adj(Ty),
    Call {
        callee: Ty,
        input: Ty,
        output: Ty,
    },
    Ctl {
        op: Ty,
        with_ctls: Ty,
    },
    Eq(Ty),
    HasField {
        record: Ty,
        name: String,
        item: Ty,
    },
    HasFunctorsIfOp {
        callee: Ty,
        functors: HashSet<Functor>,
    },
    HasIndex {
        container: Ty,
        index: Ty,
        item: Ty,
    },
    HasPartialApp {
        callee: Ty,
        missing: Ty,
        with_app: Ty,
    },
    Integral(Ty),
    Iterable {
        container: Ty,
        item: Ty,
    },
    Num(Ty),
    Unwrap {
        wrapper: Ty,
        base: Ty,
    },
}

struct Inferrer<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    constraints: Vec<Constraint>,
    tys: HashMap<NodeId, Ty>,
    next_var: u32,
}

impl Inferrer<'_> {
    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Ty {
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first = self.infer_expr(first);
                    for item in rest {
                        let item = self.infer_expr(item);
                        self.constrain(Constraint::Eq(first.clone(), item));
                    }
                    Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![first])
                }
                None => Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![self.fresh()]),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item = self.infer_expr(item);
                let size = self.infer_expr(size);
                self.constrain(Constraint::Eq(size, Ty::Prim(TyPrim::Int)));
                Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![item])
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs = self.infer_expr(lhs);
                let rhs = self.infer_expr(rhs);
                self.constrain(Constraint::Eq(lhs, rhs));
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                self.infer_binop(*op, lhs, rhs);
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignUpdate(container, index, item) => {
                self.infer_update(container, index, item);
                Ty::Tuple(Vec::new())
            }
            ExprKind::BinOp(op, lhs, rhs) => self.infer_binop(*op, lhs, rhs),
            ExprKind::Block(block) => self.infer_block(block),
            ExprKind::Call(callee, input) => {
                let callee = self.infer_expr(callee);
                let input = self.infer_expr(input);
                let output = self.fresh();
                self.constrain(Constraint::Class(Class::Call {
                    callee,
                    input,
                    output: output.clone(),
                }));
                output
            }
            ExprKind::Conjugate(within, apply) => {
                let within = self.infer_block(within);
                let apply = self.infer_block(apply);
                self.constrain(Constraint::Eq(within, Ty::Tuple(Vec::new())));
                apply
            }
            ExprKind::Fail(message) => {
                let message = self.infer_expr(message);
                self.constrain(Constraint::Eq(message, Ty::Prim(TyPrim::String)));
                Ty::Void
            }
            ExprKind::Field(record, name) => {
                let record = self.infer_expr(record);
                let item = self.fresh();
                self.constrain(Constraint::Class(Class::HasField {
                    record,
                    name: name.name.clone(),
                    item: item.clone(),
                }));
                item
            }
            ExprKind::For(item, container, body) => {
                let item = self.infer_pat(item);
                let container = self.infer_expr(container);
                self.constrain(Constraint::Class(Class::Iterable { container, item }));
                let body = self.infer_block(body);
                self.constrain(Constraint::Eq(body, Ty::Tuple(Vec::new())));
                Ty::Tuple(Vec::new())
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond = self.infer_expr(cond);
                self.constrain(Constraint::Eq(cond, Ty::Prim(TyPrim::Bool)));
                let if_true = self.infer_block(if_true);
                let if_false = if_false
                    .as_ref()
                    .map_or(Ty::Tuple(Vec::new()), |e| self.infer_expr(e));
                self.constrain(Constraint::Eq(if_true.clone(), if_false));
                if_true
            }
            ExprKind::Index(container, index) => {
                let container = self.infer_expr(container);
                let index = self.infer_expr(index);
                let item = self.fresh();
                self.constrain(Constraint::Class(Class::HasIndex {
                    container,
                    index,
                    item: item.clone(),
                }));
                item
            }
            ExprKind::Lambda(kind, input, body) => {
                let input = self.infer_pat(input);
                let body = self.infer_expr(body);
                Ty::Arrow(*kind, Box::new(input), Box::new(body), None)
            }
            ExprKind::Lit(Lit::BigInt(_)) => Ty::Prim(TyPrim::BigInt),
            ExprKind::Lit(Lit::Bool(_)) => Ty::Prim(TyPrim::Bool),
            ExprKind::Lit(Lit::Double(_)) => Ty::Prim(TyPrim::Double),
            ExprKind::Lit(Lit::Int(_)) => Ty::Prim(TyPrim::Int),
            ExprKind::Lit(Lit::Pauli(_)) => Ty::Prim(TyPrim::Pauli),
            ExprKind::Lit(Lit::Result(_)) => Ty::Prim(TyPrim::Result),
            ExprKind::Lit(Lit::String(_)) => Ty::Prim(TyPrim::String),
            ExprKind::Paren(expr) => self.infer_expr(expr),
            ExprKind::Path(path) => {
                let def = self
                    .resolutions
                    .get(&path.id)
                    .expect("path should be resolved");

                if let Some(ty) = self.globals.get(def) {
                    ty.clone()
                } else if def.package == PackageSrc::Local {
                    self.tys
                        .get(&def.node)
                        .expect("local variable should have inferred type")
                        .clone()
                } else {
                    panic!("path resolves to external package, but definition not found in globals")
                }
            }
            ExprKind::Range(start, step, end) => {
                for expr in start.iter().chain(step).chain(end) {
                    let ty = self.infer_expr(expr);
                    self.constrain(Constraint::Eq(ty, Ty::Prim(TyPrim::Int)));
                }
                Ty::Prim(TyPrim::Range)
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body = self.infer_block(body);
                self.constrain(Constraint::Eq(body, Ty::Tuple(Vec::new())));
                let until = self.infer_expr(until);
                self.constrain(Constraint::Eq(until, Ty::Prim(TyPrim::Bool)));
                if let Some(fixup) = fixup {
                    let fixup = self.infer_block(fixup);
                    self.constrain(Constraint::Eq(fixup, Ty::Tuple(Vec::new())));
                }

                Ty::Tuple(Vec::new())
            }
            ExprKind::Return(expr) => {
                self.infer_expr(expr);
                Ty::Void
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond = self.infer_expr(cond);
                self.constrain(Constraint::Eq(cond, Ty::Prim(TyPrim::Bool)));
                let if_true = self.infer_expr(if_true);
                let if_false = self.infer_expr(if_false);
                self.constrain(Constraint::Eq(if_true.clone(), if_false));
                if_true
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                self.infer_update(container, index, item)
            }
            ExprKind::Tuple(items) => {
                let items = items.iter().map(|e| self.infer_expr(e)).collect();
                Ty::Tuple(items)
            }
            ExprKind::UnOp(op, expr) => self.infer_unop(*op, expr),
            ExprKind::While(cond, body) => {
                let cond = self.infer_expr(cond);
                self.constrain(Constraint::Eq(cond, Ty::Prim(TyPrim::Bool)));
                let body = self.infer_block(body);
                self.constrain(Constraint::Eq(body, Ty::Tuple(Vec::new())));
                Ty::Tuple(Vec::new())
            }
            ExprKind::Err | ExprKind::Hole => Ty::Void,
        };

        self.tys.insert(expr.id, ty.clone());
        ty
    }

    fn infer_block(&mut self, block: &Block) -> Ty {
        todo!()
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        todo!()
    }

    fn infer_unop(&mut self, op: UnOp, expr: &Expr) -> Ty {
        todo!()
    }

    fn infer_binop(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr) -> Ty {
        todo!()
    }

    fn infer_update(&mut self, container: &Expr, index: &Expr, item: &Expr) -> Ty {
        todo!()
    }

    fn fresh(&mut self) -> Ty {
        let var = self.next_var;
        self.next_var += 1;
        Ty::Var(var)
    }

    fn constrain(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    fn solve(self) -> HashMap<NodeId, Ty> {
        todo!()
    }
}
