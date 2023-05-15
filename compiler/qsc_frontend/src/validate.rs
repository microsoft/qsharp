// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{Expr, ExprKind, Package},
    visit::{self, Visitor},
};
use qsc_data_structures::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("{0} are not currently supported")]
    NotCurrentlySupported(&'static str, #[label] Span),
}

pub(super) fn validate(package: &Package) -> Vec<Error> {
    let mut validator = Validator { errors: Vec::new() };
    validator.visit_package(package);
    validator.errors
}

struct Validator {
    errors: Vec<Error>,
}

impl Visitor<'_> for Validator {
    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Lambda(..) => self
                .errors
                .push(Error::NotCurrentlySupported("lambdas", expr.span)),
            ExprKind::Call(_, arg) if has_hole(arg) => self.errors.push(
                Error::NotCurrentlySupported("partial applications", expr.span),
            ),
            _ => {}
        };

        visit::walk_expr(self, expr);
    }
}

fn has_hole(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Hole => true,
        ExprKind::Paren(sub_expr) => has_hole(sub_expr),
        ExprKind::Tuple(sub_exprs) => sub_exprs.iter().any(has_hole),
        _ => false,
    }
}
