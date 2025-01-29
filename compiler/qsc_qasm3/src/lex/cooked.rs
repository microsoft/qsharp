// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The second lexing phase "cooks" a raw token stream, transforming them into tokens that directly
//! correspond to components in the `OpenQASM` grammar. Keywords are treated as identifiers, except `and`
//! and `or`, which are cooked into [`BinaryOperator`] so that `and=` and `or=` are lexed correctly.
//!
//! Whitespace and comment tokens are discarded; this means that cooked tokens are not necessarily
//! contiguous, so they include both a starting and ending byte offset.
//!
//! Tokens never contain substrings from the original input, but are simply labels that refer back
//! to regions in the input. Lexing never fails, but may produce error tokens.

#[cfg(test)]
mod tests;

use super::{
    raw::{self, Number, Single},
    Delim, Radix,
};
use crate::keyword::Keyword;
use enum_iterator::Sequence;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use std::{
    fmt::{self, Display, Formatter},
    iter::Peekable,
};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

#[derive(Clone, Copy, Debug, Diagnostic, Eq, Error, PartialEq)]
pub enum Error {
    #[error("expected {0} to complete {1}, found {2}")]
    #[diagnostic(code("Qsc.Lex.Incomplete"))]
    Incomplete(raw::TokenKind, TokenKind, raw::TokenKind, #[label] Span),

    #[error("expected {0} to complete {1}, found EOF")]
    #[diagnostic(code("Qsc.Lex.IncompleteEof"))]
    IncompleteEof(raw::TokenKind, TokenKind, #[label] Span),

    #[error("unterminated string literal")]
    #[diagnostic(code("Qsc.Lex.UnterminatedString"))]
    UnterminatedString(#[label] Span),

    #[error("unrecognized character `{0}`")]
    #[diagnostic(code("Qsc.Lex.UnknownChar"))]
    Unknown(char, #[label] Span),
}

impl Error {
    pub(crate) fn with_offset(self, offset: u32) -> Self {
        match self {
            Self::Incomplete(expected, token, actual, span) => {
                Self::Incomplete(expected, token, actual, span + offset)
            }
            Self::IncompleteEof(expected, token, span) => {
                Self::IncompleteEof(expected, token, span + offset)
            }
            Self::UnterminatedString(span) => Self::UnterminatedString(span + offset),
            Self::Unknown(c, span) => Self::Unknown(c, span + offset),
        }
    }

    pub(crate) fn span(self) -> Span {
        match self {
            Error::Incomplete(_, _, _, s)
            | Error::IncompleteEof(_, _, s)
            | Error::UnterminatedString(s)
            | Error::Unknown(_, s) => s,
        }
    }
}

/// A token kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum TokenKind {
    Keyword(Keyword),
    Type(Type),

    // Builtin identifiers and operations
    GPhase,
    Inv,
    Pow,
    Ctrl,
    NegCtrl,
    Dim,
    DurationOf,
    Delay,
    Reset,
    Measure,
    Barrier,

    Literal(Literal),

    // Symbols
    /// `{[(`
    Open(Delim),
    /// `}])`
    Close(Delim),

    // Punctuation
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `.`
    Dot,
    /// `,`
    Comma,
    /// `++`
    PlusPlus,
    /// `->`
    Arrow,

    // Operators,
    UnaryOperator(UnaryOperator),
    BinaryOperator(ClosedBinaryOperator),
    BinaryOperatorEq(ClosedBinaryOperator),
    ComparisonOperator(ComparisonOperator),
    Eq,

    Identifier,
    HardwareQubit,

    Whitespace,
    Comment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Type {
    Input,
    Output,
    Const,
    Readonly,
    Mutable,

    QReg,
    Qubit,

    CReg,
    Bool,
    Bit,
    Int,
    UInt,
    Float,
    Angle,
    Complex,
    Array,
    Void,

    Duration,
    Stretch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Literal {
    Bitstring,
    Boolean,
    Float,
    Imaginary,
    Integer(Radix),
    String,
    Timing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum UnaryOperator {
    /// `!`
    Bang,
    /// `-`
    Minus,
    /// `~`
    Tilde,
}

/// A binary operator that returns the same type as the type of its first operand; in other words,
/// the domain of the first operand is closed under this operation. These are candidates for
/// compound assignment operators, like `+=`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum ClosedBinaryOperator {
    /// `&`
    Amp,
    /// `|`
    Bar,
    /// `^`
    Caret,
    /// `>>`
    GtGt,
    /// `<<`
    LtLt,
    /// `-`
    Minus,
    /// `%`
    Percent,
    /// `+`
    Plus,
    /// `/`
    Slash,
    /// `*`
    Star,
    /// `**`
    StarStar,
    // TODO: missing Tilde according to qasm3Lexer.g4 to be able to express ~=
    //       But this is this a bug in the official qasm lexer?
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum ComparisonOperator {
    /// `&&`
    AmpAmp,
    /// `!=`
    BangEq,
    /// `||`
    BarBar,
    /// `==`
    EqEq,
    /// `>`
    Gt,
    /// `>=`
    GtEq,
    /// `<`
    Lt,
    /// `<=`
    LtEq,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        todo!()
    }
}

impl From<Number> for TokenKind {
    fn from(value: Number) -> Self {
        match value {
            Number::Float => Self::Literal(Literal::Float),
            Number::Int(radix) => Self::Literal(Literal::Integer(radix)),
        }
    }
}

impl Display for ClosedBinaryOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        todo!()
    }
}

pub(crate) struct Lexer<'a> {
    input: &'a str,
    len: u32,

    // This uses a `Peekable` iterator over the raw lexer, which allows for one token lookahead.
    tokens: Peekable<raw::Lexer<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            len: input
                .len()
                .try_into()
                .expect("input length should fit into u32"),
            tokens: raw::Lexer::new(input).peekable(),
        }
    }

