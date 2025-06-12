// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use super::check as check_common;
use crate::{test_utils::get_sources_and_markers, Compilation};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc::{
    line_column::{Position, Range},
    location::Location,
    PackageType,
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

fn check(source_with_cursor: &str, completions_to_check: &[&str], expect: &Expect) {
    let (compilation, cursor_position, _) = compile_with_markers(source_with_cursor);

    check_common(
        &compilation,
        "<source>",
        cursor_position,
        completions_to_check,
        expect,
    );
}

#[test]
fn in_empty_file_contains_openqasm() {
    check(
        indoc! {r#"
        ↘
    }"#},
        &["OPENQASM"],
        &expect![[r#"
            in list (sorted):
              OPENQASM (Keyword)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn in_file_after_openqasm_contains_keywords_containing_i() {
    check(
        indoc! {r#"
        OPENQASM 3.0;
        i↘
    }"#},
        &["if", "include", "input", "inv"],
        &expect![[r#"
            in list (sorted):
              if (Keyword)
                detail: None
                additional_text_edits: None
              include (Keyword)
                detail: None
                additional_text_edits: None
              input (Keyword)
                detail: None
                additional_text_edits: None
              inv (Keyword)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn in_file_after_openqasm_contains_annotations_containing_i() {
    check(
        indoc! {r#"
        OPENQASM 3.0;
        i↘
    }"#},
        &["SimulatableIntrinsic"],
        &expect![[r#"
            in list (sorted):
              SimulatableIntrinsic (Interface)
                detail: None
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn local_vars() {
    check(
        indoc! {r#"
        OPENQASM 3.0;
        input int num_samples;
        output float angle_value;
        ↘
    }"#},
        &["num_samples", "angle_value"],
        &expect![[r#"
            in list (sorted):
              angle_value (Variable)
                detail: Some("angle_value : Double")
                additional_text_edits: None
              num_samples (Variable)
                detail: Some("num_samples : Int")
                additional_text_edits: None
        "#]],
    );
}

#[test]
fn local_vars_doesnt_pick_up_variables_declared_after_cursor() {
    check(
        indoc! {r#"
        OPENQASM 3.0;
        input int num_samples;
        ↘
        output float angle_value;
    }"#},
        &["num_samples", "angle_value"],
        &expect![[r#"
            not in list:
              angle_value
            in list (sorted):
              num_samples (Variable)
                detail: Some("num_samples : Int")
                additional_text_edits: None
        "#]],
    );
}
