// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Error, ErrorKind};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::hir::{
    Functor, FunctorSet, InferFunctor, InferTy, ItemId, PrimField, PrimTy, Res, Ty, Udt,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::{self, Debug, Display, Formatter},
};

pub(super) struct Solution {
    tys: IndexMap<InferTy, Ty>,
    functors: IndexMap<InferFunctor, FunctorSet>,
}

#[derive(Clone, Debug)]
pub(super) enum Class {
    Add(Ty),
    Adj(Ty),
    Call {
        callee: Ty,
        input: ArgTy,
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
    HasIndex {
        container: Ty,
        index: Ty,
        item: Ty,
    },
    Integral(Ty),
    Iterable {
        container: Ty,
        item: Ty,
    },
    Num(Ty),
    Show(Ty),
    Unwrap {
        wrapper: Ty,
        base: Ty,
    },
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
                input: input.map(&mut f),
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

    fn check(self, udts: &HashMap<ItemId, Udt>, span: Span) -> (Vec<Constraint>, Vec<Error>) {
        match self {
            Class::Add(ty) if check_add(&ty) => (Vec::new(), Vec::new()),
            Class::Adj(ty) => check_adj(ty, span),
            Class::Call {
                callee,
                input,
                output,
            } => check_call(callee, input, output, span),
            Class::Ctl { op, with_ctls } => check_ctl(op, with_ctls, span),
            Class::Eq(ty) => check_eq(ty, span),
            Class::Exp { base, power } => check_exp(base, power, span),
            Class::HasField { record, name, item } => {
                check_has_field(udts, record, name, item, span)
            }
            Class::HasIndex {
                container,
                index,
                item,
            } => check_has_index(container, index, item, span),
            Class::Integral(ty) if check_integral(&ty) => (Vec::new(), Vec::new()),
            Class::Iterable { container, item } => check_iterable(container, item, span),
            Class::Num(ty) if check_num(&ty) => (Vec::new(), Vec::new()),
            Class::Show(ty) => check_show(ty, span),
            Class::Unwrap { wrapper, base } => check_unwrap(udts, wrapper, base, span),
            Class::Add(_) | Class::Integral(_) | Class::Num(_) => {
                (Vec::new(), vec![Error(ErrorKind::MissingClass(self, span))])
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

#[derive(Clone, Debug)]
pub(super) enum ArgTy {
    Missing,
    Present(Ty),
    Tuple(Vec<ArgTy>),
}

impl ArgTy {
    fn map(self, f: &mut impl FnMut(Ty) -> Ty) -> Self {
        match self {
            Self::Missing => Self::Missing,
            Self::Present(ty) => Self::Present(f(ty)),
            Self::Tuple(items) => Self::Tuple(items.into_iter().map(|i| i.map(f)).collect()),
        }
    }

    fn apply(&self, param: &Ty, span: Span) -> ArgAp {
        match (self, param) {
            (Self::Missing, _) => ArgAp {
                constraints: Vec::new(),
                errors: Vec::new(),
                missing: vec![param.clone()],
            },
            (Self::Present(arg_ty), param_ty) => ArgAp {
                constraints: vec![Constraint::Eq {
                    expected: param_ty.clone(),
                    actual: arg_ty.clone(),
                    span,
                }],
                errors: Vec::new(),
                missing: Vec::new(),
            },
            (Self::Tuple(args), Ty::Tuple(params)) => {
                let mut constraints = Vec::new();
                let mut errors = Vec::new();
                if args.len() != params.len() {
                    errors.push(Error(ErrorKind::Mismatch(
                        Ty::Tuple(params.clone()),
                        self.clone().into_ty(),
                        span,
                    )));
                }

                let mut missing = Vec::new();
                for (arg, param) in args.iter().zip(params) {
                    let mut ap = arg.apply(param, span);
                    constraints.append(&mut ap.constraints);
                    errors.append(&mut ap.errors);
                    if ap.missing.len() > 1 {
                        missing.push(Ty::Tuple(ap.missing));
                    } else {
                        missing.append(&mut ap.missing);
                    }
                }

                ArgAp {
                    constraints,
                    errors,
                    missing,
                }
            }
            (Self::Tuple(_), _) => ArgAp {
                constraints: Vec::new(),
                errors: vec![Error(ErrorKind::Mismatch(
                    param.clone(),
                    self.clone().into_ty(),
                    span,
                ))],
                missing: Vec::new(),
            },
        }
    }

    pub(super) fn into_ty(self) -> Ty {
        match self {
            ArgTy::Missing => Ty::Err,
            ArgTy::Present(ty) => ty,
            ArgTy::Tuple(items) => Ty::Tuple(items.into_iter().map(Self::into_ty).collect()),
        }
    }
}

struct ArgAp {
    constraints: Vec<Constraint>,
    errors: Vec<Error>,
    missing: Vec<Ty>,
}

enum Constraint {
    Class(Class, Span),
    Eq {
        expected: Ty,
        actual: Ty,
        span: Span,
    },
    Functor(Functor, FunctorSet, Span),
}

pub(super) struct Inferrer {
    constraints: VecDeque<Constraint>,
    next_ty: InferTy,
    next_functor: InferFunctor,
}

impl Inferrer {
    pub(super) fn new() -> Self {
        Self {
            constraints: VecDeque::new(),
            next_ty: InferTy::default(),
            next_functor: InferFunctor::default(),
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

    /// Returns a unique unconstrained functor variable.
    pub(super) fn fresh_functor(&mut self) -> FunctorSet {
        let fresh = self.next_functor;
        self.next_functor = fresh.successor();
        FunctorSet::Infer(fresh)
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

    /// Solves for all variables given the accumulated constraints.
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
    pending_functors: HashMap<InferFunctor, HashSet<Functor>>,
    errors: Vec<Error>,
}

impl<'a> Solver<'a> {
    fn new(udts: &'a HashMap<ItemId, Udt>) -> Self {
        Self {
            udts,
            solution: Solution {
                tys: IndexMap::new(),
                functors: IndexMap::new(),
            },
            pending_tys: HashMap::new(),
            pending_functors: HashMap::new(),
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
            Constraint::Functor(functor, functors, span) => {
                self.functor(functor, functors, span);
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
            return Vec::new();
        }

        let (constraints, mut errors) = class
            .map(|mut ty| {
                substitute_ty(&self.solution, &mut ty);
                ty
            })
            .check(self.udts, span);

        self.errors.append(&mut errors);
        constraints
    }

    fn eq(&mut self, mut expected: Ty, mut actual: Ty, span: Span) -> Vec<Constraint> {
        substitute_ty(&self.solution, &mut expected);
        substitute_ty(&self.solution, &mut actual);
        self.unify(&expected, &actual, span)
    }

    fn functor(&mut self, functor: Functor, mut functors: FunctorSet, span: Span) {
        substitute_functor(&self.solution, &mut functors);
        match (functor, functors) {
            (_, FunctorSet::CtlAdj)
            | (Functor::Adj, FunctorSet::Adj)
            | (Functor::Ctl, FunctorSet::Ctl) => {}
            (_, FunctorSet::Infer(infer)) => {
                self.pending_functors
                    .entry(infer)
                    .or_default()
                    .insert(functor);
            }
            _ => self
                .errors
                .push(Error(ErrorKind::MissingFunctor(functor, functors, span))),
        }
    }

    fn unify(&mut self, ty1: &Ty, ty2: &Ty, span: Span) -> Vec<Constraint> {
        match (ty1, ty2) {
            (Ty::Err, _)
            | (_, Ty::Err)
            | (Ty::Udt(Res::Err), Ty::Udt(_))
            | (Ty::Udt(_), Ty::Udt(Res::Err)) => Vec::new(),
            (Ty::Array(item1), Ty::Array(item2)) => self.unify(item1, item2, span),
            (
                Ty::Arrow(kind1, input1, output1, functors1),
                Ty::Arrow(kind2, input2, output2, functors2),
            ) => {
                if kind1 != kind2 {
                    self.errors
                        .push(Error(ErrorKind::Mismatch(ty1.clone(), ty2.clone(), span)));
                }

                let mut constraints = self.unify(input1, input2, span);
                constraints.append(&mut self.unify(output1, output2, span));

                match (functors1, functors2) {
                    (FunctorSet::Infer(infer1), FunctorSet::Infer(infer2)) if infer1 == infer2 => {}
                    (&FunctorSet::Infer(infer), &functors)
                    | (&functors, &FunctorSet::Infer(infer)) => {
                        constraints.append(&mut self.bind_functor(infer, functors, span));
                    }
                    _ => {
                        // TODO: We ignore incompatible functors for now, even though this is
                        // unsound. This should be fixed later.
                        // https://github.com/microsoft/qsharp/issues/150
                    }
                }

                constraints
            }
            (Ty::Infer(infer1), Ty::Infer(infer2)) if infer1 == infer2 => Vec::new(),
            (&Ty::Infer(infer), ty) | (ty, &Ty::Infer(infer)) if !contains_infer_ty(infer, ty) => {
                self.bind_ty(infer, ty.clone(), span)
            }
            (Ty::Param(name1), Ty::Param(name2)) if name1 == name2 => Vec::new(),
            (Ty::Prim(prim1), Ty::Prim(prim2)) if prim1 == prim2 => Vec::new(),
            (Ty::Tuple(items1), Ty::Tuple(items2)) => {
                if items1.len() != items2.len() {
                    self.errors
                        .push(Error(ErrorKind::Mismatch(ty1.clone(), ty2.clone(), span)));
                }

                items1
                    .iter()
                    .zip(items2)
                    .flat_map(|(item1, item2)| self.unify(item1, item2, span))
                    .collect()
            }
            (Ty::Udt(res1), Ty::Udt(res2)) if res1 == res2 => Vec::new(),
            _ => {
                self.errors
                    .push(Error(ErrorKind::Mismatch(ty1.clone(), ty2.clone(), span)));
                Vec::new()
            }
        }
    }

    fn bind_ty(&mut self, infer: InferTy, ty: Ty, span: Span) -> Vec<Constraint> {
        self.solution.tys.insert(infer, ty);
        self.pending_tys
            .remove(&infer)
            .map_or(Vec::new(), |pending| {
                pending
                    .into_iter()
                    .map(|class| Constraint::Class(class, span))
                    .collect()
            })
    }

    fn bind_functor(
        &mut self,
        infer: InferFunctor,
        functors: FunctorSet,
        span: Span,
    ) -> Vec<Constraint> {
        self.solution.functors.insert(infer, functors);
        self.pending_functors
            .remove(&infer)
            .map_or(Vec::new(), |pending| {
                pending
                    .into_iter()
                    .map(|functor| Constraint::Functor(functor, functors, span))
                    .collect()
            })
    }

    fn into_solution(self) -> (Solution, Vec<Error>) {
        (self.solution, self.errors)
    }
}

pub(super) fn substitute_ty(solution: &Solution, ty: &mut Ty) {
    match ty {
        Ty::Err | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) => {}
        Ty::Array(item) => substitute_ty(solution, item),
        Ty::Arrow(_, input, output, functors) => {
            substitute_ty(solution, input);
            substitute_ty(solution, output);
            substitute_functor(solution, functors);
        }
        Ty::Tuple(items) => {
            for item in items {
                substitute_ty(solution, item);
            }
        }
        &mut Ty::Infer(infer) => {
            if let Some(new_ty) = solution.tys.get(infer) {
                *ty = new_ty.clone();
                substitute_ty(solution, ty);
            }
        }
    }
}

fn substitute_functor(solution: &Solution, functors: &mut FunctorSet) {
    if let &mut FunctorSet::Infer(infer) = functors {
        if let Some(&new_functors) = solution.functors.get(infer) {
            *functors = new_functors;
        }
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

fn check_adj(ty: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match ty {
        Ty::Arrow(_, _, _, functors) => (
            vec![Constraint::Functor(Functor::Adj, functors, span)],
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(Class::Adj(ty), span))],
        ),
    }
}

fn check_call(callee: Ty, input: ArgTy, output: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    let Ty::Arrow(kind, callee_input, callee_output, functors) = callee else {
        return (Vec::new(), vec![Error(ErrorKind::MissingClass(
            Class::Call {
                callee,
                input,
                output,
            },
            span,
        ))]);
    };

    let mut ap = input.apply(&callee_input, span);
    if ap.missing.is_empty() {
        ap.constraints.push(Constraint::Eq {
            expected: *callee_output,
            actual: output,
            span,
        });
    } else if ap.missing.len() > 1 {
        ap.constraints.push(Constraint::Eq {
            expected: Ty::Arrow(
                kind,
                Box::new(Ty::Tuple(ap.missing)),
                callee_output,
                functors,
            ),
            actual: output,
            span,
        });
    } else if let Some(missing) = ap.missing.pop() {
        ap.constraints.push(Constraint::Eq {
            expected: Ty::Arrow(kind, Box::new(missing), callee_output, functors),
            actual: output,
            span,
        });
    }

    (ap.constraints, ap.errors)
}

fn check_ctl(op: Ty, with_ctls: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match op {
        Ty::Arrow(kind, input, output, functors) => {
            let qubit_array = Ty::Array(Box::new(Ty::Prim(PrimTy::Qubit)));
            let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *input]));
            (
                vec![
                    Constraint::Functor(Functor::Ctl, functors, span),
                    Constraint::Eq {
                        expected: Ty::Arrow(kind, ctl_input, output, functors),
                        actual: with_ctls,
                        span,
                    },
                ],
                Vec::new(),
            )
        }
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(
                Class::Ctl { op, with_ctls },
                span,
            ))],
        ),
    }
}

fn check_eq(ty: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
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
        ) => (Vec::new(), Vec::new()),
        Ty::Array(item) => (vec![Constraint::Class(Class::Eq(*item), span)], Vec::new()),
        Ty::Tuple(items) => (
            items
                .into_iter()
                .map(|item| Constraint::Class(Class::Eq(item), span))
                .collect(),
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(Class::Eq(ty), span))],
        ),
    }
}

