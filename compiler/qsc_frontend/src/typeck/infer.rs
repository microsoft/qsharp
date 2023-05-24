// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Error, ErrorKind};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::hir::{Char, Functor, InferChar, InferTy, ItemId, PrimField, PrimTy, Res, Ty, Udt};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{self, Debug, Display, Formatter},
};

pub(super) struct Solution {
    tys: IndexMap<InferTy, Ty>,
    chars: IndexMap<InferChar, Char>,
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
    Show(Ty),
    Unwrap { wrapper: Ty, base: Ty },
}

impl Class {
    fn dependencies(&self) -> Vec<&Ty> {
        match self {
            Self::Add(ty)
            | Self::Adj(ty)
            | Self::Eq(ty)
            | Self::Integral(ty)
            | Self::Num(ty)
            | Self::Show(ty) => {
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
            Self::Show(ty) => Self::Show(f(ty)),
            Self::Unwrap { wrapper, base } => Self::Unwrap {
                wrapper: f(wrapper),
                base: f(base),
            },
        }
    }

    fn check(self, udts: &HashMap<ItemId, Udt>, span: Span) -> Result<Vec<Constraint>, ClassError> {
        match self {
            Class::Add(ty) => check_add(&ty)
                .then_some(Vec::new())
                .ok_or(ClassError(Class::Add(ty), span)),
            Class::Adj(ty) => check_adj(ty, span).map(|c| vec![c]),
            Class::Call {
                callee,
                input,
                output,
            } => check_call(callee, input, output, span),
            Class::Ctl { op, with_ctls } => check_ctl(op, with_ctls, span),
            Class::Eq(ty) => check_eq(ty, span),
            Class::Exp { base, power } => check_exp(base, power, span).map(|c| vec![c]),
            Class::HasField { record, name, item } => {
                check_has_field(udts, record, name, item, span).map(|c| vec![c])
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
            Class::Show(ty) => check_show(ty, span),
            Class::Unwrap { wrapper, base } => {
                check_unwrap(udts, wrapper, base, span).map(|c| vec![c])
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
            Class::Num(ty) => write!(f, "Num<{ty}>"),
            Class::Show(ty) => write!(f, "Show<{ty}>"),
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
    Char(Functor, Char, Span),
}

struct ClassError(Class, Span);

struct UnifyError(Ty, Ty);

pub(super) struct Inferrer {
    constraints: VecDeque<Constraint>,
    next_ty: InferTy,
    next_char: InferChar,
}

impl Inferrer {
    pub(super) fn new() -> Self {
        Self {
            constraints: VecDeque::new(),
            next_ty: InferTy::default(),
            next_char: InferChar::default(),
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
    pub(super) fn fresh_ty(&mut self) -> Ty {
        let fresh = self.next_ty;
        self.next_ty = fresh.successor();
        Ty::Infer(fresh)
    }

    /// Returns a unique unconstrained characteristic variable.
    pub(super) fn fresh_char(&mut self) -> Char {
        let fresh = self.next_char;
        self.next_char = fresh.successor();
        Char::Infer(fresh)
    }

    /// Replaces all type parameters with fresh types.
    pub(super) fn freshen(&mut self, ty: &mut Ty) {
        fn freshen(solver: &mut Inferrer, params: &mut HashMap<String, Ty>, ty: &mut Ty) {
            match ty {
                Ty::Err | Ty::Infer(_) | Ty::Prim(_) | Ty::Udt(_) => {}
                Ty::Array(item) => freshen(solver, params, item),
                Ty::Arrow(_, input, output, _) => {
                    freshen(solver, params, input);
                    freshen(solver, params, output);
                }
                Ty::Param(name) => {
                    *ty = params
                        .entry(name.clone())
                        .or_insert_with(|| solver.fresh_ty())
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
    pub(super) fn solve(mut self, udts: &HashMap<ItemId, Udt>) -> (Solution, Vec<Error>) {
        // TODO: Variables that don't have a substitution should cause errors for ambiguous types.
        // However, if an unsolved variable is the result of a divergent expression, it may be OK to
        // leave it or substitute it with a concrete uninhabited type.
        // https://github.com/microsoft/qsharp/issues/152
        let mut solver = Solver::new(udts);
        while let Some(constraint) = self.constraints.pop_front() {
            self.constraints.extend(solver.constrain(constraint));
        }
        solver.into_solution()
    }
}

struct Solver<'a> {
    udts: &'a HashMap<ItemId, Udt>,
    solution: Solution,
    pending_tys: HashMap<InferTy, Vec<Class>>,
    pending_chars: HashMap<InferChar, HashSet<Functor>>,
    errors: Vec<Error>,
}

impl<'a> Solver<'a> {
    fn new(udts: &'a HashMap<ItemId, Udt>) -> Self {
        Self {
            udts,
            solution: Solution {
                tys: IndexMap::new(),
                chars: IndexMap::new(),
            },
            pending_tys: HashMap::new(),
            pending_chars: HashMap::new(),
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
            Constraint::Char(functor, characteristic, span) => {
                self.char(functor, characteristic, span);
                Vec::new()
            }
        }
    }

    fn class(&mut self, class: Class, span: Span) -> Vec<Constraint> {
        let mut unknown_dependency = false;
        for ty in class.dependencies() {
            if ty == &Ty::Err {
                unknown_dependency = true;
            } else if let Some(infer) = unknown_ty(&self.solution.tys, ty) {
                self.pending_tys
                    .entry(infer)
                    .or_default()
                    .push(class.clone());
                unknown_dependency = true;
            }
        }

        if unknown_dependency {
            Vec::new()
        } else {
            match class
                .map(|ty| substituted(&self.solution, ty))
                .check(self.udts, span)
            {
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
        substitute(&self.solution, &mut expected);
        substitute(&self.solution, &mut actual);

        let mut bind_ty = |infer, ty| {
            let constraints = if let Some(classes) = self.pending_tys.remove(&infer) {
                classes
                    .into_iter()
                    .map(|class| Constraint::Class(class, span))
                    .collect()
            } else {
                Vec::new()
            };
            self.solution.tys.insert(infer, ty);
            constraints
        };

        let mut bind_char = |infer, char: Char| {
            let constraints = if let Some(functors) = self.pending_chars.remove(&infer) {
                functors
                    .into_iter()
                    .map(|functor| Constraint::Char(functor, char.clone(), span))
                    .collect()
            } else {
                Vec::new()
            };
            self.solution.chars.insert(infer, char);
            constraints
        };

        match unify(&mut bind_ty, &mut bind_char, &expected, &actual) {
            Ok(constraints) => constraints,
            Err(UnifyError(expected, actual)) => {
                self.errors
                    .push(Error(ErrorKind::TypeMismatch(expected, actual, span)));
                Vec::new()
            }
        }
    }

    fn char(&mut self, functor: Functor, char: Char, span: Span) {
        match char {
            Char::Set(functors) if functors.contains(&functor) => {}
            Char::Set(_) => self
                .errors
                .push(Error(ErrorKind::MissingFunctor(functor, char, span))),
            Char::Infer(infer) => {
                self.pending_chars.entry(infer).or_default().insert(functor);
            }
        }
    }

    fn into_solution(self) -> (Solution, Vec<Error>) {
        (self.solution, self.errors)
    }
}

pub(super) fn substitute(solution: &Solution, ty: &mut Ty) {
    match ty {
        Ty::Err | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) => {}
        Ty::Array(item) => substitute(solution, item),
        Ty::Arrow(_, input, output, char) => {
            substitute(solution, input);
            substitute(solution, output);
            if let &mut Char::Infer(infer) = char {
                if let Some(new_char) = solution.chars.get(infer) {
                    *char = new_char.clone();
                }
            }
        }
        Ty::Tuple(items) => {
            for item in items {
                substitute(solution, item);
            }
        }
        &mut Ty::Infer(infer) => {
            if let Some(new_ty) = solution.tys.get(infer) {
                *ty = new_ty.clone();
                substitute(solution, ty);
            }
        }
    }
}

fn substituted(solution: &Solution, mut ty: Ty) -> Ty {
    substitute(solution, &mut ty);
    ty
}

fn unify(
    bind_ty: &mut impl FnMut(InferTy, Ty) -> Vec<Constraint>,
    bind_char: &mut impl FnMut(InferChar, Char) -> Vec<Constraint>,
    ty1: &Ty,
    ty2: &Ty,
) -> Result<Vec<Constraint>, UnifyError> {
    match (ty1, ty2) {
        (Ty::Err, _)
        | (_, Ty::Err)
        | (Ty::Udt(Res::Err), Ty::Udt(_))
        | (Ty::Udt(_), Ty::Udt(Res::Err)) => Ok(Vec::new()),
        (Ty::Array(item1), Ty::Array(item2)) => unify(bind_ty, bind_char, item1, item2),
        (Ty::Arrow(kind1, input1, output1, char1), Ty::Arrow(kind2, input2, output2, char2))
            if kind1 == kind2 =>
        {
            let mut constraints = unify(bind_ty, bind_char, input1, input2)?;
            constraints.append(&mut unify(bind_ty, bind_char, output1, output2)?);

            match (char1, char2) {
                (Char::Infer(infer1), Char::Infer(infer2)) if infer1 == infer2 => {}
                (&Char::Infer(infer), char) | (char, &Char::Infer(infer)) => {
                    constraints.append(&mut bind_char(infer, char.clone()));
                }
                _ => {
                    // TODO: We ignore incompatible functors until subtyping is supported, even
                    // though this is unsound.
                    // https://github.com/microsoft/qsharp/issues/150
                }
            }

            Ok(constraints)
        }
        (Ty::Infer(infer1), Ty::Infer(infer2)) if infer1 == infer2 => Ok(Vec::new()),
        (&Ty::Infer(infer), ty) | (ty, &Ty::Infer(infer)) if !contains_infer_ty(infer, ty) => {
            Ok(bind_ty(infer, ty.clone()))
        }
        (Ty::Param(name1), Ty::Param(name2)) if name1 == name2 => Ok(Vec::new()),
        (Ty::Prim(prim1), Ty::Prim(prim2)) if prim1 == prim2 => Ok(Vec::new()),
        (Ty::Tuple(items1), Ty::Tuple(items2)) if items1.len() == items2.len() => {
            let mut constraints = Vec::new();
            for (item1, item2) in items1.iter().zip(items2) {
                constraints.append(&mut unify(bind_ty, bind_char, item1, item2)?);
            }
            Ok(constraints)
        }
        (Ty::Udt(res1), Ty::Udt(res2)) if res1 == res2 => Ok(Vec::new()),
        _ => Err(UnifyError(ty1.clone(), ty2.clone())),
    }
}

fn unknown_ty(tys: &IndexMap<InferTy, Ty>, ty: &Ty) -> Option<InferTy> {
    match ty {
        &Ty::Infer(infer) => match tys.get(infer) {
            None => Some(infer),
            Some(ty) => unknown_ty(tys, ty),
        },
        _ => None,
    }
}

fn contains_infer_ty(id: InferTy, ty: &Ty) -> bool {
    match ty {
        Ty::Err | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) => false,
        Ty::Array(item) => contains_infer_ty(id, item),
        Ty::Arrow(_, input, output, _) => {
            contains_infer_ty(id, input) || contains_infer_ty(id, output)
        }
        Ty::Infer(other_id) => id == *other_id,
        Ty::Tuple(items) => items.iter().any(|ty| contains_infer_ty(id, ty)),
    }
}

fn check_add(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::Prim(PrimTy::BigInt | PrimTy::Double | PrimTy::Int | PrimTy::String) | Ty::Array(_)
    )
}

fn check_adj(ty: Ty, span: Span) -> Result<Constraint, ClassError> {
    match ty {
        Ty::Arrow(_, _, _, functors) => Ok(Constraint::Char(Functor::Adj, functors, span)),
        _ => Err(ClassError(Class::Adj(ty), span)),
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

fn check_ctl(op: Ty, with_ctls: Ty, span: Span) -> Result<Vec<Constraint>, ClassError> {
    match op {
        Ty::Arrow(kind, input, output, functors) => {
            let qubit_array = Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit)));
            let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *input]));
            Ok(vec![
                Constraint::Char(Functor::Ctl, functors.clone(), span),
                Constraint::Eq {
                    expected: Ty::Arrow(kind, ctl_input, output, functors),
                    actual: with_ctls,
                    span,
                },
            ])
        }
        _ => Err(ClassError(Class::Ctl { op, with_ctls }, span)),
    }
}

fn check_eq(ty: Ty, span: Span) -> Result<Vec<Constraint>, ClassError> {
    match ty {
        Ty::Prim(
            PrimTy::BigInt
            | PrimTy::Bool
            | PrimTy::Double
            | PrimTy::Int
            | PrimTy::Qubit
            | PrimTy::Range
            | PrimTy::Result
            | PrimTy::String
            | PrimTy::Pauli,
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
        Ty::Prim(PrimTy::BigInt) => Ok(Constraint::Eq {
            expected: Ty::Prim(PrimTy::Int),
            actual: power,
            span,
        }),
        Ty::Prim(PrimTy::Double | PrimTy::Int) => Ok(Constraint::Eq {
            expected: base,
            actual: power,
            span,
        }),
        _ => Err(ClassError(Class::Exp { base, power }, span)),
    }
}

fn check_has_field(
    udts: &HashMap<ItemId, Udt>,
    record: Ty,
    name: String,
    item: Ty,
    span: Span,
) -> Result<Constraint, ClassError> {
    match (name.parse(), &record) {
        (Ok(PrimField::Start), Ty::Prim(PrimTy::Range | PrimTy::RangeFrom))
        | (
            Ok(PrimField::Step),
            Ty::Prim(PrimTy::Range | PrimTy::RangeFrom | PrimTy::RangeTo | PrimTy::RangeFull),
        )
        | (Ok(PrimField::End), Ty::Prim(PrimTy::Range | PrimTy::RangeTo)) => Ok(Constraint::Eq {
            expected: item,
            actual: Ty::Prim(PrimTy::Int),
            span,
        }),
        (Err(()), Ty::Udt(Res::Item(id))) => {
            match udts.get(id).and_then(|udt| udt.field_ty_by_name(&name)) {
                Some(ty) => Ok(Constraint::Eq {
                    expected: item,
                    actual: ty.clone(),
                    span,
                }),
                None => Err(ClassError(Class::HasField { record, name, item }, span)),
            }
        }
        _ => Err(ClassError(Class::HasField { record, name, item }, span)),
    }
}

fn check_has_index(
    container: Ty,
    index: Ty,
    item: Ty,
    span: Span,
) -> Result<Constraint, ClassError> {
    match (container, index) {
        (Ty::Array(container_item), Ty::Prim(PrimTy::Int)) => Ok(Constraint::Eq {
            expected: *container_item,
            actual: item,
            span,
        }),
        (
            container @ Ty::Array(_),
            Ty::Prim(PrimTy::Range | PrimTy::RangeFrom | PrimTy::RangeTo | PrimTy::RangeFull),
        ) => Ok(Constraint::Eq {
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
    matches!(ty, Ty::Prim(PrimTy::BigInt | PrimTy::Int))
}

fn check_iterable(container: Ty, item: Ty, span: Span) -> Result<Constraint, ClassError> {
    match container {
        Ty::Prim(PrimTy::Range) => Ok(Constraint::Eq {
            expected: Ty::Prim(PrimTy::Int),
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
    matches!(ty, Ty::Prim(PrimTy::BigInt | PrimTy::Double | PrimTy::Int))
}

fn check_show(ty: Ty, span: Span) -> Result<Vec<Constraint>, ClassError> {
    match ty {
        Ty::Array(item) => Ok(vec![Constraint::Class(Class::Show(*item), span)]),
        Ty::Prim(_) => Ok(Vec::new()),
        Ty::Tuple(items) => Ok(items
            .into_iter()
            .map(|item| Constraint::Class(Class::Show(item), span))
            .collect()),
        _ => Err(ClassError(Class::Show(ty), span)),
    }
}

fn check_unwrap(
    udts: &HashMap<ItemId, Udt>,
    wrapper: Ty,
    base: Ty,
    span: Span,
) -> Result<Constraint, ClassError> {
    if let Ty::Udt(Res::Item(id)) = wrapper {
        if let Some(udt) = udts.get(&id) {
            return Ok(Constraint::Eq {
                expected: base,
                actual: udt.base.clone(),
                span,
            });
        }
    }

    Err(ClassError(Class::Unwrap { wrapper, base }, span))
}
