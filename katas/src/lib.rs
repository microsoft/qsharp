// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc::{
    interpret::{output::Receiver, stateless, Value},
    SourceMap,
};

use qsc_frontend::compile::{SourceContents, SourceName};

pub const EXAMPLE_ENTRY: &str = "Kata.RunExample()";

pub const EXERCISE_ENTRY: &str = "Kata.VerifyExercise()";

/// # Errors
///
/// Returns a vector of errors if compilation or evaluation failed.
///
/// # Panics
///
/// Will panic if evaluation does not return a boolean as result.
pub fn verify_exercise(
    exercise_sources: Vec<(SourceName, SourceContents)>,
    receiver: &mut impl Receiver,
) -> Result<bool, Vec<stateless::Error>> {
    let mut all_sources = vec![(
        "kataslib.qs".into(),
        include_str!("../library/katas.qs").into(),
    )];
    all_sources.extend(exercise_sources);
    let source_map = SourceMap::new(all_sources, Some(EXERCISE_ENTRY.into()));
    let context = stateless::Context::new(true, source_map)?;
    context.eval(receiver).map(|value| {
        if let Value::Bool(success) = value {
            success
        } else {
            panic!("exercise verification did not return a boolean")
        }
    })
}
