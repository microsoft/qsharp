// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod test_arrays;
#[cfg(test)]
mod test_convert;
#[cfg(test)]
mod tests;

use qsc::{
    interpret::{stateless, GenericReceiver, Value},
    SourceMap,
};

/// # Panics
///
/// Will panic if compilation fails or the result is not the same as expected.
pub fn run_stdlib_test(operation: &str, expected: &Value) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);
    let mut operation_in_namespace = String::from("namespace Test {");
    operation_in_namespace.push_str(operation);
    operation_in_namespace.push('}');

    let sources = SourceMap::new(
        [("test".into(), operation_in_namespace.into())],
        Some("Test.Test()".into()),
    );

    let context = stateless::Context::new(true, sources).expect("test should compile");
    let result = context
        .eval(&mut out)
        .expect("test should run successfully");

    assert_eq!(expected, &result);
}

/// # Panics
///
/// Will panic if compilation fails or the result is not the same as expected.
pub fn run_stdlib_test_expression(expr: &str, expected: &Value) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);

    let sources = SourceMap::new([("test".into(), "".into())], Some(expr.into()));

    let context = stateless::Context::new(true, sources).expect("test should compile");
    let result = context
        .eval(&mut out)
        .expect("test should run successfully");

    assert_eq!(expected, &result);
}
