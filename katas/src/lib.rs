// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc::{
    error::WithSource,
    stateless::{self, eval},
};
use qsc_eval::output::Receiver;
use qsc_eval::val::Value;
use qsc_frontend::compile::SourceMap;

const KATA_VERIFY: &str = "Kata.Verify()";

/// # Errors
/// Returns a vector of errors if compilation or evaluation failed.
///
/// # Panics
///
/// Will panic if Kata.Verify() does not return a Bool as result.
pub fn run_kata(
    exercise: &str,
    verify: &str,
    receiver: &mut impl Receiver,
) -> Result<bool, Vec<WithSource<stateless::Error>>> {
    let sources = SourceMap::new(
        [
            ("exercise".into(), exercise.into()),
            ("verify".into(), verify.into()),
        ],
        Some(KATA_VERIFY.into()),
    );

    // Return false if compilation or evaluation failed.
    // If evaluation succeeded, the result value must be a Bool and that's the value we should return.
    match eval(true, receiver, sources) {
        Ok(value) => match value {
            Value::Bool(value) => Ok(value),
            _ => panic!("{KATA_VERIFY} did not return a Bool value."),
        },
        Err(errors) => Err(errors),
    }
}
