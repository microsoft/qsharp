// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_data_structures::span::Span;
use qsc_frontend::{
    keyword::Keyword,
    lex::{
        concrete::{self, ConcreteToken, ConcreteTokenKind},
        cooked::{StringToken, TokenKind},
        Delim, InterpolatedEnding, InterpolatedStart,
    },
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
                if matches!(three.kind, ConcreteTokenKind::WhiteSpace) {
                    vec![TextEdit::new("", three.span.lo, three.span.hi)]
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

    let new_line_in_spaces = whitespace.contains('\n');

    use qsc_frontend::keyword::Keyword;
    use qsc_frontend::lex::cooked::ClosedBinOp;
    use ConcreteTokenKind::*;
    use TokenKind::*;
    match (&left.kind, &right.kind) {
        (Comment | Syntax(DocComment), _) => {
            // remove whitespace at the ends of comments
            effect_trim_comment(left, &mut edits, code);
            effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
        }
        (_, Comment) => {
            if new_line_in_spaces {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
        }
        (Syntax(cooked_left), Syntax(cooked_right)) => match (cooked_left, cooked_right) {
            (ClosedBinOp(ClosedBinOp::Minus), _) | (_, ClosedBinOp(ClosedBinOp::Minus)) => {
                // This case is used to ignore the spacing around a `-`.
                // This is done because we currently don't have the architecture
                // to be able to differentiate between the unary `-` and the binary `-`
                // which would have different spacing rules.
            }
            (Gt, _) | (_, Gt) | (Lt, _) | (_, Lt) => {
                // This case is used to ignore the spacing around a `<` and `>`.
                // This is done because we currently don't have the architecture
                // to be able to differentiate between the comparison operators
                // and the type-parameter delimiters which would have different
                // spacing rules.
            }
            (Semi, _) => {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
            (_, Semi) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Open(l), Close(r)) if l == r => {
                // close empty delimiter blocks, i.e. (), [], {}
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (At, Ident) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Keyword(Keyword::Internal), _) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (Keyword(Keyword::Adjoint), Keyword(Keyword::Controlled))
            | (Keyword(Keyword::Controlled), Keyword(Keyword::Adjoint)) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (Open(Delim::Brace), _)
            | (_, Close(Delim::Brace))
            | (_, Keyword(Keyword::Internal))
            | (_, Keyword(Keyword::Operation))
            | (_, Keyword(Keyword::Function))
            | (_, Keyword(Keyword::Newtype))
            | (_, Keyword(Keyword::Namespace))
            | (_, Keyword(Keyword::Open))
            | (_, Keyword(Keyword::Body))
            | (_, Keyword(Keyword::Adjoint))
            | (_, Keyword(Keyword::Controlled))
            | (_, Keyword(Keyword::Let))
            | (_, Keyword(Keyword::Mutable))
            | (_, Keyword(Keyword::Set))
            | (_, Keyword(Keyword::Use))
            | (_, Keyword(Keyword::Borrow))
            | (_, Keyword(Keyword::Fixup))
            | (_, At) => {
                effect_correct_indentation(left, whitespace, right, &mut edits, indent_level);
            }
            (_, TokenKind::Keyword(Keyword::Until))
            | (_, TokenKind::Keyword(Keyword::In))
            | (_, TokenKind::Keyword(Keyword::As))
            | (_, TokenKind::Keyword(Keyword::Elif))
            | (_, TokenKind::Keyword(Keyword::Else))
            | (_, TokenKind::Keyword(Keyword::Apply)) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, TokenKind::Keyword(Keyword::Auto))
            | (_, TokenKind::Keyword(Keyword::Distribute))
            | (_, TokenKind::Keyword(Keyword::Intrinsic))
            | (_, TokenKind::Keyword(Keyword::Invert))
            | (_, TokenKind::Keyword(Keyword::Slf)) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, _) if new_line_in_spaces => {
                effect_trim_whitespace(left, whitespace, right, &mut edits);
                // Ignore the rest of the cases if the user has a newline in the whitespace.
                // This is done because we don't currently have logic for determining when
                // lines are too long and require newlines, and we don't have logic
                // for determining what the correct indentation should be in these cases,
                // so we put this do-nothing case in to leave user code unchanged.
            }
            (String(StringToken::Interpolated(_, InterpolatedEnding::LBrace)), _)
            | (_, String(StringToken::Interpolated(InterpolatedStart::RBrace, _))) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Open(Delim::Bracket | Delim::Paren), _)
            | (_, Close(Delim::Bracket | Delim::Paren)) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (_, Open(Delim::Bracket | Delim::Paren)) => {
                if is_value_token_left(cooked_left) || is_prefix(cooked_left) {
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
            (TokenKind::DotDotDot, TokenKind::Open(Delim::Brace)) => {
                // Special case: `... {}`
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, TokenKind::Keyword(Keyword::Is))
            | (_, TokenKind::Keyword(Keyword::For))
            | (_, TokenKind::Keyword(Keyword::While))
            | (_, TokenKind::Keyword(Keyword::Repeat))
            | (_, TokenKind::Keyword(Keyword::If))
            | (_, TokenKind::Keyword(Keyword::Within))
            | (_, TokenKind::Keyword(Keyword::Return))
            | (_, TokenKind::Keyword(Keyword::Fail)) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, _) if is_value_token_right(cooked_right) => {
                if is_prefix(cooked_left) {
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    effect_single_space(left, whitespace, right, &mut edits);
                }
            }
            (_, _) if is_suffix(cooked_right) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (_, _) if is_prefix_with_space(cooked_right) => {
                if is_prefix(cooked_left) {
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    effect_single_space(left, whitespace, right, &mut edits);
                }
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
            | TokenKind::Keyword(Keyword::And)
            | TokenKind::Keyword(Keyword::Or)
            // Technically the rest are not binary ops, but has the same spacing as one
            | TokenKind::Keyword(Keyword::Not)
            | TokenKind::Keyword(Keyword::AdjointUpper)
            | TokenKind::Keyword(Keyword::ControlledUpper)
    )
}

fn is_prefix_with_space(cooked: &TokenKind) -> bool {
    matches!(cooked, TokenKind::TildeTildeTilde)
}

fn is_prefix_without_space(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::ColonColon | TokenKind::Dot | TokenKind::DotDot
    )
}

fn is_prefix(cooked: &TokenKind) -> bool {
    is_prefix_with_space(cooked)
        || is_prefix_without_space(cooked)
        || matches!(cooked, TokenKind::DotDotDot)
}

fn is_suffix(cooked: &TokenKind) -> bool {
    matches!(cooked, TokenKind::Bang | TokenKind::Comma)
}

fn is_keyword_value(keyword: &Keyword) -> bool {
    use Keyword::*;
    matches!(
        keyword,
        True | False | Zero | One | PauliI | PauliX | PauliY | PauliZ | Underscore
        // Adj and Ctl are not really values, but have the same spacing as values
        | Adj | Ctl
    )
}

/// Note that this does not include interpolated string literals
fn is_value_lit(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::BigInt(_)
            | TokenKind::Float
            | TokenKind::Ident
            | TokenKind::AposIdent
            | TokenKind::Int(_)
            | TokenKind::String(StringToken::Normal)
    )
}

