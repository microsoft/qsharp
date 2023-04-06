// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::PackageId,
    resolve::{DefId, PackageSrc, Resolutions},
};
use qsc_ast::{
    ast::{
        self, BinOp, Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Functor,
        FunctorExpr, FunctorExprKind, Lit, NodeId, Pat, PatKind, QubitInit, QubitInitKind, SetOp,
        Span, SpecBody, Stmt, StmtKind, TernOp, TyKind, TyPrim, UnOp,
    },
    visit::Visitor,
};
use std::{
    collections::{HashMap, HashSet},
    mem,
};

pub type Tys = HashMap<NodeId, Ty>;

#[derive(Clone, Debug)]
pub enum Ty {
    App(Box<Ty>, Vec<Ty>),
    Arrow(CallableKind, Box<Ty>, Box<Ty>, Option<FunctorExpr>),
    DefId(DefId),
    Never,
    Param(String),
    Prim(TyPrim),
    Tuple(Vec<Ty>),
    Var(u32),
}

struct UnifyError(Ty, Ty);

pub(super) struct Checker<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    tys: Tys,
}

impl Checker<'_> {
    pub(super) fn into_tys(self) -> Tys {
        self.tys
    }
}

impl Visitor<'_> for Checker<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        match &decl.body {
            CallableBody::Block(block) => {
                let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
                inferrer.infer_pat(&decl.input);
                let decl_output = inferrer.convert_ty(&decl.output);
                let block_output = inferrer.infer_block(block);
                inferrer.constrain(block.span, ConstraintKind::Eq(decl_output, block_output));
                self.tys.extend(inferrer.solve());
            }
            CallableBody::Specs(specs) => {
                for spec in specs {
                    match &spec.body {
                        SpecBody::Gen(_) => {}
                        SpecBody::Impl(input, block) => {
                            let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
                            inferrer.infer_pat(&decl.input);
                            inferrer.infer_pat(input);
                            let decl_output = inferrer.convert_ty(&decl.output);
                            let block_output = inferrer.infer_block(block);
                            inferrer.constrain(
                                block.span,
                                ConstraintKind::Eq(decl_output, block_output),
                            );
                            self.tys.extend(inferrer.solve());
                        }
                    }
                }
            }
        }
    }
}

pub(super) struct GlobalTable<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    package: PackageSrc,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            globals: HashMap::new(),
            package: PackageSrc::Local,
        }
    }

    pub(super) fn set_package(&mut self, package: PackageId) {
        self.package = PackageSrc::Extern(package);
    }

    pub(super) fn into_checker(self) -> Checker<'a> {
        Checker {
            resolutions: self.resolutions,
            globals: self.globals,
            tys: Tys::new(),
        }
    }
}

impl Visitor<'_> for GlobalTable<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        let id = DefId {
            package: self.package,
            node: decl.name.id,
        };
        let ty = callable_ty(self.resolutions, decl).expect("callable should have a valid type");
        self.globals.insert(id, ty);
    }
}

struct Constraint {
    span: Span,
    kind: ConstraintKind,
}

enum ConstraintKind {
    Eq(Ty, Ty),
    Class(Class),
}

#[derive(Clone, Debug)]
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

impl Class {
    fn dependencies(&self) -> Vec<&Ty> {
        match self {
            Self::Add(ty) | Self::Adj(ty) | Self::Eq(ty) | Self::Integral(ty) | Self::Num(ty) => {
                vec![ty]
            }
            Self::Call { callee, .. }
            | Self::HasFunctorsIfOp { callee, .. }
            | Self::HasPartialApp { callee, .. } => vec![callee],
            Self::Ctl { op, .. } => vec![op],
            Self::HasField { record, .. } => vec![record],
            Self::HasIndex {
                container, index, ..
            } => vec![container, index],
            Self::Iterable { container, .. } => vec![container],
            Self::Unwrap { wrapper, .. } => vec![wrapper],
        }
    }

