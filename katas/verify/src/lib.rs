// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_frontend::compile::{compile, PackageStore};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn verify_reference(reference: &str) {
    let store = PackageStore::new();
    let unit = compile(&store, [], [reference], "");
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
}
