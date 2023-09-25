// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_signature_help;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the signature help given at the cursor position matches the expected signature help.
/// The cursor position is indicated by a `↘` marker in the source text.
fn check(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_signature_help(&compilation, "<source>", cursor_offsets[0])
        .expect("Expected a signature help.");
    expect.assert_debug_eq(&actual);
}

#[test]
fn first_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar() : Unit {
                Foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : Double, z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 35,
                                    end: 45,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 1,
            }
        "#]],
    );
}

#[test]
fn mid_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar() : Unit {
                Foo(12↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : Double, z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 35,
                                    end: 45,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 1,
            }
        "#]],
    );
}

#[test]
fn second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1,↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : Double, z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 35,
                                    end: 45,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn last_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1, 1.2,↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : Double, z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 35,
                                    end: 45,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 3,
            }
        "#]],
    );
}

#[ignore = "Parser needs updating to handle `(1,, \"Four\")`"]
#[test]
fn insert_second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1,↘, "Four")
                let x = 3;
            }
        }
    "#},
        &expect![[r#""#]],
    );
}

#[test]
fn revisit_second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1, 2.↘3, "Four")
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : Double, z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 35,
                                    end: 45,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_call_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar(a : Int, b : Double) : Double { b }
            operation Baz() : Unit {
                Foo(1, Bar(↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Bar(a : Int, b : Double) : Double",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 1,
            }
        "#]],
    );
}

#[test]
fn nested_call_second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Double, z : String) : Unit {}
            operation Bar(a : Int, b : Double) : Double { b }
            operation Baz() : Unit {
                Foo(1, Bar(2,↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Bar(a : Int, b : Double) : Double",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 33,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn tuple_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : (Int, Double), z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1, ↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : (Int, Double), z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 53,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 40,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 42,
                                    end: 52,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn tuple_argument_first_item() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : (Int, Double), z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1, (↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : (Int, Double), z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 53,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 40,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 42,
                                    end: 52,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn tuple_argument_last_item() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : (Int, Double), z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2,↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : (Int, Double), z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 53,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 40,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 42,
                                    end: 52,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn tuple_argument_after_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : (Int, Double), z : String) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2, 3.0),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, y : (Int, Double), z : String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 53,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 40,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 42,
                                    end: 52,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 3,
            }
        "#]],
    );
}

#[test]
fn arguments_in_nested_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, ↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn first_inner_argument_in_nested_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 3,
            }
        "#]],
    );
}

#[test]
fn second_inner_argument_in_nested_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.3,↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 4,
            }
        "#]],
    );
}

#[test]
fn argument_after_nested_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.3, "Four"),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 5,
            }
        "#]],
    );
}

#[test]
fn argument_end_of_nested_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.3, "Four")↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn argument_nested_tuple_after_last() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.3, "Four" ↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_mismatch_after() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, 2,↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 5,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_mismatch_mid() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, 12↘3)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_mismatch_before() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1↘2, 123)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 1,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_not_enough_end() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(v : Int, (w : Double, x : String, y : Int), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.0, "Three")↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(v : Int, (w : Double, x : String, y : Int), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 67,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 56,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 48,
                                    end: 55,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 58,
                                    end: 66,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_not_enough_after() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(v : Int, (w : Double, x : String, y : Int), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.0, "Three"),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(v : Int, (w : Double, x : String, y : Int), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 67,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 56,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 48,
                                    end: 55,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 58,
                                    end: 66,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 6,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_not_enough_single_end() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.0)↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_not_enough_single_after() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2.0),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 5,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_not_enough_empty_end() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, ()↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_tuple_not_enough_empty_after() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(w : Int, (x : Double, y : String), z : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 58,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 47,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 34,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 36,
                                    end: 46,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 49,
                                    end: 57,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 5,
            }
        "#]],
    );
}

#[test]
fn nested_empty_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, (), y : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1,↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, (), y : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 36,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 25,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 27,
                                    end: 35,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_empty_tuple_mid() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, (), y : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, (), y : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 36,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 25,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 27,
                                    end: 35,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_empty_tuple_end() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, (), y : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, ()↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, (), y : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 36,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 25,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 27,
                                    end: 35,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 2,
            }
        "#]],
    );
}

#[test]
fn nested_empty_tuple_after() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, (), y : Bool) : Unit {}
            operation Bar() : Unit {
                Foo(1, (),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x : Int, (), y : Bool) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 36,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 25,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 27,
                                    end: 35,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 3,
            }
        "#]],
    );
}

#[test]
fn multi_nested_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(a : Int, (b : Int, (c : Int, d : Int), e : Int), f : Int) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2, (3, 4), 5),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(a : Int, (b : Int, (c : Int, d : Int), e : Int), f : Int) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 71,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 61,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 31,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 51,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 34,
                                    end: 41,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 43,
                                    end: 50,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 53,
                                    end: 60,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 63,
                                    end: 70,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 8,
            }
        "#]],
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn documentation_test() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Summary
            /// This is the operation `Foo`.
            /// # Input
            /// ## a
            /// This is the parameter `a`.
            /// ## b
            /// This is the parameter `b`.
            /// ## c
            /// This is the parameter `c`.
            /// ## d
            /// This is the parameter `d`.
            /// ## e
            /// This is the parameter `e`.
            /// ## f
            /// This is the parameter `f`.
            operation Foo(a : Int, (b : Int, (c : Int, d : Int), e : Int), f : Int) : Unit {}
            operation Bar() : Unit {
                Foo(1, (2, (3, 4), 5),↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(a : Int, (b : Int, (c : Int, d : Int), e : Int), f : Int) : Unit",
                        documentation: Some(
                            "This is the operation `Foo`.",
                        ),
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 71,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: Some(
                                    "This is the parameter `a`.",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 23,
                                    end: 61,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 24,
                                    end: 31,
                                },
                                documentation: Some(
                                    "This is the parameter `b`.",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 51,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 34,
                                    end: 41,
                                },
                                documentation: Some(
                                    "This is the parameter `c`.",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 43,
                                    end: 50,
                                },
                                documentation: Some(
                                    "This is the parameter `d`.",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 53,
                                    end: 60,
                                },
                                documentation: Some(
                                    "This is the parameter `e`.",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 63,
                                    end: 70,
                                },
                                documentation: Some(
                                    "This is the parameter `f`.",
                                ),
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 8,
            }
        "#]],
    );
}

#[test]
fn single_parameter_end() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(a : Int) : Unit {}
            operation Bar() : Unit {
                Foo(1↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(a : Int) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 22,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 1,
            }
        "#]],
    );
}

#[test]
fn single_parameter_after() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(a : Int) : Unit {}
            operation Bar() : Unit {
                Foo(1,↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(a : Int) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 22,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 0,
            }
        "#]],
    );
}

#[test]
fn single_parameter_before() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(a : Int) : Unit {}
            operation Bar() : Unit {
                Foo(↘ 1)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(a : Int) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 13,
                                    end: 22,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 21,
                                },
                                documentation: None,
                            },
                        ],
                    },
                ],
                active_signature: 0,
                active_parameter: 1,
            }
        "#]],
    );
}