    fn map(self, mut f: impl FnMut(Ty) -> Ty) -> Self {
        match self {
            Self::Add(ty) => Self::Add(f(ty)),
            Self::Adj(ty) => Self::Adj(f(ty)),
            Self::Call {
                callee,
                input,
                output,
            } => Self::Call {
                callee: f(callee),
                input: f(input),
                output: f(output),
            },
            Self::Ctl { op, with_ctls } => Self::Ctl {
                op: f(op),
                with_ctls: f(with_ctls),
            },
            Self::Eq(ty) => Self::Eq(f(ty)),
            Self::HasField { record, name, item } => Self::HasField {
                record: f(record),
                name,
                item: f(item),
            },
            Self::HasFunctorsIfOp { callee, functors } => Self::HasFunctorsIfOp {
                callee: f(callee),
                functors,
            },
            Self::HasIndex {
                container,
                index,
                item,
            } => Self::HasIndex {
                container: f(container),
                index: f(index),
                item: f(item),
            },
            Self::HasPartialApp {
                callee,
                missing,
                with_app,
            } => Self::HasPartialApp {
                callee: f(callee),
                missing: f(missing),
                with_app: f(with_app),
            },
            Self::Integral(ty) => Self::Integral(f(ty)),
            Self::Iterable { container, item } => Self::Iterable {
                container: f(container),
                item: f(item),
            },
            Self::Num(ty) => Self::Num(f(ty)),
            Self::Unwrap { wrapper, base } => Self::Unwrap {
                wrapper: f(wrapper),
                base: f(base),
            },
        }
    }
}

struct Inferrer<'a> {
    resolutions: &'a Resolutions,
    globals: &'a HashMap<DefId, Ty>,
    constraints: Vec<Constraint>,
    tys: Tys,
    next_var: u32,
}

