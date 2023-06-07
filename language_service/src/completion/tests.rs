// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_completions, CompletionItem, CompletionItemKind};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};

#[test]
fn in_block_contains_std_functions() {
    assert_completions_contain(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
    }"#,
        &[
            CompletionItem {
                label: "Fake".to_string(),
                kind: CompletionItemKind::Function,
            },
            CompletionItem {
                label: "FakeStdLib".to_string(),
                kind: CompletionItemKind::Module,
            },
        ],
    );
}

#[test]
fn in_namespace_contains_open() {
    assert_completions_contain(
        r#"
    namespace Test {
        ↘
        operation Test() : Unit {
        }
    }"#,
        &[CompletionItem {
            label: "open".to_string(),
            kind: CompletionItemKind::Keyword,
        }],
    );
}

#[test]
fn top_level_contains_namespace() {
    assert_completions_contain(
        r#"
        ↘
        "#,
        &[CompletionItem {
            label: "namespace".to_string(),
            kind: CompletionItemKind::Keyword,
        }],
    );
}

/// Asserts that the completion list at the given cursor position contains the expected completions.
/// The cursor position is indicated by a `↘` marker in the source text.
fn assert_completions_contain(source_with_cursor: &str, completions: &[CompletionItem]) {
    let (source, cursor_offset, _) = get_source_and_marker_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset[0]);
    for expected_completion in completions.iter() {
        assert!(
            actual_completions.items.contains(expected_completion),
            "expected to find\n{expected_completion:?}\nin:\n{actual_completions:?}"
        );
    }
}