    fn offset(&mut self) -> u32 {
        self.tokens.peek().map_or_else(|| self.len, |t| t.offset)
    }

    fn next_if_eq_single(&mut self, single: Single) -> bool {
        self.next_if_eq(raw::TokenKind::Single(single))
    }

    fn next_if_eq(&mut self, tok: raw::TokenKind) -> bool {
        self.tokens.next_if(|t| t.kind == tok).is_some()
    }

    fn expect_single(&mut self, single: Single, complete: TokenKind) -> Result<(), Error> {
        self.expect(raw::TokenKind::Single(single), complete)
    }

    fn expect(&mut self, tok: raw::TokenKind, complete: TokenKind) -> Result<(), Error> {
        if self.next_if_eq(tok) {
            Ok(())
        } else if let Some(&raw::Token { kind, offset }) = self.tokens.peek() {
            let mut tokens = self.tokens.clone();
            let hi = tokens.nth(1).map_or_else(|| self.len, |t| t.offset);
            let span = Span { lo: offset, hi };
            Err(Error::Incomplete(tok, complete, kind, span))
        } else {
            let lo = self.len;
            let span = Span { lo, hi: lo };
            Err(Error::IncompleteEof(tok, complete, span))
        }
    }

    fn cook(&mut self, token: &raw::Token) -> Result<Option<Token>, Error> {
        let kind = match token.kind {
            raw::TokenKind::Comment(raw::CommentKind::Block | raw::CommentKind::Normal)
            | raw::TokenKind::Whitespace => Ok(None),
            raw::TokenKind::Ident => {
                let ident = &self.input[(token.offset as usize)..(self.offset() as usize)];
                Ok(Some(self.ident(ident)))
            }
            raw::TokenKind::Number(number) => Ok(Some(number.into())),
            raw::TokenKind::Single(single) => self.single(single).map(Some),
            raw::TokenKind::String(raw::StringToken { terminated: true }) => {
                Ok(Some(TokenKind::Literal(Literal::String)))
            }
            raw::TokenKind::String(raw::StringToken { terminated: false }) => {
                Err(Error::UnterminatedString(Span {
                    lo: token.offset,
                    hi: token.offset,
                }))
            }
            raw::TokenKind::Unknown => {
                let c = self.input[(token.offset as usize)..]
                    .chars()
                    .next()
                    .expect("token offset should be the start of a character");
                let span = Span {
                    lo: token.offset,
                    hi: self.offset(),
                };
                Err(Error::Unknown(c, span))
            }
        }?;

        Ok(kind.map(|kind| {
            let span = Span {
                lo: token.offset,
                hi: self.offset(),
            };
            Token { kind, span }
        }))
    }

