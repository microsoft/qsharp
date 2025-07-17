// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::check_single_file;
use expect_test::expect;

// the `Iterable` class should not be in completions until we support it
#[test]
fn iterable_not_included_in_completions() {
    check_single_file(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &["Iterable"],
        &expect![[r#"
            not found:
              "Iterable"
        "#]],
    );
}

// the `Num` class should not be in completions since it was dropped
#[test]
fn num_not_included_in_completions() {
    check_single_file(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &["Num"],
        &expect![[r#"
            not found:
              "Num"
        "#]],
    );
}

#[test]
fn all_prim_classes_in_completions() {
    check_single_file(
        r"namespace Test {
            operation Test<'T: ↘
        }",
        &[
            "Eq", "Add", "Exp", "Integral", "Mod", "Mul", "Sub", "Div", "Signed", "Ord", "Show",
        ],
        &expect![[r#"
            found, sorted:
              "Add" (Class)
              "Div" (Class)
              "Eq" (Class)
              "Exp" (Class)
                detail: "Exp['Power]"
              "Integral" (Class)
              "Mod" (Class)
              "Mul" (Class)
              "Ord" (Class)
              "Show" (Class)
              "Signed" (Class)
              "Sub" (Class)
        "#]],
    );
}

#[test]
fn classes_appear_after_plus_too() {
    check_single_file(
        r"namespace Test {
            operation Test<'T: Add + ↘
        }",
        &[
            "Eq", "Add", "Exp", "Integral", "Mod", "Mul", "Sub", "Div", "Signed", "Ord", "Show",
        ],
        &expect![[r#"
            found, sorted:
              "Add" (Class)
              "Div" (Class)
              "Eq" (Class)
              "Exp" (Class)
                detail: "Exp['Power]"
              "Integral" (Class)
              "Mod" (Class)
              "Mul" (Class)
              "Ord" (Class)
              "Show" (Class)
              "Signed" (Class)
              "Sub" (Class)
        "#]],
    );
}
