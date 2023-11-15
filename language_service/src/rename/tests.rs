// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_rename, prepare_rename};
use crate::test_utils::{
    compile_notebook_with_fake_stdlib_and_markers, compile_with_fake_stdlib,
    get_source_and_marker_offsets, target_offsets_to_spans,
};
use expect_test::{expect, Expect};

/// Asserts that the rename locations given at the cursor position matches the expected rename locations.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected rename location ranges are indicated by `◉` markers in the source text.
fn check(source_with_markers: &str) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let target_spans = target_offsets_to_spans(&target_offsets);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = get_rename(&compilation, "<source>", cursor_offsets[0])
        .into_iter()
        .map(|l| l.span)
        .collect::<Vec<_>>();
    for target in &target_spans {
        assert!(actual.contains(target));
    }
    assert!(target_spans.len() == actual.len());
}

/// Asserts that the prepare rename given at the cursor position returns None.
/// The cursor position is indicated by a `↘` marker in the source text.
fn assert_no_rename(source_with_markers: &str) {
    let (source, cursor_offsets, _) = get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual = prepare_rename(&compilation, "<source>", cursor_offsets[0]);
    assert!(actual.is_none());
}

fn check_notebook(cells_with_markers: &[(&str, &str)], expect: &Expect) {
    let (compilation, cell_uri, offset, _) =
        compile_notebook_with_fake_stdlib_and_markers(cells_with_markers);
    let actual = get_rename(&compilation, &cell_uri, offset);
    expect.assert_debug_eq(&actual);
}

fn check_prepare_notebook(cells_with_markers: &[(&str, &str)], expect: &Expect) {
    let (compilation, cell_uri, offset, _) =
        compile_notebook_with_fake_stdlib_and_markers(cells_with_markers);
    let actual = prepare_rename(&compilation, &cell_uri, offset);
    expect.assert_debug_eq(&actual);
}

#[test]
fn callable_def() {
    check(
        r#"
        namespace Test {
            operation ◉Fo↘o◉(x : Int, y : Int, z : Int) : Unit {
                ◉Foo◉(x, y, z);
            }
            operation Bar(x : Int, y : Int, z : Int) : Unit {
                ◉Foo◉(x, y, z);
            }
        }
    "#,
    );
}

#[test]
fn callable_ref() {
    check(
        r#"
        namespace Test {
            operation ◉Foo◉(x : Int, y : Int, z : Int) : Unit {
                ◉Foo◉(x, y, z);
            }
            operation Bar(x : Int, y : Int, z : Int) : Unit {
                ◉Fo↘o◉(x, y, z);
            }
        }
    "#,
    );
}

#[test]
fn parameter_def() {
    check(
        r#"
        namespace Test {
            operation Foo(◉↘x◉ : Int, y : Int, z : Int) : Unit {
                let temp = ◉x◉;
                Foo(◉x◉, y, z);
            }
        }
    "#,
    );
}

#[test]
fn parameter_ref() {
    check(
        r#"
        namespace Test {
            operation Foo(◉x◉ : Int, y : Int, z : Int) : Unit {
                let temp = ◉x◉;
                Foo(◉↘x◉, y, z);
            }
        }
    "#,
    );
}

#[test]
fn local_def() {
    check(
        r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {
                let ◉t↘emp◉ = x;
                Foo(◉temp◉, y, ◉temp◉);
            }
        }
    "#,
    );
}

#[test]
fn local_ref() {
    check(
        r#"
        namespace Test {
            operation Foo(x : Int, y : Int, z : Int) : Unit {
                let ◉temp◉ = x;
                Foo(◉t↘emp◉, y, ◉temp◉);
            }
        }
    "#,
    );
}

#[test]
fn udt_def() {
    check(
        r#"
        namespace Test {
            newtype ◉F↘oo◉ = (fst : Int, snd : Int);
            operation Bar(x : ◉Foo◉) : Unit {
                let temp = ◉Foo◉(1, 2);
                Bar(temp);
            }
        }
    "#,
    );
}

#[test]
fn udt_constructor_ref() {
    check(
        r#"
        namespace Test {
            newtype ◉Foo◉ = (fst : Int, snd : Int);
            operation Bar(x : ◉Foo◉) : Unit {
                let temp = ◉F↘oo◉(1, 2);
                Bar(temp);
            }
        }
    "#,
    );
}

