// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_hover;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};

/// Asserts that the hover text at the given cursor position matches the expected hover text.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected hover span is indicated by two `◉` markers in the source text.
fn check(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_hover(&compilation, "<source>", cursor_offsets[0]);
    expect.assert_debug_eq(&actual);
}

#[test]
fn hover_callable_unit_types() {
    check(
        r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉() : Unit {}
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation Foo Unit => Unit\n```\n",
                    span: Span {
                        start: 77,
                        end: 80,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_callable_with_callable_types() {
    check(
        r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉(x : (Int => Int)) : (Int => Int) {x}
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation Foo (Int => Int is 0) => (Int => Int)\n```\n",
                    span: Span {
                        start: 77,
                        end: 80,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_call() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit { ◉B↘ar◉(); }

            operation Bar() : Unit {}
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation Bar Unit => Unit\n```\n",
                    span: Span {
                        start: 63,
                        end: 66,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let ◉↘x◉ = 3;
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nx Int\n```\n",
                    span: Span {
                        start: 83,
                        end: 84,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_ref() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let x = 3;
                let y = ◉↘x◉;
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nx Int\n```\n",
                    span: Span {
                        start: 114,
                        end: 115,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_tuple() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let (x, ◉↘y◉) = (3, 1.4);
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ny Double\n```\n",
                    span: Span {
                        start: 87,
                        end: 88,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_tuple_ref() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let (x, y) = (3, 1.4);
                let z = ◉↘y◉;
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ny Double\n```\n",
                    span: Span {
                        start: 126,
                        end: 127,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_for_loop() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                for ◉↘i◉ in 0..10 {
                    let y = i;
                }
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ni Int\n```\n",
                    span: Span {
                        start: 83,
                        end: 84,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_for_loop_ref() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                for i in 0..10 {
                    let y = ◉↘i◉;
                }
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ni Int\n```\n",
                    span: Span {
                        start: 124,
                        end: 125,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_nested_ref() {
    check(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let x = 3;
                if true {
                    let y = ◉↘x◉;
                }
            }
        }
    "#,
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nx Int\n```\n",
                    span: Span {
                        start: 144,
                        end: 145,
                    },
                },
            )
        "#]],
    );
}
