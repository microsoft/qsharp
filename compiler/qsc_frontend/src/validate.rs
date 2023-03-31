// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{
        CallableDecl, CallableKind, Expr, ExprKind, Package, Pat, PatKind, Span, Spec, SpecBody,
        SpecDecl, Ty, TyKind,
    },
    visit::{walk_callable_decl, walk_expr, walk_ty, Visitor},
};
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("callable specialization pattern requires elided pattern `...`")]
    ElidedRequired(#[label("should be `...`")] Span),

    #[error("callable specialization pattern requires elided tuple `(ident, ...)`")]
    ElidedTupleRequired(#[label("should be `(ident, ...)`")] Span),

    #[error("callable parameter `{0}` must be type annotated")]
    ParameterNotTyped(String, #[label("missing type annotation")] Span),

    #[error("adjointable/controllable operation `{0}` must return Unit")]
    NonUnitReturn(String, #[label("must return Unit")] Span),

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
            PatKind::Bind(id, ty) => {
                if ty.is_none() {
                    self.validation_errors
                        .push(Error::ParameterNotTyped(id.name.clone(), params.span));
                }
            }
            PatKind::Paren(item) => self.validate_params(item),
            PatKind::Tuple(items) => {
                items.iter().for_each(|i| self.validate_params(i));
            }
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

    fn visit_spec_decl(&mut self, decl: &'a SpecDecl) {
        match decl.spec {
            Spec::Body | Spec::Adj => {
                if let SpecBody::Impl(pat, _) = &decl.body {
                    if !is_elided(pat) {
                        self.validation_errors.push(Error::ElidedRequired(pat.span));
                    }
                }
            }
            Spec::Ctl | Spec::CtlAdj => {
                if let SpecBody::Impl(pat, _) = &decl.body {
                    if !is_elided_tuple(pat) {
                        self.validation_errors
                            .push(Error::ElidedTupleRequired(pat.span));
                    }
                }
            }
        }
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

    fn visit_ty(&mut self, ty: &'a Ty) {
        if let TyKind::Hole = ty.kind {
            self.validation_errors
                .push(Error::NotCurrentlySupported("type holes", ty.span));
        }

        walk_ty(self, ty);
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

fn is_elided(pat: &Pat) -> bool {
    match &pat.kind {
        PatKind::Elided => true,
        PatKind::Paren(pat) => is_elided(pat),
        _ => false,
    }
}

fn is_elided_tuple(pat: &Pat) -> bool {
    match &pat.kind {
        PatKind::Paren(pat) => is_elided_tuple(pat),
        PatKind::Tuple(pats) => {
            pats.len() == 2 && matches!(&pats[0].kind, PatKind::Bind(_, _)) && is_elided(&pats[1])
        }
        _ => false,
    }
}
