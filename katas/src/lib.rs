// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc_frontend::compile::{self, compile, PackageId, PackageStore};

fn compile_kata(
    verification_source: &str,
    kata_implementation: &str,
) -> Result<(PackageStore, PackageId), String> {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());
    let unit = compile(
        &store,
        [stdlib],
        [verification_source, kata_implementation],
        "Kata.Verify()",
    );

    if !unit.context.errors().is_empty() {
        let error_message = format!("Compilation errors: {:?}", unit.context.errors());
        return Err(error_message);
    }

    let id = store.insert(unit);
    Ok((store, id))
}

#[must_use]
pub fn verify_kata(verification_source: &str, kata_implementation: &str) -> bool {
    // N.B. Once evaluation works for katas, run the Verify operation.
    match compile_kata(verification_source, kata_implementation) {
        Ok((_, _)) => true,
        Err(e) => {
            println!("{e}");
            false
        }
    }
}
