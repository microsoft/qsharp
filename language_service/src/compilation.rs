// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::trace;
use qsc::{
    ast::{self},
    compile::{self, Error},
    hir::{self, PackageId},
    incremental::Compiler,
    line_column::{Encoding, Position},
    resolve,
    target::Profile,
    CompileUnit, PackageStore, PackageType, SourceMap,
};
use std::sync::Arc;

/// Represents an immutable compilation state that can be used
/// to implement language service features.
#[derive(Debug)]
pub(crate) struct Compilation {
    /// Package store, containing the current package and all its dependencies.
    pub package_store: PackageStore,
    /// The `PackageId` of the user package. User code
    /// is non-library code, i.e. all code except the std and core libs.
    pub user_package_id: PackageId,
    pub errors: Vec<Error>,
    pub kind: CompilationKind,
}

#[derive(Debug)]
pub(crate) enum CompilationKind {
    /// An open Q# project.
    /// In an `OpenProject` compilation, the user package contains
    /// one or more sources, and a target profile.
    OpenProject,
    /// A Q# notebook. In a notebook compilation, the user package
    /// contains multiple `Source`s, with each source corresponding
    /// to a cell.
    Notebook,
}

impl Compilation {
    /// Creates a new `Compilation` by compiling sources.
    pub(crate) fn new(
        sources: &[(Arc<str>, Arc<str>)],
        package_type: PackageType,
        target_profile: Profile,
    ) -> Self {
        if sources.len() == 1 {
            trace!("compiling single-file document {}", sources[0].0);
        } else {
            trace!("compiling package with {} sources", sources.len());
        }

        let source_map = SourceMap::new(sources.iter().map(|(x, y)| (x.clone(), y.clone())), None);

        let mut package_store = PackageStore::new(compile::core());
        let std_package_id =
            package_store.insert(compile::std(&package_store, target_profile.into()));

        let (unit, errors) = compile::compile(
            &package_store,
            &[std_package_id],
            source_map,
            package_type,
            target_profile.into(),
        );

        let package_id = package_store.insert(unit);

        Self {
            package_store,
            user_package_id: package_id,
            errors,
            kind: CompilationKind::OpenProject,
        }
    }

    /// Creates a new `Compilation` by compiling sources from notebook cells.
    pub(crate) fn new_notebook<I>(cells: I, target_profile: Profile) -> Self
    where
        I: Iterator<Item = (Arc<str>, Arc<str>)>,
    {
        trace!("compiling notebook");
        let mut compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            target_profile.into(),
        )
        .expect("expected incremental compiler creation to succeed");

        let mut errors = Vec::new();
        for (name, contents) in cells {
            trace!("compiling cell {name}");
            let increment = compiler
                .compile_fragments(&name, &contents, |cell_errors| {
                    errors.extend(cell_errors);
                    Ok(()) // accumulate errors without failing
                })
                .expect("compile_fragments_acc_errors should not fail");

            compiler.update(increment);
        }

        let (package_store, package_id) = compiler.into_package_store();

        Self {
            package_store,
            user_package_id: package_id,
            errors,
            kind: CompilationKind::Notebook,
        }
    }

    /// Gets the `CompileUnit` associated with user (non-library) code.
    pub fn user_unit(&self) -> &CompileUnit {
        self.package_store
            .get(self.user_package_id)
            .expect("expected to find user package")
    }

    /// Maps a source position from the user package
    /// to a package (`SourceMap`) offset.
    pub(crate) fn source_position_to_package_offset(
        &self,
        source_name: &str,
        source_position: Position,
        position_encoding: Encoding,
    ) -> u32 {
        let unit = self.user_unit();

        let source = unit
            .sources
            .find_by_name(source_name)
            .expect("source should exist in the user source map");

        let mut offset =
            source_position.to_utf8_byte_offset(position_encoding, source.contents.as_ref());

        let len = u32::try_from(source.contents.len()).expect("source length should fit into u32");
        if offset > len {
            // This can happen if the document contents are out of sync with the client's view.
            // we don't want to accidentally return an offset into the next file -
            // remap to the end of the current file.
            trace!(
                "offset {offset} out of bounds for {}, using end offset instead",
                source.name
            );
            offset = len;
        }

        source.offset + offset
    }

    /// Regenerates the compilation with the same sources but the passed in workspace configuration options.
    pub fn recompile(&mut self, package_type: PackageType, target_profile: Profile) {
        let sources = self
            .user_unit()
            .sources
            .iter()
            .map(|source| (source.name.clone(), source.contents.clone()));

        let new = match self.kind {
            CompilationKind::OpenProject => {
                Self::new(&sources.collect::<Vec<_>>(), package_type, target_profile)
            }
            CompilationKind::Notebook => Self::new_notebook(sources, target_profile),
        };
        self.package_store = new.package_store;
        self.user_package_id = new.user_package_id;
        self.errors = new.errors;
    }
}

pub(crate) trait Lookup {
    fn get_ty(&self, expr_id: ast::NodeId) -> Option<&hir::ty::Ty>;
    fn get_res(&self, id: ast::NodeId) -> Option<&resolve::Res>;
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
    /// Looks up the type of a node in user code
    fn get_ty(&self, id: ast::NodeId) -> Option<&hir::ty::Ty> {
        self.user_unit().ast.tys.terms.get(id)
    }

    /// Looks up the resolution of a node in user code
    fn get_res(&self, id: ast::NodeId) -> Option<&resolve::Res> {
        self.user_unit().ast.names.get(id)
    }

    /// Returns the hir `Item` node referred to by `item_id`,
    /// along with the `Package` and `PackageId` for the package
    /// that it was found in.
    fn resolve_item_relative_to_user_package(
        &self,
        item_id: &hir::ItemId,
    ) -> (&hir::Item, &hir::Package, hir::ItemId) {
        self.resolve_item(self.user_package_id, item_id)
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
