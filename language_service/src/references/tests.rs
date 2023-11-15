// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_references;
use crate::{
    protocol,
    test_utils::{
        compile_notebook_with_fake_stdlib_and_markers, compile_with_fake_stdlib,
        get_source_and_marker_offsets, target_offsets_to_spans,
    },
};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the reference locations given at the cursor position matches the expected reference locations.
/// The cursor position is indicated by a `↘` marker in the source text.
fn check_with_std(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_references(&compilation, "<source>", cursor_offsets[0], true);
    expect.assert_debug_eq(&actual);
}

/// Asserts that the reference locations given at the cursor position matches the expected reference locations.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected reference location ranges are indicated by `◉` markers in the source text.
fn check(source_with_markers: &str, include_declaration: bool) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let target_spans = target_offsets_to_spans(&target_offsets);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_references(
        &compilation,
        "<source>",
        cursor_offsets[0],
        include_declaration,
    )
    .into_iter()
    .map(|l| l.span)
    .collect::<Vec<_>>();
    for target in &target_spans {
        assert!(
            actual.contains(target),
            "expected {actual:?} to contain {target:?}"
        );
    }
    assert!(target_spans.len() == actual.len());
}

fn check_include_decl(source_with_markers: &str) {
    check(source_with_markers, true);
}

fn check_exclude_decl(source_with_markers: &str) {
    check(source_with_markers, false);
}

fn check_notebook_exclude_decl(cells_with_markers: &[(&str, &str)]) {
    let (compilation, cell_uri, offset, target_spans) =
        compile_notebook_with_fake_stdlib_and_markers(cells_with_markers);

    let actual = get_references(&compilation, &cell_uri, offset, false)
        .into_iter()
        .collect::<Vec<_>>();
    for target in &target_spans {
        assert!(
            actual.contains(&protocol::Location {
                source: target.0.clone(),
                span: target.1
            }),
            "expected {actual:?} to contain {target:?}"
        );
    }
    assert!(target_spans.len() == actual.len());
}

#[test]
fn std_callable_ref() {
    check_with_std(
        indoc! {r#"
        namespace Test {
            open FakeStdLib;
            operation Foo() : Unit {
                Fa↘ke();
                let x = 3;
                Fake();
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "qsharp-library-source:<std>",
                    span: Span {
                        start: 49,
                        end: 53,
                    },
                },
                Location {
                    source: "<source>",
                    span: Span {
                        start: 75,
                        end: 79,
                    },
                },
                Location {
                    source: "<source>",
                    span: Span {
                        start: 110,
                        end: 114,
                    },
                },
            ]
        "#]],
    );
}

#[test]
fn callable_def() {
    check_include_decl(
        r#"
        namespace Test {
            operation ◉F↘oo◉() : Unit {
                ◉Foo◉();
                ◉Foo◉();
            }
        }
    "#,
    );
}

#[test]
fn callable_ref() {
    check_include_decl(
        r#"
        namespace Test {
            operation ◉Foo◉() : Unit {
                ◉Fo↘o◉();
                ◉Foo◉();
            }
        }
    "#,
    );
}

#[test]
fn callable_exclude_def() {
    check_exclude_decl(
        r#"
        namespace Test {
            operation Foo() : Unit {
                ◉Fo↘o◉();
                ◉Foo◉();
            }
        }
    "#,
    );
}

#[test]
fn udt_def() {
    check_include_decl(
        r#"
        namespace Test {
            newtype ◉B↘ar◉ = (fst : Int, snd : Int);
            operation Foo(x : ◉Bar◉) : Unit {
                let bar = ◉Bar◉(1, 2);
                let baz = bar::fst;
            }
        }
    "#,
    );
}

#[test]
fn udt_ref() {
    check_include_decl(
        r#"
        namespace Test {
            newtype ◉Bar◉ = (fst : Int, snd : Int);
            operation Foo(x : ◉B↘ar◉) : Unit {
                let bar = ◉Bar◉(1, 2);
                let baz = bar::fst;
            }
        }
    "#,
    );
}

#[test]
fn udt_ref_constructor() {
    check_include_decl(
        r#"
        namespace Test {
            newtype ◉Bar◉ = (fst : Int, snd : Int);
            operation Foo(x : ◉Bar◉) : Unit {
                let bar = ◉B↘ar◉(1, 2);
                let baz = bar::fst;
            }
        }
    "#,
    );
}

#[test]
fn udt_exclude_def() {
    check_exclude_decl(
        r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo(x : ◉B↘ar◉) : Unit {
                let bar = ◉Bar◉(1, 2);
                let baz = bar::fst;
            }
        }
    "#,
    );
}

