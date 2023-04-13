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
fn test_entrypoint_attr_allowed() {
    check(
        indoc! {"
            namespace input {
                @EntryPoint()
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn test_entrypoint_attr_wrong_args() {
    check(
        indoc! {r#"
            namespace input {
                @EntryPoint("Bar")
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "#},
        &expect![[r#"
            [
                InvalidAttrArgs(
                    "()",
                    Span {
                        lo: 33,
                        hi: 40,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_unrecognized_attr() {
    check(
        indoc! {"
            namespace input {
                @Bar()
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            [
                UnrecognizedAttr(
                    "Bar",
                    Span {
                        lo: 22,
                        hi: 28,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_newtype_syntax_not_supported() {
    check(
        indoc! {"
            namespace input {
                newtype Bar = Baz : Int;
                operation Foo(a : Bar) : Unit {
                    let x = a!;
                    let y = a::Baz;
                }
            }
        "},
        &expect![[r#"
            [
                NotCurrentlySupported(
                    "newtype",
                    Span {
                        lo: 22,
                        hi: 46,
                    },
                ),
                NotCurrentlySupported(
                    "unwrap operator",
                    Span {
                        lo: 99,
                        hi: 101,
                    },
                ),
                NotCurrentlySupported(
                    "field access",
                    Span {
                        lo: 119,
                        hi: 125,
                    },
                ),
            ]
        "#]],
    );
}
