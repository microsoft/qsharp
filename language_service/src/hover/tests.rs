// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_hover;
use crate::{
    hover::Span,
    test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets},
};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the hover text at the given cursor position matches the expected hover text.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected hover span is indicated by two `◉` markers in the source text.
fn check(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_hover(&compilation, "<source>", cursor_offsets[0]).expect("Expected a hover.");
    assert_eq!(
        &actual.span,
        &Span {
            start: target_offsets[0],
            end: target_offsets[1],
        }
    );
    expect.assert_debug_eq(&actual.contents);
}

/// Asserts that there is no hover for the given test case.
fn check_none(source_with_markers: &str) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_hover(&compilation, "<source>", cursor_offsets[0]);
    assert!(actual.is_none());
}

#[test]
fn hover_callable_unit_types() {
    check(
        indoc! {r#"
        namespace Test {
            /// Doc comment
            /// with multiple lines!
            operation ◉B↘ar◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "Doc comment\nwith multiple lines!\n```qsharp\noperation Bar Unit => Unit\n```\n"
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
            "Doc comment!\n```qsharp\noperation Foo (Int => Int) => (Int => Int)\n```\n"
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
            "```qsharp\noperation Bar Unit => Unit\n```\n"
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
            "Doc comment!\n```qsharp\noperation Foo Unit => Unit is Ctl\n```\n"
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
            "Doc comment!\n```qsharp\noperation Foo (Int => Int is Adj + Ctl) => (Int => Int is Adj) is Adj\n```\n"
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
            "```qsharp\noperation Bar Unit => Unit is Adj\n```\n"
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
            "```qsharp\nx: Int\n```\n"
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
            "```qsharp\nx: Int\n```\n"
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
            "```qsharp\ny: Double\n```\n"
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
            "```qsharp\ny: Double\n```\n"
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
            "```qsharp\ni: Int\n```\n"
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
            "```qsharp\ni: Int\n```\n"
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
            "```qsharp\nx: Int\n```\n"
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
            "```qsharp\nlambda: ((Double, String) => Int)\n```\n"
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
            "```qsharp\nlambda: ((Double, String) => Int)\n```\n"
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
            "```qsharp\ny: String\n```\n"
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
            "```qsharp\ny: String\n```\n"
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
            "```qsharp\na: Int\n```\n"
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
            "```qsharp\na: Pair\n```\n"
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
            "```qsharp\nPair = (Int, snd: Int)\n```\n"
        "#]],
    );
}

#[test]
fn hover_udt_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Bar = (fst: Int, (snd : Int, Double, fourth: String), Double, sixth: Int);
            operation Foo() : ◉B↘ar◉ {
                Bar(3, (4, 2.1, "Yes"), 4.7, 2)
            }
        }
    "#},
        &expect![[r#"
            "```qsharp\nBar = (fst: Int, (snd: Int, Double, fourth: String), Double, sixth: Int)\n```\n"
        "#]],
    );
}

#[test]
fn hover_udt_ref_nested_udt() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Pair = (fst: Int, snd: Int);
            newtype Bar = (fst: Int, (snd : Int, Double, fourth: Pair), Double, sixth: Int);
            operation Foo() : ◉B↘ar◉ {
                Bar(3, (4, 2.1, Pair(14, 15)), 4.7, 2)
            }
        }
    "#},
        &expect![[r#"
            "```qsharp\nBar = (fst: Int, (snd: Int, Double, fourth: Pair), Double, sixth: Int)\n```\n"
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
            "```qsharp\nPair = (Int, snd: Int)\n```\n"
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
            "```qsharp\nPair = (Int, snd: Int)\n```\n"
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
            "```qsharp\nsnd: Int\n```\n"
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
            "```qsharp\nsnd: Int\n```\n"
        "#]],
    );
}

#[test]
fn hover_primitive_type() {
    check_none(indoc! {r#"
        namespace Test {
            newtype Pair = (◉I↘nt◉, snd : Int);
            operation Foo() : Unit {
                let a = Pair(3, 4);
                let b = a::snd;
            }
        }
    "#});
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
            "```qsharp\noperation Fake Unit => Unit\n```\n"
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
            "```qsharp\noperation FakeCtlAdj Unit => Unit is Adj + Ctl\n```\n"
        "#]],
    );
}

#[test]
fn hover_callable_summary() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Summary
            /// This is a
            /// multi-line summary!
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "This is a\nmulti-line summary!\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "#]],
    );
}

#[test]
fn hover_callable_summary_stuff_before() {
    check(
        indoc! {r#"
        namespace Test {
            /// not the summary
            /// # Summary
            /// This is a
            /// multi-line summary!
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "This is a\nmulti-line summary!\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "#]],
    );
}

#[test]
fn hover_callable_summary_other_header_before() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Not The Summary
            /// This stuff is not the summary.
            /// # Summary
            /// This is a
            /// multi-line summary!
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "This is a\nmulti-line summary!\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "#]],
    );
}

#[test]
fn hover_callable_summary_other_header_after() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Summary
            /// This is a
            /// multi-line summary!
            /// # Not The Summary
            /// This stuff is not the summary.
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "This is a\nmulti-line summary!\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "#]],
    );
}

#[test]
fn hover_callable_summary_other_headers() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Not The Summary
            /// This stuff is not the summary.
            /// # Summary
            /// This is a
            /// multi-line summary!
            /// # Also Not The Summary
            /// This stuff is also not the summary.
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "This is a\nmulti-line summary!\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "#]],
    );
}

#[test]
fn hover_callable_headers_but_no_summary() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Not The Summary
            /// This stuff is not the summary.
            /// # Also Not The Summary
            /// This stuff is also not the summary.
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r##"
            "# Not The Summary\nThis stuff is not the summary.\n# Also Not The Summary\nThis stuff is also not the summary.\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "##]],
    );
}

#[test]
fn hover_callable_summary_only_header_matches() {
    check(
        indoc! {r#"
        namespace Test {
            /// # Not The Summary
            /// This stuff is not the # Summary.
            /// # Summary
            /// This is a
            /// multi-line # Summary!
            /// # Also Not The Summary
            /// This stuff is also not the # Summary.
            operation ◉F↘oo◉() : Unit {}
        }
    "#},
        &expect![[r#"
            "This is a\nmulti-line # Summary!\n```qsharp\noperation Foo Unit => Unit\n```\n"
        "#]],
    );
}
