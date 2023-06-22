// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::hir::PackageId;
use qsc::{
    compile::{self},
    PackageStore, SourceMap,
};
use qsc::{AstUnit, Span};

/// Represents an immutable compilation state that can be used
/// to implement language service features.
pub(crate) struct Compilation {
    pub package_store: PackageStore,
    pub std_package_id: PackageId,
    pub ast_unit: AstUnit,
}

pub(crate) fn compile_document(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_package_id = package_store.insert(compile::std(&package_store));

    // Source map only contains the current document.
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (ast_unit, errors) = compile::ast(&package_store, &[std_package_id], source_map);
    Compilation {
        package_store,
        std_package_id,
        ast_unit,
    }
}

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}

pub(crate) fn map_offset(source_map: &SourceMap, source_name: &str, source_offset: u32) -> u32 {
    source_map
        .find_by_name(source_name)
        .expect("source should exist in the source map")
        .offset
        + source_offset
}
