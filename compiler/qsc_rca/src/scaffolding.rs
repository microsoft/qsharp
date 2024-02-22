// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    common::GlobalSpecId, ApplicationsGeneratorSet, CallableComputeProperties,
    ComputePropertiesLookup,
};
use qsc_data_structures::index_map::IndexMap;
use qsc_fir::{
    fir::{
        self, BlockId, ExprId, LocalItemId, PackageId, StmtId, StoreBlockId, StoreExprId,
        StoreItemId, StoreStmtId,
    },
    ty::FunctorSetValue,
};

/// Scaffolding used to build the package store compute properties.
#[derive(Debug, Default)]
pub struct PackageStoreComputeProperties(IndexMap<PackageId, PackageComputeProperties>);

impl ComputePropertiesLookup for PackageStoreComputeProperties {
    fn find_block(&self, id: StoreBlockId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.package).blocks.get(id.block)
    }

    fn find_expr(&self, id: StoreExprId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.package).exprs.get(id.expr)
    }

    fn find_item(&self, _: StoreItemId) -> Option<&crate::ItemComputeProperties> {
        unimplemented!()
    }

    fn find_stmt(&self, id: StoreStmtId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.package).stmts.get(id.stmt)
    }

    fn get_block(&self, id: StoreBlockId) -> &ApplicationsGeneratorSet {
        self.find_block(id)
            .expect("block compute properties should exist")
    }

    fn get_expr(&self, id: StoreExprId) -> &ApplicationsGeneratorSet {
        self.find_expr(id)
            .expect("expression compute properties should exist")
    }

    fn get_item(&self, _: StoreItemId) -> &crate::ItemComputeProperties {
        unimplemented!()
    }

    fn get_stmt(&self, id: StoreStmtId) -> &ApplicationsGeneratorSet {
        self.find_stmt(id)
            .expect("statement compute properties should exist")
    }
}

impl PackageStoreComputeProperties {
    pub fn find_specialization(&self, id: GlobalSpecId) -> Option<&ApplicationsGeneratorSet> {
        self.get(id.callable.package)
            .items
            .get(id.callable.item)
            .and_then(|item_compute_properties| match item_compute_properties {
                ItemComputeProperties::NonCallable => None,
                ItemComputeProperties::Specializations(specializations) => Some(specializations),
            })
            .and_then(|specializations| {
                specializations.get(SpecializationIndex::from(id.functor_set_value))
            })
    }

