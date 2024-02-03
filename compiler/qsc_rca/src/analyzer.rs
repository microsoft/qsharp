// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    scaffolding::{PackageScaffolding, PackageStoreScaffolding},
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
        let mut _scaffolding = PackageStoreScaffolding::default();
        // TODO (cesarzc): implement properly.
        //                 insert cycled specs first and then pass onto RCA.
        Self {
            compute_properties: PackageStoreComputeProperties::default(),
        }
    }

    pub fn get_package_store_compute_properties(&self) -> &PackageStoreComputeProperties {
        &self.compute_properties
    }

    pub fn update_package_compute_properties(
        &mut self,
        package_id: PackageId,
        package_store: &PackageStore,
    ) {
        // TODO (cesarzc): Implement.
    }
}
