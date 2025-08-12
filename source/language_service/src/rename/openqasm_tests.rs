// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_rename, prepare_rename};
use crate::Encoding;
use crate::test_utils::openqasm::compile_with_markers;

/// Asserts that the rename locations given at the cursor position matches the expected rename locations.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected rename location ranges are indicated by `◉` markers in the source text.
fn check(source_with_markers: &str) {
    let (compilation, cursor_position, target_spans) = compile_with_markers(source_with_markers);
    let actual = get_rename(&compilation, "<source>", cursor_position, Encoding::Utf8)
        .into_iter()
        .map(|l| l.range)
        .collect::<Vec<_>>();
    for target in &target_spans {
        assert!(actual.contains(target));
    }
    assert!(target_spans.len() == actual.len());
}

/// Asserts that the prepare rename given at the cursor position returns None.
/// The cursor position is indicated by a `↘` marker in the source text.
fn assert_no_rename(source_with_markers: &str) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_markers);
    let actual = prepare_rename(&compilation, "<source>", cursor_position, Encoding::Utf8);
    assert!(actual.is_none());
}

#[test]
fn callable_def() {
    check(
        r#"
        def ◉Fo↘o◉(int x, int y, int z) {
            ◉Foo◉(x, y, z);
        }

        def Bar(int x, int y, int z) {
            ◉Foo◉(x, y, z);
        }
    "#,
    );
}

#[test]
fn callable_ref() {
    check(
        r#"
        def ◉Foo◉(int x, int y, int z) {
            ◉Foo◉(x, y, z);
        }

        def Bar(int x, int y, int z) {
            ◉Fo↘o◉(x, y, z);
        }
    "#,
    );
}

#[test]
fn gate_def() {
    check(
        r#"
        gate ◉Fo↘o◉(x, y, z) q { }

        gate Bar(x, y, z) q {
            ◉Foo◉(x, y, z) q;
        }
    "#,
    );
}

#[test]
fn gate_ref() {
    check(
        r#"
        gate ◉Foo◉(x, y, z) q { }

        gate Bar(x, y, z) q {
            ◉Fo↘o◉(x, y, z) q;
        }
    "#,
    );
}

#[test]
fn parameter_def() {
    check(
        r#"
        def Foo(int ◉↘x◉, int y, int z) {
            int temp = ◉x◉;
            Foo(◉x◉, y, z);
        }
    "#,
    );
}

#[test]
fn parameter_ref() {
    check(
        r#"
        def Foo(int ◉x◉, int y, int z) {
            int temp = ◉x◉;
            Foo(◉↘x◉, y, z);
        }
    "#,
    );
}

#[test]
fn local_def_in_def() {
    check(
        r#"
        int temp = x;
        def Foo(int x, int y, int z) {
            int ◉t↘emp◉ = x;
            Foo(◉temp◉, y, ◉temp◉);
        }
        Foo(temp, y, temp);
    "#,
    );
}

#[test]
fn local_ref_in_def() {
    check(
        r#"
        int temp = x;
        def Foo(int x, int y, int z) {
            int ◉temp◉ = x;
            Foo(◉t↘emp◉, y, ◉temp◉);
        }
        Foo(temp, y, temp);
    "#,
    );
}

#[test]
fn local_def() {
    check(
        r#"
        def Foo(int x, int y, int z) {
            int temp = x;
            Foo(temp, y, temp);
        }
        int ◉t↘emp◉ = x;
        Foo(◉temp◉, y, ◉temp◉);
    "#,
    );
}

#[test]
fn local_ref() {
    check(
        r#"
        def Foo(int x, int y, int z) {
            int temp = x;
            Foo(temp, y, temp);
        }
        int ◉temp◉ = x;
        Foo(◉t↘emp◉, y, ◉temp◉);
    "#,
    );
}

#[test]
fn input_def() {
    check(
        r#"
        def Foo(int x, int y, int z) {
            int temp = x;
            Foo(temp, y, temp);
        }
        input int ◉t↘emp◉;
        Foo(◉temp◉, y, ◉temp◉);
    "#,
    );
}

#[test]
fn input_ref() {
    check(
        r#"
        def Foo(int x, int y, int z) {
            int temp = x;
            Foo(temp, y, temp);
        }
        input int ◉temp◉;
        Foo(◉t↘emp◉, y, ◉temp◉);
    "#,
    );
}

