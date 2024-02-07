// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Runtime Capabilities Analysis (RCA)...

mod analyzer;
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
        if self.dynamic_params_properties.is_empty() {
            write!(indent, " <empty>")?;
        } else {
            indent = set_indentation(indent, 2);
            for (para_index, param_properties) in self.dynamic_params_properties.iter().enumerate()
            {
                write!(indent, "\n[{}]: {}", para_index, param_properties)?;
            }
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
    pub dynamism_sources: FxHashSet<DynamismSource>,
}

impl Default for ComputeProperties {
    fn default() -> Self {
        Self {
            runtime_features: RuntimeFeatureFlags::empty(),
            dynamism_sources: FxHashSet::default(),
        }
    }
}

impl Display for ComputeProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut indent = set_indentation(indented(f), 0);
        write!(indent, "ComputeProperties:",)?;
        indent = set_indentation(indent, 1);
        write!(indent, "\nruntime_features: {:?}", self.runtime_features)?;
        write!(indent, "\ndynamism_sources: ")?;
        if self.dynamism_sources.is_empty() {
            _ = write!(f, "<empty>");
        } else {
            _ = write!(f, "{{");
            let mut first = true;
            for source in self.dynamism_sources.iter().sorted() {
                if !first {
                    _ = write!(f, ", ");
                }
                _ = write!(f, "{source:?}");
                first = false;
            }
            _ = write!(f, "}}");
        }

        Ok(())
    }
}

impl ComputeProperties {
    /// Creates an empty compute properties structure.
    pub fn empty() -> Self {
        Self::default()
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
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

/// The kind of compute associated to a program element.
#[derive(Debug)]
pub enum ComputeKind {
    Static,
    Dynamic,
}
