// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_hover;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};
use indoc::indoc;

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
        indoc! {r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nDoc comment!\noperation Foo Unit => Unit\n```\n",
                    span: Span {
                        start: 52,
                        end: 55,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_callable_with_callable_types() {
    check(
        indoc! {r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉(x : (Int => Int)) : (Int => Int) {x}
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nDoc comment!\noperation Foo (Int => Int) => (Int => Int)\n```\n",
                    span: Span {
                        start: 52,
                        end: 55,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_call() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit { ◉B↘ar◉(); }

            operation Bar() : Unit {}
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation Bar Unit => Unit\n```\n",
                    span: Span {
                        start: 46,
                        end: 49,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_callable_unit_types_functors() {
    check(
        indoc! {r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉() : Unit is Ctl {}
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nDoc comment!\noperation Foo Unit => Unit is Ctl\n```\n",
                    span: Span {
                        start: 52,
                        end: 55,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_callable_with_callable_types_functors() {
    check(
        indoc! {r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉(x : (Int => Int is Ctl + Adj)) : (Int => Int is Adj) is Adj {x}
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nDoc comment!\noperation Foo (Int => Int is Adj + Ctl) => (Int => Int is Adj) is Adj\n```\n",
                    span: Span {
                        start: 52,
                        end: 55,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_call_functors() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit { ◉B↘ar◉(); }

            operation Bar() : Unit is Adj {}
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation Bar Unit => Unit is Adj\n```\n",
                    span: Span {
                        start: 46,
                        end: 49,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let ◉↘x◉ = 3;
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nx: Int\n```\n",
                    span: Span {
                        start: 58,
                        end: 59,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let x = 3;
                let y = ◉↘x◉;
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nx: Int\n```\n",
                    span: Span {
                        start: 81,
                        end: 82,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_tuple() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let (x, ◉↘y◉) = (3, 1.4);
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ny: Double\n```\n",
                    span: Span {
                        start: 62,
                        end: 63,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_tuple_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let (x, y) = (3, 1.4);
                let z = ◉↘y◉;
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ny: Double\n```\n",
                    span: Span {
                        start: 93,
                        end: 94,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_for_loop() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                for ◉↘i◉ in 0..10 {
                    let y = i;
                }
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ni: Int\n```\n",
                    span: Span {
                        start: 58,
                        end: 59,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_for_loop_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                for i in 0..10 {
                    let y = ◉↘i◉;
                }
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ni: Int\n```\n",
                    span: Span {
                        start: 91,
                        end: 92,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_nested_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let x = 3;
                if true {
                    let y = ◉↘x◉;
                }
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nx: Int\n```\n",
                    span: Span {
                        start: 103,
                        end: 104,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_lambda() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let a = 3;
                let ◉la↘mbda◉ = (x, y) => a;
                let b = lambda(1.2, "yes");
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nlambda: ((Double, String) => Int)\n```\n",
                    span: Span {
                        start: 77,
                        end: 83,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_lambda_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let a = 3;
                let lambda = (x, y) => a;
                let b = ◉la↘mbda◉(1.2, "yes");
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nlambda: ((Double, String) => Int)\n```\n",
                    span: Span {
                        start: 115,
                        end: 121,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_lambda_param() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let a = 3;
                let lambda = (x, ◉↘y◉) => a;
                let b = lambda(1.2, "yes");
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ny: String\n```\n",
                    span: Span {
                        start: 90,
                        end: 91,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_lambda_param_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let lambda = (x, y) => ◉↘y◉;
                let a = lambda(1.2, "yes");
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\ny: String\n```\n",
                    span: Span {
                        start: 77,
                        end: 78,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_lambda_closure_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let a = 3;
                let lambda = (x, y) => ◉↘a◉;
                let b = lambda(1.2, "yes");
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\na: Int\n```\n",
                    span: Span {
                        start: 96,
                        end: 97,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_identifier_udt() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (fst : Int, snd : Int);
            operation Foo() : Unit {
                let a = Pair(3, 4);
                let b = ◉↘a◉;
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\na: Pair\n```\n",
                    span: Span {
                        start: 133,
                        end: 134,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_udt() {
    check(
        indoc! {r#"
        namespace Test {
            newtype ◉P↘air◉ = (Int, snd : Int);
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nPair\n```\n",
                    span: Span {
                        start: 29,
                        end: 33,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_udt_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (Int, snd : Int);
            operation Foo() : ◉P↘air◉ {
                Pair(3, 4)
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nPair\n```\n",
                    span: Span {
                        start: 76,
                        end: 80,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_udt_anno_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (Int, snd : Int);
            operation Foo() : Unit {
                let a : ◉P↘air◉ = Pair(3, 4);
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nPair\n```\n",
                    span: Span {
                        start: 99,
                        end: 103,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_udt_constructor() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (Int, snd : Int);
            operation Foo() : Unit {
                let a = ◉P↘air◉(3, 4);
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nPair\n```\n",
                    span: Span {
                        start: 99,
                        end: 103,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_udt_field() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (Int, ◉s↘nd◉ : Int);
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nsnd: Int\n```\n",
                    span: Span {
                        start: 42,
                        end: 45,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_udt_field_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (Int, snd : Int);
            operation Foo() : Unit {
                let a = Pair(3, 4);
                let b = a::◉s↘nd◉;
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\nsnd: Int\n```\n",
                    span: Span {
                        start: 130,
                        end: 133,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_primitive_type() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (◉I↘nt◉, snd : Int);
            operation Foo() : Unit {
                let a = Pair(3, 4);
                let b = a::snd;
            }
        }
    "#},
        &expect![[r#"
            None
        "#]],
    );
}

#[test]
fn hover_foreign_call() {
    check(
        indoc! {r#"
        namespace Test {
            open FakeStdLib;
            operation Foo() : Unit {
                ◉F↘ake◉();
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation Fake Unit => Unit\n```\n",
                    span: Span {
                        start: 75,
                        end: 79,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn hover_foreign_call_functors() {
    check(
        indoc! {r#"
        namespace Test {
            open FakeStdLib;
            operation Foo() : Unit {
                ◉F↘akeCtlAdj◉();
            }
        }
    "#},
        &expect![[r#"
            Some(
                Hover {
                    contents: "```qsharp\noperation FakeCtlAdj Unit => Unit is Adj + Ctl\n```\n",
                    span: Span {
                        start: 75,
                        end: 85,
                    },
                },
            )
        "#]],
    );
}
