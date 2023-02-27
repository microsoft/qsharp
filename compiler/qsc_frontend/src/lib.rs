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
    pub entry: Option<Expr>,
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

pub fn compile(files: &[&str], entry_expr: &str) -> Context {
    let (mut package, mut parse_errors) = (Package::default(), vec![]);
    for file in files {
        let (mut file_package, mut file_errors) = parse::package(file);
        package.namespaces.append(&mut file_package.namespaces);
        parse_errors.append(&mut file_errors);
    }

    let (mut entry, mut entry_parse_errors) = (None, vec![]);
    if !entry_expr.is_empty() {
        let (parsed_expr, actual_expr_parse_errors) = parse::expr(entry_expr);
        entry = parsed_expr;
        entry_parse_errors = actual_expr_parse_errors;
    }

    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);

    let mut globals = symbol::GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);

    if let Some(ref mut expr) = entry {
        assigner.visit_expr(expr);
        resolver.visit_expr(expr);
    }

    let (symbols, symbol_errors) = resolver.into_table();

    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(Into::into));
    errors.extend(entry_parse_errors.into_iter().map(Into::into));
    errors.extend(symbol_errors.into_iter().map(Into::into));

    Context {
        package,
        entry,
        symbols,
        errors,
    }
}