fn is_value_token_left(cooked: &TokenKind) -> bool {
    match cooked {
        _ if is_value_lit(cooked) => true,
        TokenKind::String(StringToken::Interpolated(_, InterpolatedEnding::Quote)) => true,
        TokenKind::Keyword(keyword) if is_keyword_value(keyword) => true,
        TokenKind::Close(_) => true, // a closed delim represents a value on the left
        _ => false,
    }
}

fn is_value_token_right(cooked: &TokenKind) -> bool {
    match cooked {
        _ if is_value_lit(cooked) => true,
        TokenKind::String(StringToken::Interpolated(InterpolatedStart::DollarQuote, _)) => true,
        TokenKind::Keyword(keyword) if is_keyword_value(keyword) => true,
        TokenKind::Open(_) => true, // an open delim represents a value on the right
        _ => false,
    }
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

fn effect_trim_whitespace(
    left: &ConcreteToken,
    whitespace: &str,
    right: &ConcreteToken,
    edits: &mut Vec<TextEdit>,
) {
    let count_newlines = whitespace.chars().filter(|c| *c == '\n').count();
    let suffix = match whitespace.rsplit_once('\n') {
        Some((_, suffix)) => suffix,
        None => "",
    };

    let mut new_whitespace = if whitespace.contains("\r\n") {
        "\r\n".repeat(count_newlines)
    } else {
        "\n".repeat(count_newlines)
    };
    new_whitespace.push_str(suffix);
    if whitespace != new_whitespace {
        edits.push(TextEdit::new(
            new_whitespace.as_str(),
            left.span.hi,
            right.span.lo,
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

    let mut new_whitespace = if whitespace.contains("\r\n") {
        "\r\n".repeat(count_newlines)
    } else {
        "\n".repeat(count_newlines)
    };
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
