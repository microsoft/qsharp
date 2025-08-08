// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use qsc::line_column::{Encoding, Position, Range};
use qsc::location::Location;
use qsc::qasm::semantic::passes::ReferenceFinder;

pub fn get_references(
    sources: &[(Arc<str>, Arc<str>)],
    source_name: &str,
    position: Position,
    position_encoding: Encoding,
) -> Vec<Location> {
    let (res, id) =
        super::find_symbol_in_sources(sources, source_name, position, position_encoding);

    let Some(id) = id else {
        return vec![];
    };

    let reference_spans = ReferenceFinder::get_references(&res.program, id, &res.symbols);
    reference_spans
        .into_iter()
        .map(|span| {
            let source = res
                .source_map
                .find_by_offset(span.lo)
                .expect("source should exist for offset");
            Location {
                source: source.name.clone(),
                range: Range::from_span(
                    position_encoding,
                    &source.contents,
                    &(span - source.offset),
                ),
            }
        })
        .collect::<Vec<_>>()
}
