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
fn foo() {
    check(
        indoc! {r#"
        namespace Test {
            operation F↘oo(x : Int, y : Double, z : String) : Unit {}
        }
    "#},
        &expect![[r#""#]],
    );
}
