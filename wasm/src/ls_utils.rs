// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(crate) fn span_contains(span: qsc_data_structures::span::Span, offset: u32) -> bool {
    offset >= span.lo && offset < span.hi
}
