// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_callable_compute_properties, CompilationContext};
use expect_test::expect;

#[ignore = "work in progress"]
#[test]
fn check_rca_for_repeated() {
    let compilation_context = CompilationContext::new();
    check_callable_compute_properties(
        &compilation_context.fir_store,
        compilation_context.get_compute_properties(),
        "Repeated",
        &expect![r#""#],
    );
}
