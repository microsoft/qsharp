// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA)...

mod analyzer;
mod applications;
mod common;
mod cycle_detection;
mod rca;
mod scaffolding;

use crate::common::set_indentation;
use bitflags::bitflags;
use indenter::indented;
use itertools::Itertools;
use qsc_data_structures::index_map::{IndexMap, Iter};
use qsc_fir::fir::{
    BlockId, ExprId, LocalItemId, PackageId, StmtId, StoreBlockId, StoreExprId, StoreItemId,
    StoreStmtId,
};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use rustc_hash::FxHashSet;
use std::{
    cmp::Ord,
    fmt::{self, Debug, Display, Formatter, Write},
};

pub use crate::analyzer::Analyzer;

/// A trait to look for the compute properties of elements in a package store.
pub trait ComputePropertiesLookup {
    /// Searches for the compute properties of a block with the specified ID.
    fn find_block(&self, id: StoreBlockId) -> Option<&ApplicationsTable>;
    /// Searches for the compute properties of an expression with the specified ID.
    fn find_expr(&self, id: StoreExprId) -> Option<&ApplicationsTable>;
    /// Searches for the compute properties of an item with the specified ID.
    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProperties>;
    /// Searches for the compute properties of a statement with the specified ID.
    fn find_stmt(&self, id: StoreStmtId) -> Option<&ApplicationsTable>;
    /// Gets the compute properties of a block.
    fn get_block(&self, id: StoreBlockId) -> &ApplicationsTable;
    /// Gets the compute properties of an expression.
    fn get_expr(&self, id: StoreExprId) -> &ApplicationsTable;
    /// Gets the compute properties of an item.
    fn get_item(&self, id: StoreItemId) -> &ItemComputeProperties;
    /// Gets the compute properties of a statement.
    fn get_stmt(&self, id: StoreStmtId) -> &ApplicationsTable;
}

/// The compute properties of a package store.
#[derive(Debug, Default)]
pub struct PackageStoreComputeProperties(IndexMap<PackageId, PackageComputeProperties>);

impl ComputePropertiesLookup for PackageStoreComputeProperties {
    fn find_block(&self, id: StoreBlockId) -> Option<&ApplicationsTable> {
        self.get(id.package)
            .and_then(|package| package.blocks.get(id.block))
    }

    fn find_expr(&self, id: StoreExprId) -> Option<&ApplicationsTable> {
        self.get(id.package)
            .and_then(|package| package.exprs.get(id.expr))
    }

    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.items.get(id.item))
    }

    fn find_stmt(&self, id: StoreStmtId) -> Option<&ApplicationsTable> {
        self.get(id.package)
            .and_then(|package| package.stmts.get(id.stmt))
    }

    fn get_block(&self, id: StoreBlockId) -> &ApplicationsTable {
        self.find_block(id)
            .expect("block compute properties should exist")
    }

    fn get_expr(&self, id: StoreExprId) -> &ApplicationsTable {
        self.find_expr(id)
            .expect("expression compute properties should exist")
    }

    fn get_item(&self, id: StoreItemId) -> &ItemComputeProperties {
        self.find_item(id)
            .expect("item compute properties should exist")
    }

    fn get_stmt(&self, id: StoreStmtId) -> &ApplicationsTable {
        self.find_stmt(id)
            .expect("statement compute properties should exist")
    }
}

impl PackageStoreComputeProperties {
    pub fn get(&self, id: PackageId) -> Option<&PackageComputeProperties> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: PackageId) -> Option<&mut PackageComputeProperties> {
        self.0.get_mut(id)
    }

    pub fn insert_block(&mut self, id: StoreBlockId, value: ApplicationsTable) {
        self.get_mut(id.package)
            .expect("package should exist")
            .blocks
            .insert(id.block, value);
    }

    pub fn insert_expr(&mut self, id: StoreExprId, value: ApplicationsTable) {
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

    pub fn insert_stmt(&mut self, id: StoreStmtId, value: ApplicationsTable) {
        self.get_mut(id.package)
            .expect("package should exist")
            .stmts
            .insert(id.stmt, value);
    }

    pub fn iter(&self) -> Iter<PackageId, PackageComputeProperties> {
        self.0.iter()
    }
}

/// The compute properties of a package.
#[derive(Debug)]
pub struct PackageComputeProperties {
    /// The compute properties of the package items.
    pub items: IndexMap<LocalItemId, ItemComputeProperties>,
    /// The compute properties of the package blocks.
    pub blocks: IndexMap<BlockId, ApplicationsTable>,
    /// The compute properties of the package statements.
    pub stmts: IndexMap<StmtId, ApplicationsTable>,
    /// The compute properties of the package expressions.
    pub exprs: IndexMap<ExprId, ApplicationsTable>,
}

