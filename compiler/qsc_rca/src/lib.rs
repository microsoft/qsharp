// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA)...

mod data_structures;
mod fir_extensions;
mod rca;
#[cfg(test)]
mod tests;

use crate::data_structures::InputParam;
use crate::fir_extensions::{CallableDeclExt, TyExt};
use crate::rca::analyze_package_and_update_compute_props;
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{
    BlockId, CallableDecl, CallableKind, ExprId, LocalItemId, NodeId, PackageId, PackageStore,
    PatId, StmtId, StoreBlockId, StoreExprId, StoreItemId, StorePatId, StoreStmtId,
};
use qsc_frontend::compile::RuntimeCapabilityFlags;

/// A trait to look for the compute properties of elements in a package store.
pub trait ComputePropsLookup {
    /// Searches for the compute properties of a block with the specified ID.
    fn find_block(&self, id: StoreBlockId) -> Option<&ElmntComputeProps>;
    /// Searches for the compute properties of an expression with the specified ID.
    fn find_expr(&self, id: StoreExprId) -> Option<&ElmntComputeProps>;
    /// Searches for the compute properties of an item with the specified ID.
    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProps>;
    /// Searches for the compute properties of a pattern with the specified ID.
    fn find_pats(&self, id: StorePatId) -> Option<&PatComputeProps>;
    /// Searches for the compute properties of a statement with the specified ID.
    fn find_stmt(&self, id: StoreStmtId) -> Option<&ElmntComputeProps>;
    /// Gets the compute properties of a block.
    fn get_block(&self, id: StoreBlockId) -> &ElmntComputeProps;
    /// Gets the compute properties of an expression.
    fn get_expr(&self, id: StoreExprId) -> &ElmntComputeProps;
    /// Gets the compute properties of an item.
    fn get_item(&self, id: StoreItemId) -> &ItemComputeProps;
    /// Gets the compute properties of a pattern.
    fn get_pats(&self, id: StorePatId) -> &PatComputeProps;
    /// Gets the compute properties of a statement.
    fn get_stmt(&self, id: StoreStmtId) -> &ElmntComputeProps;
}

/// The compute properties of a package store.
#[derive(Debug)]
pub struct PackageStoreComputeProps(IndexMap<PackageId, PackageComputeProps>);

impl ComputePropsLookup for PackageStoreComputeProps {
    fn find_block(&self, id: StoreBlockId) -> Option<&ElmntComputeProps> {
        self.get(id.package)
            .and_then(|package| package.blocks.get(id.block))
    }

    fn find_expr(&self, id: StoreExprId) -> Option<&ElmntComputeProps> {
        self.get(id.package)
            .and_then(|package| package.exprs.get(id.expr))
    }

    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProps> {
        self.get(id.package)
            .and_then(|package| package.items.get(id.item))
    }

    fn find_pats(&self, id: StorePatId) -> Option<&PatComputeProps> {
        self.get(id.package)
            .and_then(|package| package.pats.get(id.pat))
    }

    fn find_stmt(&self, id: StoreStmtId) -> Option<&ElmntComputeProps> {
        self.get(id.package)
            .and_then(|package| package.stmts.get(id.stmt))
    }

    fn get_block(&self, id: StoreBlockId) -> &ElmntComputeProps {
        self.find_block(id)
            .expect("block compute properties should exist")
    }

    fn get_expr(&self, id: StoreExprId) -> &ElmntComputeProps {
        self.find_expr(id)
            .expect("expression compute properties should exist")
    }

    fn get_item(&self, id: StoreItemId) -> &ItemComputeProps {
        self.find_item(id)
            .expect("item compute properties should exist")
    }

    fn get_pats(&self, id: StorePatId) -> &PatComputeProps {
        self.find_pats(id)
            .expect("pattern compute properties should exist")
    }

    fn get_stmt(&self, id: StoreStmtId) -> &ElmntComputeProps {
        self.find_stmt(id)
            .expect("statement compute properties should exist")
    }
}

impl PackageStoreComputeProps {
    pub fn insert_block(&mut self, id: StoreBlockId, value: ElmntComputeProps) {
        self.get_mut(id.package)
            .expect("package should exist")
            .blocks
            .insert(id.block, value);
    }

    pub fn insert_expr(&mut self, id: StoreExprId, value: ElmntComputeProps) {
        self.get_mut(id.package)
            .expect("package should exist")
            .exprs
            .insert(id.expr, value);
    }

    pub fn insert_item(&mut self, id: StoreItemId, value: ItemComputeProps) {
        self.get_mut(id.package)
            .expect("package should exist")
            .items
            .insert(id.item, value);
    }

    pub fn insert_pat(&mut self, id: StorePatId, value: PatComputeProps) {
        self.get_mut(id.package)
            .expect("package should exist")
            .pats
            .insert(id.pat, value);
    }

    pub fn insert_stmt(&mut self, id: StoreExprId, value: ElmntComputeProps) {
        self.get_mut(id.package)
            .expect("package should exist")
            .exprs
            .insert(id.expr, value);
    }

