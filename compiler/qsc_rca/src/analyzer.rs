// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    cycle_detection::detect_specializations_with_cycles,
    rca::{analyze_package, analyze_specialization_with_cyles},
    scaffolding::PackageStoreScaffolding,
    PackageStoreComputeProperties,
};
use qsc_fir::fir::{PackageId, PackageStore};

/// A runtime capabilities analyzer.
#[derive(Default)]
pub struct Analyzer {
    compute_properties: PackageStoreComputeProperties,
}

impl Analyzer {
    /// Creates a new runtime capabilities analyzer for a package store. It analyses the provided package so this is a
    /// computationally intensive operation.
    pub fn new(package_store: &PackageStore) -> Self {
        let mut scaffolding = PackageStoreScaffolding::default();
        scaffolding.initialize_packages(package_store);

        // First, we need to analyze the callable specializations with cycles. Otherwise, we cannot safely analyze the
        // rest of the items without causing an infinite analysis loop.
        for (package_id, package) in package_store {
            let specializations_with_cycles =
                detect_specializations_with_cycles(package_id, package);
            specializations_with_cycles
                .iter()
                .for_each(|specialization_id| {
                    analyze_specialization_with_cyles(
                        *specialization_id,
                        package_store,
                        &mut scaffolding,
                    )
                });
        }

        // Now we can safely analyze the rest of the items.
        for (package_id, _) in package_store {
            analyze_package(package_id, package_store, &mut scaffolding);
        }
        Self {
            compute_properties: PackageStoreComputeProperties::default(),
        }
    }

    pub fn get_package_store_compute_properties(&self) -> &PackageStoreComputeProperties {
        &self.compute_properties
    }

    pub fn update_package_compute_properties(
        &mut self,
        _package_id: PackageId,
        _package_store: &PackageStore,
    ) {
        // TODO (cesarzc): Implement.
    }
}
