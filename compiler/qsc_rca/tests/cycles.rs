// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_callable_compute_properties, CompilationContext};
use expect_test::expect;

#[ignore = "work in progress"]
#[test]
fn check_rca_for_function_direct_recursion() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo() : Unit {
            Foo();
        }"#,
    );

    check_callable_compute_properties(
        &compilation_context.fir_store,
        &compilation_context.compute_properties,
        "Foo",
        &expect![r#""#],
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_function_conditional_recursion() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(n : Int) : Unit {
            if n > 0 {
               Foo(n - 1);
            }
        }"#,
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_two_functions_cycle() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo() : Unit {
            Bar();
        }
        function Bar() : Unit {
            Foo();
        }"#,
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_function_with_int_arg_recursion() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(_ : Int) : Unit {
            Foo(0);
        }"#,
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_function_with_double_arg_recursion() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function Foo(_ : Double) : Unit {
            Foo(0);
        }"#,
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_body_recursion() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        operation Foo() : Unit {
            Foo();
        }"#,
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_adj_recursion() {
    let mut _compilation_context = CompilationContext::new();
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_ctl_recursion() {
    let mut _compilation_context = CompilationContext::new();
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_operation_ctl_adj_recursion() {
    let mut _compilation_context = CompilationContext::new();
}
