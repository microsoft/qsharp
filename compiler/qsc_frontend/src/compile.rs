// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

mod preprocess;

use crate::{
    lower::{self, Lowerer},
    resolve::{self, Names, Resolver},
    typeck::{self, Checker, Table},
};
use miette::{
    Diagnostic, MietteError, MietteSpanContents, Report, SourceCode, SourceSpan, SpanContents,
};
use qsc_ast::{
    assigner::Assigner as AstAssigner, ast, mut_visit::MutVisitor,
    validate::Validator as AstValidator, visit::Visitor as _,
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
        for (name, contents) in sources {
            offset_sources.push(Source {
                name,
                contents,
                offset: next_offset(&offset_sources),
            });
        }

        let entry_source = entry.map(|contents| Source {
            name: "<entry>".into(),
            contents,
            offset: next_offset(&offset_sources),
        });

        Self {
            sources: offset_sources,
            entry: entry_source,
        }
    }

    #[must_use]
    pub fn find_by_offset(&self, offset: u32) -> Option<&Source> {
        self.sources
            .iter()
            .chain(&self.entry)
            .rev()
            .find(|source| offset >= source.offset)
    }

    #[must_use]
    pub fn find_by_diagnostic(&self, diagnostic: &impl Diagnostic) -> Option<&Source> {
        diagnostic
            .labels()
            .and_then(|mut labels| labels.next())
            .and_then(|label| {
                self.find_by_offset(
                    label
                        .offset()
                        .try_into()
                        .expect("offset should fit into u32"),
                )
            })
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

impl SourceCode for Source {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let contents = self.contents.read_span(
            &with_offset(span, |o| o - (self.offset as usize)),
            context_lines_before,
            context_lines_after,
        )?;

        Ok(Box::new(MietteSpanContents::new_named(
            self.name.to_string(),
            contents.data(),
            with_offset(contents.span(), |o| o + (self.offset as usize)),
            contents.line(),
            contents.column(),
            contents.line_count(),
        )))
    }
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

    preprocess::Conditional { target }.visit_package(&mut ast_package);

    let mut ast_assigner = AstAssigner::new();
    ast_assigner.visit_package(&mut ast_package);
    AstValidator::default().visit_package(&ast_package);
    let mut hir_assigner = HirAssigner::new();
    let (names, name_errors) = resolve_all(store, dependencies, &mut hir_assigner, &ast_package);
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
                "core.qs".into(),
                include_str!("../../../library/core/core.qs").into(),
            ),
            (
                "qir.qs".into(),
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
            namespaces.push(namespace);
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
        namespaces: namespaces.into_boxed_slice(),
        entry,
    };

    (package, errors)
}

fn resolve_all(
    store: &PackageStore,
    dependencies: &[PackageId],
    assigner: &mut HirAssigner,
    package: &ast::Package,
) -> (Names, Vec<resolve::Error>) {
    let mut globals = resolve::GlobalTable::new();
    if let Some(unit) = store.get(PackageId::CORE) {
        globals.add_external_package(PackageId::CORE, &unit.package);
    }

    for &id in dependencies {
        let unit = store
            .get(id)
            .expect("dependency should be in package store before compilation");
        globals.add_external_package(id, &unit.package);
    }

    let mut errors = globals.add_local_package(assigner, package);
    let mut resolver = Resolver::new(globals);
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

fn with_offset(span: &SourceSpan, f: impl FnOnce(usize) -> usize) -> SourceSpan {
    SourceSpan::new(f(span.offset()).into(), span.len().into())
}

fn next_offset(sources: &[Source]) -> u32 {
    sources.last().map_or(0, |s| {
        // Leave a gap of 1 between each source so that offsets at EOF
        // get mapped to the correct source
        1 + s.offset + u32::try_from(s.contents.len()).expect("contents length should fit into u32")
    })
}

fn assert_no_errors(sources: &SourceMap, errors: &mut Vec<Error>) {
    if !errors.is_empty() {
        for error in errors.drain(..) {
            if let Some(source) = sources.find_by_diagnostic(&error) {
                eprintln!("{:?}", Report::new(error).with_source_code(source.clone()));
            } else {
                eprintln!("{:?}", Report::new(error));
            }
        }

        panic!("could not compile package");
    }
}
