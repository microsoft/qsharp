// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::Expect;
use qsc::incremental::Compiler;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::{ItemKind, LocalItemId, Package, PackageStore, StoreItemId};
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;
use qsc_rca::{ComputePropertiesLookup, PackageStoreComputeProperties};
use std::{fs::File, io::Write};

pub struct CompilationContext {
    pub compiler: Compiler,
    pub fir_store: PackageStore,
    pub compute_properties: PackageStoreComputeProperties,
}

impl CompilationContext {
    pub fn new() -> Self {
        let compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            RuntimeCapabilityFlags::all(),
        )
        .expect("should be able to create a new compiler");
        // TODO (cesarzc): use lowerer.
        let fir_store = lower_hir_package_store(compiler.package_store());
        let compute_properties = PackageStoreComputeProperties::new(&fir_store);
        Self {
            compiler,
            fir_store,
            compute_properties,
        }
    }
}

pub trait PackageStoreSearch {
    fn find_callable_id_by_name(&self, name: &str) -> Option<StoreItemId>;
}

impl PackageStoreSearch for PackageStore {
    fn find_callable_id_by_name(&self, name: &str) -> Option<StoreItemId> {
        for (package_id, package) in self.iter() {
            if let Some(item_id) = package.find_callable_id_by_name(name) {
                return Some((package_id, item_id).into());
            }
        }

        None
    }
}

pub trait PackageSearch {
    fn find_callable_id_by_name(&self, name: &str) -> Option<LocalItemId>;
}

impl PackageSearch for Package {
    fn find_callable_id_by_name(&self, name: &str) -> Option<LocalItemId> {
        for (item_id, item) in &self.items {
            if let ItemKind::Callable(callable_decl) = &item.kind {
                if callable_decl.name.name.as_ref() == name {
                    return Some(item_id);
                }
            }
        }

        None
    }
}

pub fn check_callable_compute_properties(
    fir_package_store: &impl PackageStoreSearch,
    package_store_compute_properties: &PackageStoreComputeProperties,
    callable_name: &str,
    expect: &Expect,
) {
    let callable_id = fir_package_store
        .find_callable_id_by_name(callable_name)
        .expect("callable should exist");

    let callable_compute_properties = package_store_compute_properties.get_item(callable_id);
    expect.assert_eq(&callable_compute_properties.to_string());
}

pub fn create_fir_package_store(
    lowerer: &mut Lowerer,
    hir_package_store: &HirPackageStore,
) -> PackageStore {
    let mut fir_store = PackageStore::new();
    for (id, unit) in hir_package_store {
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    fir_store
}

pub fn lower_hir_package_store(hir_package_store: &HirPackageStore) -> PackageStore {
    let mut lowerer = Lowerer::new();
    let mut fir_store = PackageStore::new();
    for (id, unit) in hir_package_store {
        fir_store.insert(
            map_hir_package_to_fir(id),
            lowerer.lower_package(&unit.package),
        );
    }
    fir_store
}

// TODO (cesarzc): for debugging purposes only, remove later.
pub fn write_fir_store_to_files(store: &PackageStore) {
    for (id, package) in store {
        let filename = format!("dbg/fir.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}

// TODO (cesarzc): for debugging purposes only, remove later.
pub fn write_compute_properties_to_files(store: &PackageStoreComputeProperties) {
    for (id, package) in store.iter() {
        let filename = format!("dbg/rca.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}
