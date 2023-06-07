// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::qsc_utils::Compilation;
use qsc::{compile, PackageStore, SourceMap};
use qsc_hir::hir::PackageId;

pub(crate) fn get_source_and_cursor_offsets(source_with_cursor: &str) -> (String, Vec<u32>) {
    let mut offsets: Vec<u32> = Vec::new();
    let mut source = source_with_cursor.to_string();
    loop {
        let cursor_offset = source.find("↘");
        match cursor_offset {
            #[allow(clippy::cast_possible_truncation)]
            Some(offset) => offsets.push(offset as u32),
            None => break,
        };
        source = source.replacen("↘", "", 1);
    }
    (source, offsets)
}

pub(crate) fn compile_with_fake_stdlib(source_name: &str, source_contents: &str) -> Compilation {
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
