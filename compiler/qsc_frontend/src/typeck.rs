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
use std::{collections::HashMap, fmt::Debug, rc::Rc};
use thiserror::Error;

pub(super) use check::{Checker, GlobalTable};

pub type Tys = IndexMap<ast::NodeId, Ty>;

pub(super) struct Udt {
    pub(super) base: Ty,
    pub(super) fields: HashMap<Rc<str>, UdtField>,
}

pub(super) struct UdtField {
    pub(super) path: Vec<usize>,
    pub(super) ty: Ty,
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub(super) struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
enum ErrorKind {
    #[error("expected {0}, found {1}")]
    TypeMismatch(Ty, Ty, #[label] Span),
    #[error("missing class instance {0}")]
    MissingClass(Class, #[label] Span),
    #[error("missing type in item signature")]
    #[diagnostic(help("types cannot be inferred for global declarations"))]
    MissingItemTy(#[label] Span),
}
