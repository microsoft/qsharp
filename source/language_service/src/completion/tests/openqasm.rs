// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use super::{CompletionItem, get_completions};
use crate::{Compilation, Encoding, test_utils::get_sources_and_markers};
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
            false,
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
    let actual_completions =
        get_completions(&compilation, "<source>", cursor_position, Encoding::Utf8);
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
fn in_empty_file_contains_openqasm() {
    check(
        indoc! {r#"
        ↘
    }"#},
        &["OPENQASM"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "OPENQASM",
                        kind: Keyword,
                        sort_text: Some(
                            "0000OPENQASM",
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
fn in_file_after_openqasm_contains_keywords_containing_i() {
    check(
        indoc! {r#"
        OPENQASM 3.0;
        i↘
    }"#},
        &["if", "include", "input", "inv"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "if",
                        kind: Keyword,
                        sort_text: Some(
                            "0000if",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "include",
                        kind: Keyword,
                        sort_text: Some(
                            "0000include",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "input",
                        kind: Keyword,
                        sort_text: Some(
                            "0000input",
                        ),
                        detail: None,
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "inv",
                        kind: Keyword,
                        sort_text: Some(
                            "0000inv",
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
fn in_file_after_openqasm_contains_annotations_containing_i() {
    check(
        indoc! {r#"
        OPENQASM 3.0;
        i↘
    }"#},
        &["SimulatableIntrinsic"],
        &expect![[r#"
            [
                Some(
                    CompletionItem {
                        label: "SimulatableIntrinsic",
                        kind: Interface,
                        sort_text: Some(
                            "0000SimulatableIntrinsic",
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
            [
                Some(
                    CompletionItem {
                        label: "num_samples",
                        kind: Variable,
                        sort_text: Some(
                            "0100num_samples",
                        ),
                        detail: Some(
                            "num_samples : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
                Some(
                    CompletionItem {
                        label: "angle_value",
                        kind: Variable,
                        sort_text: Some(
                            "0100angle_value",
                        ),
                        detail: Some(
                            "angle_value : Double",
                        ),
                        additional_text_edits: None,
                    },
                ),
            ]
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
            [
                Some(
                    CompletionItem {
                        label: "num_samples",
                        kind: Variable,
                        sort_text: Some(
                            "0100num_samples",
                        ),
                        detail: Some(
                            "num_samples : Int",
                        ),
                        additional_text_edits: None,
                    },
                ),
                None,
            ]
        "#]],
    );
}
