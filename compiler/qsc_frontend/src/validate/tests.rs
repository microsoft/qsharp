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
    let mut validator = Validator { errors: Vec::new() };
    validator.visit_namespace(ns);
    validator.errors
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
