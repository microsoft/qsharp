// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_signature_help;
use crate::{
    protocol,
    test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets},
};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the hover text at the given cursor position matches the expected hover text.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected hover span is indicated by two `◉` markers in the source text.
fn check(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_signature_help(&compilation, "<source>", cursor_offsets[0])
        .expect("Expected a signature help.");
    expect.assert_debug_eq(&actual);
}

#[test]
fn callable_unit_types() {
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
        &expect![[r#"
            SignatureHelp {
                signatures: [
                    SignatureInformation {
                        label: "operation Foo(a: Int, b: Double, c: String) : Unit",
                        documentation: None,
                        parameters: [
                            ParameterInformation {
                                label: Span {
                                    start: 14,
                                    end: 20,
                                },
                                documentation: Some(
                                    "The parameter `a`",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 22,
                                    end: 31,
                                },
                                documentation: Some(
                                    "The parameter `b`",
                                ),
                            },
                            ParameterInformation {
                                label: Span {
                                    start: 33,
                                    end: 42,
                                },
                                documentation: Some(
                                    "The parameter `c`",
                                ),
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
