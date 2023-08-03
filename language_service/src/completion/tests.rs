// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};

use super::{get_completions, CompletionItem};
use crate::test_utils::{compile_with_fake_stdlib, get_source_and_marker_offsets};

fn check(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (source, cursor_offset, _) = get_source_and_marker_offsets(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset[0]);
    let checked_completions: Vec<Option<&CompletionItem>> = completions_to_check
        .iter()
        .map(|comp| {
            actual_completions
                .items
                .iter()
                .find(|item| item.label == **comp)
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
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeWithParam",
                        kind: Function,
                        sort_text: Some(
                            "0600FakeWithParam",
                        ),
                        detail: Some(
                            "operation FakeWithParam(x: Int) : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
                Some(
                    CompletionItem {
                        label: "FakeCtlAdj",
                        kind: Function,
                        sort_text: Some(
                            "0600FakeCtlAdj",
                        ),
                        detail: Some(
                            "operation FakeCtlAdj() : Unit is Adj + Ctl",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open FakeStdLib;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_no_auto_open() {
    check(
        r#"
    namespace Test {
        open FakeStdLib;
        operation Test() : Unit {
            ↘
        }
    }"#,
        &["Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_with_alias() {
    check(
        r#"
    namespace Test {
        open FakeStdLib as Alias;
        operation Test() : Unit {
            ↘
        }
    }"#,
        &["Alias.Fake"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Alias.Fake",
                        kind: Function,
                        sort_text: Some(
                            "0600Alias.Fake",
                        ),
                        detail: Some(
                            "operation Fake() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_from_other_namespace() {
    check(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
    }
    namespace Other {
        operation Foo() : Unit {}
    }"#,
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0500Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: Some(
                            [
                                (
                                    LsSpan {
                                        start: 30,
                                        end: 30,
                                    },
                                    "open Other;\n    ",
                                ),
                            ],
                        ),
                    },
                ),
            ]
        "#]],
    );
}

#[ignore = "nested callables are not currently supported for completions"]
#[test]
fn in_block_nested_op() {
    check(
        r#"
    namespace Test {
        operation Test() : Unit {
            operation Foo() : Unit {}
            ↘
        }
    }"#,
        &["Foo"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Foo",
                        kind: Function,
                        sort_text: Some(
                            "0500Foo",
                        ),
                        detail: Some(
                            "operation Foo() : Unit",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn in_block_hidden_nested_op() {
    check(
        r#"
    namespace Test {
        operation Test() : Unit {
            ↘
        }
        operation Foo() : Unit {
            operation Bar() : Unit {}
        }
    }"#,
        &["Bar"],
        &expect![[r#"
            [
                None,
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
                    CompletionItem {
                        label: "open",
                        kind: Keyword,
                        sort_text: Some(
                            "0102open",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
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
                    CompletionItem {
                        label: "namespace",
                        kind: Keyword,
                        sort_text: Some(
                            "0101namespace",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}