fn check_exp(base: Ty, power: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match base {
        Ty::Prim(PrimTy::BigInt) => (
            vec![Constraint::Eq {
                expected: Ty::Prim(PrimTy::Int),
                actual: power,
                span,
            }],
            Vec::new(),
        ),
        Ty::Prim(PrimTy::Double | PrimTy::Int) => (
            vec![Constraint::Eq {
                expected: base,
                actual: power,
                span,
            }],
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(
                Class::Exp { base, power },
                span,
            ))],
        ),
    }
}

fn check_has_field(
    udts: &HashMap<ItemId, Udt>,
    record: Ty,
    name: String,
    item: Ty,
    span: Span,
) -> (Vec<Constraint>, Vec<Error>) {
    match (name.parse(), &record) {
        (Ok(PrimField::Start), Ty::Prim(PrimTy::Range | PrimTy::RangeFrom))
        | (
            Ok(PrimField::Step),
            Ty::Prim(PrimTy::Range | PrimTy::RangeFrom | PrimTy::RangeTo | PrimTy::RangeFull),
        )
        | (Ok(PrimField::End), Ty::Prim(PrimTy::Range | PrimTy::RangeTo)) => (
            vec![Constraint::Eq {
                expected: item,
                actual: Ty::Prim(PrimTy::Int),
                span,
            }],
            Vec::new(),
        ),
        (Err(()), Ty::Udt(Res::Item(id))) => {
            match udts.get(id).and_then(|udt| udt.field_ty_by_name(&name)) {
                Some(ty) => (
                    vec![Constraint::Eq {
                        expected: item,
                        actual: ty.clone(),
                        span,
                    }],
                    Vec::new(),
                ),
                None => (
                    Vec::new(),
                    vec![Error(ErrorKind::MissingClass(
                        Class::HasField { record, name, item },
                        span,
                    ))],
                ),
            }
        }
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(
                Class::HasField { record, name, item },
                span,
            ))],
        ),
    }
}

