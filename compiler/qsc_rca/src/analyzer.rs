// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    cycle_detection::detect_specializations_with_cycles,
    rca::{analyze_package, analyze_specialization_with_cyles},
    rca_core::CoreAnalyzer,
    rca_cyclic_callables::CyclicCallablesAnalyzer,
    scaffolding, PackageStoreComputeProperties,
};
use qsc_fir::fir::{PackageId, PackageStore};

/// A runtime capabilities analyzer.
#[derive(Default)]
pub struct Analyzer {
    compute_properties: PackageStoreComputeProperties,
}

impl Analyzer {
    /// Initializes a new runtime capabilities analyzer for a package store by analyzing the provided package store,
    /// which makes this a computationally intensive operation.
    #[must_use]
    pub fn init_and_analyze(package_store: &PackageStore) -> Self {
        let mut scaffolding = scaffolding::PackageStoreComputeProperties::init(package_store);

        // First, we need to analyze the callable specializations with cycles. Otherwise, we cannot safely analyze the
        // rest of the items without causing an infinite analysis loop.
        for (package_id, package) in package_store {
            let specializations_with_cycles =
                detect_specializations_with_cycles(package_id, package);
            for specialization_id in &specializations_with_cycles {
                analyze_specialization_with_cyles(
                    *specialization_id,
                    package_store,
                    &mut scaffolding,
                );
            }
        }

        // Now we can safely analyze the rest of the items.
        for (package_id, _) in package_store {
            analyze_package(package_id, package_store, &mut scaffolding);
        }

        // Once everything has been analyzed, flush everything to the package store compute properties.
        let mut compute_properties = PackageStoreComputeProperties::default();
        scaffolding.save(&mut compute_properties);
        Self { compute_properties }
    }

    #[must_use]
    pub fn get_package_store_compute_properties(&self) -> &PackageStoreComputeProperties {
        &self.compute_properties
    }

    pub fn update_package_compute_properties(
        &mut self,
        package_id: PackageId,
        package_store: &PackageStore,
    ) {
        // Clear the package being updated.
        let package_compute_properties = self
            .compute_properties
            .0
            .get_mut(package_id)
            .expect("package should exist");
        package_compute_properties.clear();

        // Re-analyze the package.
        let mut package_store_scaffolding =
            scaffolding::PackageStoreComputeProperties::init_and_populate(
                &mut self.compute_properties,
            );

        // First, analyze callables with cycles for the package being updated.
        let package = package_store.get(package_id);
        let specializations_with_cycles = detect_specializations_with_cycles(package_id, package);
        for specialization_id in &specializations_with_cycles {
            analyze_specialization_with_cyles(
                *specialization_id,
                package_store,
                &mut package_store_scaffolding,
            );
        }

        // Analyze the remaining items.
        analyze_package(package_id, package_store, &mut package_store_scaffolding);
        package_store_scaffolding.save(&mut self.compute_properties);
    }
}

pub struct RCA<'a> {
    package_store: &'a PackageStore,
    scaffolding: scaffolding::PackageStoreComputeProperties,
}

impl<'a> RCA<'a> {
    #[must_use]
    pub fn init(package_store: &'a PackageStore) -> Self {
        Self {
            package_store,
            scaffolding: scaffolding::PackageStoreComputeProperties::init(package_store),
        }
    }

    #[must_use]
    pub fn init_with_compute_properties(
        package_store: &'a PackageStore,
        package_store_compute_properties: PackageStoreComputeProperties,
    ) -> Self {
        Self {
            package_store,
            scaffolding: package_store_compute_properties.into(),
        }
    }

    #[must_use]
    pub fn analyze_all(self) -> PackageStoreComputeProperties {
        // First, we need to analyze the callable specializations with cycles. Otherwise, we cannot safely analyze the
        // rest of the items without causing an infinite analysis loop.
        let cyclic_callables_analyzer =
            CyclicCallablesAnalyzer::new(self.package_store, self.scaffolding);
        let scaffolding = cyclic_callables_analyzer.analyze_all();

        // Now we can safely analyze the rest of the items.
        let core_analyzer = CoreAnalyzer::new(self.package_store, scaffolding);
        core_analyzer.analyze_all().into()
    }

    #[must_use]
    pub fn analyze_package(self, package_id: PackageId) -> PackageStoreComputeProperties {
        // Even when analyzing just one package we need to first analyze cyclic callables and then the rest of the items
        // to avoid an infinite analysis loop.
        let cyclic_callables_analyzer =
            CyclicCallablesAnalyzer::new(self.package_store, self.scaffolding);
        let scaffolding = cyclic_callables_analyzer.analyze_package(package_id);
        let core_analyzer = CoreAnalyzer::new(self.package_store, scaffolding);
        core_analyzer.analyze_package(package_id).into()
    }
}
