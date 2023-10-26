// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[cfg(test)]
mod tests;

use qsc::{
    interpret::{
        output::Receiver,
        stateful::{self, Interpreter},
        Value,
    },
    PackageType, SourceContents, SourceMap, SourceName, TargetProfile,
};

pub const EXAMPLE_ENTRY: &str = "Kata.RunExample()";

pub const EXERCISE_ENTRY: &str = "Kata.Verification.CheckSolution()";

/// # Errors
///
/// Returns a vector of errors if compilation or evaluation failed.
///
/// # Panics
///
/// Will panic if evaluation does not return a boolean as result.
pub fn check_solution(
    exercise_sources: Vec<(SourceName, SourceContents)>,
    receiver: &mut impl Receiver,
) -> Result<bool, Vec<stateful::Error>> {
    let source_map = SourceMap::new(exercise_sources, Some(EXERCISE_ENTRY.into()));
    let mut interpreter: Interpreter =
        Interpreter::new(true, source_map, PackageType::Exe, TargetProfile::Full)?;
    interpreter.eval_entry(receiver).map(|value| {
        if let Value::Bool(success) = value {
            success
        } else {
            panic!("exercise verification did not return a boolean")
        }
    })
}
