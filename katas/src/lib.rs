// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc_eval::output::Receiver;
use qsc_eval::stateless::eval;

const KATA_VERIFY: &str = "Kata.Verify()";

#[must_use]
pub fn verify_kata(
    verification_source: &str,
    kata_implementation: &str,
    recv: &mut impl Receiver,
) -> bool {
    let sources = [verification_source, kata_implementation];
    run_kata(sources, recv).is_ok()
}

/// # Errors
/// Returns a vector of errors if compilation or evaluation failed.
///
/// # Panics
///
/// Will panic if Kata.Verify() does not return a Bool as result.
pub fn run_kata(
    sources: impl IntoIterator<Item = impl AsRef<str>>,
    recv: &mut impl Receiver,
) -> Result<bool, Vec<qsc_eval::stateless::Error>> {
    // Return false if compilation or evaluation failed.
    // If evaluation succeeded, the result value must be a Bool and that's the value we should return.
    let result = eval(false, KATA_VERIFY, recv, sources);
    if !result.errors.is_empty() {
        return Err(result.errors);
    }
    Ok(result
        .value
        .parse::<bool>()
        .unwrap_or_else(|_| panic!("{KATA_VERIFY} did not return a Bool value.")))
}
