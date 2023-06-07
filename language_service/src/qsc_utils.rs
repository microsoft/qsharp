// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::hir::PackageId;
use qsc::{
    compile::{self, Error},
    PackageStore, SourceMap,
};
use qsc_data_structures::span::Span;
use qsc_frontend::compile::CompileUnit;

/// Represents an immutable compilation state that can be used
/// to implement language service features.
pub(crate) struct Compilation {
    pub package_store: PackageStore,
    pub std_package_id: PackageId,
    pub compile_unit: CompileUnit,
    pub errors: Vec<Error>,
}

pub(crate) fn compile_document(source_name: &str, source_contents: &str) -> Compilation {
    let mut package_store = PackageStore::new(compile::core());
    let std_package_id = package_store.insert(compile::std(&package_store));

    // Source map only contains the current document.
    let source_map = SourceMap::new([(source_name.into(), source_contents.into())], None);
    let (compile_unit, errors) = compile::compile(&package_store, &[std_package_id], source_map);
    Compilation {
        package_store,
        std_package_id,
        compile_unit,
        errors,
    }
}

pub(crate) fn span_contains(span: Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}
