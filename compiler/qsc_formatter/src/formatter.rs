// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_frontend::lex::{
    concrete::{self, ConcreteToken, ConcreteTokenKind},
    cooked::{self, TokenKind},
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
        (Syntax(cooked_left), Syntax(cooked_right)) => match (cooked_left, cooked_right) {
            (Semi, _) => {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
            (_, Semi) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Open(Delim::Brace), Close(Delim::Brace)) => {
                // close empty brace blocks, i.e. {}
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (At, Ident) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Keyword(Keyword::Internal), _) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (Open(Delim::Brace), _)
            | (_, Close(Delim::Brace))
            | (_, Keyword(Keyword::Internal))
            | (_, Keyword(Keyword::Operation))
            | (_, Keyword(Keyword::Function))
            | (_, Keyword(Keyword::Newtype))
            | (_, Keyword(Keyword::Namespace))
            | (_, At) => {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
            (Open(Delim::Bracket | Delim::Paren), _)
            | (_, Close(Delim::Bracket | Delim::Paren)) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (_, Open(Delim::Bracket | Delim::Paren)) => {
                if is_value_token_left(cooked_left) {
                    // i.e. foo() or { foo }[3]
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    // i.e. let x = (1, 2, 3);
                    effect_single_space(left, whitespace, right, &mut edits);
                }
            }
            (_, TokenKind::DotDotDot) => {
                if is_value_token_left(cooked_left) {
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    effect_single_space(left, whitespace, right, &mut edits);
                }
            }
            (_, _) if is_value_token_right(cooked_right) => {
                if is_prefix_with_space(cooked_left)
                    || is_prefix_without_space(cooked_left)
                    || matches!(cooked_left, TokenKind::DotDotDot)
                {
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    effect_single_space(left, whitespace, right, &mut edits);
                }
            }
            (_, _) if is_suffix(cooked_right) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (_, _) if is_prefix_with_space(cooked_right) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, _) if is_prefix_without_space(cooked_right) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (_, _) if is_bin_op(cooked_right) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            _ => {}
        },
        _ => {}
    }
    edits
}

fn is_bin_op(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::Bar
            | TokenKind::BinOpEq(_)
            | TokenKind::ClosedBinOp(_)
            | TokenKind::Colon
            | TokenKind::Eq
            | TokenKind::EqEq
            | TokenKind::FatArrow
            | TokenKind::Gt
            | TokenKind::Gte
            | TokenKind::LArrow
            | TokenKind::Lt
            | TokenKind::Lte
            | TokenKind::Ne
            | TokenKind::Question
            | TokenKind::RArrow
            | TokenKind::WSlash
            | TokenKind::WSlashEq
    )
}

fn is_prefix_with_space(cooked: &TokenKind) -> bool {
    matches!(cooked, TokenKind::AposIdent | TokenKind::TildeTildeTilde)
}

fn is_prefix_without_space(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::ColonColon | TokenKind::Dot | TokenKind::DotDot
    )
}

fn is_suffix(cooked: &TokenKind) -> bool {
    matches!(cooked, TokenKind::Bang | TokenKind::Comma)
}

fn is_value_token_left(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::BigInt(_)
            | TokenKind::Float
            | TokenKind::Ident
            | TokenKind::Int(_)
            | TokenKind::String(_)
            | TokenKind::Close(_)
    )
}

fn is_value_token_right(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::BigInt(_)
            | TokenKind::Float
            | TokenKind::Ident
            | TokenKind::Int(_)
            | TokenKind::String(_)
            | TokenKind::Open(_)
    )
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
