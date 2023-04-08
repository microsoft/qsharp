// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::PackageId,
    resolve::{DefId, PackageSrc, Resolutions},
};
use miette::Diagnostic;
use qsc_ast::{
    ast::{
        self, BinOp, Block, CallableBody, CallableDecl, CallableKind, Expr, ExprKind, Functor,
        FunctorExpr, FunctorExprKind, Lit, NodeId, Package, Pat, PatKind, QubitInit, QubitInitKind,
        SetOp, Span, Spec, SpecBody, Stmt, StmtKind, TernOp, TyKind, TyPrim, UnOp,
    },
    visit::Visitor,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
    mem,
};
use thiserror::Error;

#[cfg(test)]
mod tests;

pub type Tys = HashMap<NodeId, Ty>;

#[derive(Clone, Debug)]
pub enum Ty {
    Array(Box<Ty>),
    Arrow(CallableKind, Box<Ty>, Box<Ty>, HashSet<Functor>),
    DefId(DefId),
    Err,
    Param(String),
    Prim(TyPrim),
    Tuple(Vec<Ty>),
    Var(Var),
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ty::Array(item) => write!(f, "({item})[]"),
            Ty::Arrow(kind, input, output, functors) => {
                let arrow = match kind {
                    CallableKind::Function => "->",
                    CallableKind::Operation => "=>",
                };
                write!(f, "({input}) {arrow} ({output})")?;
                if functors.contains(&Functor::Adj) && functors.contains(&Functor::Ctl) {
                    f.write_str(" is Adj + Ctl")?;
                } else if functors.contains(&Functor::Adj) {
                    f.write_str(" is Adj")?;
                } else if functors.contains(&Functor::Ctl) {
                    f.write_str(" is Ctl")?;
                }
                Ok(())
            }
            Ty::DefId(DefId {
                package: PackageSrc::Local,
                node,
            }) => write!(f, "Def<{node}>"),
            Ty::DefId(DefId {
                package: PackageSrc::Extern(package),
                node,
            }) => write!(f, "Def<{package}, {node}>"),
            Ty::Err => f.write_str("Err"),
            Ty::Param(name) => write!(f, "'{name}"),
            Ty::Prim(prim) => prim.fmt(f),
            Ty::Tuple(items) => {
                f.write_str("(")?;
                if let Some((first, rest)) = items.split_first() {
                    Display::fmt(first, f)?;
                    if rest.is_empty() {
                        f.write_str(",")?;
                    } else {
                        for item in rest {
                            f.write_str(", ")?;
                            Display::fmt(item, f)?;
                        }
                    }
                }
                f.write_str(")")
            }
            Ty::Var(id) => Display::fmt(id, f),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Var(u32);

impl Display for Var {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "?{}", self.0)
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

#[derive(Clone, Debug, Diagnostic, Error)]
pub enum Error {
    #[error("mismatched types")]
    TypeMismatch(Ty, Ty, #[label("expected {0}, found {1}")] Span),
    #[error("missing class instance")]
    MissingClass(Class, #[label("requires {0}")] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    MissingItemTy(#[label("explicit type required")] Span),
}

struct ClassError(Class, Span);

struct MissingTyError(Span);

struct UnifyError(Ty, Ty);

pub(super) struct Checker<'a> {
    resolutions: &'a Resolutions,
    globals: HashMap<DefId, Ty>,
    tys: Tys,
    errors: Vec<Error>,
}

impl Checker<'_> {
    pub(super) fn into_tys(self) -> (Tys, Vec<Error>) {
        (self.tys, self.errors)
    }
}

impl Visitor<'_> for Checker<'_> {
    fn visit_package(&mut self, package: &Package) {
        for namespace in &package.namespaces {
            self.visit_namespace(namespace);
        }

        if let Some(entry) = &package.entry {
            let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
            inferrer.infer_expr(entry);
            let (tys, errors) = inferrer.solve();
            self.tys.extend(tys);
            self.errors.extend(errors);
        }
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        match &decl.body {
            CallableBody::Block(block) => {
                let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
                inferrer.infer_pat(&decl.input);
                let decl_output = inferrer.convert_ty(&decl.output);
                let block_output = inferrer.infer_block(block).unwrap();
                inferrer.constrain(
                    block.span,
                    ConstraintKind::Eq {
                        expected: decl_output,
                        actual: block_output,
                    },
                );
                let (tys, errors) = inferrer.solve();
                self.tys.extend(tys);
                self.errors.extend(errors);
            }
            CallableBody::Specs(specs) => {
                for spec in specs {
                    match &spec.body {
                        SpecBody::Gen(_) => {}
                        SpecBody::Impl(input, block) => {
                            let mut inferrer = Inferrer::new(self.resolutions, &self.globals);

                            let callable_input_ty = inferrer.infer_pat(&decl.input);
                            let expected_input_ty = match spec.spec {
                                Spec::Body | Spec::Adj => callable_input_ty,
                                Spec::Ctl | Spec::CtlAdj => Ty::Tuple(vec![
                                    Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit))),
                                    callable_input_ty,
                                ]),
                            };
                            let actual_input_ty = inferrer.infer_pat(input);
                            inferrer.constrain(
                                input.span,
                                ConstraintKind::Eq {
                                    expected: expected_input_ty,
                                    actual: actual_input_ty,
                                },
                            );

                            let decl_output = inferrer.convert_ty(&decl.output);
                            let block_output = inferrer.infer_block(block).unwrap();
                            inferrer.constrain(
                                block.span,
                                ConstraintKind::Eq {
                                    expected: decl_output,
                                    actual: block_output,
                                },
                            );
                            let (tys, errors) = inferrer.solve();
                            self.tys.extend(tys);
                            self.errors.extend(errors);
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
    errors: Vec<Error>,
}

impl<'a> GlobalTable<'a> {
    pub(super) fn new(resolutions: &'a Resolutions) -> Self {
        Self {
            resolutions,
            globals: HashMap::new(),
            package: PackageSrc::Local,
            errors: Vec::new(),
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
            errors: self.errors,
        }
    }
}

impl Visitor<'_> for GlobalTable<'_> {
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        let (ty, errors) = callable_ty(self.resolutions, decl);
        let id = DefId {
            package: self.package,
            node: decl.name.id,
        };
        self.globals.insert(id, ty);
        for error in errors {
            self.errors.push(Error::MissingItemTy(error.0));
        }
    }
}

struct Constraint {
    span: Span,
    kind: ConstraintKind,
}

enum ConstraintKind {
    Class(Class),
    Eq { expected: Ty, actual: Ty },
}

#[derive(Clone, Debug)]
pub enum Class {
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
    Exp {
        base: Ty,
        power: Ty,
    },
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
            Self::Exp { base, .. } => vec![base],
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
            Self::Exp { base, power } => Self::Exp {
                base: f(base),
                power: f(power),
            },
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

impl Display for Class {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Class::Add(ty) => write!(f, "Add<{ty}>"),
            Class::Adj(ty) => write!(f, "Adj<{ty}>"),
            Class::Call { callee, .. } => write!(f, "Call<{callee}>"),
            Class::Ctl { op, .. } => write!(f, "Ctl<{op}>"),
            Class::Eq(ty) => write!(f, "Eq<{ty}>"),
            Class::Exp { base, .. } => write!(f, "Exp<{base}>"),
            Class::HasField { record, name, .. } => write!(f, "HasField<{record}, {name}>"),
            Class::HasFunctorsIfOp { callee, functors } => {
                write!(f, "HasFunctorsIfOp<{callee}, {functors:?}>")
            }
            Class::HasIndex {
                container, index, ..
            } => write!(f, "HasIndex<{container}, {index}>"),
            Class::HasPartialApp { callee, .. } => write!(f, "HasPartialApp<{callee}>"),
            Class::Integral(ty) => write!(f, "Integral<{ty}>"),
            Class::Iterable { container, .. } => write!(f, "Iterable<{container}>"),
            Class::Num(ty) => write!(f, "Num<{ty}"),
            Class::Unwrap { wrapper, .. } => write!(f, "Unwrap<{wrapper}>"),
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
    fn infer_expr(&mut self, expr: &Expr) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let ty = match &expr.kind {
            ExprKind::Array(items) => match items.split_first() {
                Some((first, rest)) => {
                    let first_ty = termination.update(self.infer_expr(first));
                    for item in rest {
                        let item_ty = termination.update(self.infer_expr(item));
                        self.constrain(
                            item.span,
                            ConstraintKind::Eq {
                                expected: first_ty.clone(),
                                actual: item_ty,
                            },
                        );
                    }

                    Ty::Array(Box::new(first_ty))
                }
                None => Ty::Array(Box::new(self.fresh())),
            },
            ExprKind::ArrayRepeat(item, size) => {
                let item_ty = termination.update(self.infer_expr(item));
                let size_ty = termination.update(self.infer_expr(size));
                self.constrain(
                    size.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Int),
                        actual: size_ty,
                    },
                );
                Ty::Array(Box::new(item_ty))
            }
            ExprKind::Assign(lhs, rhs) => {
                let lhs_ty = self.infer_expr(lhs).unwrap();
                let rhs_ty = termination.update(self.infer_expr(rhs));
                self.constrain(
                    lhs.span,
                    ConstraintKind::Eq {
                        expected: lhs_ty,
                        actual: rhs_ty,
                    },
                );
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                termination.update(self.infer_binop(expr.span, *op, lhs, rhs));
                Ty::Tuple(Vec::new())
            }
            ExprKind::AssignUpdate(container, index, item) => {
                termination.update(self.infer_update(expr.span, container, index, item));
                Ty::Tuple(Vec::new())
            }
            ExprKind::BinOp(op, lhs, rhs) => {
                termination.update(self.infer_binop(expr.span, *op, lhs, rhs))
            }
            ExprKind::Block(block) => termination.update(self.infer_block(block)),
            ExprKind::Call(callee, input) => {
                let callee_ty = termination.update(self.infer_expr(callee));
                let input_ty = termination.update(self.infer_expr(input));
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
                termination.update(self.infer_block(within));
                termination.update(self.infer_block(apply))
            }
            ExprKind::Fail(message) => {
                let message_ty = self.infer_expr(message).unwrap();
                self.constrain(
                    message.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::String),
                        actual: message_ty,
                    },
                );
                termination = Termination::Diverges;
                Ty::Err
            }
            ExprKind::Field(record, name) => {
                let record_ty = termination.update(self.infer_expr(record));
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
                let container_ty = termination.update(self.infer_expr(container));
                self.constrain(
                    container.span,
                    ConstraintKind::Class(Class::Iterable {
                        container: container_ty,
                        item: item_ty,
                    }),
                );

                termination.update(self.infer_block(body));
                Ty::Tuple(Vec::new())
            }
            ExprKind::If(cond, if_true, if_false) => {
                let cond_ty = termination.update(self.infer_expr(cond));
                self.constrain(
                    cond.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Bool),
                        actual: cond_ty,
                    },
                );

