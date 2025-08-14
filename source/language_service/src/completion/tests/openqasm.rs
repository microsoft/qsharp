// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{completion::tests::check, test_utils::openqasm::compile_with_markers};
use expect_test::{Expect, expect};
use indoc::indoc;

fn check_single_file(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor);

    check(
        &compilation,
        "<source>",
        cursor_position,
        completions_to_check,
        expect,
    );
}

#[test]
fn in_empty_file_contains_openqasm() {
    check_single_file(
        indoc! {r#"
        ↘
    }"#},
        &["OPENQASM"],
        &expect![[r#"
            found, sorted:
              "OPENQASM" (Keyword)
        "#]],
    );
}

#[test]
fn in_file_after_openqasm_contains_keywords_containing_i() {
    check_single_file(
        indoc! {r#"
        OPENQASM 3.0;
        i↘
    }"#},
        &["if", "include", "input", "inv"],
        &expect![[r#"
            found, sorted:
              "if" (Keyword)
              "include" (Keyword)
              "input" (Keyword)
              "inv" (Keyword)
        "#]],
    );
}

#[test]
fn in_file_after_openqasm_contains_annotations_containing_i() {
    check_single_file(
        indoc! {r#"
        OPENQASM 3.0;
        i↘
    }"#},
        &["SimulatableIntrinsic"],
        &expect![[r#"
            found, sorted:
              "SimulatableIntrinsic" (Interface)
        "#]],
    );
}

#[test]
fn local_vars() {
    check_single_file(
        indoc! {r#"
        OPENQASM 3.0;
        input int num_samples;
        output float angle_value;
        ↘
    }"#},
        &["num_samples", "angle_value"],
        &expect![[r#"
            found, sorted:
              "angle_value" (Variable)
                detail: "angle_value : Double"
              "num_samples" (Variable)
                detail: "num_samples : Int"
        "#]],
    );
}

#[test]
fn local_vars_doesnt_pick_up_variables_declared_after_cursor() {
    check_single_file(
        indoc! {r#"
        OPENQASM 3.0;
        input int num_samples;
        ↘
        output float angle_value;
    }"#},
        &["num_samples", "angle_value"],
        &expect![[r#"
            found, sorted:
              "num_samples" (Variable)
                detail: "num_samples : Int"

            not found:
              "angle_value"
        "#]],
    );
}
