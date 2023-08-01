// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{Error, ErrorKind};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    hir::{ItemId, PrimField, Res},
    ty::{
        Arrow, FunctorSet, FunctorSetValue, GenericArg, GenericParam, InferFunctorId, InferTyId,
        Prim, Scheme, Ty, Udt,
    },
};
use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    fmt::Debug,
};

#[derive(Debug, Default)]
struct Solution {
    tys: IndexMap<InferTyId, Ty>,
    functors: IndexMap<InferFunctorId, FunctorSet>,
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
            Class::Add(ty) => (
                Vec::new(),
                vec![Error(ErrorKind::MissingClassAdd(ty, span))],
            ),
            Class::Adj(ty) => check_adj(ty, span),
            Class::Call {
                callee,
                input,
                output,
            } => check_call(callee, &input, output, span),
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
            Class::Integral(ty) => (
                Vec::new(),
                vec![Error(ErrorKind::MissingClassInteger(ty, span))],
            ),
            Class::Iterable { container, item } => check_iterable(container, item, span),
            Class::Num(ty) if check_num(&ty) => (Vec::new(), Vec::new()),
            Class::Num(ty) => (
                Vec::new(),
                vec![Error(ErrorKind::MissingClassNum(ty, span))],
            ),
            Class::Show(ty) => check_show(ty, span),
            Class::Unwrap { wrapper, base } => check_unwrap(udts, wrapper, base, span),
        }
    }
}

/// Meta-level descriptions about the source of a type.
/// The compiler uses the notion of "unresolved types" to
/// represent both divergent types (return expressions, similar to
/// the `never` type), and types with insufficient information to
/// be inferred.
/// We want to generate compiler errors in the latter case,
/// so we need to track where types came from. This `TySource`
/// struct allows us to know if a type originates from a divergent
/// source, and if it doesn't, we generate an ambiguous type error.
pub(super) enum TySource {
    Divergent,
    NotDivergent { span: Span },
}

impl TySource {
    pub(super) fn not_divergent(span: Span) -> Self {
        TySource::NotDivergent { span }
    }

    pub(crate) fn divergent() -> TySource {
        TySource::Divergent
    }
}

/// An argument type and tags describing the call syntax.
#[derive(Clone, Debug)]
pub(super) enum ArgTy {
    /// A missing argument, indicating partial application.
    Hole(Ty),
    /// A given argument.
    Given(Ty),
    /// A list of arguments. This corresponds literally to tuple syntax, not to any expression of a tuple type.
    Tuple(Vec<ArgTy>),
}

impl ArgTy {
    fn map(self, f: &mut impl FnMut(Ty) -> Ty) -> Self {
        match self {
            Self::Hole(ty) => Self::Hole(f(ty)),
            Self::Given(ty) => Self::Given(f(ty)),
            Self::Tuple(items) => Self::Tuple(items.into_iter().map(|i| i.map(f)).collect()),
        }
    }

    fn apply(&self, param: &Ty, span: Span) -> App {
        match (self, param) {
            (Self::Hole(arg), _) => App {
                holes: vec![param.clone()],
                constraints: vec![Constraint::Eq {
                    expected: param.clone(),
                    actual: arg.clone(),
                    span,
                }],
                errors: Vec::new(),
            },
            (Self::Given(arg), _) => App {
                holes: Vec::new(),
                constraints: vec![Constraint::Eq {
                    expected: param.clone(),
                    actual: arg.clone(),
                    span,
                }],
                errors: Vec::new(),
            },
            (Self::Tuple(args), Ty::Tuple(params)) => {
                let mut errors = Vec::new();
                if args.len() != params.len() {
                    errors.push(Error(ErrorKind::TyMismatch(
                        Ty::Tuple(params.clone()),
                        self.to_ty(),
                        span,
                    )));
                }

                let mut holes = Vec::new();
                let mut constraints = Vec::new();
                for (arg, param) in args.iter().zip(params) {
                    let mut app = arg.apply(param, span);
                    constraints.append(&mut app.constraints);
                    errors.append(&mut app.errors);
                    if app.holes.len() > 1 {
                        holes.push(Ty::Tuple(app.holes));
                    } else {
                        holes.append(&mut app.holes);
                    }
                }

                App {
                    holes,
                    constraints,
                    errors,
                }
            }
            (Self::Tuple(_), _) => App {
                holes: Vec::new(),
                constraints: Vec::new(),
                errors: vec![Error(ErrorKind::TyMismatch(
                    param.clone(),
                    self.to_ty(),
                    span,
                ))],
            },
        }
    }

