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
use qsc_hir::{
    hir::{CallableKind, ItemId},
    ty::{FunctorSet, GenericArg, Ty, Udt},
};
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use thiserror::Error;

pub(super) use check::{Checker, GlobalTable};

#[derive(Debug, Default, Clone)]
pub struct Table {
    pub udts: FxHashMap<ItemId, Udt>,

    // AST nodes that get mapped to types are Expr, Block, Pat, and QubitInit nodes
    pub terms: IndexMap<NodeId, Ty>,
    pub generics: IndexMap<NodeId, Vec<GenericArg>>,
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub(super) struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error("expected {0}, found {1}")]
    #[diagnostic(code("Qsc.TypeCk.TyMismatch"))]
    TyMismatch(String, String, #[label] Span),
    #[error("expected {0}, found {1}")]
    #[diagnostic(code("Qsc.TypeCk.CallableMismatch"))]
    CallableMismatch(CallableKind, CallableKind, #[label] Span),
    #[error("expected {0}, found {1}")]
    #[diagnostic(code("Qsc.TypeCk.FunctorMismatch"))]
    FunctorMismatch(FunctorSet, FunctorSet, #[label] Span),
    #[error("type {0} does not support plus")]
    #[diagnostic(help("only arrays, BigInt, Double, Int and String support plus"))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassAdd"))]
    MissingClassAdd(String, #[label] Span),
    #[error("type {0} does not support the adjoint functor")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassAdj"))]
    MissingClassAdj(String, #[label] Span),
    #[error("type {0} is not callable")]
    #[diagnostic(help("only operations, functions, and newtype constructors can be called"))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassCall"))]
    MissingClassCall(String, #[label] Span),
    #[error("type {0} does not support the controlled functor")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassCtl"))]
    MissingClassCtl(String, #[label] Span),
    #[error("type {0} does not support equality")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassEq"))]
    MissingClassEq(String, #[label] Span),
    #[error("type {0} does not support exponentiation")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassExp"))]
    MissingClassExp(String, #[label] Span),
    #[error("type {0} does not have a field `{1}`")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassHasField"))]
    MissingClassHasField(String, String, #[label] Span),
    #[error("type {0} is not a struct")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassStruct"))]
    MissingClassStruct(String, #[label] Span),
    #[error("duplicate field `{1}` listed in constructor for type {0}")]
    #[diagnostic(code("Qsc.TypeCk.DuplicateField"))]
    DuplicateField(String, String, #[label] Span),
    #[error("incorrect number of field assignments for type {0}")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassCorrectFieldCount"))]
    MissingClassCorrectFieldCount(String, #[label] Span),
    #[error("type {0} cannot be indexed by type {1}")]
    #[diagnostic(help(
        "only array types can be indexed, and only Int and Range can be used as the index"
    ))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassHasIndex"))]
    MissingClassHasIndex(String, String, #[label] Span),
    #[error("type {0} is not an integer")]
    #[diagnostic(help("only BigInt and Int are integers"))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassInteger"))]
    MissingClassInteger(String, #[label] Span),
    #[error("type {0} is not iterable")]
    #[diagnostic(help("only arrays and ranges are iterable"))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassIterable"))]
    MissingClassIterable(String, #[label] Span),
    #[error("type {0} is not a number")]
    #[diagnostic(help("only BigInt, Double, and Int are numbers"))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassNum"))]
    MissingClassNum(String, #[label] Span),
    #[error("type {0} cannot be converted into a string")]
    #[diagnostic(code("Qsc.TypeCk.MissingClassShow"))]
    MissingClassShow(String, #[label] Span),
    #[error("type {0} cannot be unwrapped")]
    #[diagnostic(help("only newtypes support unwrap"))]
    #[diagnostic(code("Qsc.TypeCk.MissingClassUnwrap"))]
    MissingClassUnwrap(String, #[label] Span),
    #[error("expected superset of {0}, found {1}")]
    #[diagnostic(code("Qsc.TypeCk.MissingFunctor"))]
    MissingFunctor(FunctorSet, FunctorSet, #[label] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    #[diagnostic(code("Qsc.TypeCk.MissingItemTy"))]
    MissingItemTy(#[label] Span),
    #[error("found hole with type {0}")]
    #[diagnostic(help("replace this hole with an expression of the expected type"))]
    #[diagnostic(code("Qsc.TypeCk.TyHole"))]
    TyHole(String, #[label] Span),
    #[error("insufficient type information to infer type")]
    #[diagnostic(help("provide a type annotation"))]
    #[diagnostic(code("Qsc.TypeCk.AmbiguousTy"))]
    AmbiguousTy(#[label] Span),
}
