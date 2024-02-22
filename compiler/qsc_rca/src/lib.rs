// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA) is the process of determining the capabilities a quantum kernel needs to be able
//! to run a particular program. This implementation also identifies program elements that can be pre-computed before
//! execution on a quantum kernel and does not consider these elements when determining the capabilities. Additionally,
//! this implementation also provides details on why the program requires each capability.

mod analyzer;
mod applications;
mod common;
mod cycle_detection;
mod rca;
mod scaffolding;

use crate::common::{aggregate_compute_kind, aggregate_value_kind, set_indentation};
use bitflags::bitflags;
use indenter::indented;
use qsc_data_structures::index_map::{IndexMap, Iter};
use qsc_fir::fir::{
    BlockId, ExprId, LocalItemId, PackageId, StmtId, StoreBlockId, StoreExprId, StoreItemId,
    StoreStmtId,
};
use qsc_frontend::compile::RuntimeCapabilityFlags;
use std::{
    cmp::Ord,
    fmt::{self, Debug, Display, Formatter, Write},
};

pub use crate::analyzer::Analyzer;

/// A trait to look for the compute properties of elements in a package store.
pub trait ComputePropertiesLookup {
    /// Searches for the applications generator set of a block with the specified ID.
    fn find_block(&self, id: StoreBlockId) -> Option<&ApplicationsGeneratorSet>;
    /// Searches for the applications generator set of an expression with the specified ID.
    fn find_expr(&self, id: StoreExprId) -> Option<&ApplicationsGeneratorSet>;
    /// Searches for the compute properties of an item with the specified ID.
    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProperties>;
    /// Searches for the applications generator set of a statement with the specified ID.
    fn find_stmt(&self, id: StoreStmtId) -> Option<&ApplicationsGeneratorSet>;
    /// Gets the applications generator set of a block.
    fn get_block(&self, id: StoreBlockId) -> &ApplicationsGeneratorSet;
    /// Gets the applications generator set of an expression.
    fn get_expr(&self, id: StoreExprId) -> &ApplicationsGeneratorSet;
    /// Gets the compute properties of an item.
    fn get_item(&self, id: StoreItemId) -> &ItemComputeProperties;
    /// Gets the applications generator set of a statement.
    fn get_stmt(&self, id: StoreStmtId) -> &ApplicationsGeneratorSet;
}

/// The compute properties of a package store.
#[derive(Debug, Default)]
pub struct PackageStoreComputeProperties(IndexMap<PackageId, PackageComputeProperties>);

