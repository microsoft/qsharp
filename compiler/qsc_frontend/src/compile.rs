// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    diagnostic::OffsetError,
    id::Assigner,
    parse,
    resolve::{self, Resolutions},
    typeck::{self, Tys},
    validate::{self, validate},
};
use miette::Diagnostic;
use qsc_ast::{
    ast::{Package, Span},
    mut_visit::MutVisitor,
    visit::Visitor,
};
use std::fmt::Debug;
use std::{
    collections::{hash_map::Iter, HashMap},
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
    tys: Tys,
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
    pub fn tys(&self) -> &Tys {
        &self.tys
    }

    pub fn tys_mut(&mut self) -> &mut Tys {
        &mut self.tys
    }

    #[must_use]
    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    /// Finds the source in this context that the byte offset corresponds to. Returns the index of
    /// that source and its starting byte offset.
    #[must_use]
    pub fn source(&self, offset: usize) -> (SourceIndex, usize) {
        let (index, &offset) = self
            .offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &o)| offset >= o)
            .expect("offset should match at least one source");

        (SourceIndex(index), offset)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceIndex(pub usize);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(pub(crate) ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub(crate) enum ErrorKind {
    Parse(OffsetError<parse::Error>),
    Resolve(resolve::Error),
    Ty(typeck::Error),
    Validate(validate::Error),
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

    #[must_use]
    pub fn iter(&self) -> Iter<PackageId, CompileUnit> {
        self.units.iter()
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

    let dependencies: Vec<_> = dependencies.into_iter().collect();
    let (resolutions, resolve_errors) = resolve_all(store, dependencies.iter().copied(), &package);
    let (tys, ty_errors) = typeck_all(store, dependencies.iter().copied(), &package, &resolutions);
    let validate_errors = validate(&package);

    let mut errors = Vec::new();
    errors.extend(parse_errors.into_iter().map(|e| Error(ErrorKind::Parse(e))));
    errors.extend(
        resolve_errors
            .into_iter()
            .map(|e| Error(ErrorKind::Resolve(e))),
    );
    errors.extend(ty_errors.into_iter().map(|e| Error(ErrorKind::Ty(e))));
    errors.extend(
        validate_errors
            .into_iter()
            .map(|e| Error(ErrorKind::Validate(e))),
    );

    CompileUnit {
        package,
        context: Context {
            assigner,
            resolutions,
            tys,
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
) -> (Package, Vec<OffsetError<parse::Error>>, Vec<usize>) {
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

fn resolve_all(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &Package,
) -> (Resolutions, Vec<resolve::Error>) {
    let mut globals = resolve::GlobalTable::new();
    globals.visit_package(package);
    for dependency in dependencies {
        let unit = store
            .get(dependency)
            .expect("dependency should be in package store before compilation");
        globals.set_package(dependency);
        globals.visit_package(&unit.package);
    }

    let mut resolver = globals.into_resolver();
    resolver.visit_package(package);
    resolver.into_resolutions()
}

fn typeck_all(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &Package,
    resolutions: &Resolutions,
) -> (Tys, Vec<typeck::Error>) {
    let mut globals = typeck::GlobalTable::new(resolutions);
    globals.visit_package(package);
    for dependency in dependencies {
        let unit = store
            .get(dependency)
            .expect("dependency should be added to package store before compilation");
        globals.set_package(dependency);
        globals.visit_package(&unit.package);
    }

    let mut checker = globals.into_checker();
    checker.visit_package(package);
    checker.into_tys()
}

fn append_errors(
    errors: &mut Vec<OffsetError<parse::Error>>,
    offset: usize,
    other: Vec<parse::Error>,
) {
    let offset = offset.try_into().expect("offset should fit into isize");
    for error in other {
        errors.push(OffsetError::new(error, offset));
    }
}
