// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_frontend::compile::{self, compile, PackageStore};

pub fn verify_kata(verify_wrapper: &str, implementation: &str) {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());
    // TODO: The expression should probably be "Kata.Verify()"
    let unit = compile(&store, [stdlib], [verify_wrapper, implementation], "");
    // TODO (cesarzc): this function should probably return something rather than assert.
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
}
