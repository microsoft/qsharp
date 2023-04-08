// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{CallableDecl, CallableKind, Expr, ExprKind, Package, Span, TyKind},
    visit::{self, Visitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("adjointable/controllable operation `{0}` must return Unit")]
    NonUnitReturn(String, #[label("must return Unit")] Span),
    #[error("{0} are not currently supported")]
    NotCurrentlySupported(&'static str, #[label("not currently supported")] Span),
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
    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        if CallableKind::Operation == decl.kind && decl.functors.is_some() {
            match &decl.output.kind {
                TyKind::Tuple(items) if items.is_empty() => {}
                _ => {
                    self.errors.push(Error::NonUnitReturn(
                        decl.name.name.clone(),
                        decl.output.span,
                    ));
                }
            }
        }

        visit::walk_callable_decl(self, decl);
    }

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
