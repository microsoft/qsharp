use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{BlockId, CallableDecl, ItemKind, LocalItemId, Package, PackageId, PackageStore},
    ty::{Prim, Ty},
};

use crate::{BlockCapabilities, CallableCapabilities, PackageCapabilities, StoreCapabilities};

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
        // Initialize callables.
        let mut callables = IndexMap::<LocalItemId, Option<CallableCapabilities>>::new();
        for (id, item) in package.items.iter() {
            let capabilities = match &item.kind {
                ItemKind::Callable(c) => Some(Self::from_callable(c)),
                _ => None,
            };
            callables.insert(id, capabilities);
        }

        // Initialize blocks.
        let mut blocks = IndexMap::<BlockId, BlockCapabilities>::new();
        for (id, _) in package.blocks.iter() {
            let capabilities = BlockCapabilities {
                inherent: Vec::new(),
            };
            blocks.insert(id, capabilities);
        }
        PackageCapabilities { callables, blocks }
    }

    fn from_callable(callable: &CallableDecl) -> CallableCapabilities {
        let is_output_type_result = match callable.output {
            Ty::Prim(p) => p == Prim::Result,
            _ => false,
        };
        let is_qis_callable = callable.name.name.starts_with("__quantum__qis");
        let is_quantum_source = is_output_type_result && is_qis_callable;
        CallableCapabilities {
            is_quantum_source,
            inherent: Vec::new(),
        }
    }
}