    /// Consumes a list of tokens zero or more times.
    fn kleen_star(&mut self, tokens: &[raw::TokenKind], complete: TokenKind) -> Result<(), Error> {
        let mut iter = tokens.iter();
        while self.next_if_eq(*(iter.next().expect("tokens should have at least one token"))) {
            for token in iter {
                self.expect(*token, complete)?
            }
            iter = tokens.iter();
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn single(&mut self, single: Single) -> Result<TokenKind, Error> {
        match single {
            Single::Amp => {
                if self.next_if_eq_single(Single::Amp) {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::AmpAmp))
                } else {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::Amp))
                }
            }
            Single::At => {
                let complete = TokenKind::Keyword(Keyword::Annotation);
                self.expect(raw::TokenKind::Ident, complete)?;
                self.kleen_star(
                    &[raw::TokenKind::Single(Single::Dot), raw::TokenKind::Ident],
                    complete,
                )?;
                Ok(TokenKind::Keyword(Keyword::Annotation))
            }
            Single::Bang => {
                if self.next_if_eq_single(Single::Eq) {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::BangEq))
                } else {
                    Ok(TokenKind::UnaryOperator(UnaryOperator::Bang))
                }
            }
            Single::Bar => {
                if self.next_if_eq_single(Single::Bar) {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::BarBar))
                } else {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::Bar))
                }
            }
            Single::Caret => Ok(self.closed_bin_op(ClosedBinaryOperator::Caret)),
            Single::Close(delim) => Ok(TokenKind::Close(delim)),
            Single::Colon => Ok(TokenKind::Colon),
            Single::Comma => Ok(TokenKind::Comma),
            Single::Dot => Ok(TokenKind::Dot),
            Single::Eq => {
                if self.next_if_eq_single(Single::Eq) {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::EqEq))
                } else {
                    Ok(TokenKind::Eq)
                }
            }
            Single::Gt => {
                if self.next_if_eq_single(Single::Eq) {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::GtEq))
                } else if self.next_if_eq_single(Single::Gt) {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::GtGt))
                } else {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::Gt))
                }
            }
            Single::Lt => {
                if self.next_if_eq_single(Single::Eq) {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::LtEq))
                } else if self.next_if_eq_single(Single::Lt) {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::LtLt))
                } else {
                    Ok(TokenKind::ComparisonOperator(ComparisonOperator::Lt))
                }
            }
            Single::Minus => {
                if self.next_if_eq_single(Single::Gt) {
                    Ok(TokenKind::Arrow)
                } else {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::Minus))
                }
            }
            Single::Open(delim) => Ok(TokenKind::Open(delim)),
            Single::Percent => Ok(self.closed_bin_op(ClosedBinaryOperator::Percent)),
            Single::Plus => {
                if self.next_if_eq_single(Single::Plus) {
                    Ok(TokenKind::PlusPlus)
                } else {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::Plus))
                }
            }
            Single::Semi => Ok(TokenKind::Semicolon),
            Single::Slash => Ok(self.closed_bin_op(ClosedBinaryOperator::Slash)),
            Single::Star => {
                if self.next_if_eq_single(Single::Star) {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::StarStar))
                } else {
                    Ok(self.closed_bin_op(ClosedBinaryOperator::Star))
                }
            }
            Single::Tilde => Ok(TokenKind::UnaryOperator(UnaryOperator::Tilde)),
        }
    }

    fn closed_bin_op(&mut self, op: ClosedBinaryOperator) -> TokenKind {
        if self.next_if_eq_single(Single::Eq) {
            TokenKind::BinaryOperatorEq(op)
        } else {
            TokenKind::BinaryOperator(op)
        }
    }

    fn ident(&mut self, ident: &str) -> TokenKind {
        match ident {
            "gphase" => TokenKind::GPhase,
            "inv" => TokenKind::Inv,
            "pow" => TokenKind::Pow,
            "ctrl" => TokenKind::Ctrl,
            "negctrl" => TokenKind::NegCtrl,
            "dim" => TokenKind::Dim,
            "durationof" => TokenKind::DurationOf,
            "delay" => TokenKind::Delay,
            "reset" => TokenKind::Reset,
            "measure" => TokenKind::Measure,
            "barrier" => TokenKind::Barrier,
            "false" | "true" => TokenKind::Literal(Literal::Boolean),
            ident => ident
                .parse()
                .map(TokenKind::Keyword)
                .unwrap_or(TokenKind::Identifier),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(token) = self.tokens.next() {
            match self.cook(&token) {
                Ok(None) => {}
                Ok(Some(token)) => return Some(Ok(token)),
                Err(err) => return Some(Err(err)),
            }
        }

        None
    }
}