impl Default for PackageComputeProperties {
    fn default() -> Self {
        Self {
            items: IndexMap::new(),
            blocks: IndexMap::new(),
            stmts: IndexMap::new(),
            exprs: IndexMap::new(),
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
        Ok(())
    }
}

impl PackageComputeProperties {
    pub fn clear(&mut self) {
        self.items.clear();
        self.blocks.clear();
        self.stmts.clear();
        self.exprs.clear();
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

/// The compute properties associated to a callable or one of its elements.
#[derive(Clone, Debug)]
pub struct ApplicationsTable {
    /// The inherent compute kind when all parameters are binded to static values.
    pub inherent: ComputeKind,
    /// Each element in the vector represents the compute kind of a call application when the parameter associated
    /// to the vector index is binded to a dynamic value.
    pub dynamic_param_applications: Vec<ComputeKind>,
}

impl ApplicationsTable {
    pub fn new(params_count: usize) -> Self {
        Self {
            inherent: ComputeKind::Classical,
            dynamic_param_applications: vec![ComputeKind::Classical; params_count],
        }
    }
}

impl Display for ApplicationsTable {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ApplicationsTable:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\ninherent: {}", self.inherent)?;
        write!(indent, "\ndynamic_param_applications:")?;
        if self.dynamic_param_applications.is_empty() {
            write!(indent, " <empty>")?;
        } else {
            indent = set_indentation(indent, 2);
            for (para_index, param_compute_kind) in
                self.dynamic_param_applications.iter().enumerate()
            {
                write!(indent, "\n[{}]: {}", para_index, param_compute_kind)?;
            }
        }
        Ok(())
    }
}

impl ApplicationsTable {
    pub fn derive_application_compute_kind(
        &self,
        input_params_dynamism: &Vec<bool>,
    ) -> ComputeKind {
        assert!(self.dynamic_param_applications.len() == input_params_dynamism.len());
        let mut compute_kind = self.inherent.clone();
        for (is_dynamic, dynamic_param_application) in input_params_dynamism
            .iter()
            .zip(self.dynamic_param_applications.iter())
        {
            if *is_dynamic {
                compute_kind = aggregate_compute_kind(compute_kind, dynamic_param_application);
            }
        }
        compute_kind
    }
}

#[derive(Clone, Debug)]
pub enum ComputeKind {
    Classical,
    Quantum(QuantumProperties),
}

impl Display for ComputeKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ComputeKind::Quantum(quantum_properties) => {
                write!(f, "Quantum: {}", quantum_properties)?
            }
            ComputeKind::Classical => write!(f, "Classical")?,
        };
        Ok(())
    }
}

impl ComputeKind {
    pub fn add_dynamism_sources(&mut self, new_sources: FxHashSet<DynamismSource>) {
        let ComputeKind::Quantum(quantum_properties) = self else {
            panic!("cannot add dynamism sources to classical compute kind");
        };

        if let ValueKind::Dynamic(dynamism_sources) = &mut quantum_properties.value_kind {
            dynamism_sources.extend(new_sources);
        } else {
            quantum_properties.value_kind = ValueKind::Dynamic(new_sources);
        };
    }

    pub fn get_dynamism_sources(&self) -> Option<&FxHashSet<DynamismSource>> {
        match self {
            ComputeKind::Classical => None,
            ComputeKind::Quantum(quantum_properties) => match &quantum_properties.value_kind {
                ValueKind::Static => None,
                ValueKind::Dynamic(dynamism_sources) => Some(dynamism_sources),
            },
        }
    }

    pub fn is_dynamic(&self) -> bool {
        self.get_dynamism_sources().is_some()
    }
}

#[must_use]
fn aggregate_compute_kind(
    base_compute_kind: ComputeKind,
    delta_compute_kind: &ComputeKind,
) -> ComputeKind {
    let ComputeKind::Quantum(delta_quantum_properties) = delta_compute_kind else {
        // A classical compute kind has nothing to aggregate so just return the base with no changes.
        return base_compute_kind;
    };

    // Determine the aggregated runtime features.
    let aggregated_runtime_features = match base_compute_kind {
        ComputeKind::Classical => delta_quantum_properties.runtime_features,
        ComputeKind::Quantum(ref base_quantum_properties) => {
            base_quantum_properties.runtime_features | delta_quantum_properties.runtime_features
        }
    };

    // Determine the aggregated value kind.
    let value_kind = match &base_compute_kind {
        ComputeKind::Classical => delta_quantum_properties.value_kind.clone(),
        ComputeKind::Quantum(base_quantum_properties) => {
            match &base_quantum_properties.value_kind {
                ValueKind::Static => delta_quantum_properties.value_kind.clone(),
                ValueKind::Dynamic(_) => match &delta_quantum_properties.value_kind {
                    ValueKind::Static => base_quantum_properties.value_kind.clone(),
                    ValueKind::Dynamic(_) => ValueKind::Dynamic(FxHashSet::default()),
                },
            }
        }
    };

    // Return the aggregated compute kind.
    ComputeKind::Quantum(QuantumProperties {
        runtime_features: aggregated_runtime_features,
        value_kind,
    })
}

/// The quantum properties of a program element.
#[derive(Clone, Debug)]
pub struct QuantumProperties {
    /// The runtime features used by the program element.
    pub runtime_features: RuntimeFeatureFlags,
    /// The kind of value of the program element.
    pub value_kind: ValueKind,
}

