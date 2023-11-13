// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

pub mod preprocess;

use crate::{
    error::WithSource,
    lower::{self, Lowerer},
    resolve::{self, Names, Resolver},
    typeck::{self, Checker, Table},
};
use miette::{Diagnostic, Report};
use preprocess::TrackedName;
use qsc_ast::{
    assigner::Assigner as AstAssigner,
    ast::{self, TopLevelNode},
    mut_visit::MutVisitor,
    validate::Validator as AstValidator,
    visit::Visitor as _,
};
use qsc_data_structures::{
    index_map::{self, IndexMap},
    span::Span,
};
use qsc_hir::{
    assigner::Assigner as HirAssigner,
    global,
    hir::{self, PackageId},
    validate::Validator as HirValidator,
    visit::Visitor as _,
};
use std::{fmt::Debug, str::FromStr, sync::Arc};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetProfile {
    Full,
    Base,
}

impl TargetProfile {
    #[must_use]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Full => "Full",
            Self::Base => "Base",
        }
    }

    #[must_use]
    pub fn is_target_str(s: &str) -> bool {
        Self::from_str(s).is_ok()
    }
}

impl FromStr for TargetProfile {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Full" => Ok(TargetProfile::Full),
            "Base" => Ok(Self::Base),
            _ => Err(()),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct CompileUnit {
    pub package: hir::Package,
    pub ast: AstPackage,
    pub assigner: HirAssigner,
    pub sources: SourceMap,
    pub errors: Vec<Error>,
    pub dropped_names: Vec<TrackedName>,
}

#[derive(Debug, Default)]
pub struct AstPackage {
    pub package: ast::Package,
    pub tys: Table,
    pub names: Names,
}

#[derive(Debug, Default)]
pub struct SourceMap {
    sources: Vec<Source>,
    entry: Option<Source>,
}

impl SourceMap {
    pub fn new(
        sources: impl IntoIterator<Item = (SourceName, SourceContents)>,
        entry: Option<Arc<str>>,
    ) -> Self {
        let mut offset_sources = Vec::new();

        let entry_source = entry.map(|contents| Source {
            name: "<entry>".into(),
            contents,
            offset: 0,
        });

        let mut offset = next_offset(entry_source.as_ref());
        for (name, contents) in sources {
            let source = Source {
                name,
                contents,
                offset,
            };
            offset = next_offset(Some(&source));
            offset_sources.push(source);
        }

        Self {
            sources: offset_sources,
            entry: entry_source,
        }
    }

    pub fn push(&mut self, name: SourceName, contents: SourceContents) -> u32 {
        let offset = next_offset(self.sources.last());

        self.sources.push(Source {
            name,
            contents,
            offset,
        });

        offset
    }

