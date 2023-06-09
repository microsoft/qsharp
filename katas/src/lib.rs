// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc::{
    interpret::{output::Receiver, stateless, Value},
    SourceMap,
};

pub const EXAMPLE_ENTRY: &str = "Kata.RunExample()";

pub const EXERCISE_ENTRY: &str = "Kata.VerifyExercise()";

/// # Errors
///
/// Returns a vector of errors if compilation or evaluation failed.
///
/// # Panics
///
/// Will panic if evaluation does not return a boolean as result.
pub fn run_kata(
    sources: SourceMap,
    receiver: &mut impl Receiver,
) -> Result<bool, Vec<stateless::Error>> {
    let context = stateless::Context::new(true, sources)?;
    context.eval(receiver).map(|value| {
        if let Value::Bool(success) = value {
            success
        } else {
            panic!("kata did not return a boolean")
        }
    })
}
