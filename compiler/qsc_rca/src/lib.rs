// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA)...

mod data_structures;
mod rca;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

use crate::rca::analyze_package;
use indenter::{indented, Indented};
use qsc_data_structures::index_map::{IndexMap, Iter};
use qsc_fir::fir::{
    BlockId, ExprId, LocalItemId, NodeId, PackageId, PackageStore, PatId, StmtId, StoreBlockId,
    StoreExprId, StoreItemId, StorePatId, StoreStmtId,
};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use std::fmt::{self, Debug, Display, Formatter, Write};

fn set_indentation<'a, 'b>(
    indent: Indented<'a, Formatter<'b>>,
    level: usize,
) -> Indented<'a, Formatter<'b>> {
    match level {
        0 => indent.with_str(""),
        1 => indent.with_str("    "),
        2 => indent.with_str("        "),
        _ => unimplemented!("intentation level not supported"),
    }
}

/// A trait to look for the compute properties of elements in a package store.
pub trait ComputePropertiesLookup {
    /// Searches for the compute properties of a block with the specified ID.
    fn find_block(&self, id: StoreBlockId) -> Option<&CallableElementComputeProperties>;
    /// Searches for the compute properties of an expression with the specified ID.
    fn find_expr(&self, id: StoreExprId) -> Option<&CallableElementComputeProperties>;
    /// Searches for the compute properties of an item with the specified ID.
    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProperties>;
    /// Searches for the compute properties of a pattern with the specified ID.
    fn find_pats(&self, id: StorePatId) -> Option<&PatComputeProperties>;
    /// Searches for the compute properties of a statement with the specified ID.
    fn find_stmt(&self, id: StoreStmtId) -> Option<&CallableElementComputeProperties>;
    /// Gets the compute properties of a block.
    fn get_block(&self, id: StoreBlockId) -> &CallableElementComputeProperties;
    /// Gets the compute properties of an expression.
    fn get_expr(&self, id: StoreExprId) -> &CallableElementComputeProperties;
    /// Gets the compute properties of an item.
    fn get_item(&self, id: StoreItemId) -> &ItemComputeProperties;
    /// Gets the compute properties of a pattern.
    fn get_pats(&self, id: StorePatId) -> &PatComputeProperties;
    /// Gets the compute properties of a statement.
    fn get_stmt(&self, id: StoreStmtId) -> &CallableElementComputeProperties;
}

/// The compute properties of a package store.
#[derive(Debug)]
pub struct PackageStoreComputeProperties(IndexMap<PackageId, PackageComputeProperties>);

impl ComputePropertiesLookup for PackageStoreComputeProperties {
    fn find_block(&self, id: StoreBlockId) -> Option<&CallableElementComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.blocks.get(id.block))
    }

    fn find_expr(&self, id: StoreExprId) -> Option<&CallableElementComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.exprs.get(id.expr))
    }

    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.items.get(id.item))
    }

    fn find_pats(&self, id: StorePatId) -> Option<&PatComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.pats.get(id.pat))
    }

    fn find_stmt(&self, id: StoreStmtId) -> Option<&CallableElementComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.stmts.get(id.stmt))
    }

    fn get_block(&self, id: StoreBlockId) -> &CallableElementComputeProperties {
        self.find_block(id)
            .expect("block compute properties should exist")
    }

    fn get_expr(&self, id: StoreExprId) -> &CallableElementComputeProperties {
        self.find_expr(id)
            .expect("expression compute properties should exist")
    }

    fn get_item(&self, id: StoreItemId) -> &ItemComputeProperties {
        self.find_item(id)
            .expect("item compute properties should exist")
    }

    fn get_pats(&self, id: StorePatId) -> &PatComputeProperties {
        self.find_pats(id)
            .expect("pattern compute properties should exist")
    }

    fn get_stmt(&self, id: StoreStmtId) -> &CallableElementComputeProperties {
        self.find_stmt(id)
            .expect("statement compute properties should exist")
    }
}

impl PackageStoreComputeProperties {
    pub fn insert_block(&mut self, id: StoreBlockId, value: CallableElementComputeProperties) {
        self.get_mut(id.package)
            .expect("package should exist")
            .blocks
            .insert(id.block, value);
    }

    pub fn insert_expr(&mut self, id: StoreExprId, value: CallableElementComputeProperties) {
        self.get_mut(id.package)
            .expect("package should exist")
            .exprs
            .insert(id.expr, value);
    }

    pub fn insert_item(&mut self, id: StoreItemId, value: ItemComputeProperties) {
        self.get_mut(id.package)
            .expect("package should exist")
            .items
            .insert(id.item, value);
    }

