// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::Arc;

use qsc::line_column::{Encoding, Position};
use qsc::location::Location;

use crate::openqasm::get_reference_locations;

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

    get_reference_locations(
        position_encoding,
        &res.program,
        &res.source_map,
        &res.symbols,
        id,
    )
}
