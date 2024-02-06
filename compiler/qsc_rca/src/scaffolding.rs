// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{GlobalSpecializationId, SpecializationKind},
    ApplicationsTable, CallableComputeProperties, ComputePropertiesLookup, ItemComputeProperties,
    PackageStoreComputeProperties,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::fir::{
    BlockId, ExprId, LocalItemId, PackageId, PackageStore, StmtId, StoreBlockId, StoreExprId,
    StoreItemId, StoreStmtId,
};

/// Scaffolding used to build the package store compute properties.
#[derive(Debug, Default)]
pub struct PackageStoreScaffolding(IndexMap<PackageId, PackageScaffolding>);

impl ComputePropertiesLookup for PackageStoreScaffolding {
    fn find_block(&self, id: StoreBlockId) -> Option<&ApplicationsTable> {
        self.get(id.package)
            .and_then(|package| package.blocks.get(id.block))
    }

    fn find_expr(&self, id: StoreExprId) -> Option<&ApplicationsTable> {
        self.get(id.package)
            .and_then(|package| package.exprs.get(id.expr))
    }

    fn find_item(&self, _: StoreItemId) -> Option<&ItemComputeProperties> {
        panic!("not implemented")
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

    fn get_item(&self, _: StoreItemId) -> &ItemComputeProperties {
        panic!("not implemented")
    }

    fn get_stmt(&self, id: StoreStmtId) -> &ApplicationsTable {
        self.find_stmt(id)
            .expect("statement compute properties should exist")
    }
}

impl PackageStoreScaffolding {
    pub fn find_specialization(&self, id: GlobalSpecializationId) -> Option<&ApplicationsTable> {
        self.get(id.callable.package)
            .and_then(|package| package.items.get(id.callable.item))
            .and_then(|item_scaffolding| match item_scaffolding {
                ItemScaffolding::NonCallable => None,
                ItemScaffolding::Specializations(specializations) => Some(specializations),
            })
            .and_then(|specializations| {
                specializations.get(SpecializationIndex::from(id.specialization))
            })
    }

    pub fn flush(&mut self, package_store_compute_properties: &mut PackageStoreComputeProperties) {
        // TODO (cesarzc): drain packages too.
        for (package_id, package) in self.0.iter_mut() {
            // Flush items.
            package
                .items
                .drain()
                .for_each(|(item_id, item_scaffolding)| {
                    let item_compute_properties = ItemComputeProperties::from(item_scaffolding);
                    package_store_compute_properties
                        .insert_item((package_id, item_id).into(), item_compute_properties);
                });

            // Flush blocks.
            package
                .blocks
                .drain()
                .for_each(|(block_id, applications_table)| {
                    package_store_compute_properties
                        .insert_block((package_id, block_id).into(), applications_table)
                });

            // Flush statements.
            package
                .stmts
                .drain()
                .for_each(|(stmt_id, applications_table)| {
                    package_store_compute_properties
                        .insert_stmt((package_id, stmt_id).into(), applications_table)
                });

            // Flush expressions.
            package
                .exprs
                .drain()
                .for_each(|(expr_id, applications_table)| {
                    package_store_compute_properties
                        .insert_expr((package_id, expr_id).into(), applications_table)
                });
        }
    }

    pub fn get(&self, id: PackageId) -> Option<&PackageScaffolding> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: PackageId) -> Option<&mut PackageScaffolding> {
        self.0.get_mut(id)
    }

    pub fn get_specialization(&self, id: GlobalSpecializationId) -> &ApplicationsTable {
        self.find_specialization(id)
            .expect("specialization should exist")
    }

    pub fn initialize_packages(&mut self, package_store: &PackageStore) {
        for (package_id, _) in package_store {
            self.insert(package_id, PackageScaffolding::default())
        }
    }

