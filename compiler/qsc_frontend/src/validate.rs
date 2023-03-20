// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{CallableDecl, CallableKind, Expr, ExprKind, Package, Pat, Span, Ty, TyKind},
    visit::{walk_callable_decl, walk_expr, Visitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("adjointable/controllable operation `{0}` must return Unit")]
    NonUnitReturn(String, #[label("must return Unit")] Span),

    #[error("callable parameter `{0}` must be type annotated")]
    ParameterNotTyped(String, #[label("missing type annotation")] Span),

    #[error("{0} are not currently supported")]
    NotCurrentlySupported(&'static str, #[label("not currently supported")] Span),
}

pub(super) fn validate(package: &Package) -> Vec<Error> {
    let mut validator = Validator {
        validation_errors: Vec::new(),
    };
    validator.visit_package(package);
    validator.validation_errors
}

struct Validator {
    validation_errors: Vec<Error>,
}

impl Validator {
    fn validate_params(&mut self, params: &Pat) {
        match &params.kind {
            qsc_ast::ast::PatKind::Bind(id, ty) => match &ty {
                None => self
                    .validation_errors
                    .push(Error::ParameterNotTyped(id.name.clone(), params.span)),
                Some(t) => self.validate_type(t, params.span),
            },
            qsc_ast::ast::PatKind::Paren(item) => self.validate_params(item),
            qsc_ast::ast::PatKind::Tuple(items) => {
                items.iter().for_each(|i| self.validate_params(i));
            }
            _ => {}
        }
    }

    fn validate_type(&mut self, ty: &Ty, span: Span) {
        match &ty.kind {
            TyKind::App(ty, tys) => {
                self.validate_type(ty, span);
                tys.iter().for_each(|t| self.validate_type(t, span));
            }
            TyKind::Arrow(_, _, _, _) => self.validation_errors.push(Error::NotCurrentlySupported(
                "callables as parameters",
                span,
            )),
            TyKind::Paren(ty) => self.validate_type(ty, span),
            TyKind::Tuple(tys) => tys.iter().for_each(|t| self.validate_type(t, span)),
            _ => {}
        }
    }
}

impl<'a> Visitor<'a> for Validator {
    fn visit_callable_decl(&mut self, decl: &'a CallableDecl) {
        if CallableKind::Operation == decl.kind && decl.functors.is_some() {
            match &decl.output.kind {
                TyKind::Tuple(items) if items.is_empty() => {}
                _ => {
                    self.validation_errors.push(Error::NonUnitReturn(
                        decl.name.name.clone(),
                        decl.output.span,
                    ));
                }
            }
        }

        self.validate_params(&decl.input);
        walk_callable_decl(self, decl);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match &expr.kind {
            ExprKind::Lambda(_, _, _) => self
                .validation_errors
                .push(Error::NotCurrentlySupported("lambdas", expr.span)),
            ExprKind::Call(_, arg) if has_hole(arg) => self.validation_errors.push(
                Error::NotCurrentlySupported("partial applications", expr.span),
            ),
            _ => {}
        };
        walk_expr(self, expr);
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
