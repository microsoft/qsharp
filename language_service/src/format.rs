// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{compilation::Compilation, protocol::TextEdit};

use qsc::formatter::calculate_format_edits;
use qsc::line_column::{Encoding, Position, Range};

pub(crate) fn get_format_changes(
    compilation: &Compilation,
    source_name: &str,
    _position: Position,
    encoding: Encoding,
) -> Vec<TextEdit> {
    let contents = compilation
        .user_unit()
        .sources
        .find_by_name(source_name)
        .expect("can't find source by name")
        .contents
        .clone();

    calculate_format_edits(&contents)
        .iter()
        .map(|edit| TextEdit {
            new_text: edit.new_text.clone(),
            range: Range::from_span(encoding, &contents, &edit.span),
        })
        .collect()
}
