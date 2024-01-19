// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::PackageStoreComputeProperties;
use qsc_fir::fir::{ItemKind, LocalItemId, Package, PackageStore, StoreItemId};
use std::{fs::File, io::Write};

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
