// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::get_references;
use crate::{
    test_utils::{
        compile_notebook_with_fake_stdlib_and_markers, compile_with_fake_stdlib_and_markers,
    },
    Encoding,
};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the reference locations given at the cursor position matches the expected reference locations.
/// The cursor position is indicated by a `↘` marker in the source text.
fn check_with_std(source_with_markers: &str, expect: &Expect) {
    let (compilation, cursor_position, _) =
        compile_with_fake_stdlib_and_markers(source_with_markers);
    let actual = get_references(
        &compilation,
        "<source>",
        cursor_position,
        Encoding::Utf8,
        true,
    );
    expect.assert_debug_eq(&actual);
}

/// Asserts that the reference locations given at the cursor position matches the expected reference locations.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected reference location ranges are indicated by `◉` markers in the source text.
fn check(source_with_markers: &str, include_declaration: bool) {
    let (compilation, cursor_position, target_spans) =
        compile_with_fake_stdlib_and_markers(source_with_markers);
    let actual = get_references(
        &compilation,
        "<source>",
        cursor_position,
        Encoding::Utf8,
        include_declaration,
    )
    .into_iter()
    .map(|l| l.range)
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
    let (compilation, cell_uri, position, target_spans) =
        compile_notebook_with_fake_stdlib_and_markers(cells_with_markers);

    let actual = get_references(&compilation, &cell_uri, position, Encoding::Utf8, false)
        .into_iter()
        .collect::<Vec<_>>();
    for target in &target_spans {
        assert!(
            actual.contains(target),
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
                    range: Range {
                        start: Position {
                            line: 1,
                            column: 26,
                        },
                        end: Position {
                            line: 1,
                            column: 30,
                        },
                    },
                },
                Location {
                    source: "<source>",
                    range: Range {
                        start: Position {
                            line: 3,
                            column: 8,
                        },
                        end: Position {
                            line: 3,
                            column: 12,
                        },
                    },
                },
                Location {
                    source: "<source>",
                    range: Range {
                        start: Position {
                            line: 5,
                            column: 8,
                        },
                        end: Position {
                            line: 5,
                            column: 12,
                        },
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
                    range: Range {
                        start: Position {
                            line: 4,
                            column: 24,
                        },
                        end: Position {
                            line: 4,
                            column: 27,
                        },
                    },
                },
                Location {
                    source: "<source>",
                    range: Range {
                        start: Position {
                            line: 2,
                            column: 22,
                        },
                        end: Position {
                            line: 2,
                            column: 25,
                        },
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
                    range: Range {
                        start: Position {
                            line: 4,
                            column: 31,
                        },
                        end: Position {
                            line: 4,
                            column: 32,
                        },
                    },
                },
                Location {
                    source: "<source>",
                    range: Range {
                        start: Position {
                            line: 4,
                            column: 23,
                        },
                        end: Position {
                            line: 4,
                            column: 24,
                        },
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
