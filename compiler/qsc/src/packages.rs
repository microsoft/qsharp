// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{compile, hir::PackageId, PackageStore, TargetCapabilityFlags};
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_frontend::compile::SourceMap;
use qsc_passes::PackageType;
use qsc_project::PackageGraphSources;
use rustc_hash::FxHashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// A program that is ready to be built -- dependencies have all been built, and the user code is ready.
#[derive(Debug)]
pub struct BuildableProgram {
    pub store: PackageStore,
    pub user_code: qsc_project::PackageInfo,
    pub user_code_dependencies: Vec<(PackageId, Option<Arc<str>>)>,
    pub dependency_errors: Vec<compile::Error>,
}

impl BuildableProgram {
    /// Compiles all dependencies, populates the `PackageStore`, and prepares a `BuildableProgram` for the user code.
    #[must_use]
    pub fn new(
        capabilities: TargetCapabilityFlags,
        package_graph_sources: PackageGraphSources,
    ) -> Self {
        prepare_package_store(capabilities, package_graph_sources)
    }
}

/// Given a program config, prepare the package store by compiling all dependencies in the correct order and inserting them.
#[must_use]
pub fn prepare_package_store(
    capabilities: TargetCapabilityFlags,
    package_graph_sources: PackageGraphSources,
) -> BuildableProgram {
    let (std_id, mut package_store) = crate::compile::package_store_with_stdlib(capabilities);

    let mut canonical_package_identifier_to_package_id_mapping = FxHashMap::default();

    let (ordered_packages, user_code) = package_graph_sources
        .compilation_order()
        .expect("dependency cycle detected in package graph -- this should have been caught by the target scenario");

    let mut dependency_errors = Vec::new();
    for (package_name, package_to_compile) in ordered_packages {
        let sources: Vec<(Arc<str>, Arc<str>)> =
            package_to_compile.sources.into_iter().collect::<Vec<_>>();
        let source_map = SourceMap::new(sources, None);
        let dependencies = package_to_compile
            .dependencies
            .iter()
            .filter_map(|(alias, key)| {
                canonical_package_identifier_to_package_id_mapping
                    .get(key)
                    .copied()
                    .map(|pkg| (alias.clone(), pkg))
            })
            .collect::<FxHashMap<_, _>>();
        let dependencies = dependencies
            .iter()
            .map(|(alias, b)| (*b, Some(alias.clone())))
            .chain(std::iter::once((std_id, None)))
            .collect::<Vec<_>>();
        let (compile_unit, mut this_errors) = compile::compile(
            &package_store,
            &dependencies[..],
            source_map,
            PackageType::Lib,
            capabilities,
            LanguageFeatures::from_iter(package_to_compile.language_features),
        );

        let package_id = package_store.insert(compile_unit);
        if !this_errors.is_empty() {
            dependency_errors.append(&mut this_errors);
        }

        canonical_package_identifier_to_package_id_mapping.insert(package_name, package_id);
    }

    let user_code_dependencies = user_code
        .dependencies
        .iter()
        .filter_map(|(alias, key)| {
            canonical_package_identifier_to_package_id_mapping
                .get(key)
                .copied()
                .map(|pkg| (pkg, Some(alias.clone())))
        })
        .chain(std::iter::once((std_id, None)))
        .collect::<Vec<_>>();

    BuildableProgram {
        store: package_store,
        dependency_errors,
        user_code,
        user_code_dependencies,
    }
}