fn check_has_index(
    container: Ty,
    index: Ty,
    item: Ty,
    span: Span,
) -> (Vec<Constraint>, Vec<Error>) {
    match (container, index) {
        (Ty::Array(container_item), Ty::Prim(PrimTy::Int)) => (
            vec![Constraint::Eq {
                expected: *container_item,
                actual: item,
                span,
            }],
            Vec::new(),
        ),
        (
            container @ Ty::Array(_),
            Ty::Prim(PrimTy::Range | PrimTy::RangeFrom | PrimTy::RangeTo | PrimTy::RangeFull),
        ) => (
            vec![Constraint::Eq {
                expected: container,
                actual: item,
                span,
            }],
            Vec::new(),
        ),
        (container, index) => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(
                Class::HasIndex {
                    container,
                    index,
                    item,
                },
                span,
            ))],
        ),
    }
}

fn check_integral(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(PrimTy::BigInt | PrimTy::Int))
}

fn check_iterable(container: Ty, item: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match container {
        Ty::Prim(PrimTy::Range) => (
            vec![Constraint::Eq {
                expected: Ty::Prim(PrimTy::Int),
                actual: item,
                span,
            }],
            Vec::new(),
        ),
        Ty::Array(container_item) => (
            vec![Constraint::Eq {
                expected: *container_item,
                actual: item,
                span,
            }],
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(
                Class::Iterable { container, item },
                span,
            ))],
        ),
    }
}

fn check_num(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(PrimTy::BigInt | PrimTy::Double | PrimTy::Int))
}

fn check_show(ty: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match ty {
        Ty::Array(item) => (
            vec![Constraint::Class(Class::Show(*item), span)],
            Vec::new(),
        ),
        Ty::Prim(_) => (Vec::new(), Vec::new()),
        Ty::Tuple(items) => (
            items
                .into_iter()
                .map(|item| Constraint::Class(Class::Show(item), span))
                .collect(),
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClass(Class::Show(ty), span))],
        ),
    }
}

fn check_unwrap(
    udts: &HashMap<ItemId, Udt>,
    wrapper: Ty,
    base: Ty,
    span: Span,
) -> (Vec<Constraint>, Vec<Error>) {
    if let Ty::Udt(Res::Item(id)) = wrapper {
        if let Some(udt) = udts.get(&id) {
            return (
                vec![Constraint::Eq {
                    expected: base,
                    actual: udt.base.clone(),
                    span,
                }],
                Vec::new(),
            );
        }
    }

    (
        Vec::new(),
        vec![Error(ErrorKind::MissingClass(
            Class::Unwrap { wrapper, base },
            span,
        ))],
    )
}
