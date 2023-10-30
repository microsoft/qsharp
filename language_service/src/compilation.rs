// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::trace;
use qsc::{
    ast::{self},
    compile::{self, Error},
    hir::{self, PackageId},
    incremental::Compiler,
    CompileUnit, PackageStore, PackageType, SourceMap, TargetProfile,
};
use std::iter::successors;

/// Represents an immutable compilation state that can be used
/// to implement language service features.
/// Can be a single Q# source file or a notebook with Q# cells
/// For notebooks the compilation represents the entire notebook
pub(crate) struct Compilation {
    /// Package store, containing the current package and all dependencies
    pub package_store: PackageStore,
    /// ID of the current package
    /// For standalone Q# source files, `Package` will only contain one `Source`
    /// For notebooks, each `Source` in the `Package` represents a cell
    pub current: PackageId,
    pub errors: Vec<Error>,
    pub kind: CompilationKind,
}

pub(crate) enum CompilationKind {
    OpenDocument,
    Notebook,
}

impl Compilation {
    /// Creates a new `Compilation` by compiling source from a single open document.
    pub(crate) fn new_open_document(
        source_name: &str,
        source_contents: &str,
        package_type: PackageType,
        target_profile: TargetProfile,
    ) -> Self {
        trace!("compiling document {source_name}");
        // Source map only contains the current document.
        let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);

        let mut package_store = PackageStore::new(compile::core());
        let std_package_id = package_store.insert(compile::std(&package_store, target_profile));

        let (unit, errors) = compile::compile(
            &package_store,
            &[std_package_id],
            source_map,
            package_type,
            target_profile,
        );

        let package_id = package_store.insert(unit);

        Self {
            package_store,
            current: package_id,
            errors,
            kind: CompilationKind::OpenDocument,
        }
    }

    /// Creates a new `Compilation` by compiling sources from notebook cells.
    pub(crate) fn new_notebook<'a, I>(cells: I) -> Self
    where
        I: Iterator<Item = (&'a str, &'a str)>,
    {
        trace!("compiling notebook");
        let mut compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            TargetProfile::Full,
        )
        .expect("expected incremental compiler creation to succeed");

        let mut errors = Vec::new();
        for (name, contents) in cells {
            trace!("compiling cell {name}");
            let increment = compiler
                .compile_fragments(name, contents, |cell_errors| {
                    errors.extend(cell_errors);
                    Ok(()) // accumulate errors without failing
                })
                .expect("compile_fragments_acc_errors should not fail");

            compiler.update(increment);
        }

        let (package_store, package_id) = compiler.into_package_store();

        Self {
            package_store,
            current: package_id,
            errors,
            kind: CompilationKind::Notebook,
        }
    }

    pub fn current_unit(&self) -> &CompileUnit {
        self.package_store
            .get(self.current)
            .expect("expected to find current package")
    }

    /// Regenerates the compilation with the same sources but the passed in configuration options.
    pub fn recompile(&mut self, package_type: PackageType, target_profile: TargetProfile) {
        let sources = self.sources();

        let new = match self.kind {
            CompilationKind::OpenDocument => {
                assert!(sources.len() == 1);
                Self::new_open_document(sources[0].0, sources[0].1, package_type, target_profile)
            }
            CompilationKind::Notebook => Self::new_notebook(sources.into_iter()),
        };
        self.package_store = new.package_store;
        self.current = new.current;
        self.errors = new.errors;
    }

    /// Returns all initial sources that were used to create the compilation.
    fn sources(&self) -> Vec<(&str, &str)> {
        let sources = &self.current_unit().sources;

        // TODO: There's got to be a cleaner way
        successors(sources.find_by_offset(0), |last| {
            sources
                .find_by_offset(
                    u32::try_from(last.contents.len()).expect("source contents should fit in u32")
                        + 1,
                )
                .and_then(|s| {
                    if s.offset == last.offset {
                        None
                    } else {
                        Some(s)
                    }
                })
        })
        .map(|s| (s.name.as_ref(), s.contents.as_ref()))
        .collect()
    }
}

pub(crate) trait Lookup {
    // TODO: rename all these to resolve_* or something
    fn find_ty(&self, expr_id: ast::NodeId) -> Option<&hir::ty::Ty>;
    fn resolve_item_relative_to_user_package(
        &self,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId);
    fn resolve_item_res(
        &self,
        local_package_id: PackageId,
        res: &hir::Res,
    ) -> (&hir::Item, hir::ItemId);
    fn resolve_item(
        &self,
        local_package_id: PackageId,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId);
}

impl Lookup for Compilation {
    fn find_ty(&self, expr_id: ast::NodeId) -> Option<&hir::ty::Ty> {
        self.current_unit().ast.tys.terms.get(expr_id)
    }

    /// Returns the hir `Item` node referred to by `item_id`,
    /// along with the `Package` and `PackageId` for the package
    /// that it was found in.
    fn resolve_item_relative_to_user_package(
        &self,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        self.resolve_item(self.current, item_id)
    }

    /// Returns the hir `Item` node referred to by `res`.
    /// `Res`s can resolve to external packages, and the references
    /// are relative, so here we also need the
    /// local `PackageId` that the `res` itself came from.
    fn resolve_item_res(
        &self,
        local_package_id: PackageId,
        res: &hir::Res,
    ) -> (&hir::Item, hir::ItemId) {
        match res {
            hir::Res::Item(item_id) => {
                let (item, _, resolved_item_id) = self.resolve_item(local_package_id, item_id);
                (item, resolved_item_id)
            }
            _ => panic!("expected to find item"),
        }
    }

    /// Returns the hir `Item` node referred to by `item_id`.
    /// `ItemId`s can refer to external packages, and the references
    /// are relative, so here we also need the local `PackageId`
    /// that the `ItemId` originates from.
    fn resolve_item(
        &self,
        local_package_id: PackageId,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        // If the `ItemId` contains a package id, use that.
        // Lack of a package id means the item is in the
        // same package as the one this `ItemId` reference
        // came from. So use the local package id passed in.
        let package_id = item_id.package.unwrap_or(local_package_id);
        let package = &self
            .package_store
            .get(package_id)
            .expect("package should exist in store")
            .package;
        (
            package
                .items
                .get(item_id.item)
                .expect("item id should exist"),
            package,
            hir::ItemId {
                package: Some(package_id),
                item: item_id.item,
            },
        )
    }
}
