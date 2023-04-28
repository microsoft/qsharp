// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    lower::Lowerer,
    parse,
    resolve::{self, Resolutions},
    typeck::{self, Tys},
    validate::{self, validate},
};
use miette::{
    Diagnostic, MietteError, MietteSpanContents, Report, SourceCode, SourceSpan, SpanContents,
};
use qsc_ast::{assigner::Assigner as AstAssigner, ast, mut_visit::MutVisitor, visit::Visitor};
use qsc_data_structures::{
    index_map::{self, IndexMap},
    span::Span,
};
use qsc_hir::{
    assigner::Assigner as HirAssigner,
    hir::{self, PackageId},
};
use std::{fmt::Debug, sync::Arc};
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CompileUnit {
    pub package: hir::Package,
    pub assigner: HirAssigner,
    pub sources: SourceMap,
    pub errors: Vec<Error>,
}

#[derive(Debug)]
pub struct SourceMap {
    sources: Vec<Source>,
    entry: Option<Source>,
}

impl SourceMap {
    pub fn new(sources: impl IntoIterator<Item = (Arc<str>, Arc<str>)>, entry: Arc<str>) -> Self {
        let mut new_sources: Vec<Source> = Vec::new();
        for (name, content) in sources {
            let offset = new_sources
                .last()
                .map_or(0, |s| s.offset + s.contents.len());
            new_sources.push(Source {
                name,
                contents: content,
                offset,
            });
        }
        let sources = new_sources;

        let entry = if entry.is_empty() {
            None
        } else {
            Some(Source {
                name: "<entry>".into(),
                contents: entry,
                offset: sources.last().map_or(0, |s| s.offset + s.contents.len()),
            })
        };

        Self { sources, entry }
    }

    pub fn report(&self, error: impl Diagnostic + Send + Sync + 'static) -> Report {
        match error.labels().and_then(|mut labels| labels.next()) {
            None => Report::new(error),
            Some(label) => {
                let source = self.find_by_offset(label.offset());
                Report::new(error).with_source_code(source.clone())
            }
        }
    }

    pub(super) fn find_by_offset(&self, offset: usize) -> &Source {
        self.sources
            .iter()
            .chain(&self.entry)
            .rev()
            .find(|source| offset >= source.offset)
            .expect("offset should match at least one source")
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    pub name: Arc<str>,
    pub contents: Arc<str>,
    pub offset: usize,
}

impl SourceCode for Source {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let contents = self.contents.read_span(
            &with_offset(span, |o| o - self.offset),
            context_lines_before,
            context_lines_after,
        )?;

        Ok(Box::new(MietteSpanContents::new_named(
            self.name.to_string(),
            contents.data(),
            with_offset(contents.span(), |o| o + self.offset),
            contents.line(),
            contents.column(),
            contents.line_count(),
        )))
    }
}

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
#[error(transparent)]
pub struct Error(ErrorKind);

