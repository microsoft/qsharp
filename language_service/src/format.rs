// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compilation::Compilation,
    protocol::{Span, TextEdit},
};

use qsc::Formatter;
use qsc::RawTokenKind;

pub(crate) fn get_format_changes(
    compilation: &Compilation,
    source_name: &str,
    offset: u32,
) -> Vec<TextEdit> {
    let contents = compilation
        .user_unit()
        .sources
        .find_by_name(source_name)
        .expect("can't find source by name")
        .contents
        .clone();

    let mut edits = vec![];

    let mut formatter = Formatter::new(&contents);

    for token in formatter.tokens.iter() {}

    // This is a dummy format rule
    if !contents.starts_with("42") {
        edits.push(TextEdit {
            contents: "42\n".to_string(),
            span: Span { start: 0, end: 0 },
        });
    }

    edits
}
