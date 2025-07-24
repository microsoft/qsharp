// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use crate::{Compilation, completion::tests::check, test_utils::get_sources_and_markers};
use expect_test::{Expect, expect};
use indoc::indoc;
use qsc::{
    PackageType,
    line_column::{Position, Range},
    location::Location,
};

fn compile_project_with_markers_cursor_optional(
    sources_with_markers: &[(&str, &str)],
) -> (Compilation, Option<(String, Position)>, Vec<Location>) {
    let (sources, cursor_location, target_spans) = get_sources_and_markers(sources_with_markers);

    (
        Compilation::new_qasm(
            PackageType::Lib,
            qsc::target::Profile::Unrestricted,
            sources,
            vec![],
            &Arc::from("test project"),
        ),
        cursor_location,
        target_spans,
    )
}

pub(crate) fn compile_project_with_markers(
    sources_with_markers: &[(&str, &str)],
) -> (Compilation, String, Position, Vec<Location>) {
    let (compilation, cursor_location, target_spans) =
        compile_project_with_markers_cursor_optional(sources_with_markers);

    let (cursor_uri, cursor_offset) =
        cursor_location.expect("input string should have a cursor marker");

    (compilation, cursor_uri, cursor_offset, target_spans)
}

pub fn compile_with_markers(source_with_markers: &str) -> (Compilation, Position, Vec<Range>) {
    let (compilation, _, cursor_offset, target_spans) =
        compile_project_with_markers(&[("<source>", source_with_markers)]);
    (
        compilation,
        cursor_offset,
        target_spans.iter().map(|l| l.range).collect(),
    )
}

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
