// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_frontend::lex::{
    concrete::{self, ConcreteToken, ConcreteTokenKind},
    cooked::TokenKind,
    Delim,
};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct TextEdit {
    pub new_text: String,
    pub span: Span,
}

impl TextEdit {
    fn new(new_text: &str, lo: u32, hi: u32) -> Self {
        Self {
            new_text: new_text.to_string(),
            span: Span { lo, hi },
        }
    }
}

fn make_indent_string(level: usize) -> String {
    "    ".repeat(level)
}

/// Applies formatting rules to the give code str and returns
/// the formatted string.
pub fn format_str(code: &str) -> String {
    let mut edits = calculate_format_edits(code);
    edits.sort_by_key(|edit| edit.span.hi); // sort edits by their span's hi value from lowest to highest
    edits.reverse(); // sort from highest to lowest so that that as edits are applied they don't invalidate later applications of edits
    let mut new_code = String::from(code);

    for edit in edits {
        let range = (edit.span.lo as usize)..(edit.span.hi as usize);
        new_code.replace_range(range, &edit.new_text);
    }

    new_code
}

/// Applies formatting rules to the given code str, generating edits where
/// the source code needs to be changed to comply with the format rules.
pub fn calculate_format_edits(code: &str) -> Vec<TextEdit> {
    let tokens = concrete::ConcreteTokenIterator::new(code);
    let mut edits = vec![];

    let mut indent_level: usize = 0;

    // The sliding window used is over three adjacent tokens
    #[allow(unused_assignments)]
    let mut one = None;
    let mut two = None;
    let mut three = None;

    for token in tokens {
        // Advance the token window
        one = two;
        two = three;
        three = Some(token);

        let mut edits_for_triple = match (&one, &two, &three) {
            (Some(one), Some(two), Some(three)) => {
                match one.kind {
                    ConcreteTokenKind::Syntax(TokenKind::Open(Delim::Brace)) => indent_level += 1,
                    ConcreteTokenKind::Syntax(TokenKind::Close(Delim::Brace)) => {
                        indent_level = indent_level.saturating_sub(1)
                    }
                    ConcreteTokenKind::WhiteSpace => continue,
                    _ => {}
                }

                if matches!(one.kind, ConcreteTokenKind::WhiteSpace) {
                    // first token is whitespace, continue scanning
                    continue;
                } else if matches!(two.kind, ConcreteTokenKind::WhiteSpace) {
                    // whitespace in the middle
                    apply_rules(
                        one,
                        get_token_contents(code, two),
                        three,
                        code,
                        indent_level,
                    )
                } else {
                    // one, two are adjacent tokens with no whitespace in the middle
                    apply_rules(one, "", two, code, indent_level)
                }
            }
            (None, None, Some(three)) => {
                // Remove any whitespace at the start of a file
                if three.span.lo != 0 {
                    vec![TextEdit::new("", 0, three.span.lo)]
                } else {
                    vec![]
                }
            }
            _ => {
                // not enough tokens to apply a rule
                continue;
            }
        };

        edits.append(&mut edits_for_triple);
    }

    edits
}

fn apply_rules(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    code: &str,
    indent_level: usize,
) -> Vec<TextEdit> {
    let mut edits = vec![];
    // when we get here, neither left nor right should be whitespace

    // if the right is a close brace, the indent level should be one less
    let indent_level = if let ConcreteTokenKind::Syntax(TokenKind::Close(Delim::Brace)) = right.kind
    {
        indent_level.saturating_sub(1)
    } else {
        indent_level
    };

    use qsc_frontend::keyword::Keyword;
    use ConcreteTokenKind::*;
    use TokenKind::*;
    match (&left.kind, &right.kind) {
        (Syntax(Open(l)), Syntax(Close(r))) if l == r => {
            // close empty delimiter blocks, i.e. (), [], {}
            effect_no_space(left, whitespace, right, &mut edits);
        }
        (Comment | Syntax(DocComment), _) => {
            // remove whitespace at the ends of comments
            effect_trim_comment(left, &mut edits, code);
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (Syntax(Semi), Comment) => {
            if whitespace.contains('\n') {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
        }
        (Syntax(Semi), _) => {
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (_, Syntax(Close(Delim::Brace)))
        | (_, Syntax(Keyword(Keyword::Operation)))
        | (_, Syntax(Keyword(Keyword::Function)))
        | (_, Syntax(Keyword(Keyword::Newtype)))
        | (_, Syntax(Keyword(Keyword::Namespace)))
        | (Syntax(Open(Delim::Brace)), _) => {
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (Syntax(Close(Delim::Brace)), Syntax(Semi))
        | (Syntax(Close(Delim::Brace)), Syntax(Comma)) => {
            // remove any space between a close brace and a semicolon or comma
            effect_no_space(left, whitespace, right, &mut edits);
        }
        (Syntax(Close(Delim::Brace)), _) => {
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (Syntax(cooked_left), Syntax(cooked_right)) => match (cooked_left, cooked_right) {
            (Ident, Ident) | (Keyword(_), Ident) // single spaces around identifiers
            | (_, Colon) | (Colon, _) // single spaces around type-annotating colons
            | (Comma, _) // single space after a comma
            => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (Ident, Open(Delim::Paren)) // no space between callable name and argument tuple, i.e. foo()
            | (Ident, Open(Delim::Bracket)) // no space between array name and brackets, i.e. foo[2]
            | (Ident, Comma) // no space between identifier and following comma in a sequence
            | (Open(_), _) // no space after an open delimiter
            | (_, Close(_)) => { // no space before a close delimiter
                effect_no_space(left, whitespace, right, &mut edits);
            }
            _ => {}
        },
        _ => {}
    }
    edits
}

fn effect_no_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<TextEdit>,
) {
    if !whitespace.is_empty() {
        edits.push(TextEdit::new("", left.span.hi, right.span.lo));
    }
}

fn effect_single_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<TextEdit>,
) {
    if whitespace != " " {
        edits.push(TextEdit::new(" ", left.span.hi, right.span.lo));
    }
}

fn effect_trim_comment(left: &ConcreteToken, edits: &mut Vec<TextEdit>, code: &str) {
    let comment_contents = get_token_contents(code, left);
    let new_comment_contents = comment_contents.trim_end();
    if comment_contents != new_comment_contents {
        edits.push(TextEdit::new(
            new_comment_contents,
            left.span.lo,
            left.span.hi,
        ));
    }
}

fn effect_correct_indentation(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<TextEdit>,
    indent_level: usize,
) {
    let mut count_newlines = whitespace.chars().filter(|c| *c == '\n').count();

    // There should always be at least one newline
    if count_newlines < 1 {
        count_newlines = 1;
    }

    let mut new_whitespace = "\n".repeat(count_newlines);
    new_whitespace.push_str(&make_indent_string(indent_level));
    if whitespace != new_whitespace {
        edits.push(TextEdit::new(
            new_whitespace.as_str(),
            left.span.hi,
            right.span.lo,
        ));
    }
}

fn get_token_contents<'a>(code: &'a str, token: &ConcreteToken) -> &'a str {
    &code[token.span.lo as usize..token.span.hi as usize]
}
