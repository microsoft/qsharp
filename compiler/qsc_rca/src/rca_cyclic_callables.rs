// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::scaffolding::PackageStoreComputeProperties;
use qsc_fir::fir::{PackageId, PackageStore};

pub struct CyclicCallablesAnalyzer<'a> {
    package_store: &'a PackageStore,
    package_store_compute_properties: PackageStoreComputeProperties,
}

impl<'a> CyclicCallablesAnalyzer<'a> {
    pub fn new(
        package_store: &'a PackageStore,
        package_store_compute_properties: PackageStoreComputeProperties,
    ) -> Self {
        Self {
            package_store,
            package_store_compute_properties,
        }
    }

    pub fn analyze_all(self) -> PackageStoreComputeProperties {
        // TODO (cesarzc): implement.
        self.package_store_compute_properties
    }

    pub fn analyze_package(self, package_id: PackageId) -> PackageStoreComputeProperties {
        // TODO (cesarzc): implement.
        self.package_store_compute_properties
    }
}
