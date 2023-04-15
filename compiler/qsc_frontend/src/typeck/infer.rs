// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Error, ErrorKind};
use qsc_ast::ast;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::hir;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{self, Debug, Display, Formatter},
};

pub(super) type Substitutions = IndexMap<Var, Ty>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Var(usize);

impl Display for Var {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "?{}", self.0)
    }
}

impl From<usize> for Var {
    fn from(value: usize) -> Self {
        Var(value)
    }
}

impl From<Var> for usize {
    fn from(value: Var) -> Self {
        value.0
    }
}

#[derive(Clone, Debug)]
pub enum Ty {
    Array(Box<Ty>),
    Arrow(CallableKind, Box<Ty>, Box<Ty>, HashSet<Functor>),
    Err,
    Param(String),
    Prim(TyPrim),
    Tuple(Vec<Ty>),
    Var(Var),
}

impl Ty {
    pub(super) const UNIT: Self = Self::Tuple(Vec::new());
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

                let is = if functors.contains(&Functor::Adj) && functors.contains(&Functor::Ctl) {
                    " is Adj + Ctl"
                } else if functors.contains(&Functor::Adj) {
                    " is Adj"
                } else if functors.contains(&Functor::Ctl) {
                    " is Ctl"
                } else {
                    ""
                };

                write!(f, "({input}) {arrow} ({output}){is}")
            }
            Ty::Err => f.write_str("?"),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TyPrim {
    BigInt,
    Bool,
    Double,
    Int,
    Pauli,
    Qubit,
    Range,
    Result,
    String,
}

impl From<ast::TyPrim> for TyPrim {
    fn from(value: ast::TyPrim) -> Self {
        match value {
            ast::TyPrim::BigInt => Self::BigInt,
            ast::TyPrim::Bool => Self::Bool,
            ast::TyPrim::Double => Self::Double,
            ast::TyPrim::Int => Self::Int,
            ast::TyPrim::Pauli => Self::Pauli,
            ast::TyPrim::Qubit => Self::Qubit,
            ast::TyPrim::Range => Self::Range,
            ast::TyPrim::Result => Self::Result,
            ast::TyPrim::String => Self::String,
        }
    }
}

