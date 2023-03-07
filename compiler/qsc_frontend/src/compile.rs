// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    id::Assigner,
    parse,
    resolve::{self, GlobalTable, Resolutions},
};
use qsc_ast::{
    ast::{Package, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

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
    pub fn file_span(&self, span: Span) -> (FileIndex, Span) {
        let (index, &offset) = self
            .offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &offset)| span.lo >= offset)
            .expect("Span should match at least one offset.");

        (
            FileIndex(index),
            Span {
                lo: span.lo - offset,
                hi: span.hi - offset,
            },
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FileIndex(pub usize);

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
        self.0.fmt(f)
    }
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

impl From<resolve::Error> for Error {
    fn from(value: resolve::Error) -> Self {
        Self {
            span: value.span,
            kind: ErrorKind::Resolve(value.kind),
        }
    }
}

#[derive(Debug)]
enum ErrorKind {
    Parse(parse::ErrorKind),
    Resolve(resolve::ErrorKind),
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
    dependencies: &[PackageId],
    files: &[&str],
    entry_expr: &str,
) -> CompileUnit {
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

    let mut globals = GlobalTable::new();
    globals.visit_package(&package);
    for &dependency in dependencies {
        globals.set_package(dependency);
        let package = store
            .get(dependency)
            .expect("Dependency should be in package store.");
        globals.visit_package(&package.package);
    }

    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);
    let (resolutions, resolve_errors) = resolver.into_resolutions();
    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(Into::into));
    errors.extend(resolve_errors.into_iter().map(Into::into));

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

fn append_errors(errors: &mut Vec<parse::Error>, offset: usize, other: Vec<parse::Error>) {
    for mut error in other {
        error.span.lo += offset;
        error.span.hi += offset;
        errors.push(error);
    }
}
