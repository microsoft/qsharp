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
pub struct Edit {
    pub span: Span,
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

/// Applies formatting rules to the given code str, generating edits where
/// the source code needs to be changed to comply with the format rules.
pub fn format(code: &str) -> Vec<Edit> {
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

                if let ConcreteTokenKind::WhiteSpace = one.kind {
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
                    vec![Edit::new(0, three.span.lo, "")]
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
    let indent_level = if let ConcreteTokenKind::Syntax(TokenKind::Close(Delim::Brace)) = right.kind
    {
        indent_level.saturating_sub(1)
    } else {
        indent_level
    };

    // rule_trim_comments(left, &mut edits, code);
    // rule_close_empty_delims(left, whitespace, right, &mut edits);
    // rule_indentation(left, whitespace, right, &mut edits, indent_level);
    // rule_operator_single_space(left, whitespace, right, &mut edits);
    // rule_operator_no_space(left, whitespace, right, &mut edits);

    use qsc_frontend::keyword::Keyword;
    use ConcreteTokenKind::*;
    use TokenKind::*;
    match (&left.kind, &right.kind) {
        (Syntax(Open(l)), Syntax(Close(r))) if l == r => {
            effect_no_space(left, whitespace, right, &mut edits);
        }
        (Comment | Syntax(DocComment), _) => {
            effect_trim_comments(left, &mut edits, code);
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (Syntax(Semi), _) => match &right.kind {
            Comment => {
                if whitespace.contains('\n') {
                    effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
                }
            }
            _ => {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
        },
        (_, Syntax(Close(Delim::Brace)))
        | (_, Syntax(Keyword(Keyword::Operation)))
        | (_, Syntax(Keyword(Keyword::Function)))
        | (_, Syntax(Keyword(Keyword::Newtype)))
        | (_, Syntax(Keyword(Keyword::Namespace)))
        | (Syntax(Open(Delim::Brace)), _) => {
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (Syntax(Close(Delim::Brace)), Syntax(Semi)) => {
            effect_no_space(left, whitespace, right, &mut edits);
        }
        (Syntax(Close(Delim::Brace)), _) => {
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (Syntax(cooked_left), Syntax(cooked_right)) => match (cooked_left, cooked_right) {
            (Ident, Ident) | (Keyword(_), Ident) | (_, Colon) | (Colon, _) | (Comma, _) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (Ident, Open(Delim::Paren))
            | (Ident, Open(Delim::Bracket))
            | (Ident, Comma)
            | (Open(_), _)
            | (_, Close(_)) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            _ => {}
        },
        _ => {}
    }

    println!(
        "edits for `{}` : {edits:?}",
        &code[left.span.lo as usize..right.span.hi as usize]
    );
    edits
}

fn rule_operator_single_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    use ConcreteTokenKind::*;
    use TokenKind::*;
    if let (Syntax(cooked_left), Syntax(cooked_right)) = (&left.kind, &right.kind) {
        match (cooked_left, cooked_right) {
            (Comma, Close(_)) | (Semi, _) => {} // these have different rules
            (Ident, Ident) | (Keyword(_), Ident) | (_, Colon) | (Colon, _) | (Comma, _) => {
                effect_single_space(left, whitespace, right, edits);
            }
            _ => {}
        }
    }
}

fn rule_operator_no_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    use ConcreteTokenKind::*;
    use TokenKind::*;
    if let (Syntax(cooked_left), Syntax(cooked_right)) = (&left.kind, &right.kind) {
        match (cooked_left, cooked_right) {
            (_, Colon) | (Colon, _) | (Semi, _) => {} // Colons get single spaces, and Semi has indentation logic
            (Ident, Open(Delim::Paren))
            | (Ident, Open(Delim::Bracket))
            | (Ident, Comma)
            | (Close(Delim::Brace), Semi)
            | (Open(_), _)
            | (_, Close(_)) => {
                effect_no_space(left, whitespace, right, edits);
            }
            _ => {}
        }
    }
}

fn rule_indentation(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
    indent_level: usize,
) {
    use qsc_frontend::keyword::Keyword;
    use ConcreteTokenKind::*;
    use TokenKind::*;
    let cond = match (&left.kind, &right.kind) {
        (Comment | Syntax(DocComment), _) => true,
        (Syntax(Semi), _) => match &right.kind {
            Comment => whitespace.contains('\n'),
            _ => true,
        },
        (_, Syntax(Close(Delim::Brace)))
        | (_, Syntax(Keyword(Keyword::Operation)))
        | (_, Syntax(Keyword(Keyword::Function)))
        | (_, Syntax(Keyword(Keyword::Newtype)))
        | (_, Syntax(Keyword(Keyword::Namespace)))
        | (Syntax(Open(Delim::Brace)), _) => true,
        (Syntax(Close(Delim::Brace)), Syntax(Semi)) => false,
        (Syntax(Close(Delim::Brace)), _) => true,
        _ => false,
    };

    if cond {
        effect_correct_indentation(left, whitespace, right, edits, indent_level);
    }
}

fn rule_trim_comments(left: &ConcreteToken, edits: &mut Vec<Edit>, code: &str) {
    match &left.kind {
        ConcreteTokenKind::Comment | ConcreteTokenKind::Syntax(TokenKind::DocComment) => {
            effect_trim_comments(left, edits, code)
        }
        _ => {}
    }
}

fn rule_close_empty_delims(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    match (&left.kind, &right.kind) {
        (
            ConcreteTokenKind::Syntax(TokenKind::Open(l)),
            ConcreteTokenKind::Syntax(TokenKind::Close(r)),
        ) if l == r => {
            effect_no_space(left, whitespace, right, edits);
        }
        _ => {}
    }
}

fn effect_no_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    if !whitespace.is_empty() {
        edits.push(Edit::new(left.span.hi, right.span.lo, ""));
    }
}

fn effect_single_space(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<Edit>,
) {
    if whitespace != " " {
        edits.push(Edit::new(left.span.hi, right.span.lo, " "));
    }
}

fn effect_trim_comments(left: &ConcreteToken, edits: &mut Vec<Edit>, code: &str) {
    // fix trailing spaces on the comment
    let comment_contents = get_token_contents(code, left);
    let new_comment_contents = comment_contents.trim_end();
    if comment_contents != new_comment_contents {
        edits.push(Edit::new(left.span.lo, left.span.hi, new_comment_contents));
    }
}

fn effect_correct_indentation(
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
