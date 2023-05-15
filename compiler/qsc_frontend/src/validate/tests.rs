// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Error;
use crate::{parse, validate::Validator};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{ast::Namespace, visit::Visitor};

fn check(input: &str, expect: &Expect) {
    let (namespaces, parse_errors) = &mut parse::namespaces(input);
    assert!(parse_errors.is_empty());
    let validate_errors: Vec<_> = namespaces.iter().flat_map(validate).collect();
    expect.assert_debug_eq(&validate_errors);
}

fn validate(ns: &Namespace) -> Vec<Error> {
    let mut validator = Validator { errors: Vec::new() };
    validator.visit_namespace(ns);
    validator.errors
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
