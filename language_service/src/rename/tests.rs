// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::get_rename;
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use expect_test::{expect, Expect};
use indoc::indoc;

/// Asserts that the signature help given at the cursor position matches the expected signature help.
/// The cursor position is indicated by a `↘` marker in the source text.
fn check(source_with_markers: &str, expect: &Expect) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_rename(&compilation, "<source>", cursor_offsets[0]);
    expect.assert_debug_eq(&actual);
}

#[test]
fn callable_def() {
    check(
        indoc! {r#"
        namespace Test {
            operation Fo↘o(x : Int, y : Int, z : Int) : Unit {
                Foo(x, y, z);
            }
            operation Bar(x : Int, y : Int, z : Int) : Unit {
                Foo(x, y, z);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 79,
                    end: 82,
                },
                Span {
                    start: 161,
                    end: 164,
                },
                Span {
                    start: 31,
                    end: 34,
                },
            ]
        "#]],
    );
}

#[test]
fn callable_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {
                Foo(x, y, z);
            }
            operation Bar(x : Int, y : Int, z : Int) : Unit {
                Fo↘o(x, y, z);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 79,
                    end: 82,
                },
                Span {
                    start: 161,
                    end: 164,
                },
                Span {
                    start: 31,
                    end: 34,
                },
            ]
        "#]],
    );
}

#[test]
fn parameter_def() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(↘x : Int, y : Int, z : Int) : Unit {
                let temp = x;
                Foo(x, y, z);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 35,
                    end: 36,
                },
                Span {
                    start: 90,
                    end: 91,
                },
                Span {
                    start: 105,
                    end: 106,
                },
            ]
        "#]],
    );
}

#[test]
fn parameter_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {
                let temp = x;
                Foo(↘x, y, z);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 35,
                    end: 36,
                },
                Span {
                    start: 90,
                    end: 91,
                },
                Span {
                    start: 105,
                    end: 106,
                },
            ]
        "#]],
    );
}

#[test]
fn local_def() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {
                let t↘emp = x;
                Foo(temp, y, temp);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 83,
                    end: 87,
                },
                Span {
                    start: 105,
                    end: 109,
                },
                Span {
                    start: 114,
                    end: 118,
                },
            ]
        "#]],
    );
}

#[test]
fn local_ref() {
    check(
        indoc! {r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {
                let temp = x;
                Foo(t↘emp, y, temp);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 83,
                    end: 87,
                },
                Span {
                    start: 105,
                    end: 109,
                },
                Span {
                    start: 114,
                    end: 118,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_def() {
    check(
        indoc! {r#"
        namespace Test {
            newtype F↘oo = (fst : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                Bar(temp);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 81,
                    end: 84,
                },
                Span {
                    start: 114,
                    end: 117,
                },
                Span {
                    start: 29,
                    end: 32,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_constructor_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Foo = (fst : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = F↘oo(1, 2);
                Bar(temp);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 81,
                    end: 84,
                },
                Span {
                    start: 114,
                    end: 117,
                },
                Span {
                    start: 29,
                    end: 32,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Foo = (fst : Int, snd : Int);
            operation Bar(x : F↘oo) : Unit {
                let temp = Foo(1, 2);
                Bar(temp);
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 81,
                    end: 84,
                },
                Span {
                    start: 114,
                    end: 117,
                },
                Span {
                    start: 29,
                    end: 32,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_field_def() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Foo = (f↘st : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                let a = temp::fst;
                let b = Zip()::fst;
            }
            operation Zip() : Foo {
                Foo(1, 2)
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 147,
                    end: 150,
                },
                Span {
                    start: 175,
                    end: 178,
                },
                Span {
                    start: 36,
                    end: 39,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_field_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Foo = (fst : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                let a = temp::f↘st;
                let b = Zip()::fst;
            }
            operation Zip() : Foo {
                Foo(1, 2)
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 147,
                    end: 150,
                },
                Span {
                    start: 175,
                    end: 178,
                },
                Span {
                    start: 36,
                    end: 39,
                },
            ]
        "#]],
    );
}

#[test]
fn udt_field_complex_ref() {
    check(
        indoc! {r#"
        namespace Test {
            newtype Foo = (fst : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                let a = temp::fst;
                let b = Zip()::f↘st;
            }
            operation Zip() : Foo {
                Foo(1, 2)
            }
        }
    "#},
        &expect![[r#"
            [
                Span {
                    start: 147,
                    end: 150,
                },
                Span {
                    start: 175,
                    end: 178,
                },
                Span {
                    start: 36,
                    end: 39,
                },
            ]
        "#]],
    );
}
