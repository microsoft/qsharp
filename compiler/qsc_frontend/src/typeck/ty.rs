// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast;
use qsc_data_structures::span::Span;
use qsc_hir::hir;
use std::{
    collections::HashSet,
    fmt::{self, Debug, Display, Formatter},
};

pub(super) struct MissingTyError(pub(super) Span);

#[derive(Clone, Debug)]
pub enum Ty {
    Array(Box<Ty>),
    Arrow(CallableKind, Box<Ty>, Box<Ty>, HashSet<Functor>),
    Err,
    Param(String),
    Prim(Prim),
    Tuple(Vec<Ty>),
    Var(Var),
}

impl Ty {
    pub(super) const UNIT: Self = Self::Tuple(Vec::new());

    pub(super) fn from_ast(ty: &ast::Ty) -> (Self, Vec<MissingTyError>) {
        match &ty.kind {
            ast::TyKind::Array(item) => {
                let (item, errors) = Self::from_ast(item);
                (Ty::Array(Box::new(item)), errors)
            }
            ast::TyKind::Arrow(kind, input, output, functors) => {
                let (input, mut errors) = Self::from_ast(input);
                let (output, output_errors) = Self::from_ast(output);
                errors.extend(output_errors);
                let functors = functors.as_ref().map_or(HashSet::new(), |f| {
                    f.to_set().into_iter().map(Into::into).collect()
                });
                let ty = Ty::Arrow(kind.into(), Box::new(input), Box::new(output), functors);
                (ty, errors)
            }
            ast::TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
            ast::TyKind::Paren(inner) => Self::from_ast(inner),
            ast::TyKind::Path(_) => (Ty::Err, Vec::new()), // TODO: Resolve user-defined types.
            &ast::TyKind::Prim(prim) => (Ty::Prim(prim.into()), Vec::new()),
            ast::TyKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut errors = Vec::new();
                for item in items {
                    let (item_ty, item_errors) = Self::from_ast(item);
                    tys.push(item_ty);
                    errors.extend(item_errors);
                }
                (Ty::Tuple(tys), errors)
            }
            ast::TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
        }
    }

    pub(super) fn from_hir(ty: &hir::Ty) -> (Ty, Vec<MissingTyError>) {
        match &ty.kind {
            hir::TyKind::Array(item) => {
                let (item, errors) = Self::from_hir(item);
                (Ty::Array(Box::new(item)), errors)
            }
            hir::TyKind::Arrow(kind, input, output, functors) => {
                let (input, mut errors) = Self::from_hir(input);
                let (output, output_errors) = Self::from_hir(output);
                errors.extend(output_errors);
                let functors = functors.as_ref().map_or(HashSet::new(), |f| {
                    f.to_set().into_iter().map(Into::into).collect()
                });
                let ty = Ty::Arrow(kind.into(), Box::new(input), Box::new(output), functors);
                (ty, errors)
            }
            hir::TyKind::Hole => (Ty::Err, vec![MissingTyError(ty.span)]),
            hir::TyKind::Paren(inner) => Self::from_hir(inner),
            hir::TyKind::Path(_) => (Ty::Err, Vec::new()), // TODO: Resolve user-defined types.
            &hir::TyKind::Prim(prim) => (Ty::Prim(prim.into()), Vec::new()),
            hir::TyKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut errors = Vec::new();
                for item in items {
                    let (item_ty, item_errors) = Self::from_hir(item);
                    tys.push(item_ty);
                    errors.extend(item_errors);
                }
                (Ty::Tuple(tys), errors)
            }
            hir::TyKind::Var(name) => (Ty::Param(name.name.clone()), Vec::new()),
        }
    }

    pub(super) fn of_ast_callable(decl: &ast::CallableDecl) -> (Ty, Vec<MissingTyError>) {
        let kind = decl.kind.into();
        let (input, mut errors) = Ty::of_ast_pat(&decl.input);
        let (output, output_errors) = Ty::from_ast(&decl.output);
        errors.extend(output_errors);
        let functors = ast_callable_functors(decl);
        let ty = Ty::Arrow(kind, Box::new(input), Box::new(output), functors);
        (ty, errors)
    }

    pub(super) fn of_hir_callable(decl: &hir::CallableDecl) -> (Ty, Vec<MissingTyError>) {
        let kind = decl.kind.into();
        let (input, mut errors) = Ty::of_hir_pat(&decl.input);
        let (output, output_errors) = Ty::from_hir(&decl.output);
        errors.extend(output_errors);
        let functors = hir_callable_functors(decl);
        let ty = Ty::Arrow(kind, Box::new(input), Box::new(output), functors);
        (ty, errors)
    }

    pub(super) fn of_ast_pat(pat: &ast::Pat) -> (Ty, Vec<MissingTyError>) {
        match &pat.kind {
            ast::PatKind::Bind(_, None) | ast::PatKind::Discard(None) | ast::PatKind::Elided => {
                (Ty::Err, vec![MissingTyError(pat.span)])
            }
            ast::PatKind::Bind(_, Some(ty)) | ast::PatKind::Discard(Some(ty)) => Ty::from_ast(ty),
            ast::PatKind::Paren(inner) => Self::of_ast_pat(inner),
            ast::PatKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut errors = Vec::new();
                for item in items {
                    let (item_ty, item_errors) = Self::of_ast_pat(item);
                    tys.push(item_ty);
                    errors.extend(item_errors);
                }
                (Ty::Tuple(tys), errors)
            }
        }
    }

    pub(super) fn of_hir_pat(pat: &hir::Pat) -> (Ty, Vec<MissingTyError>) {
        match &pat.kind {
            hir::PatKind::Bind(_, None) | hir::PatKind::Discard(None) | hir::PatKind::Elided => {
                (Ty::Err, vec![MissingTyError(pat.span)])
            }
            hir::PatKind::Bind(_, Some(ty)) | hir::PatKind::Discard(Some(ty)) => Ty::from_hir(ty),
            hir::PatKind::Paren(inner) => Self::of_hir_pat(inner),
            hir::PatKind::Tuple(items) => {
                let mut tys = Vec::new();
                let mut errors = Vec::new();
                for item in items {
                    let (item_ty, item_errors) = Self::of_hir_pat(item);
                    tys.push(item_ty);
                    errors.extend(item_errors);
                }
                (Ty::Tuple(tys), errors)
            }
        }
    }
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
                let is = match (
                    functors.contains(&Functor::Adj),
                    functors.contains(&Functor::Ctl),
                ) {
                    (true, true) => " is Adj + Ctl",
                    (true, false) => " is Adj",
                    (false, true) => " is Ctl",
                    (false, false) => "",
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
pub enum Prim {
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

impl From<ast::TyPrim> for Prim {
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

impl From<hir::TyPrim> for Prim {
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
        (*value).into()
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
        (*value).into()
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Var(pub usize);

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

pub(super) fn ast_callable_functors(decl: &ast::CallableDecl) -> HashSet<Functor> {
    let mut functors = decl.functors.as_ref().map_or(HashSet::new(), |f| {
        f.to_set().into_iter().map(Into::into).collect()
    });

    if let ast::CallableBody::Specs(specs) = &decl.body {
        for spec in specs {
            match spec.spec {
                ast::Spec::Body => {}
                ast::Spec::Adj => functors.extend([Functor::Adj]),
                ast::Spec::Ctl => functors.extend([Functor::Ctl]),
                ast::Spec::CtlAdj => functors.extend([Functor::Adj, Functor::Ctl]),
            }
        }
    }

    functors
}

fn hir_callable_functors(decl: &hir::CallableDecl) -> HashSet<Functor> {
    let mut functors = decl.functors.as_ref().map_or(HashSet::new(), |f| {
        f.to_set().into_iter().map(Into::into).collect()
    });

    if let hir::CallableBody::Specs(specs) = &decl.body {
        for spec in specs {
            match spec.spec {
                hir::Spec::Body => {}
                hir::Spec::Adj => functors.extend([Functor::Adj]),
                hir::Spec::Ctl => functors.extend([Functor::Ctl]),
                hir::Spec::CtlAdj => functors.extend([Functor::Adj, Functor::Ctl]),
            }
        }
    }

    functors
}
