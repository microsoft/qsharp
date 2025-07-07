// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compilation::Compilation;
use qsc::line_column::{Encoding, Range};
use qsc::location::Location;
use qsc::{SourceMap, Span, hir::PackageId};

pub(crate) fn into_range(encoding: Encoding, span: Span, source_map: &SourceMap) -> Range {
    let lo_source = source_map
        .find_by_offset(span.lo)
        .expect("source should exist for offset");

    let hi_source = source_map
        .find_by_offset(span.hi)
        .expect("source should exist for offset");

    // Note that lo and hi offsets must always come from the same source.
    assert!(
        lo_source.offset == hi_source.offset,
        "span start and end must come from the same source"
    );

    Range::from_span(encoding, &lo_source.contents, &(span - lo_source.offset))
}

pub(crate) fn into_location(
    position_encoding: Encoding,
    compilation: &Compilation,
    span: Span,
    package_id: PackageId,
) -> Location {
    Location::from(
        span,
        package_id,
        &compilation.package_store,
        position_encoding,
    )
}
