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
    package: Package,
    entry: Option<Expr>,
    symbols: symbol::Table,
    errors: Vec<Error>,
    offsets: Vec<usize>,
}

impl Context {
    #[must_use]
    pub fn package(&self) -> &Package {
        &self.package
    }

    #[must_use]
    pub fn entry(&self) -> Option<&Expr> {
        self.entry.as_ref()
    }

    #[must_use]
    pub fn symbols(&self) -> &symbol::Table {
        &self.symbols
    }

    #[must_use]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    #[must_use]
    pub fn file_span(&self, span: Span) -> (FileId, Span) {
        let (index, &offset) = self
            .offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &offset)| span.lo >= offset)
            .expect("Span should match at least one offset.");

        (
            FileId(index),
            Span {
                lo: span.lo - offset,
                hi: span.hi - offset,
            },
        )
    }
}

pub struct FileId(pub usize);

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

struct Offsetter(usize);

impl MutVisitor for Offsetter {
    fn visit_span(&mut self, span: &mut Span) {
        span.lo += self.0;
        span.hi += self.0;
    }
}

pub fn compile(files: &[&str], entry_expr: &str) -> Context {
    let (mut package, mut parse_errors) = (Package::default(), vec![]);
    let mut offset = 0;
    let mut offsets = Vec::new();
    for file in files {
        let (mut file_package, mut file_errors) = parse::package(file);
        Offsetter(offset).visit_package(&mut file_package);
        package.namespaces.append(&mut file_package.namespaces);
        parse_errors.append(&mut file_errors);
        offsets.push(offset);
        offset += file.len();
    }

    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);
    let mut globals = symbol::GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);

    let (mut entry, entry_parse_errors) = if entry_expr.is_empty() {
        (None, Vec::new())
    } else {
        let (entry, entry_parse_errors) = parse::expr(entry_expr);
        (entry, entry_parse_errors)
    };

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
        offsets,
    }
}
