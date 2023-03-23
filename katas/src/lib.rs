// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use qsc_frontend::compile::{self, compile, PackageStore};

pub fn verify_kata(verification_source: &str, kata_implementation: &str) -> bool {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());

    // Wrap both the verification source and the kata implementation into the same namespace.
    let namespace_begin = "namespace Kata {\n";
    let namespace_end = "\n}";
    let mut wrapped_verification_source = String::from(verification_source);
    wrapped_verification_source.insert_str(0, namespace_begin);
    wrapped_verification_source.push_str(namespace_end);
    let mut wrapped_kata_implementation = String::from(kata_implementation);
    wrapped_kata_implementation.insert_str(0, namespace_begin);
    wrapped_kata_implementation.push_str(namespace_end);

    // Validate that the code successfully compiles.
    // N.B. Once evaluation works for katas, the expression to compile should be "Kata.Verify()".
    let unit = compile(
        &store,
        [stdlib],
        [wrapped_verification_source, wrapped_kata_implementation],
        "",
    );
    if !unit.context.errors().is_empty() {
        println!("Compilation errors: {:?}", unit.context.errors());
        return false;
    }

    // N.B. Once evaluation works for katas, run the Verify operation.

    return true;
}
