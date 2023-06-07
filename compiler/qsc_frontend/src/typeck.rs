// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod check;
pub(super) mod convert;
mod infer;
mod rules;
#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::ast::NodeId;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::hir::{CallableKind, FunctorSet, ItemId, Ty, Udt};
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

pub(super) use check::{Checker, GlobalTable};

pub(super) struct Table {
    pub(super) udts: HashMap<ItemId, Udt>,
    pub(super) terms: IndexMap<NodeId, Ty>,
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
    #[error("type {0} does not support add or concatenate")]
    #[diagnostic(help("only numeric types BigInt, Double, and Int support addition, and only strings and arrays support concatenation"))]
    MissingClassAdd(Ty, #[label] Span),
    #[error("type {0} does not support Adjoint functor")]
    MissingClassAdj(Ty, #[label] Span),
    #[error("expected callable type, found {0}")]
    #[diagnostic(help(
        "only operations, functions, or newtype constructors can be used as a callable"
    ))]
    MissingClassCall(Ty, #[label] Span),
    #[error("type {0} does not support Controlled functor")]
    MissingClassCtl(Ty, #[label] Span),
    #[error("type {0} does not support equality comparison")]
    #[diagnostic(help("only BigInt, Bool, Double, Int, Qubit, Range, Result, String, Paulis, and Tuples of matching types support equiality comparison."))]
    MissingClassEq(Ty, #[label] Span),
    #[error("exponentiation not supported for type {0}")]
    MissingClassExp(Ty, #[label] Span),
    #[error("type {0} does not have a field `{1}` of type {2}")]
    MissingClassHasField(Ty, String, Ty, #[label] Span),
    #[error("type {0} does not support indexing with type {1}")]
    #[diagnostic(help("only array types can be indexed, using Int or Range"))]
    MissingClassHasIndex(Ty, Ty, #[label] Span),
    #[error("type {0} is not an integral type")]
    #[diagnostic(help("only BigInt or Int can be used as an integral type"))]
    MissingClassIntegral(Ty, #[label] Span),
    #[error("type {0} is not iterable")]
    #[diagnostic(help("only arrays and ranges are iterable"))]
    MissingClassIterable(Ty, #[label] Span),
    #[error("type {0} is not a numeric type")]
    #[diagnostic(help("only BigInt, Double, or Int can be used as a numeric type"))]
    MissingClassNum(Ty, #[label] Span),
    #[error("type {0} cannot convert to string")]
    MissingClassShow(Ty, #[label] Span),
    #[error("type {0} cannot be unwrapped")]
    #[diagnostic(help("only newtype tuples support unwrap"))]
    MissingClassUnwrap(Ty, #[label] Span),
    #[error("expected superset of {0}, found {1}")]
    MissingFunctor(FunctorSet, FunctorSet, #[label] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    MissingItemTy(#[label] Span),
    #[error("found hole with type {0}")]
    #[diagnostic(help("replace this hole with an expression of the expected type"))]
    TyHole(Ty, #[label] Span),
}