impl<'a> Inferrer<'a> {
    fn new(resolutions: &'a Resolutions, globals: &'a HashMap<DefId, Ty>) -> Self {
        Self {
            resolutions,
            globals,
            constraints: Vec::new(),
            tys: Tys::new(),
            next_var: 0,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn infer_expr(&mut self, expr: &Expr) -> Ty {
        let mut divergence = false;
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_ty = self.infer_expr(first);
                    divergence = divergence || diverges(&first_ty);
                    for item in rest {
                        let item_ty = self.infer_expr(item);
                        divergence = divergence || diverges(&item_ty);
                        self.constrain(item.span, ConstraintKind::Eq(first_ty.clone(), item_ty));
                    }

                    Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![first_ty])
                }
                None => Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![self.fresh()]),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item_ty = self.infer_expr(item);
                divergence = divergence || diverges(&item_ty);
                let size_ty = self.infer_expr(size);
                divergence = divergence || diverges(&size_ty);
                self.constrain(
                    size.span,
                    ConstraintKind::Eq(size_ty, Ty::Prim(TyPrim::Int)),
                );
                Ty::App(Box::new(Ty::Prim(TyPrim::Array)), vec![item_ty])
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_ty = self.infer_expr(lhs);
                let rhs_ty = self.infer_expr(rhs);
                divergence = divergence || diverges(&rhs_ty);
                self.constrain(lhs.span, ConstraintKind::Eq(lhs_ty, rhs_ty));
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                let ty = self.infer_binop(expr.span, *op, lhs, rhs);
                divergence = divergence || diverges(&ty);
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignUpdate(container, index, item) => {
                let ty = self.infer_update(expr.span, container, index, item);
                divergence = divergence || diverges(&ty);
                Ty::Tuple(Vec::new())
            }
            ExprKind::BinOp(op, lhs, rhs) => {
                let ty = self.infer_binop(expr.span, *op, lhs, rhs);
                divergence = divergence || diverges(&ty);
                ty
            }
            ExprKind::Block(block) => {
                let ty = self.infer_block(block);
                divergence = divergence || diverges(&ty);
                ty
            }
            ExprKind::Call(callee, input) => {
                let callee_ty = self.infer_expr(callee);
                divergence = divergence || diverges(&callee_ty);
                let input_ty = self.infer_expr(input);
                divergence = divergence || diverges(&input_ty);
                let output_ty = self.fresh();
                self.constrain(
                    expr.span,
                    ConstraintKind::Class(Class::Call {
                        callee: callee_ty,
                        input: input_ty,
                        output: output_ty.clone(),
                    }),
                );
                output_ty
            }
            ExprKind::Conjugate(within, apply) => {
                let within_ty = self.infer_block(within);
                divergence = divergence || diverges(&within_ty);
                let apply_ty = self.infer_block(apply);
                divergence = divergence || diverges(&apply_ty);
                self.constrain(
                    within.span,
                    ConstraintKind::Eq(within_ty, Ty::Tuple(Vec::new())),
                );
                apply_ty
            }
            ExprKind::Fail(message) => {
                divergence = true;
                let message_ty = self.infer_expr(message);
                self.constrain(
                    message.span,
                    ConstraintKind::Eq(message_ty, Ty::Prim(TyPrim::String)),
                );
                Ty::Never
            }
            ExprKind::Field(record, name) => {
                let record_ty = self.infer_expr(record);
                divergence = divergence || diverges(&record_ty);
                let item_ty = self.fresh();
                self.constrain(
                    expr.span,
                    ConstraintKind::Class(Class::HasField {
                        record: record_ty,
                        name: name.name.clone(),
                        item: item_ty.clone(),
                    }),
                );
                item_ty
            }
            ExprKind::For(item, container, body) => {
                let item_ty = self.infer_pat(item);
                let container_ty = self.infer_expr(container);
                divergence = divergence || diverges(&container_ty);
                self.constrain(
                    container.span,
                    ConstraintKind::Class(Class::Iterable {
                        container: container_ty,
                        item: item_ty,
                    }),
                );

                let body_ty = self.infer_block(body);
                divergence = divergence || diverges(&body_ty);
                self.constrain(
                    body.span,
                    ConstraintKind::Eq(body_ty, Ty::Tuple(Vec::new())),
                );
                Ty::Tuple(Vec::new())
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_ty = self.infer_expr(cond);
                divergence = divergence || diverges(&cond_ty);
                self.constrain(
                    cond.span,
                    ConstraintKind::Eq(cond_ty, Ty::Prim(TyPrim::Bool)),
                );

                let true_ty = self.infer_block(if_true);
                let false_ty = if_false
                    .as_ref()
                    .map_or(Ty::Tuple(Vec::new()), |e| self.infer_expr(e));
                divergence = divergence || diverges(&true_ty) && diverges(&false_ty);
                self.constrain(expr.span, ConstraintKind::Eq(true_ty.clone(), false_ty));
                true_ty
            }
            ExprKind::Index(container, index) => {
                let container_ty = self.infer_expr(container);
                divergence = divergence || diverges(&container_ty);
                let index_ty = self.infer_expr(index);
                divergence = divergence || diverges(&index_ty);
                let item_ty = self.fresh();
                self.constrain(
                    expr.span,
                    ConstraintKind::Class(Class::HasIndex {
                        container: container_ty,
                        index: index_ty,
                        item: item_ty.clone(),
                    }),
                );
                item_ty
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
            ExprKind::Paren(expr) => {
                let ty = self.infer_expr(expr);
                divergence = divergence || diverges(&ty);
                ty
            }
            ExprKind::Path(path) => match self.resolutions.get(&path.id) {
                None => self.fresh(),
                Some(id) => {
                    if let Some(ty) = self.globals.get(id) {
                        self.instantiate(ty)
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
                    let ty = self.infer_expr(expr);
                    divergence = divergence || diverges(&ty);
                    self.constrain(expr.span, ConstraintKind::Eq(ty, Ty::Prim(TyPrim::Int)));
                }
                Ty::Prim(TyPrim::Range)
            }
            ExprKind::Repeat(body, until, fixup) => {
                let body_ty = self.infer_block(body);
                divergence = divergence || diverges(&body_ty);
                self.constrain(
                    body.span,
                    ConstraintKind::Eq(body_ty, Ty::Tuple(Vec::new())),
                );

                let until_ty = self.infer_expr(until);
                divergence = divergence || diverges(&until_ty);
                self.constrain(
                    until.span,
                    ConstraintKind::Eq(until_ty, Ty::Prim(TyPrim::Bool)),
                );

                if let Some(fixup) = fixup {
                    let fixup_ty = self.infer_block(fixup);
                    divergence = divergence || diverges(&fixup_ty);
                    self.constrain(
                        fixup.span,
                        ConstraintKind::Eq(fixup_ty, Ty::Tuple(Vec::new())),
                    );
                }

                Ty::Tuple(Vec::new())
            }
            ExprKind::Return(expr) => {
                divergence = true;
                self.infer_expr(expr);
                Ty::Never
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond_ty = self.infer_expr(cond);
                divergence = divergence || diverges(&cond_ty);
                self.constrain(
                    cond.span,
                    ConstraintKind::Eq(cond_ty, Ty::Prim(TyPrim::Bool)),
                );

                let true_ty = self.infer_expr(if_true);
                let false_ty = self.infer_expr(if_false);
                divergence = divergence || diverges(&true_ty) && diverges(&false_ty);
                self.constrain(expr.span, ConstraintKind::Eq(true_ty.clone(), false_ty));
                true_ty
            }
            ExprKind::TernOp(TernOp::Update, container, index, item) => {
                let ty = self.infer_update(expr.span, container, index, item);
                divergence = divergence || diverges(&ty);
                ty
            }
            ExprKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = self.infer_expr(item);
                    divergence = divergence || diverges(&ty);
                    tys.push(ty);
                }
                Ty::Tuple(tys)
            }
            ExprKind::UnOp(op, expr) => {
                let ty = self.infer_unop(*op, expr);
                divergence = divergence || diverges(&ty);
                ty
            }
            ExprKind::While(cond, body) => {
                let cond_ty = self.infer_expr(cond);
                divergence = divergence || diverges(&cond_ty);
                self.constrain(
                    cond.span,
                    ConstraintKind::Eq(cond_ty, Ty::Prim(TyPrim::Bool)),
                );

                let body_ty = self.infer_block(body);
                divergence = divergence || diverges(&body_ty);
                self.constrain(
                    body.span,
                    ConstraintKind::Eq(body_ty, Ty::Tuple(Vec::new())),
                );

                Ty::Tuple(Vec::new())
            }
            ExprKind::Err | ExprKind::Hole => self.fresh(),
        };

        let ty = if divergence { Ty::Never } else { ty };
        self.tys.insert(expr.id, ty.clone());
        ty
    }

    fn infer_block(&mut self, block: &Block) -> Ty {
        let mut divergence = false;
        let mut last = None;
        for stmt in &block.stmts {
            let ty = self.infer_stmt(stmt);
            divergence = divergence || diverges(&ty);
            last = Some(ty);
        }

        let ty = if divergence {
            Ty::Never
        } else {
            last.unwrap_or(Ty::Tuple(Vec::new()))
        };
        self.tys.insert(block.id, ty.clone());
        ty
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Ty {
        let mut divergence = false;
        let ty = match &stmt.kind {
            StmtKind::Empty => Ty::Tuple(Vec::new()),
            StmtKind::Expr(expr) => {
                let ty = self.infer_expr(expr);
                divergence = divergence || diverges(&ty);
                ty
            }
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = self.infer_expr(expr);
                divergence = divergence || diverges(&expr_ty);
                self.constrain(pat.span, ConstraintKind::Eq(pat_ty, expr_ty));
                Ty::Tuple(Vec::new())
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init_ty = self.infer_qubit_init(init);
                divergence = divergence || diverges(&init_ty);
                self.constrain(pat.span, ConstraintKind::Eq(pat_ty, init_ty));
                match block {
                    None => Ty::Tuple(Vec::new()),
                    Some(block) => {
                        let ty = self.infer_block(block);
                        divergence = divergence || diverges(&ty);
                        ty
                    }
                }
            }
            StmtKind::Semi(expr) => {
                let ty = self.infer_expr(expr);
                divergence = divergence || diverges(&ty);
                Ty::Tuple(Vec::new())
            }
        };

        let ty = if divergence { Ty::Never } else { ty };
        self.tys.insert(stmt.id, ty.clone());
        ty
    }

    fn infer_pat(&mut self, pat: &Pat) -> Ty {
        let ty = match &pat.kind {
            PatKind::Bind(name, None) => {
                let ty = self.fresh();
                self.tys.insert(name.id, ty.clone());
                ty
            }
            PatKind::Bind(name, Some(ty)) => {
                let ty = self.convert_ty(ty);
                self.tys.insert(name.id, ty.clone());
                ty
            }
            PatKind::Discard(None) | PatKind::Elided => self.fresh(),
            PatKind::Discard(Some(ty)) => self.convert_ty(ty),
            PatKind::Paren(inner) => self.infer_pat(inner),
            PatKind::Tuple(items) => {
                Ty::Tuple(items.iter().map(|item| self.infer_pat(item)).collect())
            }
        };

        self.tys.insert(pat.id, ty.clone());
        ty
    }

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Ty {
        let mut divergence = false;
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let length_ty = self.infer_expr(length);
                divergence = divergence || diverges(&length_ty);
                self.constrain(
                    length.span,
                    ConstraintKind::Eq(length_ty, Ty::Prim(TyPrim::Int)),
                );
                Ty::App(
                    Box::new(Ty::Prim(TyPrim::Array)),
                    vec![Ty::Prim(TyPrim::Qubit)],
                )
            }
            QubitInitKind::Paren(inner) => {
                let ty = self.infer_qubit_init(inner);
                divergence = divergence || diverges(&ty);
                ty
            }
            QubitInitKind::Single => Ty::Prim(TyPrim::Qubit),
            QubitInitKind::Tuple(items) => {
                let mut tys = Vec::new();
                for item in items {
                    let ty = self.infer_qubit_init(item);
                    divergence = divergence || diverges(&ty);
                    tys.push(ty);
                }
                Ty::Tuple(tys)
            }
        };

        let ty = if divergence { Ty::Never } else { ty };
        self.tys.insert(init.id, ty.clone());
        ty
    }

    fn infer_unop(&mut self, op: UnOp, expr: &Expr) -> Ty {
        let operand_ty = self.infer_expr(expr);
        let divergence = diverges(&operand_ty);
        let ty = match op {
            UnOp::Functor(Functor::Adj) => {
                self.constrain(
                    expr.span,
                    ConstraintKind::Class(Class::Adj(operand_ty.clone())),
                );
                operand_ty
            }
            UnOp::Functor(Functor::Ctl) => {
                let with_ctls = self.fresh();
                self.constrain(
                    expr.span,
                    ConstraintKind::Class(Class::Ctl {
                        op: operand_ty,
                        with_ctls: with_ctls.clone(),
                    }),
                );
                with_ctls
            }
            UnOp::Neg | UnOp::NotB | UnOp::Pos => {
                self.constrain(
                    expr.span,
                    ConstraintKind::Class(Class::Num(operand_ty.clone())),
                );
                operand_ty
            }
            UnOp::NotL => {
                self.constrain(
                    expr.span,
                    ConstraintKind::Eq(operand_ty.clone(), Ty::Prim(TyPrim::Bool)),
                );
                operand_ty
            }
            UnOp::Unwrap => todo!("user-defined types not supported"),
        };

        if divergence {
            Ty::Never
        } else {
            ty
        }
    }

    fn infer_binop(&mut self, span: Span, op: BinOp, lhs: &Expr, rhs: &Expr) -> Ty {
        let lhs_ty = self.infer_expr(lhs);
        let rhs_ty = self.infer_expr(rhs);
        let divergence = diverges(&lhs_ty) || diverges(&rhs_ty);
        self.constrain(span, ConstraintKind::Eq(lhs_ty.clone(), rhs_ty));

        let ty = match op {
            BinOp::AndL | BinOp::OrL => {
                self.constrain(
                    lhs.span,
                    ConstraintKind::Eq(lhs_ty.clone(), Ty::Prim(TyPrim::Bool)),
                );
                lhs_ty
            }
            BinOp::Eq | BinOp::Neq => {
                self.constrain(lhs.span, ConstraintKind::Class(Class::Eq(lhs_ty)));
                Ty::Prim(TyPrim::Bool)
            }
            BinOp::Add => {
                self.constrain(lhs.span, ConstraintKind::Class(Class::Add(lhs_ty.clone())));
                lhs_ty
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Exp
            | BinOp::Gt
            | BinOp::Gte
            | BinOp::Lt
            | BinOp::Lte
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Shl
            | BinOp::Shr
            | BinOp::Sub
            | BinOp::XorB => {
                self.constrain(lhs.span, ConstraintKind::Class(Class::Num(lhs_ty.clone())));
                lhs_ty
            }
        };

        if divergence {
            Ty::Never
        } else {
            ty
        }
    }

    fn infer_update(&mut self, span: Span, container: &Expr, index: &Expr, item: &Expr) -> Ty {
        let container_ty = self.infer_expr(container);
        let index_ty = self.infer_expr(index);
        let item_ty = self.infer_expr(item);
        let divergence = diverges(&container_ty) || diverges(&index_ty) || diverges(&item_ty);
        self.constrain(
            span,
            ConstraintKind::Class(Class::HasIndex {
                container: container_ty.clone(),
                index: index_ty,
                item: item_ty,
            }),
        );

        if divergence {
            Ty::Never
        } else {
            container_ty
        }
    }

    fn fresh(&mut self) -> Ty {
        let var = self.next_var;
        self.next_var += 1;
        Ty::Var(var)
    }

    fn instantiate(&mut self, ty: &Ty) -> Ty {
        fn go(fresh: &mut impl FnMut() -> Ty, vars: &mut HashMap<String, Ty>, ty: &Ty) -> Ty {
            match ty {
                Ty::App(base, args) => Ty::App(
                    Box::new(go(fresh, vars, base)),
                    args.iter().map(|arg| go(fresh, vars, arg)).collect(),
                ),
                Ty::Arrow(kind, input, output, functors) => Ty::Arrow(
                    *kind,
                    Box::new(go(fresh, vars, input)),
                    Box::new(go(fresh, vars, output)),
                    functors.clone(),
                ),
                &Ty::DefId(id) => Ty::DefId(id),
                Ty::Never => Ty::Never,
                Ty::Param(name) => vars.entry(name.clone()).or_insert_with(fresh).clone(),
                &Ty::Prim(prim) => Ty::Prim(prim),
                Ty::Tuple(items) => {
                    Ty::Tuple(items.iter().map(|item| go(fresh, vars, item)).collect())
                }
                &Ty::Var(id) => Ty::Var(id),
            }
        }

        go(&mut || self.fresh(), &mut HashMap::new(), ty)
    }

    fn convert_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &ty.kind {
            TyKind::App(base, args) => Ty::App(
                Box::new(self.convert_ty(base)),
                args.iter().map(|ty| self.convert_ty(ty)).collect(),
            ),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(
                *kind,
                Box::new(self.convert_ty(input)),
                Box::new(self.convert_ty(output)),
                functors.clone(),
            ),
            TyKind::Hole => self.fresh(),
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

    fn constrain(&mut self, span: Span, kind: ConstraintKind) {
        self.constraints.push(Constraint { span, kind });
    }

    fn solve(self) -> Tys {
        let mut substs = HashMap::new();
        let mut pending_classes: HashMap<_, Vec<_>> = HashMap::new();
        let mut constraints = self.constraints;
        let mut new_constraints = Vec::new();

        loop {
            for constraint in constraints {
                match constraint.kind {
                    ConstraintKind::Eq(ty1, ty2) => {
                        let ty1 = substitute(&substs, ty1);
                        let ty2 = substitute(&substs, ty2);
                        let new_substs = match unify(&ty1, &ty2) {
                            Ok(new_substs) => new_substs,
                            Err(UnifyError(ty1, ty2)) => panic!(
                                "types do not unify at {:?}: {ty1:?} and {ty2:?}",
                                constraint.span,
                            ),
                        };

                        for (var, _) in &new_substs {
                            if let Some(classes) = pending_classes.remove(var) {
                                new_constraints.extend(classes.into_iter().map(|class| {
                                    Constraint {
                                        span: constraint.span,
                                        kind: ConstraintKind::Class(class),
                                    }
                                }));
                            }
                        }

                        substs.extend(new_substs);
                    }
                    ConstraintKind::Class(class) => {
                        let unsolved: Vec<_> = class
                            .dependencies()
                            .into_iter()
                            .filter_map(|ty| try_var_id(&substitute(&substs, ty.clone())))
                            .collect();

                        if unsolved.is_empty() {
                            new_constraints.extend(classify(
                                constraint.span,
                                class.map(|ty| substitute(&substs, ty)),
                            ));
                        } else {
                            for var in unsolved {
                                pending_classes.entry(var).or_default().push(class.clone());
                            }
                        }
                    }
                }
            }

            if new_constraints.is_empty() {
                break;
            }

            constraints = mem::take(&mut new_constraints);
        }

        self.tys
            .into_iter()
            .map(|(id, ty)| (id, substitute(&substs, ty)))
            .collect()
    }
}

fn unify(ty1: &Ty, ty2: &Ty) -> Result<Vec<(u32, Ty)>, UnifyError> {
    match (ty1, ty2) {
        (Ty::App(base1, args1), Ty::App(base2, args2)) if args1.len() == args2.len() => {
            let mut substs = unify(base1, base2)?;
            for (arg1, arg2) in args1.iter().zip(args2) {
                substs.extend(unify(arg1, arg2)?);
            }
            Ok(substs)
        }
        (
            Ty::Arrow(kind1, input1, output1, functors1),
            Ty::Arrow(kind2, input2, output2, functors2),
        ) if kind1 == kind2
            && functor_set(functors1.as_ref()) == functor_set(functors2.as_ref()) =>
        {
            let mut substs = unify(input1, input2)?;
            substs.extend(unify(output1, output2)?);
            Ok(substs)
        }
        (Ty::DefId(def1), Ty::DefId(def2)) if def1 == def2 => Ok(Vec::new()),
        (Ty::Never, _) | (_, Ty::Never) => Ok(Vec::new()),
        (Ty::Prim(prim1), Ty::Prim(prim2)) if prim1 == prim2 => Ok(Vec::new()),
        (Ty::Tuple(items1), Ty::Tuple(items2)) if items1.len() == items2.len() => {
            let mut substs = Vec::new();
            for (item1, item2) in items1.iter().zip(items2) {
                substs.extend(unify(item1, item2)?);
            }
            Ok(substs)
        }
        (Ty::Var(var1), Ty::Var(var2)) if var1 == var2 => Ok(Vec::new()),
        (&Ty::Var(var), _) => Ok(vec![(var, ty2.clone())]),
        (_, &Ty::Var(var)) => Ok(vec![(var, ty1.clone())]),
        _ => Err(UnifyError(ty1.clone(), ty2.clone())),
    }
}

#[allow(clippy::too_many_lines)]
fn classify(span: Span, class: Class) -> Vec<Constraint> {
    match class {
        Class::Eq(Ty::Prim(
            TyPrim::BigInt
            | TyPrim::Bool
            | TyPrim::Double
            | TyPrim::Int
            | TyPrim::Qubit
            | TyPrim::Result
            | TyPrim::String
            | TyPrim::Pauli,
        ))
        | Class::Integral(Ty::Prim(TyPrim::BigInt | TyPrim::Int))
        | Class::Num(Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int))
        | Class::Add(Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int | TyPrim::String)) => {
            Vec::new()
        }
        Class::Add(Ty::App(base, _)) if matches!(*base, Ty::Prim(TyPrim::Array)) => Vec::new(),
        Class::Adj(Ty::Arrow(_, _, _, functors))
            if functor_set(functors.as_ref()).contains(&Functor::Adj) =>
        {
            Vec::new()
        }
        Class::Call {
            callee: Ty::Arrow(_, callee_input, callee_output, _),
            input,
            output,
        } => vec![
            Constraint {
                span,
                kind: ConstraintKind::Eq(input, *callee_input),
            },
            Constraint {
                span,
                kind: ConstraintKind::Eq(output, *callee_output),
            },
        ],
        Class::Ctl {
            op: Ty::Arrow(kind, input, output, functors),
            with_ctls,
        } if functor_set(functors.as_ref()).contains(&Functor::Ctl) => {
            let qubit_array = Ty::App(
                Box::new(Ty::Prim(TyPrim::Array)),
                vec![Ty::Prim(TyPrim::Qubit)],
            );
            let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *input]));
            vec![Constraint {
                span,
                kind: ConstraintKind::Eq(with_ctls, Ty::Arrow(kind, ctl_input, output, functors)),
            }]
        }
        Class::HasField { .. } => todo!("user-defined types not supported"),
        Class::HasFunctorsIfOp { callee, functors } => match callee {
            Ty::Arrow(CallableKind::Operation, _, _, callee_functors)
                if functor_set(callee_functors.as_ref()).is_superset(&functors) =>
            {
                Vec::new()
            }
            Ty::Arrow(CallableKind::Operation, _, _, _) => {
                panic!("operation is missing functors")
            }
            _ => Vec::new(),
        },
        Class::HasIndex {
            container: Ty::App(base, mut args),
            index,
            item,
        } if matches!(*base, Ty::Prim(TyPrim::Array)) && args.len() == 1 => match index {
            Ty::Prim(TyPrim::Int) => vec![Constraint {
                span,
                kind: ConstraintKind::Eq(
                    args.pop().expect("type arguments should not be empty"),
                    item,
                ),
            }],
            Ty::Prim(TyPrim::Range) => vec![Constraint {
                span,
                kind: ConstraintKind::Eq(Ty::App(base, args), item),
            }],
            _ => panic!("invalid index for array"),
        },
        Class::HasPartialApp { .. } => todo!("partial application not supported"),
        Class::Iterable {
            container: Ty::Prim(TyPrim::Range),
            item,
        } => vec![Constraint {
            span,
            kind: ConstraintKind::Eq(Ty::Prim(TyPrim::Int), item),
        }],
        Class::Iterable {
            container: Ty::App(base, mut args),
            item,
        } if matches!(*base, Ty::Prim(TyPrim::Array)) => {
            vec![Constraint {
                span,
                kind: ConstraintKind::Eq(
                    args.pop().expect("type arguments should not be empty"),
                    item,
                ),
            }]
        }
        Class::Unwrap { .. } => todo!("user-defined types not supported"),
        class => panic!("falsified class: {class:?}"),
    }
}

