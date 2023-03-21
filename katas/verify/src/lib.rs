// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_frontend::compile::{self, compile, PackageStore};

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn verify_reference(reference: &str) {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());
    let unit = compile(&store, [stdlib], [reference], "");
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
}
