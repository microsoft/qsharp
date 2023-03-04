// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{id::Assigner, lex, parse, symbol};
use miette::Diagnostic;
use qsc_ast::{
    ast::{Package, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug)]
pub struct Context {
    assigner: Assigner,
    symbols: symbol::Table,
    errors: Vec<Error>,
    offsets: Vec<usize>,
}

impl Context {
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
    pub fn offsets(&self) -> &[usize] {
        &self.offsets
    }

    #[must_use]
    pub fn find_source(&self, offset: usize) -> SourceId {
        SourceId(
            self.offsets
                .iter()
                .enumerate()
                .rev()
                .find(|(_, &o)| offset >= o)
                .expect("Span should match at least one offset.")
                .0,
        )
    }

    #[must_use]
    pub fn source_span(&self, span: Span) -> (SourceId, Span) {
        let (index, &offset) = self
            .offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &offset)| span.lo >= offset)
            .expect("Span should match at least one offset.");

        (
            SourceId(index),
            Span {
                lo: span.lo - offset,
                hi: span.hi - offset,
            },
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceId(pub usize);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub enum Error {
    Parse(parse::Error),
    Symbol(symbol::Error),
}

struct Offsetter(usize);

impl MutVisitor for Offsetter {
    fn visit_span(&mut self, span: &mut Span) {
        span.lo += self.0;
        span.hi += self.0;
    }
}

pub fn compile<T: AsRef<str>>(
    sources: impl IntoIterator<Item = T>,
    entry_expr: &str,
) -> (Package, Context) {
    let mut namespaces = Vec::new();
    let mut parse_errors = Vec::new();
    let mut offset = 0;
    let mut offsets = Vec::new();

    for source in sources {
        let source = source.as_ref();
        let (source_namespaces, errors) = parse::namespaces(source);
        for mut namespace in source_namespaces {
            Offsetter(offset).visit_namespace(&mut namespace);
            namespaces.push(namespace);
        }

        append_errors(&mut parse_errors, offset, errors);
        offsets.push(offset);
        offset += source.len();
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
    errors.extend(parse_errors.into_iter().map(Error::Parse));
    errors.extend(symbol_errors.into_iter().map(Error::Symbol));

    (
        package,
        Context {
            assigner,
            symbols,
            errors,
            offsets,
        },
    )
}

fn append_errors(errors: &mut Vec<parse::Error>, offset: usize, other: Vec<parse::Error>) {
    for error in other {
        errors.push(offset_error(offset, error));
    }
}

// TODO: Not very pretty, and brittle.
fn offset_error(offset: usize, error: parse::Error) -> parse::Error {
    match error {
        parse::Error::Lex(lex::Error::Incomplete(expected, found, single, span)) => {
            parse::Error::Lex(lex::Error::Incomplete(
                expected,
                found,
                single,
                offset_span(offset, span),
            ))
        }
        parse::Error::Lex(lex::Error::IncompleteEof(expected, found, span)) => parse::Error::Lex(
            lex::Error::IncompleteEof(expected, found, offset_span(offset, span)),
        ),
        parse::Error::Lex(lex::Error::Unknown(c, span)) => {
            parse::Error::Lex(lex::Error::Unknown(c, offset_span(offset, span)))
        }
        parse::Error::Token(expected, found, span) => {
            parse::Error::Token(expected, found, offset_span(offset, span))
        }
        parse::Error::Keyword(expected, found, span) => {
            parse::Error::Keyword(expected, found, offset_span(offset, span))
        }
        parse::Error::Rule(expected, found, span) => {
            parse::Error::Rule(expected, found, offset_span(offset, span))
        }
        parse::Error::RuleKeyword(expected, keyword, span) => {
            parse::Error::RuleKeyword(expected, keyword, offset_span(offset, span))
        }
        parse::Error::Convert(expected, found, span) => {
            parse::Error::Convert(expected, found, offset_span(offset, span))
        }
    }
}

fn offset_span(offset: usize, span: Span) -> Span {
    Span {
        lo: span.lo + offset,
        hi: span.hi + offset,
    }
}
