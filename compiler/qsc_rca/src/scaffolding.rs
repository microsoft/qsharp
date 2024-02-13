// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::{GlobalSpecId, SpecKind},
    ApplicationsTable, CallableComputeProperties, ComputePropertiesLookup, ItemComputeProperties,
    PackageComputeProperties, PackageStoreComputeProperties,
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
    pub fn find_specialization(&self, id: GlobalSpecId) -> Option<&ApplicationsTable> {
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
        assert!(package_store_compute_properties.0.is_empty());
        for (package_id, mut package_scaffolding) in self.0.drain() {
            let mut items = IndexMap::<LocalItemId, ItemComputeProperties>::new();
            for (item_id, item_scaffolding) in package_scaffolding.items.drain() {
                let item_compute_properties = ItemComputeProperties::from(item_scaffolding);
                items.insert(item_id, item_compute_properties);
            }

            let package_compute_properties = PackageComputeProperties {
                items,
                blocks: package_scaffolding.blocks,
                stmts: package_scaffolding.stmts,
                exprs: package_scaffolding.exprs,
            };
            package_store_compute_properties
                .0
                .insert(package_id, package_compute_properties);
        }
    }

    pub fn get(&self, id: PackageId) -> Option<&PackageScaffolding> {
        self.0.get(id)
    }

    pub fn get_mut(&mut self, id: PackageId) -> Option<&mut PackageScaffolding> {
        self.0.get_mut(id)
    }

    pub fn get_spec(&self, id: GlobalSpecId) -> &ApplicationsTable {
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

    pub fn insert_item(&mut self, id: StoreItemId, value: ItemScaffolding) {
        self.get_mut(id.package)
            .expect("package should exist")
            .items
            .insert(id.item, value);
    }

    pub fn insert_spec(&mut self, id: GlobalSpecId, value: ApplicationsTable) {
        let items = &mut self
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

    pub fn take(&mut self, package_store_compute_properties: &mut PackageStoreComputeProperties) {
        assert!(self.0.is_empty());
        for (package_id, mut package_compute_properties) in
            package_store_compute_properties.0.drain()
        {
            let mut items = IndexMap::<LocalItemId, ItemScaffolding>::new();
            package_compute_properties.items.drain().for_each(
                |(item_id, item_compute_properties)| {
                    let item_scaffolding = ItemScaffolding::from(item_compute_properties);
                    items.insert(item_id, item_scaffolding);
                },
            );

            let package_scaffolding = PackageScaffolding {
                items,
                blocks: package_compute_properties.blocks,
                stmts: package_compute_properties.stmts,
                exprs: package_compute_properties.exprs,
            };
            self.0.insert(package_id, package_scaffolding);
        }
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

impl From<ItemComputeProperties> for ItemScaffolding {
    fn from(value: ItemComputeProperties) -> Self {
        match value {
            ItemComputeProperties::NonCallable => ItemScaffolding::NonCallable,
            ItemComputeProperties::Callable(callable_compute_properties) => {
                ItemScaffolding::Specializations(SpecializationsScaffolding::from(
                    callable_compute_properties,
                ))
            }
        }
    }
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

impl From<SpecKind> for SpecializationIndex {
    fn from(specialization_kind: SpecKind) -> Self {
        match specialization_kind {
            SpecKind::Body => SpecializationIndex(0),
            SpecKind::Adj => SpecializationIndex(1),
            SpecKind::Ctl => SpecializationIndex(2),
            SpecKind::CtlAdj => SpecializationIndex(3),
        }
    }
}

impl From<SpecializationIndex> for SpecKind {
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

impl From<CallableComputeProperties> for SpecializationsScaffolding {
    fn from(value: CallableComputeProperties) -> Self {
        let mut specializations = SpecializationsScaffolding::default();
        specializations.insert(SpecKind::Body.into(), value.body);
        if let Some(adj_applications_table) = value.adj {
            specializations.insert(SpecKind::Adj.into(), adj_applications_table);
        }
        if let Some(ctl_applications_table) = value.ctl {
            specializations.insert(SpecKind::Ctl.into(), ctl_applications_table);
        }
        if let Some(ctl_adj_applications_table) = value.ctl_adj {
            specializations.insert(SpecKind::CtlAdj.into(), ctl_adj_applications_table);
        }
        specializations
    }
}

impl From<SpecializationsScaffolding> for CallableComputeProperties {
    fn from(value: SpecializationsScaffolding) -> Self {
        let (mut body, mut adj, mut ctl, mut ctl_adj) = (
            Option::<ApplicationsTable>::default(),
            Option::<ApplicationsTable>::default(),
            Option::<ApplicationsTable>::default(),
            Option::<ApplicationsTable>::default(),
        );
        for (specialization_index, applications_table) in value {
            match SpecKind::from(specialization_index) {
                SpecKind::Body => body = Some(applications_table),
                SpecKind::Adj => adj = Some(applications_table),
                SpecKind::Ctl => ctl = Some(applications_table),
                SpecKind::CtlAdj => ctl_adj = Some(applications_table),
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
