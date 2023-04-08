// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use self::infer::Inferrer;
use crate::{
    compile::PackageId,
    resolve::{DefId, PackageSrc, Resolutions},
};
use miette::Diagnostic;
use qsc_ast::{
    ast::{
        self, CallableBody, CallableDecl, CallableKind, Functor, FunctorExpr, FunctorExprKind,
        NodeId, Package, Pat, PatKind, SetOp, Span, Spec, SpecBody, TyKind, TyPrim,
    },
    visit::Visitor,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
};
use thiserror::Error;

mod infer;
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

impl Ty {
    const UNIT: Self = Self::Tuple(Vec::new());
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
pub(super) enum Error {
    #[error("mismatched types")]
    TypeMismatch(Ty, Ty, #[label("expected {0}, found {1}")] Span),
    #[error("missing class instance")]
    MissingClass(Class, #[label("requires {0}")] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    MissingItemTy(#[label("explicit type required")] Span),
}

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
        let id = DefId {
            package: PackageSrc::Local,
            node: decl.name.id,
        };
        let ty = self.globals.get(&id).expect("callable should have type");
        let Ty::Arrow(_, _, _, functors) = ty else { panic!("callable should have arrow type") };

        match &decl.body {
            CallableBody::Block(block) => {
                let mut inferrer = Inferrer::new(self.resolutions, &self.globals);
                inferrer.infer_pat(&decl.input);
                let decl_output = inferrer.convert_ty(&decl.output);
                if !functors.is_empty() {
                    inferrer.constrain(
                        decl.output.span,
                        ConstraintKind::Eq {
                            expected: Ty::UNIT,
                            actual: decl_output.clone(),
                        },
                    );
                }

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
                            if !functors.is_empty() {
                                inferrer.constrain(
                                    decl.output.span,
                                    ConstraintKind::Eq {
                                        expected: Ty::UNIT,
                                        actual: decl_output.clone(),
                                    },
                                );
                            }

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
