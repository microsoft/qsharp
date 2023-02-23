// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

use qsc_ast::{
    ast::{Package, Span},
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

pub fn compile(input: &str) -> Context {
    let (mut package, parse_errors) = parse::package(input);
    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);

    let mut globals = symbol::GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);
    let (symbols, symbol_errors) = resolver.into_table();

    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(Into::into));
    errors.extend(symbol_errors.into_iter().map(Into::into));

    Context {
        package,
        symbols,
        errors,
    }
}
