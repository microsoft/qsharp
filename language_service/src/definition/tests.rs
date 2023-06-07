// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_definition, Definition};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_cursor_offsets};

#[test]
fn definition_callable() {
    assert_definition(
        r#"
    namespace Test {
        operation ↘Callee() : Unit {
        }

        operation Caller() : Unit {
            C↘allee();
        }
    }
    "#,
        1, // get definition at second cursor
        0, // expect definition at first cursor
    );
}

fn assert_definition(source_with_cursor: &str, get_at_cursor: usize, definition_cursor: usize) {
    let (source, offsets) = get_source_and_cursor_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_definition = get_definition(&compilation, "<source>", offsets[get_at_cursor]);
    let expected_definition = Definition {
        source: "<source>".to_string(),
        offset: offsets[definition_cursor],
    };
    assert_eq!(
        &expected_definition, &actual_definition,
        "expected \n{expected_definition:?}\ngot:\n{actual_definition:?}"
    );
}
