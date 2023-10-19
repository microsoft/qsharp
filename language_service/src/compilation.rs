// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use log::trace;
use qsc::{
    ast,
    compile::{self, Error},
    hir::{self, ItemId, PackageId},
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

        trace!("compiled package: {:?}", package_store.get(package_id));
        trace!("errors: {:?}", &errors);

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
    fn find_item(&self, item_id: &hir::ItemId) -> (&hir::Item, &CompileUnit);
    fn find_ty(&self, expr_id: ast::NodeId) -> Option<&hir::ty::Ty>;
}

impl Lookup for Compilation {
    fn find_item(&self, id: &ItemId) -> (&hir::Item, &CompileUnit) {
        let package_id = id.package.unwrap_or(self.current);
        let unit = self
            .package_store
            .get(package_id)
            .expect("package id must be found in store");
        (
            unit.package
                .items
                .get(id.item)
                .expect("item must exist in store"), // TODO: we're hitting a panic here
            // operation Main() : Result {
            //     ## TRIGGER COMPLETION HERE ##
            //     use q = Qubit();
            //     X(q);
            //     Microsoft.Quantum.Diagnostics.DumpMachine();
            //     let r = M(q);
            //     Message($"The result of the measurement is {r}");
            //     Reset(q);
            //     r
            // }
            // Main()
            unit,
        )
    }

    fn find_ty(&self, expr_id: ast::NodeId) -> Option<&hir::ty::Ty> {
        self.current_unit().ast.tys.terms.get(expr_id)
    }
}
