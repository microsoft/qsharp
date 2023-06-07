// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_definition, Definition};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};

#[test]
fn definition_callable() {
    assert_definition(
        r#"
    namespace Test {
        operation ◉Callee() : Unit {
        }

        operation Caller() : Unit {
            C↘allee();
        }
    }
    "#,
    );
}

/// Asserts that the definition found at the given cursor position matches the expected position.
/// The cursor position is indicated by a `↘` marker in the source text.
/// The expected definition position is indicated by a `◉` marker in the source text.
fn assert_definition(source_with_markers: &str) {
    let (source, cursor_offsets, target_offsets) =
        get_source_and_marker_offsets(source_with_markers);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_definition = get_definition(&compilation, "<source>", cursor_offsets[0]);
    let expected_definition = Definition {
        source: "<source>".to_string(),
        offset: target_offsets[0],
    };
    assert_eq!(&expected_definition, &actual_definition);
}
