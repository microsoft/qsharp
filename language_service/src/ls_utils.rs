// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::PackageStore;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::hir::PackageId;

pub(crate) struct CompilationState {
    pub version: u32,
    pub package_store: PackageStore,
    pub std_package_id: PackageId,
    pub compile_unit: CompileUnit,
}

pub(crate) fn span_contains(span: qsc_data_structures::span::Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}
