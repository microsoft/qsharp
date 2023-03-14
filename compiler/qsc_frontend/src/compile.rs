// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    id::Assigner,
    lex, parse,
    resolve::{self, GlobalTable, Resolutions},
};
use miette::Diagnostic;
use qsc_ast::{
    ast::{Package, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use std::fmt::Debug;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CompileUnit {
    pub package: Package,
    pub context: Context,
}

#[derive(Debug)]
pub struct Context {
    assigner: Assigner,
    resolutions: Resolutions,
    errors: Vec<Error>,
    offsets: Vec<usize>,
}

impl Context {
    pub fn assigner_mut(&mut self) -> &mut Assigner {
        &mut self.assigner
    }

    #[must_use]
    pub fn resolutions(&self) -> &Resolutions {
        &self.resolutions
    }

    pub fn resolutions_mut(&mut self) -> &mut Resolutions {
        &mut self.resolutions
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
    pub fn find_source(&self, offset: usize) -> SourceIndex {
        SourceIndex(
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
    pub fn source_span(&self, span: Span) -> (SourceIndex, Span) {
        let (index, &offset) = self
            .offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &offset)| span.lo >= offset)
            .expect("Span should match at least one offset.");

        (
            SourceIndex(index),
            Span {
                lo: span.lo - offset,
                hi: span.hi - offset,
            },
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct SourceIndex(pub usize);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
enum ErrorKind {
    Parse(parse::Error),
    Resolve(resolve::Error),
}

#[derive(Default)]
pub struct PackageStore {
    units: HashMap<PackageId, CompileUnit>,
    next_id: PackageId,
}

impl PackageStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, unit: CompileUnit) -> PackageId {
        let id = self.next_id;
        self.next_id = PackageId(id.0 + 1);
        self.units.insert(id, unit);
        id
    }

    #[must_use]
    pub fn get(&self, id: PackageId) -> Option<&CompileUnit> {
        self.units.get(&id)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PackageId(u32);

impl Display for PackageId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

struct Offsetter(usize);

impl MutVisitor for Offsetter {
    fn visit_span(&mut self, span: &mut Span) {
        span.lo += self.0;
        span.hi += self.0;
    }
}

pub fn compile(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    entry_expr: &str,
) -> CompileUnit {
    let (mut package, parse_errors, offsets) = parse_all(sources, entry_expr);
    let mut assigner = Assigner::new();
    assigner.visit_package(&mut package);
    let (resolutions, resolve_errors) = resolve_all(store, dependencies, &package);

    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(|e| Error(ErrorKind::Parse(e))));
    errors.extend(
        resolve_errors
            .into_iter()
            .map(|e| Error(ErrorKind::Resolve(e))),
    );

    CompileUnit {
        package,
        context: Context {
            assigner,
            resolutions,
            errors,
            offsets,
        },
    }
}

#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn std() -> CompileUnit {
    let unit = compile(
        &PackageStore::new(),
        [],
        [
            include_str!("../../../library/canon.qs"),
            include_str!("../../../library/convert.qs"),
            include_str!("../../../library/core.qs"),
            include_str!("../../../library/diagnostics.qs"),
            include_str!("../../../library/internal.qs"),
            include_str!("../../../library/intrinsic.qs"),
            include_str!("../../../library/math.qs"),
            include_str!("../../../library/qir.qs"),
        ],
        "",
    );

    let errors = unit.context.errors();
    assert!(
        errors.is_empty(),
        "Failed to compile standard library: {errors:#?}"
    );

    unit
}

fn parse_all(
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    entry_expr: &str,
) -> (Package, Vec<parse::Error>, Vec<usize>) {
    let mut namespaces = Vec::new();
    let mut errors = Vec::new();
    let mut offsets = Vec::new();
    let mut offset = 0;

    for source in sources {
        let source = source.as_ref();
        let (source_namespaces, source_errors) = parse::namespaces(source);
        for mut namespace in source_namespaces {
            Offsetter(offset).visit_namespace(&mut namespace);
            namespaces.push(namespace);
        }

        append_errors(&mut errors, offset, source_errors);
        offsets.push(offset);
        offset += source.len();
    }

    let entry = if entry_expr.is_empty() {
        None
    } else {
        let (mut entry, entry_errors) = parse::expr(entry_expr);
        Offsetter(offset).visit_expr(&mut entry);
        append_errors(&mut errors, offset, entry_errors);
        offsets.push(offset);
        Some(entry)
    };

    (Package::new(namespaces, entry), errors, offsets)
}

fn resolve_all<'a>(
    store: &'a PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &'a Package,
) -> (Resolutions, Vec<resolve::Error>) {
    let mut globals = GlobalTable::new();
    globals.visit_package(package);

    for dependency in dependencies {
        globals.set_package(dependency);
        let unit = store
            .get(dependency)
            .expect("Dependency should be in package store.");
        globals.visit_package(&unit.package);
    }

    let mut resolver = globals.into_resolver();
    resolver.visit_package(package);
    resolver.into_resolutions()
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
