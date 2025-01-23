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
            not in list: 
              Iterable
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
            not in list: 
              Num
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
            in list (sorted):
              Add (Class)
                detail: None
                additional_text_edits: None
              Div (Class)
                detail: None
                additional_text_edits: None
              Eq (Class)
                detail: None
                additional_text_edits: None
              Exp (Class)
                detail: Some("Exp['Power]")
                additional_text_edits: None
              Integral (Class)
                detail: None
                additional_text_edits: None
              Mod (Class)
                detail: None
                additional_text_edits: None
              Mul (Class)
                detail: None
                additional_text_edits: None
              Ord (Class)
                detail: None
                additional_text_edits: None
              Show (Class)
                detail: None
                additional_text_edits: None
              Signed (Class)
                detail: None
                additional_text_edits: None
              Sub (Class)
                detail: None
                additional_text_edits: None
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
            in list (sorted):
              Add (Class)
                detail: None
                additional_text_edits: None
              Div (Class)
                detail: None
                additional_text_edits: None
              Eq (Class)
                detail: None
                additional_text_edits: None
              Exp (Class)
                detail: Some("Exp['Power]")
                additional_text_edits: None
              Integral (Class)
                detail: None
                additional_text_edits: None
              Mod (Class)
                detail: None
                additional_text_edits: None
              Mul (Class)
                detail: None
                additional_text_edits: None
              Ord (Class)
                detail: None
                additional_text_edits: None
              Show (Class)
                detail: None
                additional_text_edits: None
              Signed (Class)
                detail: None
                additional_text_edits: None
              Sub (Class)
                detail: None
                additional_text_edits: None
        "#]],
    );
}
