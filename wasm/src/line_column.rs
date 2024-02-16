// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::serializable_type;
use qsc::line_column;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

serializable_type! {
    Position,
    {
        pub line: u32,
        pub character: u32,
    },
    r#"export interface IPosition {
        line: number;
        character: number;
    }"#,
    IPosition
}

serializable_type! {
    Range,
    {
        pub start: Position,
        pub end: Position,
    },
    r#"export interface IRange {
        start: IPosition;
        end: IPosition;
    }"#
}

serializable_type! {
    Location,
    {
        pub source: String,
        pub span: Range,
    },
    r#"export interface ILocation {
        source: string;
        span: IRange;
    }"#,
    ILocation
}

impl From<Position> for line_column::Position {
    fn from(position: Position) -> Self {
        line_column::Position {
            line: position.line,
            column: position.character,
        }
    }
}

impl From<line_column::Position> for Position {
    fn from(position: line_column::Position) -> Self {
        Position {
            line: position.line,
            character: position.column,
        }
    }
}

impl From<qsc::line_column::Range> for Range {
    fn from(range: qsc::line_column::Range) -> Self {
        Range {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl From<qsc::location::Location> for Location {
    fn from(location: qsc::location::Location) -> Self {
        Location {
            source: location.source.to_string(),
            span: location.range.into(),
        }
    }
}