    pub fn flush(
        &mut self,
        package_store_compute_properties: &mut crate::PackageStoreComputeProperties,
    ) {
        assert!(package_store_compute_properties.0.is_empty());
        for (package_id, mut package_scaffolding) in self.0.drain() {
            let mut items = IndexMap::<LocalItemId, crate::ItemComputeProperties>::new();
            for (item_id, item_scaffolding) in package_scaffolding.items.drain() {
                let item_compute_properties = crate::ItemComputeProperties::from(item_scaffolding);
                items.insert(item_id, item_compute_properties);
            }

            let package_compute_properties = crate::PackageComputeProperties {
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

    pub fn get(&self, id: PackageId) -> &PackageComputeProperties {
        self.0
            .get(id)
            .expect("package compute properties should be present in store")
    }

    pub fn get_mut(&mut self, id: PackageId) -> &mut PackageComputeProperties {
        self.0
            .get_mut(id)
            .expect("package compute properties should be present in store")
    }

    pub fn get_spec(&self, id: GlobalSpecId) -> &ApplicationsGeneratorSet {
        self.find_specialization(id)
            .expect("specialization should exist")
    }

    pub fn initialize_packages(&mut self, package_store: &fir::PackageStore) {
        for (package_id, _) in package_store {
            self.insert(package_id, PackageComputeProperties::default());
        }
    }

    pub fn insert(&mut self, id: PackageId, value: PackageComputeProperties) {
        self.0.insert(id, value);
    }

    pub fn insert_item(&mut self, id: StoreItemId, value: ItemComputeProperties) {
        self.get_mut(id.package).items.insert(id.item, value);
    }

    pub fn insert_spec(&mut self, id: GlobalSpecId, value: ApplicationsGeneratorSet) {
        let items = &mut self.get_mut(id.callable.package).items;
        if let Some(item_compute_properties) = items.get_mut(id.callable.item) {
            if let ItemComputeProperties::Specializations(specializations) = item_compute_properties
            {
                // The item already exists but not the specialization.
                specializations.insert(SpecializationIndex::from(id.functor_set_value), value);
            } else {
                panic!("item should be a callable");
            }
        } else {
            // Insert both the specialization and the item.
            let mut specializations = IndexMap::new();
            specializations.insert(SpecializationIndex::from(id.functor_set_value), value);
            items.insert(
                id.callable.item,
                ItemComputeProperties::Specializations(specializations),
            );
        }
    }

    pub fn take(
        &mut self,
        package_store_compute_properties: &mut crate::PackageStoreComputeProperties,
    ) {
        assert!(self.0.is_empty());
        for (package_id, mut package_compute_properties) in
            package_store_compute_properties.0.drain()
        {
            let mut items = IndexMap::<LocalItemId, ItemComputeProperties>::new();
            package_compute_properties.items.drain().for_each(
                |(item_id, item_compute_properties)| {
                    let item_compute_properties =
                        ItemComputeProperties::from(item_compute_properties);
                    items.insert(item_id, item_compute_properties);
                },
            );

            let package_compute_properties = PackageComputeProperties {
                items,
                blocks: package_compute_properties.blocks,
                stmts: package_compute_properties.stmts,
                exprs: package_compute_properties.exprs,
            };
            self.0.insert(package_id, package_compute_properties);
        }
    }
}

/// Scaffolding used to build the compute properties of a package.
#[derive(Debug, Default)]
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

/// Scaffolding used to build the compute properties of an item.
#[derive(Debug, Default)]
pub enum ItemComputeProperties {
    #[default]
    NonCallable,
    Specializations(SpecializationsComputeProperties),
}

impl From<crate::ItemComputeProperties> for ItemComputeProperties {
    fn from(value: crate::ItemComputeProperties) -> Self {
        match value {
            crate::ItemComputeProperties::NonCallable => ItemComputeProperties::NonCallable,
            crate::ItemComputeProperties::Callable(callable_compute_properties) => {
                ItemComputeProperties::Specializations(SpecializationsComputeProperties::from(
                    callable_compute_properties,
                ))
            }
        }
    }
}

impl From<ItemComputeProperties> for crate::ItemComputeProperties {
    fn from(value: ItemComputeProperties) -> Self {
        match value {
            ItemComputeProperties::NonCallable => crate::ItemComputeProperties::NonCallable,
            ItemComputeProperties::Specializations(specializations) => {
                crate::ItemComputeProperties::Callable(CallableComputeProperties::from(
                    specializations,
                ))
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

impl From<SpecializationIndex> for FunctorSetValue {
    fn from(value: SpecializationIndex) -> Self {
        match value {
            SpecializationIndex(0) => Self::Empty,
            SpecializationIndex(1) => Self::Adj,
            SpecializationIndex(2) => Self::Ctl,
            SpecializationIndex(3) => Self::CtlAdj,
            _ => panic!("invalid specialization index"),
        }
    }
}

impl From<FunctorSetValue> for SpecializationIndex {
    fn from(value: FunctorSetValue) -> Self {
        match value {
            FunctorSetValue::Empty => SpecializationIndex(0),
            FunctorSetValue::Adj => SpecializationIndex(1),
            FunctorSetValue::Ctl => SpecializationIndex(2),
            FunctorSetValue::CtlAdj => SpecializationIndex(3),
        }
    }
}

pub type SpecializationsComputeProperties = IndexMap<SpecializationIndex, ApplicationsGeneratorSet>;

impl From<CallableComputeProperties> for SpecializationsComputeProperties {
    fn from(value: CallableComputeProperties) -> Self {
        let mut specializations = SpecializationsComputeProperties::default();
        specializations.insert(FunctorSetValue::Empty.into(), value.body);
        if let Some(adj_applications_table) = value.adj {
            specializations.insert(FunctorSetValue::Adj.into(), adj_applications_table);
        }
        if let Some(ctl_applications_table) = value.ctl {
            specializations.insert(FunctorSetValue::Ctl.into(), ctl_applications_table);
        }
        if let Some(ctl_adj_applications_table) = value.ctl_adj {
            specializations.insert(FunctorSetValue::CtlAdj.into(), ctl_adj_applications_table);
        }
        specializations
    }
}

impl From<SpecializationsComputeProperties> for CallableComputeProperties {
    fn from(value: SpecializationsComputeProperties) -> Self {
        let (mut body, mut adj, mut ctl, mut ctl_adj) = (
            Option::<ApplicationsGeneratorSet>::default(),
            Option::<ApplicationsGeneratorSet>::default(),
            Option::<ApplicationsGeneratorSet>::default(),
            Option::<ApplicationsGeneratorSet>::default(),
        );
        for (specialization_index, applications_table) in value {
            match specialization_index.into() {
                FunctorSetValue::Empty => body = Some(applications_table),
                FunctorSetValue::Adj => adj = Some(applications_table),
                FunctorSetValue::Ctl => ctl = Some(applications_table),
                FunctorSetValue::CtlAdj => ctl_adj = Some(applications_table),
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
