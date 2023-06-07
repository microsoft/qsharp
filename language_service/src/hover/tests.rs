// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_hover, Hover, Span};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};
use indoc::indoc;

#[test]
fn hover_callable() {
    assert_hover(
        r#"
        namespace Test {
            /// Doc comment!
            operation ◉F↘oo◉() : Unit {
            }
        }
    "#,
        Some(indoc!(
            r#"```qsharp
        operation Foo() : Unit
        ```

        Doc comment!


        "#
        )),
    );
}

/// Asserts that the hover text at the given cursor position matches the expected hover text.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected hover span is indicated by two `◉` markers in the source text.
fn assert_hover(source_with_markers: &str, expected_contents: Option<&str>) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_hover = get_hover(&compilation, "<source>", cursor_offsets[0]);
    let expected_hover: Option<Hover> = expected_contents.map(|expected_contents| Hover {
        contents: expected_contents.to_string(),
        span: Span {
            start: target_offsets[0],
            end: target_offsets[1],
        },
    });
    assert_eq!(&expected_hover, &actual_hover);
}