#[test]
fn std_udt_ref() {
    check_with_std(
        indoc! {r#"
        namespace Test {
            open FakeStdLib;
            operation Foo(x : U↘dt) : Unit {}
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "qsharp-library-source:<std>",
                    span: Span {
                        start: 210,
                        end: 213,
                    },
                },
                Location {
                    source: "<source>",
                    span: Span {
                        start: 60,
                        end: 63,
                    },
                },
            ]
        "#]],
    );
}

#[test]
fn field_def() {
    check_include_decl(
        r#"
        namespace Test {
            newtype Bar = (◉f↘st◉ : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::◉fst◉;
            }
        }
    "#,
    );
}

#[test]
fn field_ref() {
    check_include_decl(
        r#"
        namespace Test {
            newtype Bar = (◉fst◉ : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::◉f↘st◉;
            }
        }
    "#,
    );
}

#[test]
fn field_exclude_def() {
    check_exclude_decl(
        r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::◉f↘st◉;
            }
        }
    "#,
    );
}

#[test]
fn std_field_ref() {
    check_with_std(
        indoc! {r#"
        namespace Test {
            open FakeStdLib;
            operation Foo() : Unit {
                let bar = Udt(1, 2);
                let baz = bar::↘x;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "qsharp-library-source:<std>",
                    span: Span {
                        start: 217,
                        end: 218,
                    },
                },
                Location {
                    source: "<source>",
                    span: Span {
                        start: 119,
                        end: 120,
                    },
                },
            ]
        "#]],
    );
}

#[test]
fn local_def() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let ◉z↘ip◉ = 3;
                let zap = ◉zip◉;
            }
            operation Bar() : Unit {
                let zip = 3;
                let zap = zip;
            }
        }
    "#,
    );
}

#[test]
fn local_ref() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let ◉zip◉ = 3;
                let zap = ◉z↘ip◉;
            }
            operation Bar() : Unit {
                let zip = 3;
                let zap = zip;
            }
        }
    "#,
    );
}

#[test]
fn param_def() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo(◉b↘ar◉ : Int) : Unit {
                let lambda = (bar, baz) => {
                    let zip = bar;
                }
                let zip = ◉bar◉;
            }
        }
    "#,
    );
}

#[test]
fn param_ref() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo(bar : Int) : Unit {
                let lambda = (◉bar◉, baz) => {
                    let zip = ◉b↘ar◉;
                }
                let zip = bar;
            }
        }
    "#,
    );
}

#[test]
fn local_shadow_def() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let bar = 3;
                {
                    let ◉b↘ar◉ = 2.0;
                    let baz = ◉bar◉;
                }
                let baz = bar;
            }
        }
    "#,
    );
}

#[test]
fn local_shadow_ref() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let bar = 3;
                {
                    let ◉bar◉ = 2.0;
                    let baz = ◉ba↘r◉;
                }
                let baz = bar;
            }
        }
    "#,
    );
}

#[test]
fn local_exclude_def() {
    check_exclude_decl(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let b↘ar = 3;
                let baz = ◉bar◉;
            }
        }
    "#,
    );
}

#[test]
fn ty_param_def() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo<◉'↘T◉>(x : ◉'T◉) : ◉'T◉ { x }
        }
    "#,
    );
}

#[test]
fn ty_param_ref() {
    check_include_decl(
        r#"
        namespace Test {
            operation Foo<◉'T◉>(x : ◉'↘T◉) : ◉'T◉ { x }
        }
    "#,
    );
}

#[test]
fn notebook_across_cells() {
    check_notebook_exclude_decl(&[
        ("cell1", "operation Callee() : Unit {}"),
        ("cell2", "◉C↘allee◉();"),
        ("cell3", "◉Callee◉();"),
    ]);
}

#[test]
fn notebook_defined_in_later_cell() {
    check_notebook_exclude_decl(&[
        ("cell1", "C↘allee();"),
        ("cell2", "operation Callee() : Unit {}"),
    ]);
}
