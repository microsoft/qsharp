// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    cycle_detection::detect_specializations_with_cycles,
    rca::{analyze_package, analyze_specialization_with_cyles},
    scaffolding::{self},
    PackageStoreComputeProperties,
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
        let mut scaffolding = scaffolding::PackageStoreComputeProperties::default();
        scaffolding.initialize_packages(package_store);

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
        scaffolding.flush(&mut compute_properties);
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
        let mut package_store_scaffolding = scaffolding::PackageStoreComputeProperties::default();
        let package = package_store.get(package_id);
        package_store_scaffolding.take(&mut self.compute_properties);

        // First, analyze callables with cycles for the package being updated.
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
        package_store_scaffolding.flush(&mut self.compute_properties);
    }
}