impl From<hir::TyPrim> for TyPrim {
    fn from(value: hir::TyPrim) -> Self {
        match value {
            hir::TyPrim::BigInt => Self::BigInt,
            hir::TyPrim::Bool => Self::Bool,
            hir::TyPrim::Double => Self::Double,
            hir::TyPrim::Int => Self::Int,
            hir::TyPrim::Pauli => Self::Pauli,
            hir::TyPrim::Qubit => Self::Qubit,
            hir::TyPrim::Range => Self::Range,
            hir::TyPrim::Result => Self::Result,
            hir::TyPrim::String => Self::String,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableKind {
    Function,
    Operation,
}

impl From<ast::CallableKind> for CallableKind {
    fn from(value: ast::CallableKind) -> Self {
        match value {
            ast::CallableKind::Function => Self::Function,
            ast::CallableKind::Operation => Self::Operation,
        }
    }
}

impl From<&ast::CallableKind> for CallableKind {
    fn from(value: &ast::CallableKind) -> Self {
        match value {
            ast::CallableKind::Function => Self::Function,
            ast::CallableKind::Operation => Self::Operation,
        }
    }
}

impl From<hir::CallableKind> for CallableKind {
    fn from(value: hir::CallableKind) -> Self {
        match value {
            hir::CallableKind::Function => Self::Function,
            hir::CallableKind::Operation => Self::Operation,
        }
    }
}

impl From<&hir::CallableKind> for CallableKind {
    fn from(value: &hir::CallableKind) -> Self {
        match value {
            hir::CallableKind::Function => Self::Function,
            hir::CallableKind::Operation => Self::Operation,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Functor {
    Adj,
    Ctl,
}

impl From<ast::Functor> for Functor {
    fn from(value: ast::Functor) -> Self {
        match value {
            ast::Functor::Adj => Self::Adj,
            ast::Functor::Ctl => Self::Ctl,
        }
    }
}

impl From<hir::Functor> for Functor {
    fn from(value: hir::Functor) -> Self {
        match value {
            hir::Functor::Adj => Self::Adj,
            hir::Functor::Ctl => Self::Ctl,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) enum Class {
    Add(Ty),
    Adj(Ty),
    Call { callee: Ty, input: Ty, output: Ty },
    Ctl { op: Ty, with_ctls: Ty },
    Eq(Ty),
    Exp { base: Ty, power: Ty },
    HasField { record: Ty, name: String, item: Ty },
    HasIndex { container: Ty, index: Ty, item: Ty },
    Integral(Ty),
    Iterable { container: Ty, item: Ty },
    Num(Ty),
    Unwrap { wrapper: Ty, base: Ty },
}

impl Class {
    fn dependencies(&self) -> Vec<&Ty> {
        match self {
            Self::Add(ty) | Self::Adj(ty) | Self::Eq(ty) | Self::Integral(ty) | Self::Num(ty) => {
                vec![ty]
            }
            Self::Call { callee, .. } => vec![callee],
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
            Self::HasIndex {
                container,
                index,
                item,
            } => Self::HasIndex {
                container: f(container),
                index: f(index),
                item: f(item),
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

    fn check(self, span: Span) -> Result<Vec<Constraint>, ClassError> {
        match self {
            Class::Add(ty) => check_add(&ty)
                .then_some(Vec::new())
                .ok_or(ClassError(Class::Add(ty), span)),
            Class::Adj(ty) => check_adj(&ty)
                .then_some(Vec::new())
                .ok_or(ClassError(Class::Adj(ty), span)),
            Class::Call {
                callee,
                input,
                output,
            } => check_call(callee, input, output, span),
            Class::Ctl { op, with_ctls } => check_ctl(op, with_ctls, span).map(|c| vec![c]),
            Class::Eq(ty) => check_eq(ty, span),
            Class::Exp { base, power } => check_exp(base, power, span).map(|c| vec![c]),
            Class::HasField { record, name, item } => {
                // TODO: If the record type is a user-defined type, look up its fields.
                // https://github.com/microsoft/qsharp/issues/148
                Err(ClassError(Class::HasField { record, name, item }, span))
            }
            Class::HasIndex {
                container,
                index,
                item,
            } => check_has_index(container, index, item, span).map(|c| vec![c]),
            Class::Integral(ty) => check_integral(&ty)
                .then_some(Vec::new())
                .ok_or(ClassError(Class::Integral(ty), span)),
            Class::Iterable { container, item } => {
                check_iterable(container, item, span).map(|c| vec![c])
            }
            Class::Num(ty) => check_num(&ty)
                .then_some(Vec::new())
                .ok_or(ClassError(Class::Num(ty), span)),
            Class::Unwrap { wrapper, base } => {
                // TODO: If the wrapper type is a user-defined type, look up its underlying type.
                // https://github.com/microsoft/qsharp/issues/148
                Err(ClassError(Class::Unwrap { wrapper, base }, span))
            }
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
            Class::HasIndex {
                container, index, ..
            } => write!(f, "HasIndex<{container}, {index}>"),
            Class::Integral(ty) => write!(f, "Integral<{ty}>"),
            Class::Iterable { container, .. } => write!(f, "Iterable<{container}>"),
            Class::Num(ty) => write!(f, "Num<{ty}"),
            Class::Unwrap { wrapper, .. } => write!(f, "Unwrap<{wrapper}>"),
        }
    }
}

enum Constraint {
    Class(Class, Span),
    Eq {
        expected: Ty,
        actual: Ty,
        span: Span,
    },
}

struct ClassError(Class, Span);

struct UnifyError(Ty, Ty);

pub(super) struct Inferrer {
    constraints: VecDeque<Constraint>,
    next_var: Var,
}

impl Inferrer {
    pub(super) fn new() -> Self {
        Self {
            constraints: VecDeque::new(),
            next_var: Var(0),
        }
    }

    /// Introduces an equality constraint between the expected and actual types.
    pub(super) fn eq(&mut self, span: Span, expected: Ty, actual: Ty) {
        self.constraints.push_back(Constraint::Eq {
            expected,
            actual,
            span,
        });
    }

    /// Introduces a class constraint.
    pub(super) fn class(&mut self, span: Span, class: Class) {
        self.constraints.push_back(Constraint::Class(class, span));
    }

    /// Returns a unique unconstrained type variable.
    pub(super) fn fresh(&mut self) -> Ty {
        let var = self.next_var;
        self.next_var = Var(var.0 + 1);
        Ty::Var(var)
    }

    /// Replaces all type parameters with fresh types.
    pub(super) fn freshen(&mut self, ty: &mut Ty) {
        fn freshen(solver: &mut Inferrer, params: &mut HashMap<String, Ty>, ty: &mut Ty) {
            match ty {
                Ty::Err | Ty::Prim(_) | Ty::Var(_) => {}
                Ty::Array(item) => freshen(solver, params, item),
                Ty::Arrow(_, input, output, _) => {
                    freshen(solver, params, input);
                    freshen(solver, params, output);
                }
                Ty::Param(name) => {
                    *ty = params
                        .entry(name.clone())
                        .or_insert_with(|| solver.fresh())
                        .clone();
                }
                Ty::Tuple(items) => {
                    for item in items {
                        freshen(solver, params, item);
                    }
                }
            }
        }

        freshen(self, &mut HashMap::new(), ty);
    }

    /// Solves for all type variables given the accumulated constraints.
    pub(super) fn solve(mut self) -> (Substitutions, Vec<Error>) {
        // TODO: Variables that don't have a substitution should cause errors for ambiguous types.
        // However, if an unsolved variable is the result of a divergent expression, it may be OK to
        // leave it or substitute it with a concrete uninhabited type.
        // https://github.com/microsoft/qsharp/issues/152
        let mut solver = Solver::new();
        while let Some(constraint) = self.constraints.pop_front() {
            self.constraints.extend(solver.constrain(constraint));
        }
        solver.into_substs()
    }
}

struct Solver {
    substs: Substitutions,
    pending: HashMap<Var, Vec<Class>>,
    errors: Vec<Error>,
}

impl Solver {
    fn new() -> Self {
        Self {
            substs: Substitutions::new(),
            pending: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn constrain(&mut self, constraint: Constraint) -> Vec<Constraint> {
        match constraint {
            Constraint::Class(class, span) => self.class(class, span),
            Constraint::Eq {
                expected,
                actual,
                span,
            } => self.eq(expected, actual, span),
        }
    }

    fn class(&mut self, class: Class, span: Span) -> Vec<Constraint> {
        let mut unknown_dependency = false;
        for ty in class.dependencies() {
            if let Some(var) = unknown_var(&self.substs, ty) {
                self.pending.entry(var).or_default().push(class.clone());
                unknown_dependency = true;
            }
        }

        if unknown_dependency {
            Vec::new()
        } else {
            match class.map(|ty| substituted(&self.substs, ty)).check(span) {
                Ok(constraints) => constraints,
                Err(ClassError(class, span)) => {
                    self.errors
                        .push(Error(ErrorKind::MissingClass(class, span)));
                    Vec::new()
                }
            }
        }
    }

    fn eq(&mut self, mut expected: Ty, mut actual: Ty, span: Span) -> Vec<Constraint> {
        substitute(&self.substs, &mut expected);
        substitute(&self.substs, &mut actual);
        let mut constraints = Vec::new();

        let mut bind = |var, ty| {
            self.substs.insert(var, ty);
            if let Some(classes) = self.pending.remove(&var) {
                constraints.extend(
                    classes
                        .into_iter()
                        .map(|class| Constraint::Class(class, span)),
                );
            }
        };

        match unify(&expected, &actual, &mut bind) {
            Ok(()) => {}
            Err(UnifyError(expected, actual)) => {
                self.errors
                    .push(Error(ErrorKind::TypeMismatch(expected, actual, span)));
            }
        }

        constraints
    }

    fn into_substs(self) -> (Substitutions, Vec<Error>) {
        (self.substs, self.errors)
    }
}

pub(super) fn substitute(substs: &Substitutions, ty: &mut Ty) {
    match ty {
        Ty::Err | Ty::Param(_) | Ty::Prim(_) => {}
        Ty::Array(item) => substitute(substs, item),
        Ty::Arrow(_, input, output, _) => {
            substitute(substs, input);
            substitute(substs, output);
        }
        Ty::Tuple(items) => {
            for item in items {
                substitute(substs, item);
            }
        }
        &mut Ty::Var(var) => {
            if let Some(new_ty) = substs.get(var) {
                *ty = new_ty.clone();
                substitute(substs, ty);
            }
        }
    }
}

fn substituted(substs: &Substitutions, mut ty: Ty) -> Ty {
    substitute(substs, &mut ty);
    ty
}

fn unify(ty1: &Ty, ty2: &Ty, bind: &mut impl FnMut(Var, Ty)) -> Result<(), UnifyError> {
    match (ty1, ty2) {
        (Ty::Array(item1), Ty::Array(item2)) => unify(item1, item2, bind),
        (Ty::Arrow(kind1, input1, output1, _), Ty::Arrow(kind2, input2, output2, _))
            if kind1 == kind2 =>
        {
            // TODO: We ignore functors until subtyping is supported. This is unsound, but the
            // alternative is disallowing valid programs.
            // https://github.com/microsoft/qsharp/issues/150
            unify(input1, input2, bind)?;
            unify(output1, output2, bind)?;
            Ok(())
        }
        (Ty::Param(name1), Ty::Param(name2)) if name1 == name2 => Ok(()),
        (Ty::Prim(prim1), Ty::Prim(prim2)) if prim1 == prim2 => Ok(()),
        (Ty::Tuple(items1), Ty::Tuple(items2)) if items1.len() == items2.len() => {
            for (item1, item2) in items1.iter().zip(items2) {
                unify(item1, item2, bind)?;
            }
            Ok(())
        }
        (Ty::Var(var1), Ty::Var(var2)) if var1 == var2 => Ok(()),
        (&Ty::Var(var), _) => {
            bind(var, ty2.clone());
            Ok(())
        }
        (_, &Ty::Var(var)) => {
            bind(var, ty1.clone());
            Ok(())
        }
        _ => Err(UnifyError(ty1.clone(), ty2.clone())),
    }
}

fn unknown_var(substs: &Substitutions, ty: &Ty) -> Option<Var> {
    match ty {
        &Ty::Var(var) => match substs.get(var) {
            None => Some(var),
            Some(ty) => unknown_var(substs, ty),
        },
        _ => None,
    }
}

fn check_add(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int | TyPrim::String) | Ty::Array(_)
    )
}

fn check_adj(ty: &Ty) -> bool {
    match ty {
        Ty::Arrow(_, _, _, functors) => functors.contains(&Functor::Adj),
        _ => false,
    }
}

fn check_call(
    callee: Ty,
    input: Ty,
    output: Ty,
    span: Span,
) -> Result<Vec<Constraint>, ClassError> {
    match callee {
        Ty::Arrow(_, callee_input, callee_output, _) => Ok(vec![
            Constraint::Eq {
                expected: *callee_input,
                actual: input,
                span,
            },
            Constraint::Eq {
                expected: *callee_output,
                actual: output,
                span,
            },
        ]),
        _ => Err(ClassError(
            Class::Call {
                callee,
                input,
                output,
            },
            span,
        )),
    }
}

fn check_ctl(op: Ty, with_ctls: Ty, span: Span) -> Result<Constraint, ClassError> {
    match op {
        Ty::Arrow(kind, input, output, functors) if functors.contains(&Functor::Ctl) => {
            let qubit_array = Ty::Array(Box::new(Ty::Prim(TyPrim::Qubit)));
            let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *input]));
            Ok(Constraint::Eq {
                expected: Ty::Arrow(kind, ctl_input, output, functors),
                actual: with_ctls,
                span,
            })
        }
        _ => Err(ClassError(Class::Ctl { op, with_ctls }, span)),
    }
}

fn check_eq(ty: Ty, span: Span) -> Result<Vec<Constraint>, ClassError> {
    match ty {
        Ty::Prim(
            TyPrim::BigInt
            | TyPrim::Bool
            | TyPrim::Double
            | TyPrim::Int
            | TyPrim::Qubit
            | TyPrim::Range
            | TyPrim::Result
            | TyPrim::String
            | TyPrim::Pauli,
        ) => Ok(Vec::new()),
        Ty::Array(item) => Ok(vec![Constraint::Class(Class::Eq(*item), span)]),
        Ty::Tuple(items) => Ok(items
            .into_iter()
            .map(|item| Constraint::Class(Class::Eq(item), span))
            .collect()),
        _ => Err(ClassError(Class::Eq(ty), span)),
    }
}

fn check_exp(base: Ty, power: Ty, span: Span) -> Result<Constraint, ClassError> {
    match base {
        Ty::Prim(TyPrim::BigInt) => Ok(Constraint::Eq {
            expected: Ty::Prim(TyPrim::Int),
            actual: power,
            span,
        }),
        Ty::Prim(TyPrim::Double | TyPrim::Int) => Ok(Constraint::Eq {
            expected: base,
            actual: power,
            span,
        }),
        _ => Err(ClassError(Class::Exp { base, power }, span)),
    }
}

fn check_has_index(
    container: Ty,
    index: Ty,
    item: Ty,
    span: Span,
) -> Result<Constraint, ClassError> {
    match (container, index) {
        (Ty::Array(container_item), Ty::Prim(TyPrim::Int)) => Ok(Constraint::Eq {
            expected: *container_item,
            actual: item,
            span,
        }),
        (container @ Ty::Array(_), Ty::Prim(TyPrim::Range)) => Ok(Constraint::Eq {
            expected: container,
            actual: item,
            span,
        }),
        (container, index) => Err(ClassError(
            Class::HasIndex {
                container,
                index,
                item,
            },
            span,
        )),
    }
}

fn check_integral(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(TyPrim::BigInt | TyPrim::Int))
}

fn check_iterable(container: Ty, item: Ty, span: Span) -> Result<Constraint, ClassError> {
    match container {
        Ty::Prim(TyPrim::Range) => Ok(Constraint::Eq {
            expected: Ty::Prim(TyPrim::Int),
            actual: item,
            span,
        }),
        Ty::Array(container_item) => Ok(Constraint::Eq {
            expected: *container_item,
            actual: item,
            span,
        }),
        _ => Err(ClassError(Class::Iterable { container, item }, span)),
    }
}

fn check_num(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(TyPrim::BigInt | TyPrim::Double | TyPrim::Int))
}
