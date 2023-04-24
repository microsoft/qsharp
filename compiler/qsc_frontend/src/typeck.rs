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
use qsc_ast::ast;
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::hir::Ty;
use std::fmt::Debug;
use thiserror::Error;

pub(super) use check::{Checker, GlobalTable};

pub type Tys = IndexMap<ast::NodeId, Ty>;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub(super) struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error("mismatched types")]
    TypeMismatch(Ty, Ty, #[label("expected {0}, found {1}")] Span),
    #[error("missing class instance")]
    MissingClass(Class, #[label("requires {0}")] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    MissingItemTy(#[label("explicit type required")] Span),
}