fn substitute(substs: &HashMap<u32, Ty>, ty: Ty) -> Ty {
    match ty {
        Ty::App(base, args) => Ty::App(
            Box::new(substitute(substs, *base)),
            args.into_iter()
                .map(|arg| substitute(substs, arg))
                .collect(),
        ),
        Ty::Arrow(kind, input, output, functors) => Ty::Arrow(
            kind,
            Box::new(substitute(substs, *input)),
            Box::new(substitute(substs, *output)),
            functors,
        ),
        Ty::DefId(id) => Ty::DefId(id),
        Ty::Never => Ty::Never,
        Ty::Param(name) => Ty::Param(name),
        Ty::Prim(prim) => Ty::Prim(prim),
        Ty::Tuple(items) => Ty::Tuple(
            items
                .into_iter()
                .map(|item| substitute(substs, item))
                .collect(),
        ),
        Ty::Var(var) => match substs.get(&var) {
            Some(new_ty) => substitute(substs, new_ty.clone()),
            None => Ty::Var(var),
        },
    }
}

fn functor_set(expr: Option<&FunctorExpr>) -> HashSet<Functor> {
    match expr {
        None => HashSet::new(),
        Some(expr) => match &expr.kind {
            FunctorExprKind::BinOp(op, lhs, rhs) => {
                let lhs = functor_set(Some(lhs));
                let rhs = functor_set(Some(rhs));
                match op {
                    SetOp::Union => lhs.union(&rhs).copied().collect(),
                    SetOp::Intersect => lhs.intersection(&rhs).copied().collect(),
                }
            }
            &FunctorExprKind::Lit(functor) => HashSet::from([functor]),
            FunctorExprKind::Paren(expr) => functor_set(Some(expr)),
        },
    }
}