                let true_ty = self.infer_block(if_true);
                let false_ty = if_false
                    .as_ref()
                    .map_or(Fallible::Convergent(Ty::Tuple(Vec::new())), |e| {
                        self.infer_expr(e)
                    });
                let (true_ty, false_ty) = termination.update_and(true_ty, false_ty);
                self.constrain(
                    expr.span,
                    ConstraintKind::Eq {
                        expected: true_ty.clone(),
                        actual: false_ty,
                    },
                );
                true_ty
            }
            ExprKind::Index(container, index) => {
                let container_ty = termination.update(self.infer_expr(container));
                let index_ty = termination.update(self.infer_expr(index));
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
                    let ty = termination.update(self.infer_expr(expr));
                    self.constrain(
                        expr.span,
                        ConstraintKind::Eq {
                            expected: Ty::Prim(TyPrim::Int),
                            actual: ty,
                        },
                    );
                }
                Ty::Prim(TyPrim::Range)
            }
            ExprKind::Repeat(body, until, fixup) => {
                termination.update(self.infer_block(body));
                let until_ty = termination.update(self.infer_expr(until));
                self.constrain(
                    until.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Bool),
                        actual: until_ty,
                    },
                );

                if let Some(fixup) = fixup {
                    termination.update(self.infer_block(fixup));
                }

                Ty::Tuple(Vec::new())
            }
            ExprKind::Return(expr) => {
                self.infer_expr(expr);
                termination = Termination::Diverges;
                Ty::Err
            }
            ExprKind::TernOp(TernOp::Cond, cond, if_true, if_false) => {
                let cond_ty = termination.update(self.infer_expr(cond));
                self.constrain(
                    cond.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Bool),
                        actual: cond_ty,
                    },
                );

                let true_ty = self.infer_expr(if_true);
                let false_ty = self.infer_expr(if_false);
                let (true_ty, false_ty) = termination.update_and(true_ty, false_ty);
                self.constrain(
                    expr.span,
                    ConstraintKind::Eq {
                        expected: true_ty.clone(),
                        actual: false_ty,
                    },
                );
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
                self.constrain(
                    cond.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Bool),
                        actual: cond_ty,
                    },
                );

                termination.update(self.infer_block(body));
                Ty::Tuple(Vec::new())
            }
            ExprKind::Err | ExprKind::Hole => self.fresh(),
        };

        let ty = if termination.diverges() {
            self.fresh()
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
            self.fresh()
        } else {
            last.unwrap_or(Ty::Tuple(Vec::new()))
        };
        self.tys.insert(block.id, ty.clone());
        termination.wrap(ty)
    }

    fn infer_stmt(&mut self, stmt: &Stmt) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let ty = match &stmt.kind {
            StmtKind::Empty => Ty::Tuple(Vec::new()),
            StmtKind::Expr(expr) => termination.update(self.infer_expr(expr)),
            StmtKind::Local(_, pat, expr) => {
                let pat_ty = self.infer_pat(pat);
                let expr_ty = termination.update(self.infer_expr(expr));
                self.constrain(
                    pat.span,
                    ConstraintKind::Eq {
                        expected: expr_ty,
                        actual: pat_ty,
                    },
                );
                Ty::Tuple(Vec::new())
            }
            StmtKind::Qubit(_, pat, init, block) => {
                let pat_ty = self.infer_pat(pat);
                let init_ty = termination.update(self.infer_qubit_init(init));
                self.constrain(
                    pat.span,
                    ConstraintKind::Eq {
                        expected: init_ty,
                        actual: pat_ty,
                    },
                );
                match block {
                    None => Ty::Tuple(Vec::new()),
                    Some(block) => termination.update(self.infer_block(block)),
                }
            }
            StmtKind::Semi(expr) => {
                termination.update(self.infer_expr(expr));
                Ty::Tuple(Vec::new())
            }
        };

        let ty = if termination.diverges() {
            self.fresh()
        } else {
            ty
        };
        self.tys.insert(stmt.id, ty.clone());
        termination.wrap(ty)
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

    fn infer_qubit_init(&mut self, init: &QubitInit) -> Fallible<Ty> {
        let mut termination = Termination::Converges;
        let ty = match &init.kind {
            QubitInitKind::Array(length) => {
                let length_ty = termination.update(self.infer_expr(length));
                self.constrain(
                    length.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Int),
                        actual: length_ty,
                    },
                );
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
            self.fresh()
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
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Bool),
                        actual: operand_ty.clone(),
                    },
                );
                operand_ty
            }
            UnOp::Unwrap => todo!("user-defined types not supported"),
        };

        termination.wrap(if termination.diverges() {
            self.fresh()
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
                self.constrain(
                    span,
                    ConstraintKind::Eq {
                        expected: lhs_ty.clone(),
                        actual: rhs_ty,
                    },
                );
                self.constrain(
                    lhs.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Bool),
                        actual: lhs_ty.clone(),
                    },
                );
                lhs_ty
            }
            BinOp::Eq | BinOp::Neq => {
                self.constrain(
                    span,
                    ConstraintKind::Eq {
                        expected: lhs_ty.clone(),
                        actual: rhs_ty,
                    },
                );
                self.constrain(lhs.span, ConstraintKind::Class(Class::Eq(lhs_ty)));
                Ty::Prim(TyPrim::Bool)
            }
            BinOp::Add => {
                self.constrain(
                    span,
                    ConstraintKind::Eq {
                        expected: lhs_ty.clone(),
                        actual: rhs_ty,
                    },
                );
                self.constrain(lhs.span, ConstraintKind::Class(Class::Add(lhs_ty.clone())));
                lhs_ty
            }
            BinOp::Gt | BinOp::Gte | BinOp::Lt | BinOp::Lte => {
                self.constrain(
                    span,
                    ConstraintKind::Eq {
                        expected: lhs_ty.clone(),
                        actual: rhs_ty,
                    },
                );
                self.constrain(lhs.span, ConstraintKind::Class(Class::Num(lhs_ty)));
                Ty::Prim(TyPrim::Bool)
            }
            BinOp::AndB
            | BinOp::Div
            | BinOp::Mod
            | BinOp::Mul
            | BinOp::OrB
            | BinOp::Sub
            | BinOp::XorB => {
                self.constrain(
                    span,
                    ConstraintKind::Eq {
                        expected: lhs_ty.clone(),
                        actual: rhs_ty,
                    },
                );
                self.constrain(lhs.span, ConstraintKind::Class(Class::Num(lhs_ty.clone())));
                lhs_ty
            }
            BinOp::Exp => {
                self.constrain(
                    span,
                    ConstraintKind::Class(Class::Exp {
                        base: lhs_ty.clone(),
                        power: rhs_ty,
                    }),
                );
                lhs_ty
            }
            BinOp::Shl | BinOp::Shr => {
                self.constrain(
                    lhs.span,
                    ConstraintKind::Class(Class::Integral(lhs_ty.clone())),
                );
                self.constrain(
                    rhs.span,
                    ConstraintKind::Eq {
                        expected: Ty::Prim(TyPrim::Int),
                        actual: rhs_ty,
                    },
                );
                lhs_ty
            }
        };

        termination.wrap(if termination.diverges() {
            self.fresh()
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
        self.constrain(
            span,
            ConstraintKind::Class(Class::HasIndex {
                container: container_ty.clone(),
                index: index_ty,
                item: item_ty,
            }),
        );

        termination.wrap(if termination.diverges() {
            self.fresh()
        } else {
            container_ty
        })
    }

    fn fresh(&mut self) -> Ty {
        let var = self.next_var;
        self.next_var += 1;
        Ty::Var(Var(var))
    }

    fn instantiate(&mut self, ty: &Ty) -> Ty {
        fn go(fresh: &mut impl FnMut() -> Ty, vars: &mut HashMap<String, Ty>, ty: &Ty) -> Ty {
            match ty {
                Ty::Array(item) => Ty::Array(Box::new(go(fresh, vars, item))),
                Ty::Arrow(kind, input, output, functors) => Ty::Arrow(
                    *kind,
                    Box::new(go(fresh, vars, input)),
                    Box::new(go(fresh, vars, output)),
                    functors.clone(),
                ),
                &Ty::DefId(id) => Ty::DefId(id),
                Ty::Err => Ty::Err,
                Ty::Param(name) => vars.entry(name.clone()).or_insert_with(fresh).clone(),
                &Ty::Prim(prim) => Ty::Prim(prim),
                Ty::Tuple(items) => {
                    Ty::Tuple(items.iter().map(|item| go(fresh, vars, item)).collect())
                }
                &Ty::Var(var) => Ty::Var(var),
            }
        }

        go(&mut || self.fresh(), &mut HashMap::new(), ty)
    }

    fn convert_ty(&mut self, ty: &ast::Ty) -> Ty {
        match &ty.kind {
            TyKind::Array(item) => Ty::Array(Box::new(self.convert_ty(item))),
            TyKind::Arrow(kind, input, output, functors) => Ty::Arrow(
                *kind,
                Box::new(self.convert_ty(input)),
                Box::new(self.convert_ty(output)),
                functor_set(functors.as_ref()),
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

    fn solve(self) -> (Tys, Vec<Error>) {
        let mut substs = HashMap::new();
        let mut pending_classes: HashMap<_, Vec<_>> = HashMap::new();
        let mut constraints = self.constraints;
        let mut new_constraints = Vec::new();
        let mut errors = Vec::new();

        loop {
            for constraint in constraints {
                match constraint.kind {
                    ConstraintKind::Class(class) => {
                        let unsolved: Vec<_> = class
                            .dependencies()
                            .into_iter()
                            .filter_map(|ty| try_var(&substitute(&substs, ty.clone())))
                            .collect();

                        if unsolved.is_empty() {
                            match classify(constraint.span, class.map(|ty| substitute(&substs, ty)))
                            {
                                Ok(new) => new_constraints.extend(new),
                                Err(error) => {
                                    errors.push(Error::MissingClass(error.0, error.1));
                                }
                            }
                        } else {
                            for var in unsolved {
                                pending_classes.entry(var).or_default().push(class.clone());
                            }
                        }
                    }
                    ConstraintKind::Eq { expected, actual } => {
                        let ty1 = substitute(&substs, expected);
                        let ty2 = substitute(&substs, actual);
                        let new_substs = match unify(&ty1, &ty2) {
                            Ok(new_substs) => new_substs,
                            Err(UnifyError(ty1, ty2)) => {
                                errors.push(Error::TypeMismatch(ty1, ty2, constraint.span));
                                Vec::new()
                            }
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
                }
            }

            if new_constraints.is_empty() {
                break;
            }

            constraints = mem::take(&mut new_constraints);
        }

        let tys = self
            .tys
            .into_iter()
            .map(|(id, ty)| (id, substitute(&substs, ty)))
            .collect();
        (tys, errors)
    }
}

fn unify(ty1: &Ty, ty2: &Ty) -> Result<Vec<(Var, Ty)>, UnifyError> {
    match (ty1, ty2) {
        (Ty::Array(item1), Ty::Array(item2)) => unify(item1, item2),
        // TODO: Ignoring functors is unsound, but we don't know which one should be a subset of the
        // other until subtyping is supported.
        (Ty::Arrow(kind1, input1, output1, _), Ty::Arrow(kind2, input2, output2, _))
            if kind1 == kind2 =>
        {
            let mut substs = unify(input1, input2)?;
            substs.extend(unify(output1, output2)?);
            Ok(substs)
        }
        (Ty::DefId(def1), Ty::DefId(def2)) if def1 == def2 => Ok(Vec::new()),
        (Ty::Err, _) | (_, Ty::Err) => Ok(Vec::new()),
        (Ty::Param(name1), Ty::Param(name2)) if name1 == name2 => Ok(Vec::new()),
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
fn classify(span: Span, class: Class) -> Result<Vec<Constraint>, ClassError> {
    match class {
        Class::Eq(Ty::Prim(
            TyPrim::BigInt
            | TyPrim::Bool
            | TyPrim::Double
            | TyPrim::Int
            | TyPrim::Qubit
            | TyPrim::Range
            | TyPrim::Result
            | TyPrim::String
            | TyPrim::Pauli,
        ))
        | Class::Integral(Ty::Prim(TyPrim::BigInt | TyPrim::Int))
        | Class::Num(Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int))
        | Class::Add(Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int | TyPrim::String)) => {
            Ok(Vec::new())
        }
        Class::Add(Ty::Array(_)) => Ok(Vec::new()),
        Class::Adj(Ty::Arrow(_, _, _, functors)) if functors.contains(&Functor::Adj) => {
            Ok(Vec::new())
        }
        Class::Call {
            callee: Ty::Arrow(_, callee_input, callee_output, _),
            input,
            output,
        } => Ok(vec![
            Constraint {
                span,
                kind: ConstraintKind::Eq {
                    expected: *callee_input,
                    actual: input,
                },
            },
            Constraint {
                span,
                kind: ConstraintKind::Eq {
                    expected: *callee_output,
                    actual: output,
                },
            },
        ]),
        Class::Ctl {
            op: Ty::Arrow(kind, input, output, functors),
            with_ctls,
        } if functors.contains(&Functor::Ctl) => {
            let qubit_array = Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit)));
            let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *input]));
            Ok(vec![Constraint {
                span,
                kind: ConstraintKind::Eq {
                    expected: Ty::Arrow(kind, ctl_input, output, functors),
                    actual: with_ctls,
                },
            }])
        }
        Class::Eq(Ty::Array(item)) => Ok(vec![Constraint {
            span,
            kind: ConstraintKind::Class(Class::Eq(*item)),
        }]),
        Class::Eq(Ty::Tuple(items)) => Ok(items
            .into_iter()
            .map(|item| Constraint {
                span,
                kind: ConstraintKind::Class(Class::Eq(item)),
            })
            .collect()),
        Class::Exp {
            base: Ty::Prim(TyPrim::BigInt),
            power,
        } => Ok(vec![Constraint {
            span,
            kind: ConstraintKind::Eq {
                expected: Ty::Prim(TyPrim::Int),
                actual: power,
            },
        }]),
        Class::Exp {
            base: base @ Ty::Prim(TyPrim::Double | TyPrim::Int),
            power,
        } => Ok(vec![Constraint {
            span,
            kind: ConstraintKind::Eq {
                expected: base,
                actual: power,
            },
        }]),
        Class::HasField { .. } => todo!("user-defined types not supported"),
        Class::HasFunctorsIfOp { callee, functors } => match callee {
            Ty::Arrow(CallableKind::Operation, _, _, callee_functors)
                if callee_functors.is_subset(&functors) =>
            {
                Ok(Vec::new())
            }
            Ty::Arrow(CallableKind::Operation, _, _, _) => Err(ClassError(
                Class::HasFunctorsIfOp { callee, functors },
                span,
            )),
            _ => Ok(Vec::new()),
        },
        Class::HasIndex {
            container: Ty::Array(container_item),
            index,
            item,
        } => match index {
            Ty::Prim(TyPrim::Int) => Ok(vec![Constraint {
                span,
                kind: ConstraintKind::Eq {
                    expected: *container_item,
                    actual: item,
                },
            }]),
            Ty::Prim(TyPrim::Range) => Ok(vec![Constraint {
                span,
                kind: ConstraintKind::Eq {
                    expected: Ty::Array(container_item),
                    actual: item,
                },
            }]),
            _ => Err(ClassError(
                Class::HasIndex {
                    container: Ty::Array(container_item),
                    index,
                    item,
                },
                span,
            )),
        },
        Class::HasPartialApp { .. } => todo!("partial application not supported"),
        Class::Iterable {
            container: Ty::Prim(TyPrim::Range),
            item,
        } => Ok(vec![Constraint {
            span,
            kind: ConstraintKind::Eq {
                expected: Ty::Prim(TyPrim::Int),
                actual: item,
            },
        }]),
        Class::Iterable {
            container: Ty::Array(container_item),
            item,
        } => Ok(vec![Constraint {
            span,
            kind: ConstraintKind::Eq {
                expected: *container_item,
                actual: item,
            },
        }]),
        Class::Unwrap { .. } => todo!("user-defined types not supported"),
        class if class.dependencies().iter().any(|ty| matches!(ty, Ty::Err)) => Ok(Vec::new()),
        class => Err(ClassError(class, span)),
    }
}

