// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Error, ErrorKind};
use crate::resolve::{DefId, PackageSrc};
use qsc_ast::ast::{CallableKind, Functor, Span, TyPrim};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
    mem,
};

pub(super) type Substitutions = HashMap<Var, Ty>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Var(u32);

impl Display for Var {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "?{}", self.0)
    }
}

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

#[derive(Clone, Debug)]
pub(super) enum Class {
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

struct Constraint {
    span: Span,
    kind: ConstraintKind,
}

enum ConstraintKind {
    Class(Class),
    Eq { expected: Ty, actual: Ty },
}

pub(super) struct Solver {
    constraints: Vec<Constraint>,
    next_var: u32,
}

impl Solver {
    pub(super) fn new() -> Self {
        Self {
            constraints: Vec::new(),
            next_var: 0,
        }
    }

    pub(super) fn eq(&mut self, span: Span, expected: Ty, actual: Ty) {
        let kind = ConstraintKind::Eq { expected, actual };
        self.constraints.push(Constraint { span, kind });
    }

    pub(super) fn class(&mut self, span: Span, class: Class) {
        let kind = ConstraintKind::Class(class);
        self.constraints.push(Constraint { span, kind });
    }

    pub(super) fn fresh(&mut self) -> Ty {
        let var = self.next_var;
        self.next_var += 1;
        Ty::Var(Var(var))
    }

    pub(super) fn instantiate(&mut self, ty: &Ty) -> Ty {
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

    pub(super) fn solve(self) -> (Substitutions, Vec<Error>) {
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
                                    errors.push(Error(ErrorKind::MissingClass(error.0, error.1)));
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
                                errors.push(Error(ErrorKind::TypeMismatch(
                                    ty1,
                                    ty2,
                                    constraint.span,
                                )));
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

        (substs, errors)
    }
}

struct ClassError(Class, Span);

struct UnifyError(Ty, Ty);

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

pub(super) fn substitute(substs: &Substitutions, ty: Ty) -> Ty {
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

fn try_var(ty: &Ty) -> Option<Var> {
    match ty {
        &Ty::Var(var) => Some(var),
        _ => None,
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
