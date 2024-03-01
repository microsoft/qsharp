// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{core, cyclic_callables, scaffolding, PackageStoreComputeProperties};
use qsc_fir::fir::{PackageId, PackageStore};

/// A runtime capabilities analyzer.
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
            cyclic_callables::Analyzer::new(self.package_store, self.scaffolding);
        let scaffolding = cyclic_callables_analyzer.analyze_all();

        // Now we can safely analyze the rest of the items.
        let core_analyzer = core::Analyzer::new(self.package_store, scaffolding);
        core_analyzer.analyze_all().into()
    }

    #[must_use]
    pub fn analyze_package(self, package_id: PackageId) -> PackageStoreComputeProperties {
        // Even when analyzing just one package we need to first analyze cyclic callables and then the rest of the items
        // to avoid an infinite analysis loop.
        let cyclic_callables_analyzer =
            cyclic_callables::Analyzer::new(self.package_store, self.scaffolding);
        let scaffolding = cyclic_callables_analyzer.analyze_package(package_id);
        let core_analyzer = core::Analyzer::new(self.package_store, scaffolding);
        core_analyzer.analyze_package(package_id).into()
    }
}