    pub(super) fn to_ty(&self) -> Ty {
        match self {
            ArgTy::Hole(ty) | ArgTy::Given(ty) => ty.clone(),
            ArgTy::Tuple(items) => Ty::Tuple(items.iter().map(Self::to_ty).collect()),
        }
    }
}

/// The result of applying an argument to a callable.
struct App {
    /// The types of all missing arguments in order and preserving their recursive tuple structure.
    holes: Vec<Ty>,
    /// The constraints implied by the call.
    constraints: Vec<Constraint>,
    /// The errors from the call.
    errors: Vec<Error>,
}

#[derive(Debug)]
enum Constraint {
    Class(Class, Span),
    Eq {
        expected: Ty,
        actual: Ty,
        span: Span,
    },
    Superset {
        expected: FunctorSetValue,
        actual: FunctorSet,
        span: Span,
    },
}

pub(super) struct Inferrer {
    solver: Solver,
    constraints: VecDeque<Constraint>,
    /// Metadata about the construction of types.
    ty_metadata: IndexMap<InferTyId, TySource>,
    next_ty: InferTyId,
    next_functor: InferFunctorId,
}

impl Inferrer {
    pub(super) fn new() -> Self {
        Self {
            solver: Solver::new(),
            constraints: VecDeque::new(),
            next_ty: InferTyId::default(),
            next_functor: InferFunctorId::default(),
            ty_metadata: IndexMap::default(),
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
    pub(super) fn fresh_ty(&mut self, meta: TySource) -> Ty {
        let fresh = self.next_ty;
        self.next_ty = fresh.successor();
        self.ty_metadata.insert(fresh, meta);
        Ty::Infer(fresh)
    }

    /// Returns a unique unconstrained functor variable.
    pub(super) fn fresh_functor(&mut self) -> FunctorSet {
        let fresh = self.next_functor;
        self.next_functor = fresh.successor();
        FunctorSet::Infer(fresh)
    }

    /// Instantiates the type scheme.
    pub(super) fn instantiate(&mut self, scheme: &Scheme, span: Span) -> (Arrow, Vec<GenericArg>) {
        let args: Vec<_> = scheme
            .params()
            .iter()
            .map(|param| match param {
                GenericParam::Ty => GenericArg::Ty(self.fresh_ty(TySource::not_divergent(span))),
                GenericParam::Functor(expected) => {
                    let actual = self.fresh_functor();
                    self.constraints.push_back(Constraint::Superset {
                        expected: *expected,
                        actual,
                        span,
                    });
                    GenericArg::Functor(actual)
                }
            })
            .collect();

        let ty = scheme
            .instantiate(&args)
            .expect("scheme should instantiate with fresh arguments");

        (ty, args)
    }

    /// Solves for all variables given the accumulated constraints.
    pub(super) fn solve(&mut self, udts: &HashMap<ItemId, Udt>) -> Vec<Error> {
        while let Some(constraint) = self.constraints.pop_front() {
            for constraint in self.solver.constrain(udts, constraint).into_iter().rev() {
                self.constraints.push_front(constraint);
            }
        }
        let unresolved_ty_errs = self.find_unresolved_types();
        self.solver.default_functors(self.next_functor);
        self.solver
            .errors
            .drain(..)
            .chain(unresolved_ty_errs.into_iter())
            .collect()
    }

    fn find_unresolved_types(&mut self) -> Vec<Error> {
        self.ty_metadata
            .drain()
            .filter_map(|(id, meta)| {
                if self.solver.solution.tys.get(id).is_none() {
                    match meta {
                        TySource::Divergent => {
                            // here, we are resolving all divergent types to the unit type.
                            self.solver.solution.tys.insert(id, Ty::UNIT);
                            None
                        }
                        TySource::NotDivergent { span } => {
                            Some(Error(ErrorKind::AmbiguousTy(span)))
                        }
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub(super) fn substitute_ty(&mut self, ty: &mut Ty) {
        substitute_ty(&self.solver.solution, ty);
    }

    pub(super) fn substitute_functor(&mut self, functors: &mut FunctorSet) {
        substitute_functor(&self.solver.solution, functors);
    }
}

#[derive(Debug)]
struct Solver {
    solution: Solution,
    pending_tys: HashMap<InferTyId, Vec<Class>>,
    pending_functors: HashMap<InferFunctorId, FunctorSetValue>,
    errors: Vec<Error>,
}

impl Solver {
    fn new() -> Self {
        Self {
            solution: Solution::default(),
            pending_tys: HashMap::new(),
            pending_functors: HashMap::new(),
            errors: Vec::new(),
        }
    }

    fn constrain(
        &mut self,
        udts: &HashMap<ItemId, Udt>,
        constraint: Constraint,
    ) -> Vec<Constraint> {
        match constraint {
            Constraint::Class(class, span) => self.class(udts, class, span),
            Constraint::Eq {
                expected,
                actual,
                span,
            } => self.eq(expected, actual, span),
            Constraint::Superset {
                expected,
                actual,
                span,
            } => {
                self.superset(expected, actual, span);
                Vec::new()
            }
        }
    }

    fn class(&mut self, udts: &HashMap<ItemId, Udt>, class: Class, span: Span) -> Vec<Constraint> {
        let unknown_dependency = class.dependencies().into_iter().any(|ty| {
            if ty == &Ty::Err {
                true
            } else if let Some(infer) = unknown_ty(&self.solution.tys, ty) {
                self.pending_tys
                    .entry(infer)
                    .or_default()
                    .push(class.clone());
                true
            } else {
                false
            }
        });

        if unknown_dependency {
            Vec::new()
        } else {
            let (constraints, mut errors) = class
                .map(|ty| substituted_ty(&self.solution, ty))
                .check(udts, span);
            self.errors.append(&mut errors);
            constraints
        }
    }

    fn eq(&mut self, mut expected: Ty, mut actual: Ty, span: Span) -> Vec<Constraint> {
        substitute_ty(&self.solution, &mut expected);
        substitute_ty(&self.solution, &mut actual);
        self.unify(&expected, &actual, span)
    }

    fn superset(&mut self, expected: FunctorSetValue, mut actual: FunctorSet, span: Span) {
        substitute_functor(&self.solution, &mut actual);
        match (expected, actual) {
            (_, FunctorSet::Value(FunctorSetValue::CtlAdj))
            | (FunctorSetValue::Empty, _)
            | (FunctorSetValue::Adj, FunctorSet::Value(FunctorSetValue::Adj))
            | (FunctorSetValue::Ctl, FunctorSet::Value(FunctorSetValue::Ctl)) => {}
            (expected, FunctorSet::Infer(infer)) => match self.pending_functors.entry(infer) {
                Entry::Occupied(mut entry) => {
                    let functors = entry.get_mut();
                    *functors = functors.union(&expected);
                }
                Entry::Vacant(entry) => {
                    entry.insert(expected);
                }
            },
            (expected, actual) => self.errors.push(Error(ErrorKind::MissingFunctor(
                FunctorSet::Value(expected),
                actual,
                span,
            ))),
        }
    }

    fn unify(&mut self, ty1: &Ty, ty2: &Ty, span: Span) -> Vec<Constraint> {
        match (ty1, ty2) {
            (Ty::Err, _)
            | (_, Ty::Err)
            | (Ty::Udt(Res::Err), Ty::Udt(_))
            | (Ty::Udt(_), Ty::Udt(Res::Err)) => Vec::new(),
            (Ty::Array(item1), Ty::Array(item2)) => self.unify(item1, item2, span),
            (Ty::Arrow(arrow1), Ty::Arrow(arrow2)) => {
                if arrow1.kind != arrow2.kind {
                    self.errors.push(Error(ErrorKind::CallableMismatch(
                        arrow1.kind,
                        arrow2.kind,
                        span,
                    )));
                }

                let mut constraints = self.unify(&arrow1.input, &arrow2.input, span);
                constraints.append(&mut self.unify(&arrow1.output, &arrow2.output, span));

                match (arrow1.functors, arrow2.functors) {
                    (FunctorSet::Value(value1), FunctorSet::Value(value2)) if value1 == value2 => {}
                    (FunctorSet::Infer(infer1), FunctorSet::Infer(infer2)) if infer1 == infer2 => {}
                    (FunctorSet::Infer(infer), functors) | (functors, FunctorSet::Infer(infer)) => {
                        constraints.append(&mut self.bind_functor(infer, functors, span));
                    }
                    _ => {
                        self.errors.push(Error(ErrorKind::FunctorMismatch(
                            arrow1.functors,
                            arrow2.functors,
                            span,
                        )));
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
                        .push(Error(ErrorKind::TyMismatch(ty1.clone(), ty2.clone(), span)));
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
                    .push(Error(ErrorKind::TyMismatch(ty1.clone(), ty2.clone(), span)));
                Vec::new()
            }
        }
    }

    fn bind_ty(&mut self, infer: InferTyId, ty: Ty, span: Span) -> Vec<Constraint> {
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
        infer: InferFunctorId,
        functors: FunctorSet,
        span: Span,
    ) -> Vec<Constraint> {
        self.solution.functors.insert(infer, functors);
        self.pending_functors
            .remove(&infer)
            .map_or(Vec::new(), |expected| {
                vec![Constraint::Superset {
                    expected,
                    actual: functors,
                    span,
                }]
            })
    }

    fn default_functors(&mut self, functor_end: InferFunctorId) {
        let mut functor = InferFunctorId::default();
        while functor < functor_end {
            if !self.solution.functors.contains_key(functor) {
                let value = self.pending_functors.remove(&functor).unwrap_or_default();
                self.solution
                    .functors
                    .insert(functor, FunctorSet::Value(value));
            }

            functor = functor.successor();
        }
    }
}

fn substitute_ty(solution: &Solution, ty: &mut Ty) {
    match ty {
        Ty::Err | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) => {}
        Ty::Array(item) => substitute_ty(solution, item),
        Ty::Arrow(arrow) => {
            substitute_ty(solution, &mut arrow.input);
            substitute_ty(solution, &mut arrow.output);
            substitute_functor(solution, &mut arrow.functors);
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

fn substituted_ty(solution: &Solution, mut ty: Ty) -> Ty {
    substitute_ty(solution, &mut ty);
    ty
}

fn substitute_functor(solution: &Solution, functors: &mut FunctorSet) {
    if let &mut FunctorSet::Infer(infer) = functors {
        if let Some(&new_functors) = solution.functors.get(infer) {
            *functors = new_functors;
            substitute_functor(solution, functors);
        }
    }
}

fn unknown_ty(tys: &IndexMap<InferTyId, Ty>, ty: &Ty) -> Option<InferTyId> {
    match ty {
        &Ty::Infer(infer) => match tys.get(infer) {
            None => Some(infer),
            Some(ty) => unknown_ty(tys, ty),
        },
        _ => None,
    }
}

fn contains_infer_ty(id: InferTyId, ty: &Ty) -> bool {
    match ty {
        Ty::Err | Ty::Param(_) | Ty::Prim(_) | Ty::Udt(_) => false,
        Ty::Array(item) => contains_infer_ty(id, item),
        Ty::Arrow(arrow) => {
            contains_infer_ty(id, &arrow.input) || contains_infer_ty(id, &arrow.output)
        }
        Ty::Infer(other_id) => id == *other_id,
        Ty::Tuple(items) => items.iter().any(|ty| contains_infer_ty(id, ty)),
    }
}

fn check_add(ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::Prim(Prim::BigInt | Prim::Double | Prim::Int | Prim::String) | Ty::Array(_)
    )
}

fn check_adj(ty: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match ty {
        Ty::Arrow(arrow) => (
            vec![Constraint::Superset {
                expected: FunctorSetValue::Adj,
                actual: arrow.functors,
                span,
            }],
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClassAdj(ty, span))],
        ),
    }
}

fn check_call(callee: Ty, input: &ArgTy, output: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    let Ty::Arrow(arrow) = callee else {
        return (Vec::new(), vec![Error(ErrorKind::MissingClassCall(
            callee,
            span,
        ))]);
    };

    let mut app = input.apply(&arrow.input, span);
    let expected = if app.holes.len() > 1 {
        Ty::Arrow(Box::new(Arrow {
            kind: arrow.kind,
            input: Box::new(Ty::Tuple(app.holes)),
            output: arrow.output,
            functors: arrow.functors,
        }))
    } else if let Some(hole) = app.holes.pop() {
        Ty::Arrow(Box::new(Arrow {
            kind: arrow.kind,
            input: Box::new(hole),
            output: arrow.output,
            functors: arrow.functors,
        }))
    } else {
        *arrow.output
    };

    app.constraints.push(Constraint::Eq {
        expected,
        actual: output,
        span,
    });
    (app.constraints, app.errors)
}

fn check_ctl(op: Ty, with_ctls: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    let Ty::Arrow(arrow) = op else {
        return (
            Vec::new(),
            vec![Error(ErrorKind::MissingClassCtl(
                op,
                span,
            ))],
        );
    };

    let qubit_array = Ty::Array(Box::new(Ty::Prim(Prim::Qubit)));
    let ctl_input = Box::new(Ty::Tuple(vec![qubit_array, *arrow.input]));
    (
        vec![
            Constraint::Superset {
                expected: FunctorSetValue::Ctl,
                actual: arrow.functors,
                span,
            },
            Constraint::Eq {
                expected: Ty::Arrow(Box::new(Arrow {
                    kind: arrow.kind,
                    input: ctl_input,
                    output: arrow.output,
                    functors: arrow.functors,
                })),
                actual: with_ctls,
                span,
            },
        ],
        Vec::new(),
    )
}

fn check_eq(ty: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match ty {
        Ty::Prim(
            Prim::BigInt
            | Prim::Bool
            | Prim::Double
            | Prim::Int
            | Prim::Qubit
            | Prim::Range
            | Prim::Result
            | Prim::String
            | Prim::Pauli,
        ) => (Vec::new(), Vec::new()),
        Ty::Array(item) => (vec![Constraint::Class(Class::Eq(*item), span)], Vec::new()),
        Ty::Tuple(items) => (
            items
                .into_iter()
                .map(|item| Constraint::Class(Class::Eq(item), span))
                .collect(),
            Vec::new(),
        ),
        _ => (Vec::new(), vec![Error(ErrorKind::MissingClassEq(ty, span))]),
    }
}

fn check_exp(base: Ty, power: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match base {
        Ty::Prim(Prim::BigInt) => (
            vec![Constraint::Eq {
                expected: Ty::Prim(Prim::Int),
                actual: power,
                span,
            }],
            Vec::new(),
        ),
        Ty::Prim(Prim::Double | Prim::Int) => (
            vec![Constraint::Eq {
                expected: base,
                actual: power,
                span,
            }],
            Vec::new(),
        ),
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClassExp(base, span))],
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
        (Ok(PrimField::Start), Ty::Prim(Prim::Range | Prim::RangeFrom))
        | (
            Ok(PrimField::Step),
            Ty::Prim(Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull),
        )
        | (Ok(PrimField::End), Ty::Prim(Prim::Range | Prim::RangeTo)) => (
            vec![Constraint::Eq {
                expected: item,
                actual: Ty::Prim(Prim::Int),
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
                    vec![Error(ErrorKind::MissingClassHasField(record, name, span))],
                ),
            }
        }
        _ => (
            Vec::new(),
            vec![Error(ErrorKind::MissingClassHasField(record, name, span))],
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
        (Ty::Array(container_item), Ty::Prim(Prim::Int)) => (
            vec![Constraint::Eq {
                expected: *container_item,
                actual: item,
                span,
            }],
            Vec::new(),
        ),
        (
            container @ Ty::Array(_),
            Ty::Prim(Prim::Range | Prim::RangeFrom | Prim::RangeTo | Prim::RangeFull),
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
            vec![Error(ErrorKind::MissingClassHasIndex(
                container, index, span,
            ))],
        ),
    }
}

fn check_integral(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(Prim::BigInt | Prim::Int))
}

fn check_iterable(container: Ty, item: Ty, span: Span) -> (Vec<Constraint>, Vec<Error>) {
    match container {
        Ty::Prim(Prim::Range) => (
            vec![Constraint::Eq {
                expected: Ty::Prim(Prim::Int),
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
            vec![Error(ErrorKind::MissingClassIterable(container, span))],
        ),
    }
}

fn check_num(ty: &Ty) -> bool {
    matches!(ty, Ty::Prim(Prim::BigInt | Prim::Double | Prim::Int))
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
            vec![Error(ErrorKind::MissingClassShow(ty, span))],
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
                    actual: udt.get_pure_ty(),
                    span,
                }],
                Vec::new(),
            );
        }
    }

    (
        Vec::new(),
        vec![Error(ErrorKind::MissingClassUnwrap(wrapper, span))],
    )
}
