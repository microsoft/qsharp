// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::CompilationContext;
//use expect_test::expect;

#[test]
fn check_two_calls_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        operation Foo() : Unit {
            Bar();
        }
        operation Bar() : Unit {
            Foo();
        }"#,
    );
}