#[test]
fn udt_ref() {
    check(
        r#"
        namespace Test {
            newtype ◉Foo◉ = (fst : Int, snd : Int);
            operation Bar(x : ◉F↘oo◉) : Unit {
                let temp = ◉Foo◉(1, 2);
                Bar(temp);
            }
        }
    "#,
    );
}

#[test]
fn udt_field_def() {
    check(
        r#"
        namespace Test {
            newtype Foo = (◉f↘st◉ : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                let a = temp::◉fst◉;
                let b = Zip()::◉fst◉;
            }
            operation Zip() : Foo {
                Foo(1, 2)
            }
        }
    "#,
    );
}

#[test]
fn udt_field_ref() {
    check(
        r#"
        namespace Test {
            newtype Foo = (◉fst◉ : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                let a = temp::◉f↘st◉;
                let b = Zip()::◉fst◉;
            }
            operation Zip() : Foo {
                Foo(1, 2)
            }
        }
    "#,
    );
}

#[test]
fn udt_field_complex_ref() {
    check(
        r#"
        namespace Test {
            newtype Foo = (◉fst◉ : Int, snd : Int);
            operation Bar(x : Foo) : Unit {
                let temp = Foo(1, 2);
                let a = temp::◉fst◉;
                let b = Zip()::◉f↘st◉;
            }
            operation Zip() : Foo {
                Foo(1, 2)
            }
        }
    "#,
    );
}

#[test]
fn no_rename_namespace() {
    assert_no_rename(
        r#"
        namespace Te↘st {
            operation Foo() : Unit {}

        }
    "#,
    );
}

#[test]
fn no_rename_keyword() {
    assert_no_rename(
        r#"
        namespace Test {
            ope↘ration Foo() : Unit {}

        }
    "#,
    );
}

#[test]
fn no_rename_non_udt_type() {
    assert_no_rename(
        r#"
        namespace Test {
            operation Foo() : Un↘it {}

        }
    "#,
    );
}

#[test]
fn no_rename_string() {
    assert_no_rename(
        r#"
        namespace Test {
            operation Foo() : Unit {
                let temp = "He↘llo World!"
            }

        }
    "#,
    );
}

#[test]
fn no_rename_comment() {
    assert_no_rename(
        r#"
        namespace Test {
            // He↘llo World!
            operation Foo() : Unit {}

        }
    "#,
    );
}

#[test]
fn no_rename_std_item() {
    assert_no_rename(
        r#"
        namespace Test {
            operation Foo() : Unit {
                F↘ake();
            }

        }
    "#,
    );
}

#[test]
fn no_rename_non_id_character() {
    assert_no_rename(
        r#"
        namespace Test {
            operation Foo() ↘: Unit {
                Fake();
            }

        }
    "#,
    );
}

#[test]
fn no_rename_std_udt_return_type() {
    assert_no_rename(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Foo() : U↘dt {
        }
    }
    "#,
    );
}

#[test]
fn ty_param_def() {
    check(
        r#"
        namespace Test {
            operation Foo<'◉↘T◉>(x : '◉T◉) : '◉T◉ { x }
        }
    "#,
    );
}

#[test]
fn ty_param_ref() {
    check(
        r#"
        namespace Test {
            operation Foo<'◉T◉>(x : '◉↘T◉) : '◉T◉ { x }
        }
    "#,
    );
}

#[test]
fn notebook_rename_defined_in_later_cell() {
    check_prepare_notebook(
        &[
            ("cell1", "C↘allee();"),
            ("cell2", "operation Callee() : Unit {}"),
        ],
        &expect![[r#"
            None
        "#]],
    );
}

#[test]
fn notebook_rename_across_cells() {
    check_notebook(
        &[
            ("cell1", "operation Callee() : Unit {}"),
            ("cell2", "◉C↘allee◉();"),
        ],
        &expect![[r#"
            [
                Location {
                    source: "cell1",
                    span: Span {
                        start: 10,
                        end: 16,
                    },
                },
                Location {
                    source: "cell2",
                    span: Span {
                        start: 0,
                        end: 6,
                    },
                },
            ]
        "#]],
    );
}
