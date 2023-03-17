// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{parse::namespaces, validate::Validator};
use qsc_ast::visit::Visitor;

pub(super) fn check(input: &str) {
    let (parsed, errs) = &mut namespaces(input);
    assert!(errs.is_empty());
    let validator = &mut Validator {};
    parsed
        .iter_mut()
        .for_each(|ns| validator.visit_namespace(ns));
}

#[test]
#[should_panic(expected = "Callable parameters must be type annotated.")]
fn test_untyped_params() {
    check("namespace input { operation Foo(a, b, c) : Unit {} }");
}

#[test]
#[should_panic(expected = "Callable parameters must be type annotated.")]
fn test_untyped_nested_params() {
    check("namespace input { operation Foo(a : Int, (b : Int, c), d : Int) : Unit {} }");
}

#[test]
#[should_panic(expected = "Callables as parameters are not currently supported.")]
fn test_callable_params() {
    check("namespace input { operation Foo(a : Int -> Int) : Unit {} }");
}

#[test]
#[should_panic(expected = "Callables as parameters are not currently supported.")]
fn test_callable_nested_params() {
    check(
        "namespace input { operation Foo(a : Int, (b : Int, c : Int => Int), d : Int) : Unit {} }",
    );
}

#[test]
#[should_panic(expected = "Adjointable and Controllable Operations must return Unit.")]
fn test_adj_return_int() {
    check("namespace input { operation Foo() : Int is Adj {} }");
}

#[test]
#[should_panic(expected = "Adjointable and Controllable Operations must return Unit.")]
fn test_ctl_return_int() {
    check("namespace input { operation Foo() : Int is Ctl {} }");
}

#[test]
#[should_panic(expected = "Lambdas are not currently supported.")]
fn test_lambda() {
    check("namespace input { operation Foo() : Int { let lambda = (x, y) -> x + y; return lambda(1, 2); } }");
}

#[test]
#[should_panic(expected = "Partial applications are not currently supported.")]
fn test_partial() {
    check(
        r#"
namespace input {
    operation Foo(x : Int, y : Int) : Unit {}
    operation Bar() : Unit {
        let foo = Foo(_, 2);
        foo(1);
    }
}"#,
    );
}
