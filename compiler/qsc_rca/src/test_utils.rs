// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_fir::fir::PackageStore;
use std::{fs::File, io::Write};

// TODO (cesarzc): for debugging purposes only, remove later.
pub fn write_fir_store_to_files(store: &PackageStore) {
    for (id, package) in store {
        let filename = format!("dbg/fir.package{id}.txt");
        let mut package_file = File::create(filename).expect("File could be created");
        let package_string = format!("{package}");
        write!(package_file, "{package_string}").expect("Writing to file should succeed.");
    }
}
