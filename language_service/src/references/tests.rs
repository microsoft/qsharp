// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_references;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};
use indoc::indoc;

fn check_with_std(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_references(&compilation, "<source>", cursor_offsets[0], true);
    expect.assert_debug_eq(&actual);
}

fn check(source_with_markers: &str, include_declaration: bool) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_references(
        &compilation,
        "<source>",
        cursor_offsets[0],
        include_declaration,
    )
    .into_iter()
    .map(|l| l.span.start)
    .collect::<Vec<_>>();
    let count = target_offsets.len();
    for target in target_offsets {
        assert!(actual.contains(&target));
    }
    assert!(count == actual.len());
}

fn check_include_decl(source_with_markers: &str) {
    check(source_with_markers, true);
}

fn check_exclude_decl(source_with_markers: &str) {
    check(source_with_markers, false);
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
                    offset: 49,
                },
                Location {
                    source: "<source>",
                    offset: 75,
                },
                Location {
                    source: "<source>",
                    offset: 110,
                },
            ]
        "#]],
    );
}

#[test]
fn callable_def() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation ◉F↘oo() : Unit {
                ◉Foo();
                ◉Foo();
            }
        }
    "#});
}

#[test]
fn callable_ref() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation ◉Foo() : Unit {
                ◉Fo↘o();
                ◉Foo();
            }
        }
    "#});
}

#[test]
fn callable_exclude_def() {
    check_exclude_decl(indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                ◉Fo↘o();
                ◉Foo();
            }
        }
    "#});
}

#[test]
fn udt_def() {
    check_include_decl(indoc! {r#"
        namespace Test {
            newtype ◉B↘ar = (fst : Int, snd : Int);
            operation Foo(x : ◉Bar) : Unit {
                let bar = ◉Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#});
}

#[test]
fn udt_ref() {
    check_include_decl(indoc! {r#"
        namespace Test {
            newtype ◉Bar = (fst : Int, snd : Int);
            operation Foo(x : ◉B↘ar) : Unit {
                let bar = ◉Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#});
}

#[test]
fn udt_ref_constructor() {
    check_include_decl(indoc! {r#"
        namespace Test {
            newtype ◉Bar = (fst : Int, snd : Int);
            operation Foo(x : ◉Bar) : Unit {
                let bar = ◉B↘ar(1, 2);
                let baz = bar::fst;
            }
        }
    "#});
}

#[test]
fn udt_exclude_def() {
    check_exclude_decl(indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo(x : ◉B↘ar) : Unit {
                let bar = ◉Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#});
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
                    offset: 210,
                },
                Location {
                    source: "<source>",
                    offset: 60,
                },
            ]
        "#]],
    );
}

#[test]
fn field_def() {
    check_include_decl(indoc! {r#"
        namespace Test {
            newtype Bar = (◉f↘st : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::◉fst;
            }
        }
    "#});
}

#[test]
fn field_ref() {
    check_include_decl(indoc! {r#"
        namespace Test {
            newtype Bar = (◉fst : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::◉f↘st;
            }
        }
    "#});
}

#[test]
fn field_exclude_def() {
    check_exclude_decl(indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::◉f↘st;
            }
        }
    "#});
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
                    offset: 217,
                },
                Location {
                    source: "<source>",
                    offset: 119,
                },
            ]
        "#]],
    );
}

#[test]
fn local_def() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let ◉z↘ip = 3;
                let zap = ◉zip;
            }
            operation Bar() : Unit {
                let zip = 3;
                let zap = zip;
            }
        }
    "#});
}

#[test]
fn local_ref() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let ◉zip = 3;
                let zap = ◉z↘ip;
            }
            operation Bar() : Unit {
                let zip = 3;
                let zap = zip;
            }
        }
    "#});
}

#[test]
fn param_def() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation Foo(◉b↘ar : Int) : Unit {
                let lambda = (bar, baz) => {
                    let zip = bar;
                }
                let zip = ◉bar;
            }
        }
    "#});
}

#[test]
fn param_ref() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation Foo(bar : Int) : Unit {
                let lambda = (◉bar, baz) => {
                    let zip = ◉b↘ar;
                }
                let zip = bar;
            }
        }
    "#});
}

#[test]
fn local_shadow_def() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let bar = 3;
                {
                    let ◉b↘ar = 2.0;
                    let baz = ◉bar;
                }
                let baz = bar;
            }
        }
    "#});
}

#[test]
fn local_shadow_ref() {
    check_include_decl(indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let bar = 3;
                {
                    let ◉bar = 2.0;
                    let baz = ◉ba↘r;
                }
                let baz = bar;
            }
        }
    "#});
}

#[test]
fn local_exclude_def() {
    check_exclude_decl(indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let b↘ar = 3;
                let baz = ◉bar;
            }
        }
    "#});
}
