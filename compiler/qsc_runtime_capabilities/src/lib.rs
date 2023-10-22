use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{LocalItemId, PackageId, PackageStore};

#[derive(Debug)]
pub enum RuntimeCapability {
    ConditionalForwardBranching,
    QubitReuse,
    IntegerComputations,
    FloatingPointComputationg,
    BackwardsBranching,
    UserDefinedFunctionCalls,
    HigherLevelConstructs,
}

#[derive(Debug)]
pub struct CallableCapabilities {
    pub inherent: Vec<RuntimeCapability>,
}

#[derive(Debug)]
pub struct PackageCapabilities {
    pub callables: IndexMap<LocalItemId, Option<CallableCapabilities>>,
}

pub struct StoreCapabilities(pub IndexMap<PackageId, PackageCapabilities>);

pub fn analyze_store_capabilities(_store: &PackageStore) -> StoreCapabilities {
    let store_capabilities = IndexMap::new();
    StoreCapabilities(store_capabilities)
}