fn try_var_id(ty: &Ty) -> Option<u32> {
    match ty {
        &Ty::Var(var) => Some(var),
        _ => None,
    }
}

fn callable_ty(resolutions: &Resolutions, decl: &CallableDecl) -> Option<Ty> {
    let input = try_pat_ty(resolutions, &decl.input)?;
    let output = try_convert_ty(resolutions, &decl.output)?;
    Some(Ty::Arrow(
        decl.kind,
        Box::new(input),
        Box::new(output),
        decl.functors.clone(),
    ))
}

fn try_convert_ty(resolutions: &Resolutions, ty: &ast::Ty) -> Option<Ty> {
    match &ty.kind {
        TyKind::App(base, args) => {
            let base = try_convert_ty(resolutions, base)?;
            let args = args
                .iter()
                .map(|arg| try_convert_ty(resolutions, arg))
                .collect::<Option<_>>()?;
            Some(Ty::App(Box::new(base), args))
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let input = try_convert_ty(resolutions, input)?;
            let output = try_convert_ty(resolutions, output)?;
            Some(Ty::Arrow(
                *kind,
                Box::new(input),
                Box::new(output),
                functors.clone(),
            ))
        }
        TyKind::Hole => None,
        TyKind::Paren(inner) => try_convert_ty(resolutions, inner),
        TyKind::Path(path) => Some(Ty::DefId(
            *resolutions.get(&path.id).expect("path should be resolved"),
        )),
        &TyKind::Prim(prim) => Some(Ty::Prim(prim)),
        TyKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| try_convert_ty(resolutions, item))
                .collect::<Option<_>>()?;
            Some(Ty::Tuple(items))
        }
        TyKind::Var(name) => Some(Ty::Param(name.name.clone())),
    }
}

fn try_pat_ty(resolutions: &Resolutions, pat: &Pat) -> Option<Ty> {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => None,
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => try_convert_ty(resolutions, ty),
        PatKind::Paren(inner) => try_pat_ty(resolutions, inner),
        PatKind::Tuple(items) => {
            let items = items
                .iter()
                .map(|item| try_pat_ty(resolutions, item))
                .collect::<Option<_>>()?;
            Some(Ty::Tuple(items))
        }
    }
}

fn diverges(ty: &Ty) -> bool {
    matches!(ty, &Ty::Never)
}
