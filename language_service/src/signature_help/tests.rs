// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_signature_help;
use crate::{
    protocol::SignatureHelpContext,
    test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets},
};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the hover text at the given cursor position matches the expected hover text.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected hover span is indicated by two `◉` markers in the source text.
fn check(source_with_markers: &str, context: SignatureHelpContext, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_signature_help(&compilation, "<source>", cursor_offsets[0], context)
        .expect("Expected a signature help.");
    expect.assert_debug_eq(&actual);
}

#[test]
fn first_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar() : Unit {
                Foo(↘)
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some("(".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x: Int, y: Double, z: String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 42,
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
fn mid_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar() : Unit {
                Foo(12↘)
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 3,
            trigger_character: None,
            is_retrigger: true,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x: Int, y: Double, z: String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 42,
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
fn second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar() : Unit {
                Foo(1,↘)
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x: Int, y: Double, z: String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 42,
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
fn last_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar() : Unit {
                Foo(1, 1.2,↘)
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x: Int, y: Double, z: String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 42,
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

#[ignore = "Parser needs updating to handle `(1,, \"Four\")`"]
#[test]
fn insert_second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar() : Unit {
                Foo(1,↘, "Four")
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#""#]],
    );
}

#[test]
fn revisit_second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar() : Unit {
                Foo(1, 2.↘3, "Four")
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(x: Int, y: Double, z: String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 42,
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
fn nested_call_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar(a: Int, b: Double) : Double { b }
            operation Baz() : Unit {
                Foo(1, Bar(↘))
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Bar(a: Int, b: Double) : Double",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
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
fn nested_call_second_argument() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x: Int, y: Double, z: String) : Unit {}
            operation Bar(a: Int, b: Double) : Double { b }
            operation Baz() : Unit {
                Foo(1, Bar(2,↘))
                let x = 3;
            }
        }
    "#},
        SignatureHelpContext {
            trigger_kind: 2,
            trigger_character: Some(",".to_string()),
            is_retrigger: false,
            active_signature_help: None,
        },
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Bar(a: Int, b: Double) : Double",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: None,
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
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
