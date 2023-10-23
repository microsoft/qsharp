use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{ItemKind, Package, PackageId, PackageStore};

use crate::{CallableCapabilities, PackageCapabilities, StoreCapabilities};

pub struct Analyzer {
    stores: IndexMap<PackageId, PackageCapabilities>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            stores: IndexMap::new(),
        }
    }

    pub fn analyze_runtime_capabilities(&mut self, store: &PackageStore) -> StoreCapabilities {
        self.initialize_from_store(store);
        StoreCapabilities(self.stores.drain().collect())
    }

    fn initialize_from_store(&mut self, store: &PackageStore) {
        for (id, package) in store.0.iter() {
            let mut package_capabilities = PackageCapabilities::new();
            self.initialize_from_package(&mut package_capabilities, package);
            self.stores.insert(id, package_capabilities);
        }
    }

    fn initialize_from_package(
        &mut self,
        package_capabilities: &mut PackageCapabilities,
        package: &Package,
    ) {
        for (id, item) in package.items.iter() {
            let initial_capabilities = match item.kind {
                ItemKind::Callable(_) => Some(CallableCapabilities::new()),
                _ => None,
            };
            package_capabilities
                .callables
                .insert(id, initial_capabilities);
        }
    }
}
