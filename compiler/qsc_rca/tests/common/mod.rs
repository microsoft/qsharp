// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::Expect;
use itertools::Itertools;
use qsc::incremental::Compiler;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_eval::{debug::map_hir_package_to_fir, lower::Lowerer};
use qsc_fir::fir::{ItemKind, LocalItemId, Package, PackageStore, StoreItemId};
use qsc_frontend::compile::{PackageStore as HirPackageStore, RuntimeCapabilityFlags, SourceMap};
use qsc_passes::PackageType;
use qsc_rca::{Analyzer, ComputePropertiesLookup, PackageStoreComputeProperties};
use std::{fs::File, io::Write};

pub struct CompilationContext {
    pub compiler: Compiler,
    pub fir_store: PackageStore,
    pub compute_properties: PackageStoreComputeProperties,
    lowerer: Lowerer,
}

impl CompilationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_compute_properties(&self) -> &PackageStoreComputeProperties {
        //self.analyzer.get_package_store_compute_properties()
        &self.compute_properties
    }

    pub fn update(&mut self, source: &str) {
        let increment = self
            .compiler
            .compile_fragments_fail_fast("rca-test", source)
            .expect("code should compile");
        let package_id = map_hir_package_to_fir(self.compiler.package_id());
        let fir_package = self.fir_store.get_mut(package_id);
        self.lowerer
            .lower_and_update_package(fir_package, &increment.hir);
        self.compiler.update(increment);

        // Clear the compute properties of the package to update.
        let package_compute_properties = self.compute_properties.get_mut(package_id);
        package_compute_properties.clear();
        let analyzer = Analyzer::init_with_compute_properties(
            &self.fir_store,
            self.compute_properties.clone(),
        );
        self.compute_properties = analyzer.analyze_package(package_id);
        write_fir_store_to_files(&self.fir_store);
        write_compute_properties_to_files(&self.compute_properties);
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        let compiler = Compiler::new(
            true,
            SourceMap::default(),
            PackageType::Lib,
            RuntimeCapabilityFlags::all(),
            LanguageFeatures::default(),
        )
        .expect("should be able to create a new compiler");
        let mut lowerer = Lowerer::new();
        let fir_store = lower_hir_package_store(&mut lowerer, compiler.package_store());
        let analyzer = Analyzer::init(&fir_store);
        let compute_properties = analyzer.analyze_all();
        Self {
            compiler,
            fir_store,
            lowerer,
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

pub fn check_last_statement_compute_properties(
    package_store_compute_properties: &PackageStoreComputeProperties,
    expect: &Expect,
) {
    let last_package_id = package_store_compute_properties
        .iter()
        .map(|(package_id, _)| package_id)
        .sorted()
        .last()
        .expect("at least one package should exist");
    let package_compute_properties = package_store_compute_properties.get(last_package_id);
    let last_statement_id = package_compute_properties
        .stmts
        .iter()
        .map(|(stmt_id, _)| stmt_id)
        .sorted()
        .last()
        .expect("at least one statement should exist");
    let stmt_compute_properties = package_compute_properties
        .stmts
        .get(last_statement_id)
        .expect("statement compute properties should exist");
    expect.assert_eq(&stmt_compute_properties.to_string());
}

fn lower_hir_package_store(
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
