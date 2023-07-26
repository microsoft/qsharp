// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{get_completions, CompletionItemKind};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};

/// Asserts that the completion list at the given cursor position contains the expected completions.
/// The cursor position is indicated by a `↘` marker in the source text.
fn assert_completions_contain(
    source_with_cursor: &str,
    completions: &[(&str, CompletionItemKind)],
) {
    let (source, cursor_offset, _) = get_source_and_marker_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset[0]);
    for expected_completion in completions.iter() {
        assert!(
            actual_completions
                .items
                .iter()
                .any(|c| c.kind == expected_completion.1 && c.label == expected_completion.0),
            "expected to find\n{expected_completion:?}\nin:\n{actual_completions:?}"
        );
    }
}

fn assert_completions_contain_detail(
    source_with_cursor: &str,
    completions: &[(&str, CompletionItemKind, Option<String>)],
) {
    let (source, cursor_offset, _) = get_source_and_marker_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset[0]);
    for expected_completion in completions.iter() {
        assert!(
            actual_completions
                .items
                .iter()
                .any(|c| c.kind == expected_completion.1
                    && c.label == expected_completion.0
                    && c.detail == expected_completion.2),
            "expected to find\n{expected_completion:?}\nin:\n{actual_completions:?}"
        );
    }
}

#[test]
fn in_block_contains_std_functions() {
    assert_completions_contain_detail(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
    }"#,
        &[
            (
                "Fake",
                CompletionItemKind::Function,
                Some("operation Fake() : Unit".to_owned()),
            ),
            (
                "FakeWithParam",
                CompletionItemKind::Function,
                Some("operation FakeWithParam(x: Int) : Unit".to_owned()),
            ),
            (
                "FakeCtlAdj",
                CompletionItemKind::Function,
                Some("operation FakeCtlAdj() : Unit is Adj + Ctl".to_owned()),
            ),
            ("FakeStdLib", CompletionItemKind::Module, None),
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
        &[("open", CompletionItemKind::Keyword)],
    );
}

#[test]
fn top_level_contains_namespace() {
    assert_completions_contain(
        r#"
        namespace Test {}
        ↘
        "#,
        &[("namespace", CompletionItemKind::Keyword)],
    );
}
