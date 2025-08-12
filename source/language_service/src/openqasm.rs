// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod definition;
mod references;
mod rename;
use std::sync::Arc;

pub use definition::get_definition;
use log::trace;
use qsc::SourceMap;
use qsc::line_column::Encoding;
use qsc::line_column::Position;
use qsc::line_column::Range;
use qsc::location::Location;
use qsc::qasm::semantic::ast::Program;
use qsc::qasm::semantic::passes::ReferenceFinder;
use qsc::qasm::semantic::passes::SymbolFinder;
use qsc::qasm::semantic::symbols::SymbolId;
use qsc::qasm::semantic::symbols::SymbolTable;
pub use references::get_references;
pub use rename::get_rename;
pub use rename::prepare_rename;

/// Tries to find a symbol in the given source at the specified position.
/// returns the semantic parse result and the symbol ID if found.
fn find_symbol_in_sources(
    sources: &[(Arc<str>, Arc<str>)],
    source_name: &str,
    position: Position,
    position_encoding: Encoding,
) -> (
    qsc::qasm::semantic::QasmSemanticParseResult,
    Option<qsc::qasm::semantic::symbols::SymbolId>,
) {
    let res = qsc::qasm::semantic::parse_sources(sources);
    let offset = source_position_to_package_offset(
        &res.source_map,
        source_name,
        position,
        position_encoding,
    );
    let id = SymbolFinder::get_symbol_at_offset(&res.program, offset, &res.symbols);
    (res, id)
}

/// Maps a source position from the user package
/// to a package (`SourceMap`) offset.
fn source_position_to_package_offset(
    sources: &SourceMap,
    source_name: &str,
    source_position: Position,
    position_encoding: Encoding,
) -> u32 {
    let source = sources
        .find_by_name(source_name)
        .expect("source should exist in the user source map");

    let mut offset =
        source_position.to_utf8_byte_offset(position_encoding, source.contents.as_ref());

    let len = u32::try_from(source.contents.len()).expect("source length should fit into u32");
    if offset > len {
        // This can happen if the document contents are out of sync with the client's view.
        // we don't want to accidentally return an offset into the next file -
        // remap to the end of the current file.
        trace!(
            "offset {offset} out of bounds for {}, using end offset instead",
            source.name
        );
        offset = len;
    }

    source.offset + offset
}

fn map_spans_to_source_locations(
    position_encoding: Encoding,
    source_map: &SourceMap,
    spans: Vec<qsc::Span>,
) -> Vec<Location> {
    spans
        .into_iter()
        .map(|span| {
            let source = source_map
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

fn get_reference_locations(
    position_encoding: Encoding,
    program: &Program,
    source_map: &SourceMap,
    symbols: &SymbolTable,
    id: SymbolId,
) -> Vec<Location> {
    let reference_spans = ReferenceFinder::get_references(program, id, symbols);
    map_spans_to_source_locations(position_encoding, source_map, reference_spans)
}
