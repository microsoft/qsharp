// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_references;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};
use indoc::indoc;

fn check(source_with_markers: &str, expect: &Expect, include_declaration: bool) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_references(
        &compilation,
        "<source>",
        cursor_offsets[0],
        include_declaration,
    );
    expect.assert_debug_eq(&actual);
}

fn check_include_decl(source_with_markers: &str, expect: &Expect) {
    check(source_with_markers, expect, true);
}

fn check_exclude_decl(source_with_markers: &str, expect: &Expect) {
    check(source_with_markers, expect, false);
}

#[test]
fn std_callable_ref() {
    check_include_decl(
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
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation F↘oo() : Unit {
                Foo();
                Foo();
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 31,
                },
                Location {
                    source: "<source>",
                    offset: 54,
                },
                Location {
                    source: "<source>",
                    offset: 69,
                },
            ]
        "#]],
    );
}

#[test]
fn callable_ref() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                Fo↘o();
                Foo();
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 31,
                },
                Location {
                    source: "<source>",
                    offset: 54,
                },
                Location {
                    source: "<source>",
                    offset: 69,
                },
            ]
        "#]],
    );
}

#[test]
fn callable_exclude_def() {
    check_exclude_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                Fo↘o();
                Foo();
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 54,
                },
                Location {
                    source: "<source>",
                    offset: 69,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_def() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            newtype B↘ar = (fst : Int, snd : Int);
            operation Foo(x : Bar) : Unit {
                let bar = Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 29,
                },
                Location {
                    source: "<source>",
                    offset: 81,
                },
                Location {
                    source: "<source>",
                    offset: 113,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_ref() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo(x : B↘ar) : Unit {
                let bar = Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 29,
                },
                Location {
                    source: "<source>",
                    offset: 81,
                },
                Location {
                    source: "<source>",
                    offset: 113,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_ref_constructor() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo(x : Bar) : Unit {
                let bar = B↘ar(1, 2);
                let baz = bar::fst;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 29,
                },
                Location {
                    source: "<source>",
                    offset: 81,
                },
                Location {
                    source: "<source>",
                    offset: 113,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_exclude_def() {
    check_exclude_decl(
        indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo(x : B↘ar) : Unit {
                let bar = Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 81,
                },
                Location {
                    source: "<source>",
                    offset: 113,
                },
            ]
        "#]],
    );
}

#[test]
fn std_udt_ref() {
    check_include_decl(
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
    check_include_decl(
        indoc! {r#"
        namespace Test {
            newtype Bar = (f↘st : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::fst;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 36,
                },
                Location {
                    source: "<source>",
                    offset: 140,
                },
            ]
        "#]],
    );
}

#[test]
fn field_ref() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::f↘st;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 36,
                },
                Location {
                    source: "<source>",
                    offset: 140,
                },
            ]
        "#]],
    );
}

#[test]
fn field_exclude_def() {
    check_exclude_decl(
        indoc! {r#"
        namespace Test {
            newtype Bar = (fst : Int, snd : Int);
            operation Foo() : Unit {
                let bar = Bar(1, 2);
                let baz = bar::f↘st;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 140,
                },
            ]
        "#]],
    );
}

#[test]
fn std_field_ref() {
    check_include_decl(
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
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let z↘ip = 3;
                let zap = zip;
            }
            operation Bar() : Unit {
                let zip = 3;
                let zap = zip;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 58,
                },
                Location {
                    source: "<source>",
                    offset: 85,
                },
            ]
        "#]],
    );
}

#[test]
fn local_ref() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let zip = 3;
                let zap = z↘ip;
            }
            operation Bar() : Unit {
                let zip = 3;
                let zap = zip;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 58,
                },
                Location {
                    source: "<source>",
                    offset: 85,
                },
            ]
        "#]],
    );
}

#[test]
fn param_def() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo(b↘ar : Int) : Unit {
                let lambda = (bar, baz) => {
                    let zip = bar;
                }
                let zip = bar;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 35,
                },
                Location {
                    source: "<source>",
                    offset: 147,
                },
            ]
        "#]],
    );
}

#[test]
fn param_ref() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo(bar : Int) : Unit {
                let lambda = (bar, baz) => {
                    let zip = b↘ar;
                }
                let zip = bar;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 77,
                },
                Location {
                    source: "<source>",
                    offset: 114,
                },
            ]
        "#]],
    );
}

#[test]
fn local_shadow_def() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let bar = 3;
                {
                    let b↘ar = 2.0;
                    let baz = bar;
                }
                let baz = bar;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 93,
                },
                Location {
                    source: "<source>",
                    offset: 126,
                },
            ]
        "#]],
    );
}

#[test]
fn local_shadow_ref() {
    check_include_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let bar = 3;
                {
                    let bar = 2.0;
                    let baz = ba↘r;
                }
                let baz = bar;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 93,
                },
                Location {
                    source: "<source>",
                    offset: 126,
                },
            ]
        "#]],
    );
}

#[test]
fn local_exclude_def() {
    check_exclude_decl(
        indoc! {r#"
        namespace Test {
            operation Foo() : Unit {
                let b↘ar = 3;
                let baz = bar;
            }
        }
    "#},
        &expect![[r#"
            [
                Location {
                    source: "<source>",
                    offset: 85,
                },
            ]
        "#]],
    );
}
