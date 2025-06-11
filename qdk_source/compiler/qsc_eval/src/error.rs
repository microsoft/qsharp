// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::SourceSpan;
use qsc_data_structures::span::Span;
use qsc_hir::hir::PackageId;
use std::fmt::Debug;

#[derive(Clone, Copy, Debug)]
pub struct PackageSpan {
    pub package: PackageId,
    pub span: Span,
}

impl From<PackageSpan> for SourceSpan {
    fn from(value: PackageSpan) -> Self {
        Self::from((value.span.lo as usize)..(value.span.hi as usize))
    }
}
