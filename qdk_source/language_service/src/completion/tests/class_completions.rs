// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::check;
use expect_test::expect;

// the `Iterable` class should not be in completions until we support it
#[test]
fn iterable_not_included_in_completions() {
    check(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &["Iterable"],
        &expect![[r#"
            [
                None,
            ]
        "#]],
    );
}

// the `Num` class should not be in completions since it was dropped
#[test]
fn num_not_included_in_completions() {
    check(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &["Num"],
        &expect![[r#"
            [
                None,
            ]
        "#]],
    );
}

#[test]
fn all_prim_classes_in_completions() {
    check(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &[
            "Eq", "Add", "Exp", "Integral", "Mod", "Mul", "Sub", "Div", "Signed", "Ord", "Show",
        ],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Eq",
                        kind: Class,
                        sort_text: Some(
                            "0100Eq",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Add",
                        kind: Class,
                        sort_text: Some(
                            "0100Add",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Exp",
                        kind: Class,
                        sort_text: Some(
                            "0100Exp",
                        ),
                        detail: Some(
                            "Exp['Power]",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Integral",
                        kind: Class,
                        sort_text: Some(
                            "0100Integral",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Mod",
                        kind: Class,
                        sort_text: Some(
                            "0100Mod",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Mul",
                        kind: Class,
                        sort_text: Some(
                            "0100Mul",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Sub",
                        kind: Class,
                        sort_text: Some(
                            "0100Sub",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Div",
                        kind: Class,
                        sort_text: Some(
                            "0100Div",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Signed",
                        kind: Class,
                        sort_text: Some(
                            "0100Signed",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Ord",
                        kind: Class,
                        sort_text: Some(
                            "0100Ord",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Show",
                        kind: Class,
                        sort_text: Some(
                            "0100Show",
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
fn classes_appear_after_plus_too() {
    check(
        r"namespace Test {
            operation Test<'T: Add + ↘
        }",
        &[
            "Eq", "Add", "Exp", "Integral", "Mod", "Mul", "Sub", "Div", "Signed", "Ord", "Show",
        ],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "Eq",
                        kind: Class,
                        sort_text: Some(
                            "0100Eq",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Add",
                        kind: Class,
                        sort_text: Some(
                            "0100Add",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Exp",
                        kind: Class,
                        sort_text: Some(
                            "0100Exp",
                        ),
                        detail: Some(
                            "Exp['Power]",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Integral",
                        kind: Class,
                        sort_text: Some(
                            "0100Integral",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Mod",
                        kind: Class,
                        sort_text: Some(
                            "0100Mod",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Mul",
                        kind: Class,
                        sort_text: Some(
                            "0100Mul",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Sub",
                        kind: Class,
                        sort_text: Some(
                            "0100Sub",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Div",
                        kind: Class,
                        sort_text: Some(
                            "0100Div",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Signed",
                        kind: Class,
                        sort_text: Some(
                            "0100Signed",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Ord",
                        kind: Class,
                        sort_text: Some(
                            "0100Ord",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "Show",
                        kind: Class,
                        sort_text: Some(
                            "0100Show",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
            ]
        "#]],
    );
}