    pub fn get(&self, id: PackageId) -> Option<&PackageComputeProps> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: PackageId) -> Option<&mut PackageComputeProps> {
        self.0.get_mut(id)
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
    /// The compute properties of the package items.
    pub items: IndexMap<LocalItemId, ItemComputeProps>,
    /// The compute properties of the package blocks.
    pub blocks: IndexMap<BlockId, ElmntComputeProps>,
    /// The compute properties of the package statements.
    pub stmts: IndexMap<StmtId, ElmntComputeProps>,
    /// The compute properties of the package expressions.
    pub exprs: IndexMap<ExprId, ElmntComputeProps>,
    /// The compute properties of the package patterns.
    pub pats: IndexMap<PatId, PatComputeProps>,
}

impl Default for PackageComputeProps {
    fn default() -> Self {
        Self {
            items: IndexMap::new(),
            blocks: IndexMap::new(),
            stmts: IndexMap::new(),
            exprs: IndexMap::new(),
            pats: IndexMap::new(),
        }
    }
}

impl PackageComputeProps {
    pub fn clear(&mut self) {
        self.items.clear();
        self.blocks.clear();
        self.stmts.clear();
        self.exprs.clear();
        self.pats.clear();
    }
}

/// The compute properties of an item.
#[derive(Debug)]
pub enum ItemComputeProps {
    /// The compute properties of a callable.
    Callable(CallableComputeProps),
    /// The compute properties of a non-callable (for completeness only).
    NonCallable,
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

impl CallableComputeProps {
    pub fn from_instrinsic<'a>(
        callable: &CallableDecl,
        input_params: impl Iterator<Item = &'a InputParam>,
    ) -> Self {
        assert!(callable.is_intrinsic());
        match callable.kind {
            CallableKind::Function => Self::from_instrinsic_function(callable, input_params),
            CallableKind::Operation => Self::from_instrinsic_operation(callable, input_params),
        }
    }

    fn from_instrinsic_function<'a>(
        callable: &CallableDecl,
        input_params: impl Iterator<Item = &'a InputParam>,
    ) -> Self {
        assert!(matches!(callable.kind, CallableKind::Function));

        // Functions are purely classical, so no runtime capabilities are needed and cannot be an inherent quantum
        // source.
        let inherent = ComputeProps {
            rt_caps: RuntimeCapabilityFlags::empty(),
            quantum_source: None,
        };

        // For each parameter, its properties when it is used as a dynamic argument in a particular application depend
        // on the parameter type.
        let mut dyn_params = Vec::new();
        for param in input_params {
            let param_rt_caps = param.ty.infer_rt_caps();
            let param_compute_props = ComputeProps {
                rt_caps: param_rt_caps,
                quantum_source: Some(QuantumSource::Intrinsic),
            };
            dyn_params.push(param_compute_props);
        }

        // Construct the callable compute properties.
        let body = AppsTbl {
            inherent,
            dyn_params,
        };
        Self {
            body,
            adj: None,
            ctl: None,
            ctl_adj: None,
        }
    }

    fn from_instrinsic_operation<'a>(
        callable: &CallableDecl,
        _input_params: impl Iterator<Item = &'a InputParam>,
    ) -> Self {
        assert!(matches!(callable.kind, CallableKind::Operation));

        // TODO (cesarzc): Implement this properly.
        let body = AppsTbl {
            inherent: ComputeProps {
                rt_caps: RuntimeCapabilityFlags::empty(),
                quantum_source: None,
            },
            dyn_params: Vec::new(),
        };
        Self {
            body,
            adj: None,
            ctl: None,
            ctl_adj: None,
        }
    }
}

/// The compute properties of pattern.
#[derive(Debug)]
pub enum PatComputeProps {
    /// A local discard. No compute properties tracked.
    LocalDiscard,
    /// A local node with compute properties tracked.
    LocalNode(NodeId, ElmntComputeProps),
    /// A local tuple. No compute properties tracked because it is not a node.
    LocalTuple(Vec<PatId>),
    ///
    InputParamDiscard,
    /// An input parameter. No compute properties tracked.
    InputParamNode(NodeId),
    /// An input parameter(s) tuple. No compute properties tracked.
    InputParamTuple(Vec<PatId>),
}

/// The compute properties of an element.
#[derive(Debug)]
pub enum ElmntComputeProps {
    /// An application dependent element.
    AppDependent(AppsTbl),
    /// An application independent element.
    AppIndependent(ComputeProps),
}

/// The compute properties associated to an application table.
#[derive(Debug)]
pub struct AppsTbl {
    /// The inherent compute properties of all applications.
    pub inherent: ComputeProps,
    /// The compute properties for each parameter when it corresponds to a dynamic argument in a particular callable
    /// application.
    pub dyn_params: Vec<ComputeProps>,
}

/// The tracked compute properties.
#[derive(Debug)]
pub struct ComputeProps {
    /// The runtime capabilities.
    pub rt_caps: RuntimeCapabilityFlags,
    /// The quantum source, if any.
    pub quantum_source: Option<QuantumSource>,
}

/// A quantum source.
#[derive(Debug)]
pub enum QuantumSource {
    /// An intrinsic quantum source.
    Intrinsic,
    /// A quantum source that comes from another expression.
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
