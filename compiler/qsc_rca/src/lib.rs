// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA)...

mod cycle_detection;
mod data_structures;
mod rca;

use crate::rca::analyze_package;
use bitflags::bitflags;
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
    pub fn get(&self, id: PackageId) -> Option<&PackageComputeProperties> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: PackageId) -> Option<&mut PackageComputeProperties> {
        self.0.get_mut(id)
    }

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

    pub fn iter(&self) -> Iter<PackageId, PackageComputeProperties> {
        self.0.iter()
    }

    /// Creates a structure that contains compute properties for a package store.
    /// Note: this is a computationally intensive operation.
    pub fn new(package_store: &PackageStore) -> Self {
        // Create package store compute properties with empty properties for each package.
        let mut package_store_compute_properties = IndexMap::new();
        for (id, _) in package_store.iter() {
            package_store_compute_properties.insert(id, PackageComputeProperties::default());
        }
        let mut package_store_compute_properties = Self(package_store_compute_properties);

        // Analyze each package in the store.
        for (package_id, _) in package_store.iter() {
            // N.B. It is assumed than when a package is analyzed, all the callables in other package that it depends
            // on have already been analyzed.
            analyze_package(
                package_id,
                package_store,
                &mut package_store_compute_properties,
            );
        }
        package_store_compute_properties
    }

    /// Updates the compute properties of the specified package using the contents of the passed package store.
    /// Note: this operation only updates the compute properties of a specific package, but it does not update the
    /// compute properties of other packages that depend on the package being reanalyzed.
    pub fn reanalyze_package(&mut self, id: PackageId, package_store: &PackageStore) {
        let package = self.get_mut(id).expect("package should exist");
        package.clear();
        analyze_package(id, package_store, self);
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
    /// An invalid element compute properties. TODO (cesarzc): remove once implementation is complete.
    Invalid,
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
            CallableElementComputeProperties::Invalid => write!(f, "Invalid"),
        }
    }
}

/// The compute properties associated to an application table.
#[derive(Clone, Debug)]
pub struct ApplicationsTable {
    /// The inherent compute properties present in all applications.
    /// N.B. These are the properties of the callable when all its parameters are binded to static values.
    pub inherent_properties: ComputeProperties,
    /// The compute properties of a callable application when a parameter is binded to a dynamic value.
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

/// The tracked compute properties of a program element.
#[derive(Clone, Debug)]
pub struct ComputeProperties {
    /// The runtime features used by the program element.
    pub runtime_features: RuntimeFeatureFlags,
    /// The sources of dynamism, if any.
    pub dynamism_sources: Vec<DynamismSource>,
}

impl Default for ComputeProperties {
    fn default() -> Self {
        Self {
            runtime_features: RuntimeFeatureFlags::empty(),
            dynamism_sources: Vec::new(),
        }
    }
}

impl Display for ComputeProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ComputeProperties:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nruntime_features: {:?}", self.runtime_features)?;
        if !self.dynamism_sources.is_empty() {
            write!(indent, "\ndynamism_sources: {:?}", self.dynamism_sources)?;
        }

        Ok(())
    }
}

