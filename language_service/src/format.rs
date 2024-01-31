// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compilation::Compilation,
    protocol::{Span, TextEdit},
};

use qsc::formatter::Formatter;
use qsc::RawToken;
use qsc::RawTokenKind;
use regex_lite::Regex;

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

    let formatter = Formatter::new(&contents);

    edits.extend(RemoveTrailingWhitespace(&formatter.tokens, &contents));

    // This is a dummy format rule
    // if !contents.starts_with("42") {
    //     edits.push(TextEdit {
    //         contents: "42\n".to_string(),
    //         span: Span { start: 0, end: 0 },
    //     });
    // }

    edits
}

fn RemoveTrailingWhitespace(tokens: &[RawToken], contents: &str) -> Vec<TextEdit> {
    let mut edits = vec![];

    let trailing_spaces = Regex::new(r"(?<spaces>[ \t]+)(?<newline>\n|\r\n)").unwrap();

    for i in 0..tokens.len() {
        let curr = &tokens[i];
        if matches!(curr.kind, RawTokenKind::Whitespace) {
            let lo: usize = curr.offset.try_into().unwrap();
            let hi: usize = if i + 1 < tokens.len() {
                let next = &tokens[i + 1];
                next.offset.try_into().unwrap()
            } else {
                contents.len()
            };
            let text = contents.get(lo..hi).unwrap();
            for capture in trailing_spaces.captures_iter(text) {
                let range = capture.name("spaces").unwrap().range();
                let length = range.len();
                let start = curr.offset + TryInto::<u32>::try_into(range.start).unwrap();
                let end = curr.offset + TryInto::<u32>::try_into(range.end).unwrap();
                edits.push(TextEdit {
                    contents: String::new(),
                    span: Span { start, end },
                });
            }
        }
    }

    edits
}
