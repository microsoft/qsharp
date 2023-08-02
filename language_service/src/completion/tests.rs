// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};

use super::{get_completions, CompletionItemKind};
use crate::{
    qsc_utils::LsSpan,
    test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets},
};

fn check(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (source, cursor_offset, _) = get_source_and_marker_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset[0]);
    let checked_completions: Vec<
        Option<(
            &String,
            CompletionItemKind,
            &Option<String>,
            &Option<Vec<(LsSpan, String)>>,
        )>,
    > = completions_to_check
        .iter()
        .map(|comp| {
            actual_completions.items.iter().find_map(|item| {
                if item.label == **comp {
                    Some((
                        &item.label,
                        item.kind,
                        &item.detail,
                        &item.additional_text_edits,
                    ))
                } else {
                    None
                }
            })
        })
        .collect();

    expect.assert_debug_eq(&checked_completions);
}

#[test]
fn in_block_contains_std_functions() {
    check(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
    }"#,
        &["Fake", "FakeWithParam", "FakeCtlAdj"],
        &expect![[r#"
            [
                Some(
                    (
                        "Fake",
                        Function,
                        Some(
                            "operation Fake() : Unit",
                        ),
                        Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "This is a new Open!",
                                ),
                            ],
                        ),
                    ),
                ),
                Some(
                    (
                        "FakeWithParam",
                        Function,
                        Some(
                            "operation FakeWithParam(x: Int) : Unit",
                        ),
                        Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "This is a new Open!",
                                ),
                            ],
                        ),
                    ),
                ),
                Some(
                    (
                        "FakeCtlAdj",
                        Function,
                        Some(
                            "operation FakeCtlAdj() : Unit is Adj + Ctl",
                        ),
                        Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "This is a new Open!",
                                ),
                            ],
                        ),
                    ),
                ),
            ]
        "#]],
    );
}

#[test]
fn in_namespace_contains_open() {
    check(
        r#"
    namespace Test {
        ↘
        operation Test() : Unit {
        }
    }"#,
        &["open"],
        &expect![[r#"
            [
                Some(
                    (
                        "open",
                        Keyword,
                        None,
                    ),
                ),
            ]
        "#]],
    );
}

#[test]
fn top_level_contains_namespace() {
    check(
        r#"
        namespace Test {}
        ↘
        "#,
        &["namespace"],
        &expect![[r#"
            [
                Some(
                    (
                        "namespace",
                        Keyword,
                        None,
                    ),
                ),
            ]
        "#]],
    );
}
