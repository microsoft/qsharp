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

#[derive(Clone, Copy)]
enum NewlineContext {
    NoContext,
    Newlines,
    Spaces,
}

#[derive(Clone, Copy)]
enum TypeParameterListState {
    NoState,
    SeenCallableKeyword,
    SeenCallableName,
    InTypeParamList,
}

struct FormatterState<'a> {
    code: &'a str,
    indent_level: usize,
    delim_newlines_stack: Vec<NewlineContext>,
    type_param_state: TypeParameterListState,
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

    let mut state = FormatterState {
        code,
        indent_level: 0,
        delim_newlines_stack: vec![],
        type_param_state: TypeParameterListState::NoState,
    };

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
                if matches!(one.kind, ConcreteTokenKind::WhiteSpace) {
                    // first token is whitespace, continue scanning
                    continue;
                } else if matches!(two.kind, ConcreteTokenKind::WhiteSpace) {
                    // whitespace in the middle
                    apply_rules(one, get_token_contents(code, two), three, &mut state)
                } else {
                    // one, two are adjacent tokens with no whitespace in the middle
                    apply_rules(one, "", two, &mut state)
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
    state: &mut FormatterState,
) -> Vec<TextEdit> {
    let mut edits = vec![];
    // when we get here, neither left nor right should be whitespace

    let are_newlines_in_spaces = whitespace.contains('\n');
    let mut newline_context = state
        .delim_newlines_stack
        .last()
        .map_or(NewlineContext::NoContext, |b| *b);

    use qsc_frontend::keyword::Keyword;
    use qsc_frontend::lex::cooked::ClosedBinOp;
    use ConcreteTokenKind::*;
    use TokenKind::*;

    // Save the left token's status as a delimiter before updating the delimiter state
    let left_delim_state = to_delim_enum(&left.kind, state.type_param_state);

    // If we are leaving a type param list, reset the state
    if matches!(&left.kind, Syntax(Gt))
        && matches!(
            state.type_param_state,
            TypeParameterListState::InTypeParamList
        )
    {
        state.type_param_state = TypeParameterListState::NoState
    }

    match &right.kind {
        Comment => {
            // comments don't update state
        }
        Syntax(Keyword(Keyword::Operation | Keyword::Function)) => {
            state.type_param_state = TypeParameterListState::SeenCallableKeyword;
        }
        Syntax(Ident)
            if matches!(
                state.type_param_state,
                TypeParameterListState::SeenCallableKeyword
            ) =>
        {
            state.type_param_state = TypeParameterListState::SeenCallableName;
        }
        Syntax(Lt)
            if matches!(
                state.type_param_state,
                TypeParameterListState::SeenCallableName
            ) =>
        {
            state.type_param_state = TypeParameterListState::InTypeParamList;
        }
        Syntax(AposIdent | Comma | Gt)
            if matches!(
                state.type_param_state,
                TypeParameterListState::InTypeParamList
            ) =>
        {
            // type param identifiers and commas don't take us out of the type parameter list context
            // Gt only takes us out of the list once we are past it (it is the left-hand token)
        }
        _ => {
            state.type_param_state = TypeParameterListState::NoState;
        }
    }

    // Save the right token's status as a delimiter after updating the delimiter state
    let right_delim_state = to_delim_enum(&right.kind, state.type_param_state);

    let does_right_required_newline =
        matches!(&right.kind, Syntax(cooked_right) if is_newline_keyword_or_ampersat(cooked_right));

    match (left_delim_state, right_delim_state) {
        (Delimiter::Open, Delimiter::Close) => {
            // Don't change the indentation if empty sequence.
        }
        (Delimiter::Open, _) if are_newlines_in_spaces => {
            newline_context = NewlineContext::Newlines;
            state.delim_newlines_stack.push(newline_context);
            state.indent_level += 1;
        }
        (Delimiter::Open, _) if does_right_required_newline => {
            newline_context = NewlineContext::Newlines;
            state.delim_newlines_stack.push(newline_context);
            state.indent_level += 1;
        }
        (Delimiter::Open, _) if matches!(right.kind, Comment) => {
            newline_context = NewlineContext::Newlines;
            state.delim_newlines_stack.push(newline_context);
            state.indent_level += 1;
        }
        (Delimiter::Open, _) => {
            newline_context = NewlineContext::Spaces;
            state.delim_newlines_stack.push(newline_context);
        }
        (_, Delimiter::Close) => {
            state.delim_newlines_stack.pop();
            if matches!(newline_context, NewlineContext::Newlines) {
                state.indent_level = state.indent_level.saturating_sub(1);
            }
        }
        _ => {}
    }

    match (&left.kind, &right.kind) {
        (Comment | Syntax(DocComment), _) => {
            // remove whitespace at the ends of comments
            effect_trim_comment(left, &mut edits, state.code);
            effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
        }
        (_, Comment) if matches!(left_delim_state, Delimiter::Open) => {
            effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
        }
        (_, Comment) => {
            if are_newlines_in_spaces {
                effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
            }
            // else do nothing, preserving the user's spaces before the comment
        }
        (Syntax(cooked_left), Syntax(cooked_right)) => match (cooked_left, cooked_right) {
            (ClosedBinOp(ClosedBinOp::Minus), _) | (_, ClosedBinOp(ClosedBinOp::Minus)) => {
                // This case is used to ignore the spacing around a `-`.
                // This is done because we currently don't have the architecture
                // to be able to differentiate between the unary `-` and the binary `-`
                // which would have different spacing rules.
            }
            (Semi, _) if matches!(newline_context, NewlineContext::Spaces) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (Semi, _) => {
                effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
            }
            (_, Semi) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Open(l), Close(r)) if l == r => {
                // close empty delimiter blocks, i.e. (), [], {}
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Lt, Gt)
                if matches!(
                    state.type_param_state,
                    TypeParameterListState::InTypeParamList
                ) =>
            {
                // close empty delimiter blocks <>
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (_, _)
                if matches!(left_delim_state, Delimiter::Open)
                    && matches!(newline_context, NewlineContext::Newlines) =>
            {
                effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
            }
            (Comma, _) if matches!(newline_context, NewlineContext::Newlines) => {
                effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
            }
            (Comma, _) => {
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, _)
                if matches!(right_delim_state, Delimiter::Close)
                    && matches!(newline_context, NewlineContext::Newlines) =>
            {
                effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
            }
            (Open(Delim::Bracket | Delim::Paren), _)
            | (_, Close(Delim::Bracket | Delim::Paren)) => {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Lt, _) | (_, Gt)
                if matches!(
                    state.type_param_state,
                    TypeParameterListState::InTypeParamList
                ) =>
            {
                effect_no_space(left, whitespace, right, &mut edits);
            }
            (Open(Delim::Brace), _) | (_, Close(Delim::Brace)) => {
                effect_single_space(left, whitespace, right, &mut edits);
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
            (_, _) if does_right_required_newline => {
                effect_correct_indentation(left, whitespace, right, &mut edits, state.indent_level);
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
            (_, _) if are_newlines_in_spaces => {
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
            (_, Open(Delim::Brace)) => {
                // Special-case braces to always have a leading single space
                effect_single_space(left, whitespace, right, &mut edits);
            }
            (_, _) if matches!(right_delim_state, Delimiter::Open) => {
                // Otherwise, all open delims have the same logic
                if is_value_token_left(cooked_left, left_delim_state) || is_prefix(cooked_left) {
                    // i.e. foo() or { foo }[3]
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    // i.e. let x = (1, 2, 3);
                    effect_single_space(left, whitespace, right, &mut edits);
                }
            }
            (_, TokenKind::DotDotDot) => {
                if is_value_token_left(cooked_left, left_delim_state) {
                    effect_no_space(left, whitespace, right, &mut edits);
                } else {
                    effect_single_space(left, whitespace, right, &mut edits);
                }
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
            (_, _) if is_value_token_right(cooked_right, right_delim_state) => {
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

#[derive(Clone, Copy)]
enum Delimiter {
    Open,
    Close,
    NonDelim,
}

fn to_delim_enum(kind: &ConcreteTokenKind, type_param_state: TypeParameterListState) -> Delimiter {
    match kind {
        ConcreteTokenKind::Syntax(TokenKind::Open(_)) => Delimiter::Open,
        ConcreteTokenKind::Syntax(TokenKind::Lt)
            if matches!(type_param_state, TypeParameterListState::InTypeParamList) =>
        {
            Delimiter::Open
        }
        ConcreteTokenKind::Syntax(TokenKind::Close(_)) => Delimiter::Close,
        ConcreteTokenKind::Syntax(TokenKind::Gt)
            if matches!(type_param_state, TypeParameterListState::InTypeParamList) =>
        {
            Delimiter::Close
        }
        _ => Delimiter::NonDelim,
    }
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

fn is_newline_keyword_or_ampersat(cooked: &TokenKind) -> bool {
    use Keyword::*;
    matches!(
        cooked,
        TokenKind::At
            | TokenKind::Keyword(
                Internal
                    | Operation
                    | Function
                    | Newtype
                    | Namespace
                    | Open
                    | Body
                    | Adjoint
                    | Controlled
                    | Let
                    | Mutable
                    | Set
                    | Use
                    | Borrow
                    | Fixup
            )
    )
}

/// Note that this does not include interpolated string literals
fn is_value_lit(cooked: &TokenKind) -> bool {
    matches!(
        cooked,
        TokenKind::BigInt(_)
            | TokenKind::Float
            | TokenKind::Int(_)
            | TokenKind::String(StringToken::Normal)
    )
}

fn is_value_token_left(cooked: &TokenKind, delim_state: Delimiter) -> bool {
    // a closed delim represents a value on the left
    if matches!(delim_state, Delimiter::Close) {
        return true;
    }

    match cooked {
        _ if is_value_lit(cooked) => true,
        TokenKind::Keyword(keyword) if is_keyword_value(keyword) => true,
        TokenKind::Ident
        | TokenKind::AposIdent
        | TokenKind::String(StringToken::Interpolated(_, InterpolatedEnding::Quote)) => true,
        _ => false,
    }
}

fn is_value_token_right(cooked: &TokenKind, delim_state: Delimiter) -> bool {
    // an open delim represents a value on the right
    if matches!(delim_state, Delimiter::Open) {
        return true;
    }

    match cooked {
        _ if is_value_lit(cooked) => true,
        TokenKind::Keyword(keyword) if is_keyword_value(keyword) => true,
        TokenKind::Ident
        | TokenKind::AposIdent
        | TokenKind::String(StringToken::Interpolated(InterpolatedStart::DollarQuote, _)) => true,
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
