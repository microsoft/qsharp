use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        BlockId, CallableDecl, ExprId, ItemKind, LocalItemId, Package, PackageId, PackageStore,
        PatId, StmtId,
    },
    ty::{Prim, Ty},
};

use crate::{CapsBundle, StoreCapabilities};

#[derive(Debug)]
struct PackageCapsScaffolding {
    pub callables: IndexMap<LocalItemId, Option<CallableCapsScaffolding>>,
    pub blocks: IndexMap<BlockId, Option<BlockCapsScaffolding>>,
    pub stmts: IndexMap<StmtId, Option<CapsBundle>>,
    pub exprs: IndexMap<ExprId, Option<CapsBundle>>,
    pub pats: IndexMap<PatId, Option<CapsBundle>>,
}

// CONSIDER (cesarzc): Might need to do this a per specialization basis.
#[derive(Debug)]
struct CallableCapsScaffolding {
    pub intrinsic_caps: Option<CapsBundle>,
    pub parameter_caps: Option<Vec<CapsBundle>>,
}

// CONSIDER (cesarzc): This seems the same
#[derive(Debug)]
struct BlockCapsScaffolding {
    pub intrinsic_caps: Option<CapsBundle>,
    pub parameter_caps: Option<Vec<CapsBundle>>,
}

pub struct Analyzer {
    store: IndexMap<PackageId, PackageCapsScaffolding>,
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
        // TODO (cesarzc): should convert the store somehow.
        StoreCapabilities(IndexMap::new())
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
    pub fn from_package(package: &Package) -> PackageCapsScaffolding {
        // Initialize callables.
        let mut callables = IndexMap::<LocalItemId, Option<CallableCapsScaffolding>>::new();
        for (id, item) in package.items.iter() {
            let capabilities = match &item.kind {
                ItemKind::Callable(c) => Some(Self::from_callable(c)),
                _ => None,
            };
            callables.insert(id, capabilities);
        }

        // Initialize blocks.
        let mut blocks = IndexMap::<BlockId, Option<BlockCapsScaffolding>>::new();
        for (id, _) in package.blocks.iter() {
            blocks.insert(id, None);
        }

        // Initialize statements.
        let mut stmts = IndexMap::<StmtId, Option<CapsBundle>>::new();
        for (id, _) in package.stmts.iter() {
            stmts.insert(id, None);
        }

        // Initialize expressions.
        let mut exprs = IndexMap::<ExprId, Option<CapsBundle>>::new();
        for (id, _) in package.exprs.iter() {
            exprs.insert(id, None);
        }

        // Initialize patterns.
        let mut pats = IndexMap::<PatId, Option<CapsBundle>>::new();
        for (id, _) in package.pats.iter() {
            pats.insert(id, None);
        }

        PackageCapsScaffolding {
            callables,
            blocks,
            stmts,
            exprs,
            pats,
        }
    }

    fn from_callable(callable: &CallableDecl) -> CallableCapsScaffolding {
        // TODO (cesarzc): Separate into from_function and from_operation.

        // Parameter capabilities for QIS callables depend on the parameter type.
        // E.g.: Int -> {IntegerComputations}, Double -> {FloatingPointComputations}, Qubit -> {}.
        let is_qis_callable = callable.name.name.starts_with("__quantum__qis");
        // TODO (cesarzc): Implement.
        let parameter_caps = None;

        //
        let is_output_type_result = match callable.output {
            Ty::Prim(p) => p == Prim::Result,
            _ => false,
        };
        let is_quantum_source = is_output_type_result && is_qis_callable;
        let mut intrinsic_caps = None;
        if is_quantum_source {
            intrinsic_caps = Some(CapsBundle {
                is_quantum_source: true,
                caps: Vec::new(),
            });
        }

        CallableCapsScaffolding {
            intrinsic_caps,
            parameter_caps,
        }
    }
}