    #[must_use]
    pub fn find_by_offset(&self, offset: u32) -> Option<&Source> {
        self.sources
            .iter()
            .rev()
            .chain(&self.entry)
            .find(|source| offset >= source.offset)
    }

    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&Source> {
        self.sources.iter().find(|s| s.name.as_ref() == name)
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    pub name: SourceName,
    pub contents: SourceContents,
    pub offset: u32,
}

pub type SourceName = Arc<str>;

pub type SourceContents = Arc<str>;

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(pub(super) ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
pub(super) enum ErrorKind {
    #[error("syntax error")]
    Parse(#[from] qsc_parse::Error),
    #[error("name error")]
    Resolve(#[from] resolve::Error),
    #[error("type error")]
    Type(#[from] typeck::Error),
    #[error(transparent)]
    Lower(#[from] lower::Error),
}

pub struct PackageStore {
    core: global::Table,
    units: IndexMap<PackageId, CompileUnit>,
    next_id: PackageId,
}

impl PackageStore {
    #[must_use]
    pub fn new(core: CompileUnit) -> Self {
        let table = global::iter_package(Some(PackageId::CORE), &core.package).collect();
        let mut units = IndexMap::new();
        units.insert(PackageId::CORE, core);
        Self {
            core: table,
            units,
            next_id: PackageId::CORE.successor(),
        }
    }

    #[must_use]
    pub fn core(&self) -> &global::Table {
        &self.core
    }

    pub fn insert(&mut self, unit: CompileUnit) -> PackageId {
        let id = self.next_id;
        self.next_id = id.successor();
        self.units.insert(id, unit);
        id
    }

    #[must_use]
    pub fn get(&self, id: PackageId) -> Option<&CompileUnit> {
        self.units.get(id)
    }

    #[must_use]
    pub fn iter(&self) -> Iter {
        Iter(self.units.iter())
    }

    /// "Opens" the package store. This inserts an empty
    /// package into the store, which will be considered
    /// the open package and which can be incrementally updated.
    #[must_use]
    pub fn open(mut self) -> OpenPackageStore {
        let id = self.next_id;
        self.next_id = id.successor();
        self.units.insert(id, CompileUnit::default());

        OpenPackageStore {
            store: self,
            open: id,
        }
    }
}

/// A package store that contains one mutable `CompileUnit`.
pub struct OpenPackageStore {
    store: PackageStore,
    open: PackageId,
}

impl OpenPackageStore {
    /// Returns a reference to the underlying, immutable,
    /// package store.
    #[must_use]
    pub fn package_store(&self) -> &PackageStore {
        &self.store
    }

    /// Returns the ID of the open package.
    #[must_use]
    pub fn open_package_id(&self) -> PackageId {
        self.open
    }

    /// Returns a mutable reference to the open package,
    /// along with a reference to the core library that can be used
    /// to perform passes.
    #[must_use]
    pub fn get_open_mut(&mut self) -> (&global::Table, &mut CompileUnit) {
        let id = self.open;

        (
            &self.store.core,
            self.store
                .units
                .get_mut(id)
                .expect("open package id should exist in store"),
        )
    }

    /// Consumes the `OpenPackageStore` and returns a `PackageStore`
    /// along with the id of the formerly open package.
    #[must_use]
    pub fn into_package_store(self) -> (PackageStore, PackageId) {
        (self.store, self.open)
    }
}

pub struct Iter<'a>(index_map::Iter<'a, PackageId, CompileUnit>);

impl<'a> Iterator for Iter<'a> {
    type Item = (PackageId, &'a CompileUnit);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub(super) struct Offsetter(pub(super) u32);

impl MutVisitor for Offsetter {
    fn visit_span(&mut self, span: &mut Span) {
        span.lo += self.0;
        span.hi += self.0;
    }
}

pub fn compile(
    store: &PackageStore,
    dependencies: &[PackageId],
    sources: SourceMap,
    target: TargetProfile,
) -> CompileUnit {
    let (mut ast_package, parse_errors) = parse_all(&sources);

    let mut cond_compile = preprocess::Conditional::new(target);
    cond_compile.visit_package(&mut ast_package);
    let dropped_names = cond_compile.into_names();

    let mut ast_assigner = AstAssigner::new();
    ast_assigner.visit_package(&mut ast_package);
    AstValidator::default().visit_package(&ast_package);
    let mut hir_assigner = HirAssigner::new();
    let (names, name_errors) = resolve_all(
        store,
        dependencies,
        &mut hir_assigner,
        &ast_package,
        dropped_names.clone(),
    );
    let (tys, ty_errors) = typeck_all(store, dependencies, &ast_package, &names);
    let mut lowerer = Lowerer::new();
    let package = lowerer
        .with(&mut hir_assigner, &names, &tys)
        .lower_package(&ast_package);
    HirValidator::default().visit_package(&package);
    let lower_errors = lowerer.drain_errors();

    let errors = parse_errors
        .into_iter()
        .map(Into::into)
        .chain(name_errors.into_iter().map(Into::into))
        .chain(ty_errors.into_iter().map(Into::into))
        .chain(lower_errors.into_iter().map(Into::into))
        .map(Error)
        .collect();

    CompileUnit {
        package,
        ast: AstPackage {
            package: ast_package,
            tys,
            names,
        },
        assigner: hir_assigner,
        sources,
        errors,
        dropped_names,
    }
}

/// Compiles the core library.
///
/// # Panics
///
/// Panics if the core library does not compile without errors.
#[must_use]
pub fn core() -> CompileUnit {
    let store = PackageStore {
        core: global::Table::default(),
        units: IndexMap::new(),
        next_id: PackageId::CORE,
    };

    let sources = SourceMap::new(
        [
            (
                "core/core.qs".into(),
                include_str!("../../../library/core/core.qs").into(),
            ),
            (
                "core/qir.qs".into(),
                include_str!("../../../library/core/qir.qs").into(),
            ),
        ],
        None,
    );

    let mut unit = compile(&store, &[], sources, TargetProfile::Base);
    assert_no_errors(&unit.sources, &mut unit.errors);
    unit
}

/// Compiles the standard library.
///
/// # Panics
///
/// Panics if the standard library does not compile without errors.
#[must_use]
pub fn std(store: &PackageStore, target: TargetProfile) -> CompileUnit {
    let sources = SourceMap::new(
        [
            (
                "arithmetic.qs".into(),
                include_str!("../../../library/std/arithmetic.qs").into(),
            ),
            (
                "arithmetic_internal.qs".into(),
                include_str!("../../../library/std/arithmetic_internal.qs").into(),
            ),
            (
                "arrays.qs".into(),
                include_str!("../../../library/std/arrays.qs").into(),
            ),
            (
                "canon.qs".into(),
                include_str!("../../../library/std/canon.qs").into(),
            ),
            (
                "convert.qs".into(),
                include_str!("../../../library/std/convert.qs").into(),
            ),
            (
                "core.qs".into(),
                include_str!("../../../library/std/core.qs").into(),
            ),
            (
                "diagnostics.qs".into(),
                include_str!("../../../library/std/diagnostics.qs").into(),
            ),
            (
                "internal.qs".into(),
                include_str!("../../../library/std/internal.qs").into(),
            ),
            (
                "intrinsic.qs".into(),
                include_str!("../../../library/std/intrinsic.qs").into(),
            ),
            (
                "math.qs".into(),
                include_str!("../../../library/std/math.qs").into(),
            ),
            (
                "measurement.qs".into(),
                include_str!("../../../library/std/measurement.qs").into(),
            ),
            (
                "qir.qs".into(),
                include_str!("../../../library/std/qir.qs").into(),
            ),
            (
                "random.qs".into(),
                include_str!("../../../library/std/random.qs").into(),
            ),
        ],
        None,
    );

    let mut unit = compile(store, &[PackageId::CORE], sources, target);
    assert_no_errors(&unit.sources, &mut unit.errors);
    unit
}

fn parse_all(sources: &SourceMap) -> (ast::Package, Vec<qsc_parse::Error>) {
    let mut namespaces = Vec::new();
    let mut errors = Vec::new();
    for source in &sources.sources {
        let (source_namespaces, source_errors) = qsc_parse::namespaces(&source.contents);
        for mut namespace in source_namespaces {
            Offsetter(source.offset).visit_namespace(&mut namespace);
            namespaces.push(TopLevelNode::Namespace(namespace));
        }

        append_parse_errors(&mut errors, source.offset, source_errors);
    }

    let entry = sources
        .entry
        .as_ref()
        .filter(|source| !source.contents.is_empty())
        .map(|source| {
            let (mut entry, entry_errors) = qsc_parse::expr(&source.contents);
            Offsetter(source.offset).visit_expr(&mut entry);
            append_parse_errors(&mut errors, source.offset, entry_errors);
            entry
        });

    let package = ast::Package {
        id: ast::NodeId::default(),
        nodes: namespaces.into_boxed_slice(),
        entry,
    };

    (package, errors)
}

fn resolve_all(
    store: &PackageStore,
    dependencies: &[PackageId],
    assigner: &mut HirAssigner,
    package: &ast::Package,
    mut dropped_names: Vec<TrackedName>,
) -> (Names, Vec<resolve::Error>) {
    let mut globals = resolve::GlobalTable::new();
    if let Some(unit) = store.get(PackageId::CORE) {
        globals.add_external_package(PackageId::CORE, &unit.package);
        dropped_names.extend(unit.dropped_names.iter().cloned());
    }

    for &id in dependencies {
        let unit = store
            .get(id)
            .expect("dependency should be in package store before compilation");
        globals.add_external_package(id, &unit.package);
        dropped_names.extend(unit.dropped_names.iter().cloned());
    }

    let mut errors = globals.add_local_package(assigner, package);
    let mut resolver = Resolver::new(globals, dropped_names);
    resolver.with(assigner).visit_package(package);
    let (names, mut resolver_errors) = resolver.into_names();
    errors.append(&mut resolver_errors);
    (names, errors)
}

fn typeck_all(
    store: &PackageStore,
    dependencies: &[PackageId],
    package: &ast::Package,
    names: &Names,
) -> (typeck::Table, Vec<typeck::Error>) {
    let mut globals = typeck::GlobalTable::new();
    if let Some(unit) = store.get(PackageId::CORE) {
        globals.add_external_package(PackageId::CORE, &unit.package);
    }

    for &id in dependencies {
        let unit = store
            .get(id)
            .expect("dependency should be added to package store before compilation");
        globals.add_external_package(id, &unit.package);
    }

    let mut checker = Checker::new(globals);
    checker.check_package(names, package);
    checker.into_table()
}

fn append_parse_errors(
    errors: &mut Vec<qsc_parse::Error>,
    offset: u32,
    other: Vec<qsc_parse::Error>,
) {
    for error in other {
        errors.push(error.with_offset(offset));
    }
}

fn next_offset(last_source: Option<&Source>) -> u32 {
    // Leave a gap of 1 between each source so that offsets at EOF
    // get mapped to the correct source
    last_source.map_or(0, |s| {
        1 + s.offset + u32::try_from(s.contents.len()).expect("contents length should fit into u32")
    })
}

fn assert_no_errors(sources: &SourceMap, errors: &mut Vec<Error>) {
    if !errors.is_empty() {
        for error in errors.drain(..) {
            eprintln!("{:?}", Report::new(WithSource::from_map(sources, error)));
        }

        panic!("could not compile package");
    }
}
