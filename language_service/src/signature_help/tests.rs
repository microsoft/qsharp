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

#[test]
fn indirect_local_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {}
            operation Bar() : Unit {
                let foo = Foo;
                foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, Int, Int) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 16,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 10,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 12,
                                    end: 15,
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
fn indirect_array_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {}
            operation Bar() : Unit {
                let foo = [Foo];
                foo[0](↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, Int, Int) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 16,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 10,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 12,
                                    end: 15,
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
fn indirect_block_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {}
            operation Bar() : Unit {
                ({ Foo }(↘))
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, Int, Int) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 16,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 10,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 12,
                                    end: 15,
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
fn indirect_unresolved_lambda_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Bar() : Unit {
                let foo = (x, y, z) => {};
                foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((?1, ?2, ?3) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 13,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 4,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 6,
                                    end: 8,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 10,
                                    end: 12,
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
fn indirect_partially_resolved_lambda_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Bar() : Unit {
                let foo = (x, y, z) => {};
                foo(1, ↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, ?2, ?3) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 14,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 9,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 11,
                                    end: 13,
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
fn indirect_resolved_lambda_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Bar() : Unit {
                let foo = (x, y, z) => {};
                foo(1, 2, 3);
                foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, Int, Int) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 16,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 10,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 12,
                                    end: 15,
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
fn controlled_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit is Ctl {}
            operation Bar() : Unit {
                Controlled Foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Qubit[], (Int, Int, Int)) => Unit is Ctl)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 27,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 9,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 11,
                                    end: 26,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 12,
                                    end: 15,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 17,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 25,
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
fn double_controlled_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit is Ctl {}
            operation Bar() : Unit {
                Controlled Controlled Foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Qubit[], (Qubit[], (Int, Int, Int))) => Unit is Ctl)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 38,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 9,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 11,
                                    end: 37,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 12,
                                    end: 19,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 21,
                                    end: 36,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 25,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 27,
                                    end: 30,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 32,
                                    end: 35,
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
fn partial_application_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {}
            operation Bar() : Unit {
                let foo = Foo(1, _, _);
                foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, Int) => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 11,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 10,
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
fn indirect_no_params_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let foo = Foo;
                foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "(Unit => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 5,
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
fn indirect_single_param_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int) : Unit {
                let foo = Foo;
                foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "(Int => Unit)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 4,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 4,
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
fn udt_constructor_call() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Foo = (fst : Int, snd : Double);
            operation Bar(x : Int) : Unit {
                let foo = Foo(↘)
                let x = 3;
            }
        }
    "#},
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "((Int, Double) -> Foo)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 14,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 2,
                                    end: 5,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 7,
                                    end: 13,
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
fn std_callable_with_udt() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Udt {
            TakesUdt(↘)
        }
    }
    "#,
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "function TakesUdt(input : Udt) : Udt",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 17,
                                    end: 30,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 18,
                                    end: 29,
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
fn indirect_callable_with_std_udt_args() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Udt {
            let callee = TakesUdt;
            callee(↘)
        }
    }
    "#,
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "(Udt -> Udt)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 4,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 4,
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
fn indirect_callable_with_std_udt() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Unit {
            let fn = UdtFn((x) -> x);
            fn!(↘)
        }
    }
    "#,
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "(Int -> Int)",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 4,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 1,
                                    end: 4,
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
fn indirect_callable_with_std_udt_with_params() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : Unit {
            let fn = UdtFnWithUdtParams(TakesUdt);
            fn!(↘)
        }
    }
    "#,
        &expect![[r#"
    SignatureHelp {
        signatures: [
            SignatureInformation {
                label: "(Udt -> Udt)",
                documentation: None,
                parameters: [
                    ParameterInformation {
                        label: Span {
                            start: 1,
                            end: 4,
                        },
                        documentation: None,
                    },
                    ParameterInformation {
                        label: Span {
                            start: 1,
                            end: 4,
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