    pub fn insert(&mut self, id: PackageId, value: PackageScaffolding) {
        self.0.insert(id, value);
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

    pub fn insert_item(&mut self, id: StoreItemId, value: ItemScaffolding) {
        self.get_mut(id.package)
            .expect("package should exist")
            .items
            .insert(id.item, value);
    }

    pub fn insert_spec(&mut self, id: GlobalSpecializationId, value: ApplicationsTable) {
        let mut items = &mut self
            .get_mut(id.callable.package)
            .expect("package should exist")
            .items;
        if let Some(item_scaffolding) = items.get_mut(id.callable.item) {
            if let ItemScaffolding::Specializations(specializations) = item_scaffolding {
                // The item already exists but not the specialization.
                specializations.insert(SpecializationIndex::from(id.specialization), value);
            } else {
                panic!("item should be a callable");
            }
        } else {
            // Insert both the specialization and the item.
            let mut specializations = IndexMap::new();
            specializations.insert(SpecializationIndex::from(id.specialization), value);
            items.insert(
                id.callable.item,
                ItemScaffolding::Specializations(specializations),
            );
        }
    }

    pub fn insert_stmt(&mut self, id: StoreExprId, value: ApplicationsTable) {
        self.get_mut(id.package)
            .expect("package should exist")
            .exprs
            .insert(id.expr, value);
    }

    pub fn take(&mut self, package_store_compute_properties: &mut PackageStoreComputeProperties) {
        // TODO (cesarzc): implement.
    }
}

/// Scaffolding used to build the compute properties of a package.
#[derive(Debug, Default)]
pub struct PackageScaffolding {
    /// The compute properties of the package items.
    pub items: IndexMap<LocalItemId, ItemScaffolding>,
    /// The compute properties of the package blocks.
    pub blocks: IndexMap<BlockId, ApplicationsTable>,
    /// The compute properties of the package statements.
    pub stmts: IndexMap<StmtId, ApplicationsTable>,
    /// The compute properties of the package expressions.
    pub exprs: IndexMap<ExprId, ApplicationsTable>,
}

/// Scaffolding used to build the compute properties of an item.
#[derive(Debug, Default)]
pub enum ItemScaffolding {
    #[default]
    NonCallable,
    Specializations(SpecializationsScaffolding),
}

impl From<ItemScaffolding> for ItemComputeProperties {
    fn from(value: ItemScaffolding) -> Self {
        match value {
            ItemScaffolding::NonCallable => ItemComputeProperties::NonCallable,
            ItemScaffolding::Specializations(specializations) => {
                ItemComputeProperties::Callable(CallableComputeProperties::from(specializations))
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct SpecializationIndex(usize);

impl From<SpecializationIndex> for usize {
    fn from(value: SpecializationIndex) -> Self {
        value.0
    }
}

impl From<usize> for SpecializationIndex {
    fn from(value: usize) -> Self {
        SpecializationIndex(value)
    }
}

impl From<SpecializationKind> for SpecializationIndex {
    fn from(specialization_kind: SpecializationKind) -> Self {
        match specialization_kind {
            SpecializationKind::Body => SpecializationIndex(0),
            SpecializationKind::Adj => SpecializationIndex(1),
            SpecializationKind::Ctl => SpecializationIndex(2),
            SpecializationKind::CtlAdj => SpecializationIndex(4),
        }
    }
}

impl From<SpecializationIndex> for SpecializationKind {
    fn from(value: SpecializationIndex) -> Self {
        match value {
            SpecializationIndex(0) => Self::Body,
            SpecializationIndex(1) => Self::Adj,
            SpecializationIndex(2) => Self::Ctl,
            SpecializationIndex(3) => Self::CtlAdj,
            _ => panic!("invalid specialization index"),
        }
    }
}

pub type SpecializationsScaffolding = IndexMap<SpecializationIndex, ApplicationsTable>;

impl From<SpecializationsScaffolding> for CallableComputeProperties {
    fn from(value: SpecializationsScaffolding) -> Self {
        let (mut body, mut adj, mut ctl, mut ctl_adj) = (
            Option::<ApplicationsTable>::default(),
            Option::<ApplicationsTable>::default(),
            Option::<ApplicationsTable>::default(),
            Option::<ApplicationsTable>::default(),
        );
        for (specialization_index, applications_table) in value {
            match SpecializationKind::from(specialization_index) {
                SpecializationKind::Body => body = Some(applications_table),
                SpecializationKind::Adj => adj = Some(applications_table),
                SpecializationKind::Ctl => ctl = Some(applications_table),
                SpecializationKind::CtlAdj => ctl_adj = Some(applications_table),
            };
        }

        CallableComputeProperties {
            body: body.expect("body should exist"),
            adj,
            ctl,
            ctl_adj,
        }
    }
}