bitflags! {
    /// Runtime features represent anything a program can do that is more complex than executing quantum operations on
    /// statically allocated qubits and using constant arguments.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RuntimeFeatureFlags: u64 {
        /// An intrinsic callable uses a dynamic `Result` argument.
        const IntrinsicApplicationUsesDynamicResult = 0b0001;
        /// An intrinsic callable uses a dynamic `Bool` argument.
        const IntrinsicApplicationUsesDynamicBool = 0b0010;
        /// An intrinsic callable uses a dynamic `Int` argument.
        const IntrinsicApplicationUsesDynamicInt = 0b0100;
        /// An intrinsic callable uses a dynamic `Pauli` argument.
        const IntrinsicApplicationUsesDynamicPauli = 0b1000;
        /// An intrinsic callable uses a dynamic `Range` argument.
        const IntrinsicApplicationUsesDynamicRange = 0b0001_0000;
        /// An intrinsic callable uses a dynamic `Double` argument.
        const IntrinsicApplicationUsesDynamicDouble = 0b010_0000;
        /// An intrinsic callable uses a dynamic `Qubit` argument.
        const IntrinsicApplicationUsesDynamicQubit = 0b0100_0000;
        /// An intrinsic callable uses a dynamic `BigInt` argument.
        const IntrinsicApplicationUsesDynamicBigInt = 0b1000_0000;
        /// An intrinsic callable uses a dynamic `String` argument.
        const IntrinsicApplicationUsesDynamicString = 0b0001_0000_0000;
        /// An intrinsic callable uses a dynamic array argument.
        const IntrinsicApplicationUsesDynamicArray = 0b0010_0000_0000;
        /// An intrinsic callable uses a dynamic tuple argument.
        const IntrinsicApplicationUsesDynamicTuple = 0b0100_0000_0000;
        /// An intrinsic callable uses a dynamic UDT argument.
        const IntrinsicApplicationUsesDynamicUdt = 0b1000_0000_0000;
        /// An intrinsic callable uses a dynamic arrow function argument.
        const IntrinsicApplicationUsesDynamicArrowFunction = 0b0001_0000_0000_0000;
        /// An intrinsic callable uses a dynamic arrow operation argument.
        const IntrinsicApplicationUsesDynamicArrowOperation = 0b0010_0000_0000_0000;
        /// A function with cycles used with a dynamic argument.
        const CycledFunctionApplicationUsesDynamicArg = 0b0100_0000_0000_0000;
        /// An operation specialization with cycles is used.
        const CycledOperationSpecializationApplication = 0b1000_0000_0000_0000;
    }
}

impl RuntimeFeatureFlags {
    /// Maps program contructs to runtime capabilities.
    pub fn runtime_capabilities(&self) -> RuntimeCapabilityFlags {
        let mut runtume_capabilities = RuntimeCapabilityFlags::empty();
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicResult) {
            runtume_capabilities |= RuntimeCapabilityFlags::ForwardBranching;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicBool) {
            runtume_capabilities |= RuntimeCapabilityFlags::ForwardBranching;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicInt) {
            runtume_capabilities |= RuntimeCapabilityFlags::IntegerComputations;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicPauli) {
            runtume_capabilities |= RuntimeCapabilityFlags::IntegerComputations;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicRange) {
            runtume_capabilities |= RuntimeCapabilityFlags::IntegerComputations;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicDouble) {
            runtume_capabilities |= RuntimeCapabilityFlags::FloatingPointComputations;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicQubit) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicBigInt) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicString) {
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicArray) {
            // N.B. Mapped runtime capabilities can be more nuanced by taking into account the contained type.
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicTuple) {
            // N.B. Mapped runtime capabilities can be more nuanced by taking into account the contained types.
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicUdt) {
            // N.B. Mapped runtime capabilities can be more nuanced by taking into account the type of each UDT item.
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicArrowFunction) {
            // N.B. Mapped runtime capabilities can be more nuanced by taking into account the input and output types.
            runtume_capabilities |= RuntimeCapabilityFlags::HigherLevelConstructs;
        }
        if self.contains(RuntimeFeatureFlags::IntrinsicApplicationUsesDynamicArrowOperation) {
            // N.B. Mapped runtime capabilities can be more nuanced by taking into account the input and output types.
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::CycledFunctionApplicationUsesDynamicArg) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        if self.contains(RuntimeFeatureFlags::CycledOperationSpecializationApplication) {
            runtume_capabilities |= RuntimeCapabilityFlags::all();
        }
        runtume_capabilities
    }
}

/// A source of dynamism.
#[derive(Clone, Copy, Debug)]
pub enum DynamismSource {
    /// An intrinsic dynamism source.
    Intrinsic,
    /// An assumed dynamism source.
    Assumed,
    /// A dynamism source that comes from an expression.
    Expr(ExprId),
}

impl Display for DynamismSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            DynamismSource::Intrinsic => write!(f, "Intrinsic",),
            DynamismSource::Assumed => write!(f, "Assumed",),
            DynamismSource::Expr(expr_id) => write!(f, "Expr: {}", expr_id),
        }
    }
}
