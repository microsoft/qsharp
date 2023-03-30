// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc_eval::{output::GenericReceiver, val::Value, Evaluator};
use qsc_frontend::compile::{self, compile, PackageId, PackageStore};
use qsc_passes::globals::extract_callables;
use std::io;

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
    // Compile and run the kata.
    let verification_result =
        compile_kata(verification_source, kata_implementation).and_then(|(store, id)| {
            let globals = extract_callables(&store);
            let mut stdout = io::stdout();
            let mut out = GenericReceiver::new(&mut stdout);
            let evaluator = Evaluator::from_store(&store, id, &globals, &mut out);
            let unit = store
                .get(id)
                .expect("Compile unit should be in package store");
            let expr = unit
                .package
                .entry
                .as_ref()
                .expect("Entry expression should be present");
            evaluator
                .eval_expr(expr)
                .map_err(|_| String::from("Runtime error"))
        });

    // Return false if compilation or evaluation failed.
    // If evaluation succeeded, the result value must be a Bool and that's the value we should return.
    match verification_result {
        Ok((result, _)) => match result {
            Value::Bool(b) => b,
            _ => false,
        },
        Err(_e) => false,
    }
}
