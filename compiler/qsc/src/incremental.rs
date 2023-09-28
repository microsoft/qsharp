// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, core, std};
use miette::Diagnostic;
use qsc_frontend::{
    compile::{OpenPackageStore, PackageStore, SourceMap, TargetProfile},
    error::WithSource,
    incremental::Increment,
};
use qsc_hir::hir::PackageId;
use qsc_passes::{PackageType, PassContext};

/// An incremental Q# compiler.
pub struct Compiler {
    /// A package store that contains the current, mutable, `CompileUnit`
    /// as well as all its immutable dependencies.
    store: OpenPackageStore,
    /// The ID of the source package. The source package
    /// is made up of the initial sources passed in when creating the compiler.
    source_package_id: PackageId,
    /// Context for passes that is reused across incremental compilations.
    passes: PassContext,
    /// The frontend incremental compiler.
    frontend: qsc_frontend::incremental::Compiler,
}

/// An incremental compiler error.
pub type Errors = Vec<WithSource<compile::Error>>;

impl Compiler {
    /// Creates a new incremental compiler, compiling the passed in sources.
    /// # Errors
    /// If compiling the sources fails, compiler errors are returned.
    pub fn new(
        include_std: bool,
        sources: SourceMap,
        package_type: PackageType,
        target: TargetProfile,
    ) -> Result<Self, Errors> {
        let core = core();
        let mut store = PackageStore::new(core);
        let mut dependencies = Vec::new();
        if include_std {
            let std = std(&store, target);
            let id = store.insert(std);
            dependencies.push(id);
        }

        let (unit, errors) = compile(&store, &dependencies, sources, package_type, target);
        if !errors.is_empty() {
            return Err(into_errors_with_source(errors, &unit.sources));
        }

        let source_package_id = store.insert(unit);
        dependencies.push(source_package_id);

        let frontend = qsc_frontend::incremental::Compiler::new(&store, dependencies, target);
        let store = store.open();

        Ok(Self {
            store,
            source_package_id,
            frontend,
            passes: PassContext::new(target),
        })
    }

    /// Compiles Q# fragments. Fragments are Q# code that can contain
    /// top-level statements as well as namespaces. A notebook cell
    /// or an interpreter entry is an example of fragments.
    ///
    /// This method returns the AST and HIR packages that were created as a result of
    /// the compilation, however it does *not* update the current compilation.
    ///
    /// The caller can use the returned packages to perform passes,
    /// get information about the newly added items, or do other modifications.
    /// It is then the caller's responsibility to merge
    /// these packages into the current `CompileUnit` using the `update()` method.
    pub fn compile_fragments(
        &mut self,
        source_name: &str,
        source_contents: &str,
    ) -> Result<Increment, Errors> {
        let (core, unit) = self.store.get_open_mut();

        let mut increment = self
            .frontend
            .compile_fragments(unit, source_name, source_contents)
            .map_err(into_errors)?;

        let pass_errors = self.passes.run_default_passes(
            &mut increment.hir,
            &mut unit.assigner,
            core,
            PackageType::Lib,
        );

        if !pass_errors.is_empty() {
            return Err(into_errors_with_source(pass_errors, &unit.sources));
        }

        Ok(increment)
    }

    /// Compiles an entry expression.
    ///
    /// This method returns the AST and HIR packages that were created as a result of
    /// the compilation, however it does *not* update the current compilation.
    ///
    /// The caller can use the returned packages to perform passes,
    /// get information about the newly added items, or do other modifications.
    /// It is then the caller's responsibility to merge
    /// these packages into the current `CompileUnit` using the `update()` method.
    pub fn compile_expr(&mut self, expr: &str) -> Result<Increment, Errors> {
        let (core, unit) = self.store.get_open_mut();

        let mut increment = self
            .frontend
            .compile_expr(unit, "<entry>", expr)
            .map_err(into_errors)?;

        let pass_errors = self.passes.run_default_passes(
            &mut increment.hir,
            &mut unit.assigner,
            core,
            PackageType::Lib,
        );

        if !pass_errors.is_empty() {
            return Err(into_errors_with_source(pass_errors, &unit.sources));
        }

        Ok(increment)
    }

    /// Updates the current compilation with the AST and HIR packages,
    /// and any associated context, returned from a previous incremental compilation.
    pub fn update(&mut self, new: Increment) {
        let (_, unit) = self.store.get_open_mut();

        // Extend the AST and HIR packages
        unit.ast.package.extend(new.ast.package);
        unit.package.extend(new.hir);

        // The new `Increment` will contain the names and tys
        // from the original package as well, so just
        // replace the current tables instead of extending.
        unit.ast.names = new.ast.names;
        unit.ast.tys = new.ast.tys;
    }

    /// Returns a reference to the underlying package store.
    #[must_use]
    pub fn package_store(&self) -> &PackageStore {
        self.store.package_store()
    }

    /// Returns ID of the current `CompileUnit`.
    #[must_use]
    pub fn package_id(&self) -> PackageId {
        self.store.open_package_id()
    }

    /// Returns the ID of the source package created from the sources
    /// passed in during inital creation.
    #[must_use]
    pub fn source_package_id(&self) -> PackageId {
        self.source_package_id
    }

    /// Consumes the incremental compiler and returns an immutable package store.
    /// This method can be used to finalize the compilation.
    #[must_use]
    pub fn into_package_store(self) -> (PackageStore, PackageId) {
        self.store.into_package_store()
    }
}

fn into_errors_with_source<T>(errors: Vec<T>, sources: &SourceMap) -> Errors
where
    compile::Error: From<T>,
{
    errors
        .into_iter()
        .map(|e| WithSource::from_map(sources, e.into()))
        .collect()
}

fn into_errors<T>(errors: Vec<WithSource<T>>) -> Errors
where
    compile::Error: From<T>,
    T: Diagnostic,
{
    errors
        .into_iter()
        .map(qsc_frontend::error::WithSource::into_with_source)
        .collect()
}