fn substitute(substs: &HashMap<Var, Ty>, ty: Ty) -> Ty {
    match ty {
        Ty::Array(item) => Ty::Array(Box::new(substitute(substs, *item))),
        Ty::Arrow(kind, input, output, functors) => Ty::Arrow(
            kind,
            Box::new(substitute(substs, *input)),
            Box::new(substitute(substs, *output)),
            functors,
        ),
        Ty::DefId(id) => Ty::DefId(id),
        Ty::Err => Ty::Err,
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

fn try_var(ty: &Ty) -> Option<Var> {
    match ty {
        &Ty::Var(var) => Some(var),
        _ => None,
    }
}

fn callable_ty(resolutions: &Resolutions, decl: &CallableDecl) -> (Ty, Vec<MissingTyError>) {
    let (input, mut errors) = try_pat_ty(resolutions, &decl.input);
    let (output, output_errors) = try_convert_ty(resolutions, &decl.output);
    errors.extend(output_errors);

    let sig_functors = functor_set(decl.functors.as_ref());
    let body_functors = match &decl.body {
        CallableBody::Block(_) => HashSet::new(),
        CallableBody::Specs(specs) => specs
            .iter()
            .flat_map(|spec| match spec.spec {
                Spec::Body => Vec::new(),
                Spec::Adj => vec![Functor::Adj],
                Spec::Ctl => vec![Functor::Ctl],
                Spec::CtlAdj => vec![Functor::Adj, Functor::Ctl],
            })
            .collect(),
    };

    let functors = sig_functors.union(&body_functors).copied().collect();
    let ty = Ty::Arrow(decl.kind, Box::new(input), Box::new(output), functors);
    (ty, errors)
}

fn try_convert_ty(resolutions: &Resolutions, ty: &ast::Ty) -> (Ty, Vec<MissingTyError>) {
    match &ty.kind {
        TyKind::Array(item) => {
            let (new_item, errors) = try_convert_ty(resolutions, item);
            (Ty::Array(Box::new(new_item)), errors)
        }
        TyKind::Arrow(kind, input, output, functors) => {
            let (input, mut errors) = try_convert_ty(resolutions, input);
            let (output, output_errors) = try_convert_ty(resolutions, output);
            errors.extend(output_errors);
            let ty = Ty::Arrow(
                *kind,
                Box::new(input),
                Box::new(output),
                functor_set(functors.as_ref()),
            );
            (ty, errors)
        }
        TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
        TyKind::Paren(inner) => try_convert_ty(resolutions, inner),
        TyKind::Path(path) => (
            resolutions
                .get(&path.id)
                .copied()
                .map_or(Ty::Err, Ty::DefId),
            Vec::new(),
        ),
        &TyKind::Prim(prim) => (Ty::Prim(prim), Vec::new()),
        TyKind::Tuple(items) => {
            let mut new_items = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (new_item, item_errors) = try_convert_ty(resolutions, item);
                new_items.push(new_item);
                errors.extend(item_errors);
            }
            (Ty::Tuple(new_items), errors)
        }
        TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
    }
}

fn try_pat_ty(resolutions: &Resolutions, pat: &Pat) -> (Ty, Vec<MissingTyError>) {
    match &pat.kind {
        PatKind::Bind(_, None) | PatKind::Discard(None) | PatKind::Elided => {
            (Ty::Err, vec![MissingTyError(pat.span)])
        }
        PatKind::Bind(_, Some(ty)) | PatKind::Discard(Some(ty)) => try_convert_ty(resolutions, ty),
        PatKind::Paren(inner) => try_pat_ty(resolutions, inner),
        PatKind::Tuple(items) => {
            let mut new_items = Vec::new();
            let mut errors = Vec::new();
            for item in items {
                let (new_item, item_errors) = try_pat_ty(resolutions, item);
                new_items.push(new_item);
                errors.extend(item_errors);
            }
            (Ty::Tuple(new_items), errors)
        }
    }
}
