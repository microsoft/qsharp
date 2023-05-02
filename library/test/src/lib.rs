// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]

#[cfg(test)]
mod tests;

use qsc::{
    interpret::{stateless, GenericReceiver, Value},
    SourceMap,
};

pub fn run_stdlib_test(expr: &str, expected: &Value) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);

    let sources = SourceMap::new([("test".into(), "".into())], Some(expr.into()));

    let context = stateless::Context::new(true, sources).expect("test should compile");
    let result = context
        .eval(&mut out)
        .expect("test should run successfully");

    assert_eq!(expected, &result);
}