#[derive(Clone, Debug, Diagnostic, Error)]
#[diagnostic(transparent)]
pub(crate) enum ErrorKind {
    #[error("syntax error")]
    Parse(#[from] parse::Error),
    #[error("name error")]
    Resolve(#[from] resolve::Error),
    #[error("type error")]
    Type(#[from] typeck::Error),
    #[error("validation error")]
    Validate(#[from] validate::Error),
}

#[derive(Default)]
pub struct PackageStore {
    units: IndexMap<PackageId, CompileUnit>,
    next_id: PackageId,
}

impl PackageStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
    pub fn get_entry_expr(&self, id: PackageId) -> Option<&hir::Expr> {
        self.get(id).and_then(|unit| unit.package.entry.as_ref())
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
    sources: SourceMap,
) -> CompileUnit {
    let (mut package, parse_errors) = parse_all(&sources);
    let mut assigner = AstAssigner::new();
    assigner.visit_package(&mut package);

    let dependencies: Vec<_> = dependencies.into_iter().collect();
    let (resolutions, resolve_errors) = resolve_all(store, dependencies.iter().copied(), &package);
    let (tys, ty_errors) = typeck_all(store, dependencies.iter().copied(), &package, &resolutions);
    let validate_errors = validate(&package);
    let mut lowerer = Lowerer::new();
    let package = lowerer.with(&resolutions, &tys).lower_package(&package);

    let errors = parse_errors
        .into_iter()
        .map(Into::into)
        .chain(resolve_errors.into_iter().map(Into::into))
        .chain(ty_errors.into_iter().map(Into::into))
        .chain(validate_errors.into_iter().map(Into::into))
        .map(Error)
        .collect();

    CompileUnit {
        package,
        assigner: lowerer.into_assigner(),
        sources,
        errors,
    }
}

#[must_use]
pub fn std() -> CompileUnit {
    let sources = SourceMap::new(
        [
            (
                "arrays.qs".into(),
                include_str!("../../../library/arrays.qs").into(),
            ),
            (
                "canon.qs".into(),
                include_str!("../../../library/canon.qs").into(),
            ),
            (
                "convert.qs".into(),
                include_str!("../../../library/convert.qs").into(),
            ),
            (
                "core.qs".into(),
                include_str!("../../../library/core.qs").into(),
            ),
            (
                "diagnostics.qs".into(),
                include_str!("../../../library/diagnostics.qs").into(),
            ),
            (
                "internal.qs".into(),
                include_str!("../../../library/internal.qs").into(),
            ),
            (
                "intrinsic.qs".into(),
                include_str!("../../../library/intrinsic.qs").into(),
            ),
            (
                "math.qs".into(),
                include_str!("../../../library/math.qs").into(),
            ),
            (
                "qir.qs".into(),
                include_str!("../../../library/qir.qs").into(),
            ),
            (
                "random.qs".into(),
                include_str!("../../../library/random.qs").into(),
            ),
        ],
        "".into(),
    );

    compile(&PackageStore::new(), [], sources)
}

fn parse_all(sources: &SourceMap) -> (ast::Package, Vec<parse::Error>) {
    let mut namespaces = Vec::new();
    let mut errors = Vec::new();
    for source in &sources.sources {
        let (source_namespaces, source_errors) = parse::namespaces(&source.contents);
        for mut namespace in source_namespaces {
            Offsetter(source.offset).visit_namespace(&mut namespace);
            namespaces.push(namespace);
        }

        append_parse_errors(&mut errors, source.offset, source_errors);
    }

    let entry = if let Some(entry_source) = &sources.entry {
        let (mut entry, entry_errors) = parse::expr(&entry_source.contents);
        Offsetter(entry_source.offset).visit_expr(&mut entry);
        append_parse_errors(&mut errors, entry_source.offset, entry_errors);
        Some(entry)
    } else {
        None
    };

    let package = ast::Package {
        id: ast::NodeId::default(),
        namespaces,
        entry,
    };

    (package, errors)
}

fn resolve_all(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &ast::Package,
) -> (Resolutions, Vec<resolve::Error>) {
    let mut globals = resolve::GlobalTable::new();
    globals.add_local_package(package);

    for id in dependencies {
        let unit = store
            .get(id)
            .expect("dependency should be in package store before compilation");
        globals.add_external_package(id, &unit.package);
    }

    let mut resolver = globals.into_resolver();
    resolver.visit_package(package);
    resolver.into_resolutions()
}

fn typeck_all(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &ast::Package,
    resolutions: &Resolutions,
) -> (Tys, Vec<typeck::Error>) {
    let mut globals = typeck::GlobalTable::new();
    globals.add_local_package(resolutions, package);

    for id in dependencies {
        let unit = store
            .get(id)
            .expect("dependency should be added to package store before compilation");
        globals.add_external_package(id, &unit.package);
    }

    let mut checker = globals.into_checker();
    checker.check_package(resolutions, package);
    checker.into_tys()
}

fn append_parse_errors(errors: &mut Vec<parse::Error>, offset: usize, other: Vec<parse::Error>) {
    for error in other {
        errors.push(error.with_offset(offset));
    }
}

fn with_offset(span: &SourceSpan, f: impl FnOnce(usize) -> usize) -> SourceSpan {
    SourceSpan::new(f(span.offset()).into(), span.len().into())
}
