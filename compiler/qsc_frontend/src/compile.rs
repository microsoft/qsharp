// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    diagnostic::OffsetError,
    id::{AstAssigner, HirAssigner},
    lower::Lowerer,
    parse,
    resolve::{self, Link, Resolutions},
    typeck::{self, Tys},
    validate::{self, validate},
};
use miette::Diagnostic;
use qsc_ast::{ast, mut_visit::MutVisitor as AstMutVisitor, visit::Visitor as AstVisitor};
use qsc_data_structures::span::Span;
use qsc_hir::{hir, visit::Visitor as HirVisitor};
use std::{
    collections::{hash_map::Iter, HashMap},
    fmt::{self, Debug, Display, Formatter},
};
use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct CompileUnit {
    pub package: hir::Package,
    pub context: Context,
}

#[derive(Debug)]
pub struct Context {
    assigner: HirAssigner,
    resolutions: Resolutions<hir::NodeId>,
    tys: Tys<hir::NodeId>,
    errors: Vec<Error>,
    offsets: Vec<usize>,
}

impl Context {
    pub fn assigner_mut(&mut self) -> &mut HirAssigner {
        &mut self.assigner
    }

    #[must_use]
    pub fn resolutions(&self) -> &Resolutions<hir::NodeId> {
        &self.resolutions
    }

    pub fn resolutions_mut(&mut self) -> &mut Resolutions<hir::NodeId> {
        &mut self.resolutions
    }

    #[must_use]
    pub fn tys(&self) -> &Tys<hir::NodeId> {
        &self.tys
    }

    pub fn tys_mut(&mut self) -> &mut Tys<hir::NodeId> {
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
    Type(typeck::Error),
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
    pub fn get_entry_expr(&self, id: PackageId) -> Option<&hir::Expr> {
        self.get(id).and_then(|unit| unit.package.entry.as_ref())
    }

    #[must_use]
    pub fn get_resolutions(&self, id: PackageId) -> Option<&Resolutions<hir::NodeId>> {
        self.get(id).map(|unit| unit.context.resolutions())
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

impl AstMutVisitor for Offsetter {
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
    let mut assigner = AstAssigner::new();
    AstMutVisitor::visit_package(&mut assigner, &mut package);

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
    errors.extend(ty_errors.into_iter().map(|e| Error(ErrorKind::Type(e))));
    errors.extend(
        validate_errors
            .into_iter()
            .map(|e| Error(ErrorKind::Validate(e))),
    );

    let mut lowerer = Lowerer::new();
    let package = lowerer.lower_package(&package);

    let resolutions = resolutions
        .into_iter()
        .filter_map(|(id, link)| {
            let id = lowerer.get_id(id)?;
            let link = match link {
                Link::Internal(node) => Link::Internal(
                    lowerer
                        .get_id(node)
                        .expect("lowered node should not resolve to deleted node"),
                ),
                Link::External(package, node) => Link::External(package, node),
            };
            Some((id, link))
        })
        .collect();

    let tys = tys
        .into_iter()
        .filter_map(|(id, ty)| lowerer.get_id(id).map(|id| (id, ty)))
        .collect();

    CompileUnit {
        package,
        context: Context {
            assigner: lowerer.into_assigner(),
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
) -> (ast::Package, Vec<OffsetError<parse::Error>>, Vec<usize>) {
    let mut namespaces = Vec::new();
    let mut errors = Vec::new();
    let mut offsets = Vec::new();
    let mut offset = 0;

    for source in sources {
        let source = source.as_ref();
        let (source_namespaces, source_errors) = parse::namespaces(source);
        for mut namespace in source_namespaces {
            AstMutVisitor::visit_namespace(&mut Offsetter(offset), &mut namespace);
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
        AstMutVisitor::visit_expr(&mut Offsetter(offset), &mut entry);
        append_errors(&mut errors, offset, entry_errors);
        offsets.push(offset);
        Some(entry)
    };

    (ast::Package::new(namespaces, entry), errors, offsets)
}

fn resolve_all(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &ast::Package,
) -> (Resolutions<ast::NodeId>, Vec<resolve::Error>) {
    let mut globals = resolve::GlobalTable::new();
    AstVisitor::visit_package(&mut globals, package);

    for dependency in dependencies {
        let unit = store
            .get(dependency)
            .expect("dependency should be in package store before compilation");
        globals.set_package(dependency);
        HirVisitor::visit_package(&mut globals, &unit.package);
    }

    let mut resolver = globals.into_resolver();
    AstVisitor::visit_package(&mut resolver, package);
    resolver.into_resolutions()
}

fn typeck_all(
    store: &PackageStore,
    dependencies: impl IntoIterator<Item = PackageId>,
    package: &ast::Package,
    resolutions: &Resolutions<ast::NodeId>,
) -> (Tys<ast::NodeId>, Vec<typeck::Error>) {
    let mut globals = typeck::GlobalTable::new(resolutions);
    AstVisitor::visit_package(&mut globals, package);

    for dependency in dependencies {
        let unit = store
            .get(dependency)
            .expect("dependency should be added to package store before compilation");
        globals.set_package(dependency);
        HirVisitor::visit_package(&mut globals, &unit.package);
    }

    let mut checker = globals.into_checker();
    AstVisitor::visit_package(&mut checker, package);
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
