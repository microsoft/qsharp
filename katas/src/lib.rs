// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc_frontend::compile::{self, compile, PackageStore};

#[must_use]
pub fn verify_kata(verification_source: &str, kata_implementation: &str) -> bool {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());

    // Validate that the code successfully compiles.
    // N.B. Once evaluation works for katas, the expression to compile should be "Kata.Verify()".
    let unit = compile(
        &store,
        [stdlib],
        [verification_source, kata_implementation],
        "",
    );
    if !unit.context.errors().is_empty() {
        println!("Compilation errors: {:?}", unit.context.errors());
        return false;
    }

    // N.B. Once evaluation works for katas, run the Verify operation.

    true
}
