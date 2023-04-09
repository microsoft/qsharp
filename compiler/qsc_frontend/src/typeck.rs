// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod check;
mod infer;
mod solve;
#[cfg(test)]
mod tests;

use self::solve::Class;
use miette::Diagnostic;
use qsc_ast::ast::{Functor, FunctorExpr, FunctorExprKind, NodeId, SetOp, Span};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};
use thiserror::Error;

pub(super) use check::GlobalTable;
pub use solve::Ty;

pub type Tys = HashMap<NodeId, Ty>;

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