#[test]
fn output_def() {
    check(
        r#"
        def Foo(int x, int y, int z) {
            int temp = x;
            Foo(temp, y, temp);
        }
        output int ◉t↘emp◉;
        Foo(◉temp◉, y, ◉temp◉);
    "#,
    );
}

#[test]
fn output_ref() {
    check(
        r#"
        def Foo(int x, int y, int z) {
            int temp = x;
            Foo(temp, y, temp);
        }
        output int ◉temp◉;
        Foo(◉t↘emp◉, y, ◉temp◉);
    "#,
    );
}

#[test]
fn no_rename_openqasm_header() {
    assert_no_rename(
        r#"
    OP↘ENQASM 3.0;
    "#,
    );
}

#[test]
fn no_rename_keyword() {
    assert_no_rename(
        r#"
    inc↘lude "stdgates.inc";
    "#,
    );
}

#[test]
fn no_rename_type() {
    assert_no_rename(
        r#"
    in↘t x;
    "#,
    );
}

#[test]
fn no_rename_string_literal() {
    assert_no_rename(
        r#"
    include "He↘llo World!";
    "#,
    );
}

#[test]
fn rename_for_loop_iter_def() {
    check(
        r#"
    def Foo(int x, int y, int z) {}
    for int ◉i↘ndex◉ in [0:10] {
        int temp = ◉index◉;
        Foo(◉index◉, 0, 7 * ◉index◉ + 3);
    }
    "#,
    );
}

#[test]
fn rename_for_loop_iter_ref() {
    check(
        r#"
    def Foo(int x, int y, int z) {}
    for int ◉index◉ in [0:10] {
        int temp = ◉↘index◉;
        Foo(◉index◉, 0, 7 * ◉index◉ + 3);
    }
    "#,
    );
}

#[test]
fn no_rename_comment() {
    assert_no_rename(
        r#"
    OPENQASM 3.0;
    // He↘llo World!
    include "stdgates.inc";
    "#,
    );
}

#[test]
fn no_rename_std_item() {
    assert_no_rename(
        r#"
    OPENQASM 3.0;
    include "stdgates.inc";

    // Built-in operation identifier shouldn't be renameable
    qubit[1] q;
    ↘x q[0];
    "#,
    );
}

#[test]
fn no_rename_intrinsic_3_item() {
    assert_no_rename(
        r#"
    OPENQASM 3.0;
    // Built-in operation identifier shouldn't be renameable
    qubit q;
    ↘U(0.0, 0.0, 0.0) q;
    "#,
    );
}

#[test]
fn no_rename_intrinsic_2_item() {
    assert_no_rename(
        r#"
    OPENQASM 2.0;
    // Built-in operation identifier shouldn't be renameable
    qubit q;
    ↘U(0.0, 0.0, 0.0) q;
    "#,
    );
}

#[test]
fn no_rename_intrinsic_const() {
    assert_no_rename(
        r#"
    float i = ↘pi * 7. / 8.;
    "#,
    );
}

#[test]
fn no_rename_non_id_character() {
    assert_no_rename(
        r#"
    // Non-identifier character '='
    int x ↘= 0;
    "#,
    );
}

#[test]
fn ty_param_def() {
    check(
        r#"
    // Use a parameter identifier to model rename
    def Foo(int ◉↘t◉) -> int { return ◉t◉; }
    "#,
    );
}

#[test]
fn ty_param_ref() {
    check(
        r#"
    def Foo(int ◉t◉) -> int { return ◉↘t◉; }
    "#,
    );
}

#[test]
#[ignore = "not yet implemented"]
fn sized_ty_param_ref() {
    check(
        r#"
    OPENQASM 3.0;
    include "stdgates.inc";
    const int ◉size◉ = 5;
    def Foo(int[◉↘size◉] t) -> int { return t; }
    const int[◉size◉] u = 10;
    float[◉size◉] v = 3.14;
    complex[float[◉size◉]] w = 1.0 + 2.0i;
    uint[◉size◉] x = 0;
    array[int[◉size◉]] y = [1, 2, 3, 4, 5];
    array[complex[float[◉size◉ - 3]]] z = [1.0 + 2.0i, 3.0 + 4.0i];
    "#,
    );
}
