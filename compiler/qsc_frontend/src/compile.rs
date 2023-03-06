// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{id::Assigner, parse, symbol};
use qsc_ast::{
    ast::{Package, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};

#[derive(Debug)]
pub struct Context<'a> {
    assigner: Assigner,
    symbols: symbol::Table,
    errors: Vec<Error>,
    offsets: Vec<usize>,
    deps: Vec<&'a Package>,
}

impl<'a> Context<'a> {
    pub fn assigner_mut(&mut self) -> &mut Assigner {
        &mut self.assigner
    }

    #[must_use]
    pub fn symbols(&self) -> &symbol::Table {
        &self.symbols
    }

    pub fn symbols_mut(&mut self) -> &mut symbol::Table {
        &mut self.symbols
    }

    #[must_use]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    #[must_use]
    pub fn deps(&self) -> &[&'a Package] {
        &self.deps
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

#[derive(Debug, Eq, PartialEq)]
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

pub fn compile<'a>(
    files: &[&str],
    entry_expr: &str,
    deps: Vec<&'a Package>,
) -> (Package, Context<'a>) {
    let mut namespaces = Vec::new();
    let mut parse_errors = Vec::new();
    let mut offset = 0;
    let mut offsets = Vec::new();

    for file in files {
        let (file_namespaces, errors) = parse::namespaces(file);
        for mut namespace in file_namespaces {
            Offsetter(offset).visit_namespace(&mut namespace);
            namespaces.push(namespace);
        }

        append_errors(&mut parse_errors, offset, errors);
        offsets.push(offset);
        offset += file.len();
    }

    let entry = if entry_expr.is_empty() {
        None
    } else {
        let (mut entry, errors) = parse::expr(entry_expr);
        Offsetter(offset).visit_expr(&mut entry);
        append_errors(&mut parse_errors, offset, errors);
        offsets.push(offset);
        Some(entry)
    };

    let mut package = Package::new(namespaces, entry);
    let mut assigner = Assigner::new();
    assigner.visit_package(&mut package);
    let mut globals = symbol::GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);
    let (symbols, symbol_errors) = resolver.into_table();
    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(Into::into));
    errors.extend(symbol_errors.into_iter().map(Into::into));

    (
        package,
        Context {
            assigner,
            symbols,
            errors,
            offsets,
            deps,
        },
    )
}

fn append_errors(errors: &mut Vec<parse::Error>, offset: usize, other: Vec<parse::Error>) {
    for mut error in other {
        error.span.lo += offset;
        error.span.hi += offset;
        errors.push(error);
    }
}
