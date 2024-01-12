// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA)...

mod rca;
#[cfg(test)]
mod tests;

use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{ExprId, LocalItemId, PackageId, PackageStore, PatId};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use rca::analyze_package_and_update_compute_props;

/// The compute properties of a package store.
#[derive(Debug)]
pub struct PackageStoreComputeProps(IndexMap<PackageId, PackageComputeProps>);

impl PackageStoreComputeProps {
    pub fn get(&self, id: PackageId) -> &PackageComputeProps {
        self.0
            .get(id)
            .expect("package compute properties does not exist")
    }

    pub fn get_mut(&mut self, id: PackageId) -> &mut PackageComputeProps {
        self.0
            .get_mut(id)
            .expect("package compute properties does not exist")
    }

    pub fn with_empty_packages(fir_store: &PackageStore) -> Self {
        let mut package_store_compute_props = IndexMap::new();
        for (id, _) in fir_store.iter() {
            package_store_compute_props.insert(id, PackageComputeProps::default());
        }
        Self(package_store_compute_props)
    }
}

/// The compute properties of a package.
#[derive(Debug)]
pub struct PackageComputeProps {
    pub items: IndexMap<LocalItemId, ItemComputeProps>,
}

impl Default for PackageComputeProps {
    fn default() -> Self {
        Self {
            items: IndexMap::new(),
        }
    }
}

impl PackageComputeProps {
    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// The compute properties of an item.
#[derive(Debug)]
pub enum ItemComputeProps {
    NonCallable,
    Callable(CallableComputeProps),
}

/// The compute properties of a callable.
#[derive(Debug)]
pub struct CallableComputeProps {
    /// The compute properties of the callable body.
    pub body: AppsTbl,
    /// The compute properties of the adjoint specialization.
    pub adj: Option<AppsTbl>,
    /// The compute properties of the controlled specialization.
    pub ctl: Option<AppsTbl>,
    /// The compute properties of the controlled adjoint specialization.
    pub ctl_adj: Option<AppsTbl>,
}

/// The compute properties associated to an application table.
#[derive(Debug)]
pub struct AppsTbl {
    /// The inherent compute properties of all applications.
    pub inherent: ComputeProps,
    /// The compute properties for each dynamic parameter in the application.
    pub dynamic_params: Vec<ComputeProps>,
}

/// The tracked compute properties.
#[derive(Debug)]
pub struct ComputeProps {
    pub rt_caps: RuntimeCapabilityFlags,
    pub quantum_source: Option<QuantumSource>,
}

/// A quantum source.
#[derive(Debug)]
pub enum QuantumSource {
    Intrinsic,
    InputParam(PatId),
    Expr(ExprId),
}

/// The runtime capabilities analyzer.
#[derive(Debug)]
pub struct Analyzer {
    /// The compute properties of the package store.
    compute_props: PackageStoreComputeProps,
    /// The ID of the opened package.
    _open_package_id: PackageId,
}

impl Analyzer {
    pub fn get_package_store_compute_props(&self) -> &PackageStoreComputeProps {
        &self.compute_props
    }

    pub fn new(fir_store: &PackageStore, open_package_id: PackageId) -> Self {
        let mut compute_props = PackageStoreComputeProps::with_empty_packages(fir_store);

        // Analyze each package in the store.
        for (package_id, _) in fir_store.iter() {
            analyze_package_and_update_compute_props(package_id, fir_store, &mut compute_props);
        }
        Self {
            compute_props,
            _open_package_id: open_package_id,
        }
    }
}
