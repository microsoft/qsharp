// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use qsc_ast::{
    ast::{Expr, Package, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};

mod id;
mod lex;
mod parse;
pub mod symbol;

#[derive(Debug)]
pub struct Context {
    pub package: Package,
    pub expr: Option<Expr>,
    pub symbols: symbol::Table,
    pub errors: Vec<Error>,
}

#[allow(dead_code)] // TODO: Format errors for display.
#[derive(Debug)]
pub struct Error {
    span: Span,
    kind: ErrorKind,
}

impl From<parse::Error> for Error {
    fn from(value: parse::Error) -> Self {
        Self {
            span: value.span,
            kind: ErrorKind::Parse(value.kind),
        }
    }
}

impl From<symbol::Error> for Error {
    fn from(value: symbol::Error) -> Self {
        Self {
            span: value.span,
            kind: ErrorKind::Symbol(value.kind),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    Parse(parse::ErrorKind),
    Symbol(symbol::ErrorKind),
}

pub fn compile(input: &[&str], expr_input: Option<&str>) -> Context {
    let input = (*input).join("\n");
    let (mut package, parse_errors) = parse::package(&input);

    let (mut expr, mut expr_parse_errors) = (None, vec![]);
    if let Some(expr_input) = expr_input {
        let (parsed_expr, actual_expr_parse_errors) = parse::expr(expr_input);
        expr = parsed_expr;
        expr_parse_errors = actual_expr_parse_errors;
    }

    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);

    let mut globals = symbol::GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);

    if let Some(ref mut expr) = expr {
        assigner.visit_expr(expr);
        resolver.visit_expr(expr);
    }

    let (symbols, symbol_errors) = resolver.into_table();

    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(Into::into));
    errors.extend(expr_parse_errors.into_iter().map(Into::into));
    errors.extend(symbol_errors.into_iter().map(Into::into));

    Context {
        package,
        expr,
        symbols,
        errors,
    }
}
