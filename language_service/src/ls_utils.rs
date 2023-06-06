// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc::PackageStore;
use qsc_frontend::compile::CompileUnit;
use qsc_hir::hir::PackageId;

pub(crate) fn span_contains(span: qsc_data_structures::span::Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}
pub(crate) struct CompilationState {
    pub store: PackageStore,
    pub std: PackageId,
    pub compile_unit: Option<CompileUnit>,
}
