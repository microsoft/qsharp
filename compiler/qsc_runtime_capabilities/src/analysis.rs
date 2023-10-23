use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{CallableDecl, ItemKind, LocalItemId, Package, PackageId, PackageStore};

use crate::{CallableCapabilities, PackageCapabilities, StoreCapabilities};

pub struct Analyzer {
    store: IndexMap<PackageId, PackageCapabilities>,
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    pub fn new() -> Self {
        Self {
            store: IndexMap::new(),
        }
    }

    pub fn analyze_runtime_capabilities(&mut self, store: &PackageStore) -> StoreCapabilities {
        self.initialize(store);
        StoreCapabilities(self.store.drain().collect())
    }

    fn initialize(&mut self, store: &PackageStore) {
        for (id, package) in store.0.iter() {
            let capabilities = Initializer::from_package(package);
            self.store.insert(id, capabilities);
        }
    }
}

struct Initializer;

impl Initializer {
    pub fn from_package(package: &Package) -> PackageCapabilities {
        let mut callables = IndexMap::<LocalItemId, Option<CallableCapabilities>>::new();
        for (id, item) in package.items.iter() {
            let capabilities = match item.kind {
                ItemKind::Callable(_) => Some(CallableCapabilities::new()),
                _ => None,
            };
            callables.insert(id, capabilities);
        }
        PackageCapabilities { callables }
    }
}
