// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{parse::namespaces, validate::Validator};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{ast::Namespace, visit::Visitor};

use super::Error;

fn check(input: &str, expect: &Expect) {
    let (parsed, errs) = &mut namespaces(input);
    assert!(errs.is_empty());
    let errs: Vec<Error> = parsed.iter().flat_map(validate).collect();
    expect.assert_debug_eq(&errs);
}

fn validate(ns: &Namespace) -> Vec<Error> {
    let mut validator = Validator {
        validation_errors: Vec::new(),
    };
    validator.visit_namespace(ns);
    validator.validation_errors
}

#[test]
fn test_untyped_params() {
    check(
        "namespace input { operation Foo(a, b, c) : Unit {} }",
        &expect![[r#"
            [
                ParameterNotTyped(
                    "a",
                    Span {
                        lo: 32,
                        hi: 33,
                    },
                ),
                ParameterNotTyped(
                    "b",
                    Span {
                        lo: 35,
                        hi: 36,
                    },
                ),
                ParameterNotTyped(
                    "c",
                    Span {
                        lo: 38,
                        hi: 39,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_untyped_nested_params() {
    check(
        "namespace input { operation Foo(a : Int, (b : Int, c), d : Int) : Unit {} }",
        &expect![[r#"
            [
                ParameterNotTyped(
                    "c",
                    Span {
                        lo: 51,
                        hi: 52,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_callable_params() {
    check(
        "namespace input { operation Foo(a : Int -> Int) : Unit {} }",
        &expect![[r#"
            [
                NotCurrentlySupported(
                    "callables as parameters",
                    Span {
                        lo: 32,
                        hi: 46,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_callable_nested_params() {
    check(
        "namespace input { operation Foo(a : Int, (b : Int, c : Int => Int), d : Int) : Unit {} }",
        &expect![[r#"
            [
                NotCurrentlySupported(
                    "callables as parameters",
                    Span {
                        lo: 51,
                        hi: 65,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_adj_return_int() {
    check(
        "namespace input { operation Foo() : Int is Adj {} }",
        &expect![[r#"
            [
                NonUnitReturn(
                    "Foo",
                    Span {
                        lo: 36,
                        hi: 39,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_ctl_return_int() {
    check(
        "namespace input { operation Foo() : Int is Ctl {} }",
        &expect![[r#"
            [
                NonUnitReturn(
                    "Foo",
                    Span {
                        lo: 36,
                        hi: 39,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_lambda() {
    check("namespace input { operation Foo() : Int { let lambda = (x, y) -> x + y; return lambda(1, 2); } }",
    &expect![[r#"
        [
            NotCurrentlySupported(
                "lambdas",
                Span {
                    lo: 55,
                    hi: 70,
                },
            ),
        ]
    "#]],);
}

#[test]
fn test_partial() {
    check(
        indoc! {"
            namespace input {
                operation Foo(x : Int, y : Int) : Unit {}
                operation Bar() : Unit {
                    let foo = Foo(_, 2);
                    foo(1);
                }
            }
        "},
        &expect![[r#"
            [
                NotCurrentlySupported(
                    "partial applications",
                    Span {
                        lo: 111,
                        hi: 120,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_type_hole_param() {
    check(
        "namespace input { operation Foo(a : Int, b : _) : Unit { return b; } }",
        &expect![[r#"
            [
                NotCurrentlySupported(
                    "type holes",
                    Span {
                        lo: 45,
                        hi: 46,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_nested_type_hole_param() {
    check(
        indoc! {"
            namespace input {
                operation Foo(a : Int, b : (Int, _, Double)) : Unit {
                    let (_, x, _) = b;
                    return x;
                }
            }
        "},
        &expect![[r#"
            [
                NotCurrentlySupported(
                    "type holes",
                    Span {
                        lo: 55,
                        hi: 56,
                    },
                ),
            ]
        "#]],
    );
}
