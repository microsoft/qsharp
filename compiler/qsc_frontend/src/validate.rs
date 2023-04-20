// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use miette::Diagnostic;
use qsc_ast::{
    ast::{Attr, Expr, ExprKind, Item, ItemKind, Package, UnOp},
    visit::{self, Visitor},
};
use qsc_data_structures::span::Span;
use thiserror::Error;

#[derive(Clone, Debug, Diagnostic, Error)]
pub(super) enum Error {
    #[error("invalid attribute arguments, expected {0}")]
    InvalidAttrArgs(&'static str, #[label("invalid attribute arguments")] Span),
    #[error("{0} are not currently supported")]
    NotCurrentlySupported(&'static str, #[label("not currently supported")] Span),
    #[error("unrecognized attribute {0}")]
    #[diagnostic(help("supported attributes are: `EntryPoint`"))]
    UnrecognizedAttr(String, #[label("unrecognized attribute")] Span),
}

pub(super) fn validate(package: &Package) -> Vec<Error> {
    let mut validator = Validator { errors: Vec::new() };
    validator.visit_package(package);
    validator.errors
}

struct Validator {
    errors: Vec<Error>,
}

impl Validator {
    fn validate_attrs(&mut self, attrs: &[Attr]) {
        for attr in attrs {
            match attr.name.name.as_str() {
                "EntryPoint" => match &attr.arg.kind {
                    ExprKind::Tuple(args) if args.is_empty() => {}
                    _ => self
                        .errors
                        .push(Error::InvalidAttrArgs("()", attr.arg.span)),
                },
                _ => self
                    .errors
                    .push(Error::UnrecognizedAttr(attr.name.name.clone(), attr.span)),
            }
        }
    }
}

impl Visitor<'_> for Validator {
    fn visit_item(&mut self, item: &Item) {
        self.validate_attrs(&item.attrs);
        if matches!(&item.kind, ItemKind::Ty(..)) {
            self.errors
                .push(Error::NotCurrentlySupported("newtype", item.span));
        }
        visit::walk_item(self, item);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Lambda(..) => self
                .errors
                .push(Error::NotCurrentlySupported("lambdas", expr.span)),
            ExprKind::Call(_, arg) if has_hole(arg) => self.errors.push(
                Error::NotCurrentlySupported("partial applications", expr.span),
            ),
            ExprKind::UnOp(UnOp::Unwrap, _) => self
                .errors
                .push(Error::NotCurrentlySupported("unwrap operator", expr.span)),
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