impl Default for QuantumProperties {
    fn default() -> Self {
        Self {
            runtime_features: RuntimeFeatureFlags::empty(),
            value_kind: ValueKind::Static,
        }
    }
}

impl Display for QuantumProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "QuantumProperties:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nruntime_features: {:?}", self.runtime_features)?;
        write!(indent, "\nvalue_kind: {}", self.value_kind)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum ValueKind {
    Static,
    Dynamic(FxHashSet<DynamismSource>),
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ValueKind::Dynamic(dynamism_sources) => {
                write!(f, "Dynamic: ")?;
                write!(f, "{{")?;
                let mut first = true;
                for source in dynamism_sources.iter().sorted() {
                    if !first {
                        _ = write!(f, ", ");
                    }
                    write!(f, "{source:?}")?;
                    first = false;
                }
                write!(f, "}}")?;
            }
            ValueKind::Static => {
                write!(f, "Static")?;
            }
        };
        Ok(())
    }
}

bitflags! {
    /// Runtime features represent anything a program can do that is more complex than executing quantum operations on
    /// statically allocated qubits and using constant arguments.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RuntimeFeatureFlags: u64 {
        /// Use of a dynamic `Result`.
        const UseOfDynamicResult = 0b0001;
        /// Use of a dynamic `Bool`.
        const UseOfDynamicBool = 0b0010;
        /// Use of a dynamic `Int`.
        const UseOfDynamicInt = 0b0100;
        /// Use of a dynamic `Pauli`.
        const UseOfDynamicPauli = 0b1000;
        /// Use of a dynamic `Range`.
        const UseOfDynamicRange = 0b0001_0000;
        /// Use of a dynamic `Double`.
        const UseOfDynamicDouble = 0b010_0000;
        /// Use of a dynamic `Qubit`.
        const UseOfDynamicQubit = 0b0100_0000;
        /// Use of a dynamic `BigInt`.
        const UseOfDynamicBigInt = 0b1000_0000;
        /// Use of a dynamic `String`.
        const UseOfDynamicString = 0b0001_0000_0000;
        /// Use of a dynamic array.
        const UseOfDynamicArray = 0b0010_0000_0000;
        /// Use of a dynamic tuple.
        const UseOfDynamicTuple = 0b0100_0000_0000;
        /// Use of a dynamic UDT.
        const UseOfDynamicUdt = 0b1000_0000_0000;
        /// Use of a dynamic arrow function.
        const UseOfDynamicArrowFunction = 0b0001_0000_0000_0000;
        /// Use of a dynamic arrow operation.
        const UseOfDynamicArrowOperation = 0b0010_0000_0000_0000;
        /// Use of a dynamic generic.
        const UseOfDynamicGeneric = 0b0100_0000_0000_0000;
        /// A function with cycles used with a dynamic argument.
        const CycledFunctionUsesDynamicArg = 0b1000_0000_0000_0000;
        /// An operation specialization with cycles is used.
        const CycledOperation = 0b0001_0000_0000_0000_0000;
        /// A callee expression is dynamic.
        const DynamicCallee = 0b0010_0000_0000_0000_0000;
        /// A callee expression could not be resolved to a specific callable.
        const UnresolvedCallee = 0b0100_0000_0000_0000_0000;
    }
}

impl RuntimeFeatureFlags {
    /// Maps program contructs to runtime capabilities.
    pub fn runtime_capabilities(&self) -> RuntimeCapabilityFlags {
        let mut runtume_capabilities = RuntimeCapabilityFlags::empty();
        if self.contains(RuntimeFeatureFlags::UseOfDynamicResult) {
            runtume_capabilities |= RuntimeCapabilityFlags::ForwardBranching;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicBool) {
            runtume_capabilities |= RuntimeCapabilityFlags::ForwardBranching;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicInt) {
            runtume_capabilities |= RuntimeCapabilityFlags::IntegerComputations;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicPauli) {
            runtume_capabilities |= RuntimeCapabilityFlags::IntegerComputations;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicRange) {
            runtume_capabilities |= RuntimeCapabilityFlags::IntegerComputations;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicDouble) {
            runtume_capabilities |= RuntimeCapabilityFlags::FloatingPointComputations;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicQubit) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicBigInt) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicString) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicArray) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicTuple) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicUdt) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicArrowFunction) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicArrowOperation) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicGeneric) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::CycledFunctionUsesDynamicArg) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::CycledOperation) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::DynamicCallee) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::UnresolvedCallee) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        runtume_capabilities
    }
}

/// A source of dynamism.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DynamismSource {
    /// An intrinsic dynamism source.
    Intrinsic,
    /// An assumed dynamism source.
    Assumed,
    /// A dynamism source that comes from an expression.
    Expr(ExprId),
    /// A dynamism source that comes from a statement.
    Stmt(StmtId),
}

impl Display for DynamismSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            DynamismSource::Intrinsic => write!(f, "Intrinsic",),
            DynamismSource::Assumed => write!(f, "Assumed",),
            DynamismSource::Expr(expr_id) => write!(f, "Expr: {}", expr_id),
            DynamismSource::Stmt(stmt_id) => write!(f, "Stmt: {}", stmt_id),
        }
    }
}
