// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::keyword::Keyword;

use super::{
    concrete::{self, ConcreteToken, ConcreteTokenKind},
    //raw::{Lexer, Single, TokenKind},
    Delim,
    TokenKind,
};
use qsc_data_structures::span::Span;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Edit {
    #[allow(dead_code)] // TODO: nobody's using this yet except for tests
    pub span: Span,
    #[allow(dead_code)] // TODO: nobody's using this yet except for tests
    pub new_text: String,
}

impl Edit {
    fn new(lo: u32, hi: u32, new_text: &str) -> Self {
        Self {
            span: Span { lo, hi },
            new_text: new_text.to_string(),
        }
    }
}

fn make_indent_string(level: usize) -> String {
    "    ".repeat(level)
}

pub fn format(code: &str) -> Vec<Edit> {
    let tokens = concrete::ConcreteTokenIterator::new(code);
    let mut edits = vec![];

    let mut indent_level = 0;

    #[allow(unused_assignments)] // there's probably a better way of doing this, but this works
    let mut one = None;
    let mut two = None;
    let mut three = None;

    for token in tokens {
        // Advance the trio of tokens
        one = two;
        two = three;
        three = Some(token);

        let mut edits_for_triple = match (&one, &two, &three) {
            (Some(one), Some(two), Some(three)) => {
                // if the token is a {, increase the indent level
                if let ConcreteTokenKind::Cooked(TokenKind::Open(Delim::Brace)) = one.kind {
                    indent_level += 1;
                }

                // if the token is a }, decrease the indent level
                if let ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)) = one.kind {
                    #[allow(clippy::implicit_saturating_sub)]
                    if indent_level > 0 {
                        indent_level -= 1;
                    }
                }

                if let ConcreteTokenKind::WhiteSpace = one.kind {
                    // first token is whitespace, continue scanning
                    continue;
                } else if let ConcreteTokenKind::WhiteSpace = two.kind {
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
                    vec![Edit::new(0, three.span.lo, "")]
                } else {
                    vec![]
                }
            }
            _ => {
                // not enough tokens to apply a rule
                // TODO: we'll probably need to handle end-of-file cases here
                continue;
            }
        };

        edits.append(&mut edits_for_triple);
    }

    edits
}

fn fix_whitespace(whitespace: &str, indent_level: usize) -> String {
    //
    // when you see newline, insert the indent string
    // and trim until the next newline or the end of the string
    //

    let mut count_newlines = whitespace.chars().filter(|c| *c == '\n').count();

    // There should always be at least one newline
    if count_newlines < 1 {
        count_newlines = 1;
    }
    let mut new = "\n".repeat(count_newlines);
    new.push_str(&make_indent_string(indent_level));
    new
}

fn apply_rules(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    code: &str,
    indent_level: usize,
) -> Vec<Edit> {
    let mut edits = vec![];
    // when we get here, neither left nor right should be whitespace

    // if the right is a close brace, the indent level should be one less
    let indent_level = if let ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)) = right.kind
    {
        if indent_level > 0 {
            indent_level - 1
        } else {
            indent_level
        }
    } else {
        indent_level
    };

    match (&left.kind, &right.kind) {
        (
            ConcreteTokenKind::Cooked(TokenKind::Open(l)),
            ConcreteTokenKind::Cooked(TokenKind::Close(r)),
        ) if l == r => {
            rule_no_space(left, whitespace, right, &mut edits);
        }
        (ConcreteTokenKind::Comment | ConcreteTokenKind::Cooked(TokenKind::DocComment), _) => {
            rule_trim_comments(left, &mut edits, code);
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (ConcreteTokenKind::Cooked(TokenKind::Semi), _) => match &right.kind {
            ConcreteTokenKind::Comment => {
                if whitespace.contains('\n') {
                    rule_indentation(left, whitespace, right, &mut edits, indent_level);
                }
            }
            _ => {
                rule_indentation(left, whitespace, right, &mut edits, indent_level);
            }
        },
        (_, ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace))) => {
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (
            _,
            ConcreteTokenKind::Cooked(TokenKind::Keyword(
                Keyword::Operation | Keyword::Function | Keyword::Newtype | Keyword::Namespace,
            )),
        ) => {
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (ConcreteTokenKind::Cooked(TokenKind::Open(Delim::Brace)), _) => {
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (
            ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)),
            ConcreteTokenKind::Cooked(TokenKind::Semi),
        ) => {
            rule_no_space(left, whitespace, right, &mut edits);
        }
        (ConcreteTokenKind::Cooked(TokenKind::Close(Delim::Brace)), _) => {
            rule_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (ConcreteTokenKind::Cooked(cooked_left), ConcreteTokenKind::Cooked(cooked_right)) => {
            match (cooked_left, cooked_right) {
                (TokenKind::Ident, TokenKind::Ident)
                | (TokenKind::Keyword(_), TokenKind::Ident)
                | (_, TokenKind::Colon)
                | (TokenKind::Colon, _)
                | (TokenKind::Comma, _) => {
                    rule_single_space(left, whitespace, right, &mut edits);
                }
                (TokenKind::Ident, TokenKind::Open(Delim::Paren))
                | (TokenKind::Ident, TokenKind::Open(Delim::Bracket))
                | (TokenKind::Ident, TokenKind::Comma)
                | (TokenKind::Open(_), _)
                | (_, TokenKind::Close(_)) => {
                    rule_no_space(left, whitespace, right, &mut edits);
                }
                _ => {}
            }
        }
        _ => {}
    }

    println!(
        "edits for `{}` : {edits:?}",
        &code[left.span.lo as usize..right.span.hi as usize]
    );
    edits
}

fn rule_no_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    if !whitespace.is_empty() {
        edits.push(Edit::new(left.span.hi, right.span.lo, ""));
    }
}

fn rule_single_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    if whitespace != " " {
        edits.push(Edit::new(left.span.hi, right.span.lo, " "));
    }
}

fn rule_trim_comments(left: &ConcreteToken, edits: &mut Vec<Edit>, code: &str) {
    // fix trailing spaces on the comment
    let comment_contents = get_token_contents(code, left);
    let new_comment_contents = comment_contents.trim_end();
    if comment_contents != new_comment_contents {
        edits.push(Edit::new(left.span.lo, left.span.hi, new_comment_contents));
    }
}

fn rule_indentation(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
    indent_level: usize,
) {
    // if the middle whitespace contains a new line, we need to
    // fix the indentation
    let new_whitespace = fix_whitespace(whitespace, indent_level);
    if whitespace != new_whitespace {
        edits.push(Edit::new(
            left.span.hi,
            right.span.lo,
            new_whitespace.as_str(),
        ));
    }
}

fn get_token_contents<'a>(code: &'a str, token: &ConcreteToken) -> &'a str {
    &code[token.span.lo as usize..token.span.hi as usize]
}
