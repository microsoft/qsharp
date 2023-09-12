// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::SourceSpan;
use qsc_data_structures::span::Span;
use qsc_hir::hir::PackageId;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub struct GlobalSpan {
    pub package: PackageId,
    pub span: Span,
}

impl From<GlobalSpan> for SourceSpan {
    fn from(value: GlobalSpan) -> Self {
        Self::from((value.span.lo as usize)..(value.span.hi as usize))
    }
}
