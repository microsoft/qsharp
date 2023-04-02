// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{parse::namespaces, validate::validate};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::ast::{NodeId, Package};

use super::Error;

fn check(input: &str, expect: &Expect) {
    let (parsed, errs) = namespaces(input);
    assert!(errs.is_empty());
    let errs: Vec<Error> = validate(&Package {
        id: NodeId::zero(),
        namespaces: parsed,
        entry: None,
    });
    expect.assert_debug_eq(&errs);
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

#[test]
fn test_elided_required() {
    check(
        indoc! {"
            namespace input {
                operation Foo(a : Int) : Unit is Adj + Ctl {
                    body a {}
                    controlled (ctls, ...) {}
                }
            }
        "},
        &expect![[r#"
            [
                ElidedRequired(
                    Span {
                        lo: 80,
                        hi: 81,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_elided_tuple_required() {
    check(
        indoc! {"
            namespace input {
                operation Foo(a : Int) : Unit is Adj + Ctl {
                    body ... {}
                    controlled ... {}
                }
            }
        "},
        &expect![[r#"
            [
                ElidedTupleRequired(
                    Span {
                        lo: 106,
                        hi: 109,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_main_args_single() {
    check(
        indoc! {"
            namespace test {
                operation main(a : Int) : Unit {}
            }
        "},
        &expect![[r#"
            [
                MainArgs(
                    Span {
                        lo: 35,
                        hi: 44,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_main_args_tuple() {
    check(
        indoc! {"
            namespace test {
                operation main(a : Int, b : Int) : Unit {}
            }
        "},
        &expect![[r#"
            [
                MainArgs(
                    Span {
                        lo: 35,
                        hi: 53,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_main_no_args_allowed() {
    check(
        indoc! {"
            namespace test {
                operation main() : Unit {}
            }
        "},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn test_main_spec_decl() {
    check(
        indoc! {"
            namespace test {
                operation main() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            [
                MainSpecDecl(
                    Span {
                        lo: 21,
                        hi: 72,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_duplicate_main() {
    check(
        indoc! {"
            namespace test {
                operation main() : Unit {}
            }
            namespace test2 {
                operation main() : Unit {}
            }
            namespace test3 {
                operation main() : Unit {}
            }
        "},
        &expect![[r#"
            [
                DuplicateMain(
                    Span {
                        lo: 82,
                        hi: 86,
                    },
                ),
                DuplicateMain(
                    Span {
                        lo: 133,
                        hi: 137,
                    },
                ),
            ]
        "#]],
    );
}
