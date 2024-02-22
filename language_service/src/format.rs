// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{compilation::Compilation, protocol::TextEdit};

use qsc::formatter::format;
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

    format(&contents)
        .iter()
        .map(|edit| TextEdit {
            contents: edit.new_text.clone(),
            span: Range::from_span(encoding, &contents, &edit.span),
        })
        .collect()
}