    pub fn insert_pat(&mut self, id: StorePatId, value: PatComputeProperties) {
        self.get_mut(id.package)
            .expect("package should exist")
            .pats
            .insert(id.pat, value);
    }

    pub fn insert_stmt(&mut self, id: StoreExprId, value: CallableElementComputeProperties) {
        self.get_mut(id.package)
            .expect("package should exist")
            .exprs
            .insert(id.expr, value);
    }

    pub fn get(&self, id: PackageId) -> Option<&PackageComputeProperties> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: PackageId) -> Option<&mut PackageComputeProperties> {
        self.0.get_mut(id)
    }

    pub fn iter(&self) -> Iter<PackageId, PackageComputeProperties> {
        self.0.iter()
    }

    pub fn with_empty_packages(fir_store: &PackageStore) -> Self {
        let mut package_store_compute_props = IndexMap::new();
        for (id, _) in fir_store.iter() {
            package_store_compute_props.insert(id, PackageComputeProperties::default());
        }
        Self(package_store_compute_props)
    }
}

/// The compute properties of a package.
#[derive(Debug)]
pub struct PackageComputeProperties {
    /// The compute properties of the package items.
    pub items: IndexMap<LocalItemId, ItemComputeProperties>,
    /// The compute properties of the package blocks.
    pub blocks: IndexMap<BlockId, CallableElementComputeProperties>,
    /// The compute properties of the package statements.
    pub stmts: IndexMap<StmtId, CallableElementComputeProperties>,
    /// The compute properties of the package expressions.
    pub exprs: IndexMap<ExprId, CallableElementComputeProperties>,
    /// The compute properties of the package patterns.
    pub pats: IndexMap<PatId, PatComputeProperties>,
}

impl Default for PackageComputeProperties {
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

impl Display for PackageComputeProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "Package:")?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nItems:")?;
        indent = set_indentation(indent, 2);
        for (item_id, item) in self.items.iter() {
            write!(indent, "\nItem {item_id}: {item}")?;
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nBlocks:")?;
        indent = set_indentation(indent, 2);
        for (block_id, block) in self.blocks.iter() {
            write!(indent, "\nBlock {block_id}: {block}")?;
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nStmts:")?;
        indent = set_indentation(indent, 2);
        for (stmt_id, stmt) in self.stmts.iter() {
            write!(indent, "\nStmt {stmt_id}: {stmt}")?;
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nExprs:")?;
        indent = set_indentation(indent, 2);
        for (expr_id, expr) in self.exprs.iter() {
            write!(indent, "\nExpr {expr_id}: {expr}")?;
        }
        indent = set_indentation(indent, 1);
        write!(indent, "\nPats:")?;
        indent = set_indentation(indent, 2);
        for (pat_id, pat) in self.pats.iter() {
            write!(indent, "\nPat {pat_id}: {pat}")?;
        }
        Ok(())
    }
}

impl PackageComputeProperties {
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
pub enum ItemComputeProperties {
    /// The compute properties of a callable.
    Callable(CallableComputeProperties),
    /// The compute properties of a non-callable (for completeness only).
    NonCallable,
}

impl Display for ItemComputeProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            ItemComputeProperties::Callable(callable_compute_properties) => {
                write!(f, "Callable: {}", callable_compute_properties)
            }
            ItemComputeProperties::NonCallable => write!(f, "NonCallable"),
        }
    }
}

/// The compute properties of a callable.
#[derive(Debug)]
pub struct CallableComputeProperties {
    /// The compute properties of the callable body.
    pub body: ApplicationsTable,
    /// The compute properties of the adjoint specialization.
    pub adj: Option<ApplicationsTable>,
    /// The compute properties of the controlled specialization.
    pub ctl: Option<ApplicationsTable>,
    /// The compute properties of the controlled adjoint specialization.
    pub ctl_adj: Option<ApplicationsTable>,
}

impl Display for CallableComputeProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "CallableComputeProperties:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nbody: {}", self.body)?;
        match &self.adj {
            Some(spec) => write!(indent, "\nadj: {spec}")?,
            None => write!(indent, "\nadj: <none>")?,
        }
        match &self.ctl {
            Some(spec) => write!(indent, "\nctl: {spec}")?,
            None => write!(indent, "\nctl: <none>")?,
        }
        match &self.ctl_adj {
            Some(spec) => write!(indent, "\nctl-adj: {spec}")?,
            None => write!(indent, "\nctl-adj: <none>")?,
        }
        Ok(())
    }
}

