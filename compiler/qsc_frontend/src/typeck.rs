// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod check;
pub(super) mod convert;
mod infer;
mod rules;
#[cfg(test)]
mod tests;

use self::infer::Class;
use miette::Diagnostic;
use qsc_ast::ast::NodeId;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{
    hir::{CallableKind, ItemId},
    ty::{FunctorSet, GenericArg, Ty, Udt},
};
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

pub(super) use check::{Checker, GlobalTable};

pub(super) struct Table {
    pub(super) udts: HashMap<ItemId, Udt>,
    pub(super) terms: IndexMap<NodeId, Ty>,
    pub(super) generics: IndexMap<NodeId, Vec<GenericArg>>,
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub(super) struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error("expected {0}, found {1}")]
    TyMismatch(Ty, Ty, #[label] Span),
    #[error("expected {0}, found {1}")]
    CallableMismatch(CallableKind, CallableKind, #[label] Span),
    #[error("expected {0}, found {1}")]
    FunctorMismatch(FunctorSet, FunctorSet, #[label] Span),
    #[error("missing class instance {0}")]
    MissingClass(Class, #[label] Span),
    #[error("expected superset of {0}, found {1}")]
    MissingFunctor(FunctorSet, FunctorSet, #[label] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    MissingItemTy(#[label] Span),
    #[error("found hole with type {0}")]
    #[diagnostic(help("replace this hole with an expression of the expected type"))]
    TyHole(Ty, #[label] Span),
}
