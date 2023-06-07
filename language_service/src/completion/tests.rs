// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    completion::{CompletionItem, CompletionItemKind},
    qsc_utils::Compilation,
};
use qsc::{compile, PackageStore, SourceMap};
use qsc_hir::hir::PackageId;

use super::get_completions;

#[test]
fn in_block_contains_std_functions() {
    assert_completions_contain(
        r#"
    namespace Test {
        operation Test() : Unit {
            |>
        }
    }"#,
        &[
            CompletionItem {
                label: "Fake".to_string(),
                kind: CompletionItemKind::Function,
            },
            CompletionItem {
                label: "FakeStdLib".to_string(),
                kind: CompletionItemKind::Module,
            },
        ],
    );
}

#[test]
fn in_namespace_contains_open() {
    assert_completions_contain(
        r#"
    namespace Test {
        |>
        operation Test() : Unit {
        }
    }"#,
        &[CompletionItem {
            label: "open".to_string(),
            kind: CompletionItemKind::Keyword,
        }],
    );
}

#[test]
fn top_level_contains_namespace() {
    assert_completions_contain(
        r#"
        |>
        "#,
        &[CompletionItem {
            label: "namespace".to_string(),
            kind: CompletionItemKind::Keyword,
        }],
    );
}

fn assert_completions_contain(source_with_cursor: &str, completions: &[CompletionItem]) {
    let (source, cursor_offset) = get_source_and_cursor_offset(source_with_cursor);
    let compilation = compile_with_fake_stdlib("<source>", &source);
    let actual_completions = get_completions(&compilation, "<source>", cursor_offset);
    for expected_completion in completions.iter() {
        assert!(
            actual_completions.items.contains(expected_completion),
            "expected to find\n{expected_completion:?}\nin:\n{actual_completions:?}"
        );
    }
}

fn get_source_and_cursor_offset(source_with_cursor: &str) -> (String, u32) {
    #[allow(clippy::cast_possible_truncation)]
    let cursor_offset = source_with_cursor
        .find("|>")
        .expect("string should contain cursor") as u32;
    let source = source_with_cursor.replace("|>", "");
    (source, cursor_offset)
}

fn compile_with_fake_stdlib(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_source_map = SourceMap::new(
        [(
            "<std>".into(),
            "namespace FakeStdLib { operation Fake() : Unit {} }".into(),
        )],
        None,
    );
    let (std_compile_unit, std_errors) =
        compile::compile(&package_store, &[PackageId::CORE], std_source_map);
    assert!(std_errors.is_empty());
    let std_package_id = package_store.insert(std_compile_unit);
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (compile_unit, errors) = compile::compile(&package_store, &[std_package_id], source_map);
    Compilation {
        package_store,
        std_package_id,
        compile_unit,
        errors,
    }
}