impl ComputePropertiesLookup for PackageStoreComputeProperties {
    fn find_block(&self, id: StoreBlockId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.package)
            .and_then(|package| package.blocks.get(id.block))
    }

    fn find_expr(&self, id: StoreExprId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.package)
            .and_then(|package| package.exprs.get(id.expr))
    }

    fn find_item(&self, id: StoreItemId) -> Option<&ItemComputeProperties> {
        self.get(id.package)
            .and_then(|package| package.items.get(id.item))
    }

    fn find_stmt(&self, id: StoreStmtId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.package)
            .and_then(|package| package.stmts.get(id.stmt))
    }

    fn get_block(&self, id: StoreBlockId) -> &ApplicationsGeneratorSet {
        self.find_block(id)
            .expect("block compute properties should exist")
    }

    fn get_expr(&self, id: StoreExprId) -> &ApplicationsGeneratorSet {
        self.find_expr(id)
            .expect("expression compute properties should exist")
    }

    fn get_item(&self, id: StoreItemId) -> &ItemComputeProperties {
        self.find_item(id)
            .expect("item compute properties should exist")
    }

    fn get_stmt(&self, id: StoreStmtId) -> &ApplicationsGeneratorSet {
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

    pub fn insert_block(&mut self, id: StoreBlockId, value: ApplicationsGeneratorSet) {
        self.get_mut(id.package)
            .expect("package should exist")
            .blocks
            .insert(id.block, value);
    }

    pub fn insert_expr(&mut self, id: StoreExprId, value: ApplicationsGeneratorSet) {
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

    pub fn insert_stmt(&mut self, id: StoreStmtId, value: ApplicationsGeneratorSet) {
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
    /// The applications generator sets of the package blocks.
    pub blocks: IndexMap<BlockId, ApplicationsGeneratorSet>,
    /// The applications generator sets of the package statements.
    pub stmts: IndexMap<StmtId, ApplicationsGeneratorSet>,
    /// The applications generator sets of the package expressions.
    pub exprs: IndexMap<ExprId, ApplicationsGeneratorSet>,
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
    /// The applications generator set for the callable's body.
    pub body: ApplicationsGeneratorSet,
    /// The applications generator set for the callable's adjoint specialization.
    pub adj: Option<ApplicationsGeneratorSet>,
    /// The applications generator set for the callable's controlled specialization.
    pub ctl: Option<ApplicationsGeneratorSet>,
    /// The applications generator set for the callable's controlled adjoint specialization.
    pub ctl_adj: Option<ApplicationsGeneratorSet>,
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

/// A set of compute properties associated to a callable or one of its elements, from which the properties of any
/// particular call application can be derived.
#[derive(Clone, Debug)]
pub struct ApplicationsGeneratorSet {
    /// The inherent compute kind of a program element, which is determined by binding all the parameters it depends on
    /// to static values.
    pub inherent: ComputeKind,
    /// Each element in the vector represents the compute kind of a call application when the parameter associated to
    /// the vector index is bound to a dynamic value.
    pub dynamic_param_applications: Vec<ComputeKind>,
}

impl Display for ApplicationsGeneratorSet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ApplicationsGeneratorSet:",)?;
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

impl ApplicationsGeneratorSet {
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

#[derive(Clone, Copy, Debug)]
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
    pub fn aggregate_value_kind(&mut self, value_kind: ValueKind) {
        let Self::Quantum(quantum_properties) = self else {
            panic!("only quantum compute kinds can aggregate value kinds");
        };

        quantum_properties.value_kind =
            aggregate_value_kind(quantum_properties.value_kind, &value_kind);
    }

    pub fn is_value_dynamic(&self) -> bool {
        match self {
            Self::Classical => false,
            Self::Quantum(quantum_properties) => match quantum_properties.value_kind {
                ValueKind::Static => false,
                ValueKind::Dynamic => true,
            },
        }
    }

    pub fn with_runtime_features(runtime_features: RuntimeFeatureFlags) -> Self {
        Self::Quantum(QuantumProperties::with_runtime_features(runtime_features))
    }
}

/// The quantum properties of a program element.
#[derive(Clone, Copy, Debug)]
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

impl QuantumProperties {
    fn with_runtime_features(runtime_features: RuntimeFeatureFlags) -> Self {
        Self {
            runtime_features,
            value_kind: ValueKind::Static,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ValueKind {
    Static,
    Dynamic,
}

impl Display for ValueKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            ValueKind::Dynamic => {
                write!(f, "Dynamic")?;
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
    pub struct RuntimeFeatureFlags: u32 {
        /// Use of a dynamic `Bool`.
        const UseOfDynamicBool = 1 << 0;
        /// Use of a dynamic `Int`.
        const UseOfDynamicInt = 1 << 1;
        /// Use of a dynamic `Pauli`.
        const UseOfDynamicPauli = 1 << 2;
        /// Use of a dynamic `Range`.
        const UseOfDynamicRange = 1 << 3;
        /// Use of a dynamic `Double`.
        const UseOfDynamicDouble = 1 << 4;
        /// Use of a dynamic `Qubit`.
        const UseOfDynamicQubit = 1 << 5;
        /// Use of a dynamic `BigInt`.
        const UseOfDynamicBigInt = 1 << 6;
        /// Use of a dynamic `String`.
        const UseOfDynamicString = 1 << 7;
        /// Use of a dynamic array.
        const UseOfDynamicArray = 1 << 8;
        /// Use of a dynamic tuple.
        const UseOfDynamicTuple = 1 << 9;
        /// Use of a dynamic UDT.
        const UseOfDynamicUdt = 1 << 10;
        /// Use of a dynamic arrow function.
        const UseOfDynamicArrowFunction = 1 << 11;
        /// Use of a dynamic arrow operation.
        const UseOfDynamicArrowOperation = 1 << 12;
        /// Use of a dynamic generic.
        const UseOfDynamicGeneric = 1 << 13;
        /// A function with cycles used with a dynamic argument.
        const CyclicFunctionUsesDynamicArg = 1 << 14;
        /// An operation specialization with cycles is used.
        const CyclicOperation = 1 << 15;
        /// A callee expression is dynamic.
        const DynamicCallee = 1 << 16;
        /// A callee expression could not be resolved to a specific callable.
        const UnresolvedCallee = 1 << 17;
        /// A UDT constructor was used with a dynamic argument(s).
        const UdtConstructorUsesDynamicArg = 1 << 18;
        /// Forward branching on dynamic value.
        const ForwardBranchingOnDynamicValue = 1 << 19;
        /// Qubit allocation that happens within a dynamic scope.
        const DynamicQubitAllocation = 1 << 20;
        /// Result allocation that happens within a dynamic scope.
        const DynamicResultAllocation = 1 << 21;
        /// Use of a dynamic index to access or update an array.
        const UseOfDynamicIndex = 1 << 22;
        /// Use of a closure.
        const Closure = 1 << 23;
        /// Use of a runtime failure.
        const Failure = 1 << 24;
    }
}

impl RuntimeFeatureFlags {
    /// Maps program contructs to runtime capabilities.
    pub fn runtime_capabilities(&self) -> RuntimeCapabilityFlags {
        let mut runtume_capabilities = RuntimeCapabilityFlags::empty();
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
        if self.contains(RuntimeFeatureFlags::CyclicFunctionUsesDynamicArg) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::CyclicOperation) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::DynamicCallee) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::UnresolvedCallee) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::UdtConstructorUsesDynamicArg) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::ForwardBranchingOnDynamicValue) {
            runtume_capabilities |= RuntimeCapabilityFlags::ForwardBranching;
        }
        if self.contains(RuntimeFeatureFlags::DynamicQubitAllocation) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::DynamicResultAllocation) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::UseOfDynamicIndex) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::Closure) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::Failure) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        runtume_capabilities
    }
}
