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

#[test]
fn all_prim_classes_in_completions() {
    check(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &["Eq", "Add", "Exp", "Integral", "Num", "Show"],
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
                        label: "Num",
                        kind: Class,
                        sort_text: Some(
                            "0100Num",
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
        &["Eq", "Add", "Exp", "Integral", "Num", "Show"],
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
                        label: "Num",
                        kind: Class,
                        sort_text: Some(
                            "0100Num",
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