/// The compute properties of pattern.
#[derive(Debug)]
pub enum PatComputeProperties {
    /// A local discard. No compute properties tracked.
    LocalDiscard,
    /// A local node with compute properties tracked.
    LocalNode(NodeId, CallableElementComputeProperties),
    /// A local tuple. No compute properties tracked because it is not a node.
    LocalTuple(Vec<PatId>),
    /// A discarded input parameter.
    InputParamDiscard,
    /// An input parameter. No compute properties tracked.
    InputParamNode(NodeId),
    /// An input parameter(s) tuple. No compute properties tracked.
    InputParamTuple(Vec<PatId>),
}

impl Display for PatComputeProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            PatComputeProperties::LocalDiscard => write!(f, "LocalDiscard"),
            PatComputeProperties::LocalNode(node_id, element_compute_properties) => {
                write!(f, "LocalNode ({}): {}", node_id, element_compute_properties)
            }
            PatComputeProperties::LocalTuple(tuple) => write!(f, "LocalTuple: {:?}", tuple),
            PatComputeProperties::InputParamDiscard => write!(f, "InputParamDiscard"),
            PatComputeProperties::InputParamNode(node_id) => {
                write!(f, "InputParamNode: {}", node_id)
            }
            PatComputeProperties::InputParamTuple(tuple) => {
                write!(f, "InputParamTuple: {:?}", tuple)
            }
        }
    }
}

/// The compute properties of a callable element.
#[derive(Debug)]
pub enum CallableElementComputeProperties {
    /// An application dependent element.
    ApplicationDependent(ApplicationsTable),
    /// An application independent element.
    ApplicationIndependent(ComputeProperties),
}

impl Display for CallableElementComputeProperties {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            CallableElementComputeProperties::ApplicationDependent(applications_table) => {
                write!(f, "ApplicationDependent: {}", applications_table)
            }
            CallableElementComputeProperties::ApplicationIndependent(compute_properties) => {
                write!(f, "ApplicationIndependent: {}", compute_properties)
            }
        }
    }
}

/// The compute properties associated to an application table.
#[derive(Debug)]
pub struct ApplicationsTable {
    /// The inherent compute properties of all applications.
    pub inherent_properties: ComputeProperties,
    /// The compute properties for each parameter when it corresponds to a dynamic argument in a particular callable
    /// application.
    pub dynamic_params_properties: Vec<ComputeProperties>,
}

impl Display for ApplicationsTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ApplicationsTable:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\ninherent: {}", self.inherent_properties)?;
        write!(indent, "\ndynamic_params_properties:")?;
        indent = set_indentation(indent, 2);
        for (para_index, param_properties) in self.dynamic_params_properties.iter().enumerate() {
            write!(indent, "\n[{}]: {}", para_index, param_properties)?;
        }
        Ok(())
    }
}

/// The tracked compute properties.
#[derive(Debug)]
pub struct ComputeProperties {
    /// The runtime capabilities.
    pub runtime_capabilities: RuntimeCapabilityFlags,
    /// The quantum source, if any.
    pub quantum_source: Option<QuantumSource>,
}

impl Display for ComputeProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ComputeProperties:",)?;
        indent = set_indentation(indent, 1);
        write!(
            indent,
            "\nruntime_capabilities: {:?}",
            self.runtime_capabilities
        )?;
        if let Some(quantum_source) = self.quantum_source {
            write!(indent, "\nquantum_source: {quantum_source}")?;
        }

        Ok(())
    }
}

/// A quantum source.
#[derive(Clone, Copy, Debug)]
pub enum QuantumSource {
    /// An intrinsic quantum source.
    Intrinsic,
    /// A quantum source that comes from another expression.
    Expr(ExprId),
}

impl Display for QuantumSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            QuantumSource::Intrinsic => write!(f, "Intrinsic",),
            QuantumSource::Expr(expr_id) => write!(f, "Expr: {}", expr_id),
        }
    }
}

/// The runtime capabilities analyzer.
#[derive(Debug)]
pub struct Analyzer {
    /// The compute properties of the package store.
    compute_properties: PackageStoreComputeProperties,
    /// The ID of the opened package.
    _open_package_id: PackageId,
}

impl Analyzer {
    pub fn get_package_store_compute_properties(&self) -> &PackageStoreComputeProperties {
        &self.compute_properties
    }

    pub fn new(fir_store: &PackageStore, open_package_id: PackageId) -> Self {
        let mut compute_properties = PackageStoreComputeProperties::with_empty_packages(fir_store);

        // Analyze each package in the store.
        for (package_id, _) in fir_store.iter() {
            analyze_package(package_id, fir_store, &mut compute_properties);
        }
        Self {
            compute_properties,
            _open_package_id: open_package_id,
        }
    }
}
