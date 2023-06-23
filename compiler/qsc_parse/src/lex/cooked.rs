// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The second lexing phase "cooks" a raw token stream, transforming them into tokens that directly
//! correspond to components in the Q# grammar. Keywords are treated as identifiers, except `and`
//! and `or`, which are cooked into [`ClosedBinOp`] so that `and=` and `or=` are lexed correctly.
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
    Delim, InterpolatedEnding, InterpolatedStart, Radix,
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
pub(crate) enum Error {
    #[error("expected `{0}` to complete {1}, found {2}")]
    #[diagnostic(code("Qsc.Lex.Incomplete"))]
    Incomplete(raw::Single, TokenKind, raw::TokenKind, #[label] Span),

    #[error("expected `{0}` to complete {1}, found EOF")]
    #[diagnostic(code("Qsc.Lex.IncompleteEof"))]
    IncompleteEof(raw::Single, TokenKind, #[label] Span),

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
}

/// A token kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub(crate) enum TokenKind {
    /// `'`
    Apos,
    /// `@`
    At,
    /// `!`
    Bang,
    /// `|`
    Bar,
    /// A big integer literal.
    BigInt(Radix),
    /// A closed binary operator followed by an equals token.
    BinOpEq(ClosedBinOp),
    /// A closing delimiter.
    Close(Delim),
    /// A closed binary operator not followed by an equals token.
    ClosedBinOp(ClosedBinOp),
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `,`
    Comma,
    /// A doc comment.
    DocComment,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `...`
    DotDotDot,
    /// End of file.
    Eof,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `=>`
    FatArrow,
    /// A floating-point literal.
    Float,
    /// `>`
    Gt,
    /// `>=`
    Gte,
    /// An identifier.
    Ident,
    /// An integer literal.
    Int(Radix),
    /// A keyword.
    Keyword(Keyword),
    /// `<-`
    LArrow,
    /// `<`
    Lt,
    /// `<=`
    Lte,
    /// `!=`
    Ne,
    /// An opening delimiter.
    Open(Delim),
    /// `?`
    Question,
    /// `->`
    RArrow,
    /// `;`
    Semi,
    /// A string literal.
    String(StringToken),
    /// `~~~`
    TildeTildeTilde,
    /// `w/`
    WSlash,
    /// `w/=`
    WSlashEq,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TokenKind::Apos => f.write_str("`'`"),
            TokenKind::At => f.write_str("`@`"),
            TokenKind::Bang => f.write_str("`!`"),
            TokenKind::Bar => f.write_str("`|`"),
            TokenKind::BigInt(_) => f.write_str("big integer"),
            TokenKind::BinOpEq(op) => write!(f, "`{op}=`"),
            TokenKind::Close(Delim::Brace) => f.write_str("`}`"),
            TokenKind::Close(Delim::Bracket) => f.write_str("`]`"),
            TokenKind::Close(Delim::Paren) => f.write_str("`)`"),
            TokenKind::ClosedBinOp(op) => write!(f, "`{op}`"),
            TokenKind::Colon => f.write_str("`:`"),
            TokenKind::ColonColon => f.write_str("`::`"),
            TokenKind::Comma => f.write_str("`,`"),
            TokenKind::DocComment => f.write_str("doc comment"),
            TokenKind::Dot => f.write_str("`.`"),
            TokenKind::DotDot => f.write_str("`..`"),
            TokenKind::DotDotDot => f.write_str("`...`"),
            TokenKind::Eof => f.write_str("EOF"),
            TokenKind::Eq => f.write_str("`=`"),
            TokenKind::EqEq => f.write_str("`==`"),
            TokenKind::FatArrow => f.write_str("`=>`"),
            TokenKind::Float => f.write_str("float"),
            TokenKind::Gt => f.write_str("`>`"),
            TokenKind::Gte => f.write_str("`>=`"),
            TokenKind::Ident => f.write_str("identifier"),
            TokenKind::Int(_) => f.write_str("integer"),
            TokenKind::Keyword(keyword) => write!(f, "keyword `{keyword}`"),
            TokenKind::LArrow => f.write_str("`<-`"),
            TokenKind::Lt => f.write_str("`<`"),
            TokenKind::Lte => f.write_str("`<=`"),
            TokenKind::Ne => f.write_str("`!=`"),
            TokenKind::Open(Delim::Brace) => f.write_str("`{`"),
            TokenKind::Open(Delim::Bracket) => f.write_str("`[`"),
            TokenKind::Open(Delim::Paren) => f.write_str("`(`"),
            TokenKind::Question => f.write_str("`?`"),
            TokenKind::RArrow => f.write_str("`->`"),
            TokenKind::Semi => f.write_str("`;`"),
            TokenKind::String(_) => f.write_str("string"),
            TokenKind::TildeTildeTilde => f.write_str("`~~~`"),
            TokenKind::WSlash => f.write_str("`w/`"),
            TokenKind::WSlashEq => f.write_str("`w/=`"),
        }
    }
}

impl From<Number> for TokenKind {
    fn from(value: Number) -> Self {
        match value {
            Number::BigInt(radix) => Self::BigInt(radix),
            Number::Float => Self::Float,
            Number::Int(radix) => Self::Int(radix),
        }
    }
}

/// A binary operator that returns the same type as the type of its first operand; in other words,
/// the domain of the first operand is closed under this operation. These are candidates for
/// compound assignment operators, like `+=`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub(crate) enum ClosedBinOp {
    /// `&&&`
    AmpAmpAmp,
    /// `and`
    And,
    /// `|||`
    BarBarBar,
    /// `^`
    Caret,
    /// `^^^`
    CaretCaretCaret,
    /// `>>>`
    GtGtGt,
    /// `<<<`
    LtLtLt,
    /// `-`
    Minus,
    /// `or`
    Or,
    /// `%`
    Percent,
    /// `+`
    Plus,
    /// `/`
    Slash,
    /// `*`
    Star,
}

impl Display for ClosedBinOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            ClosedBinOp::AmpAmpAmp => "&&&",
            ClosedBinOp::And => "and",
            ClosedBinOp::BarBarBar => "|||",
            ClosedBinOp::Caret => "^",
            ClosedBinOp::CaretCaretCaret => "^^^",
            ClosedBinOp::GtGtGt => ">>>",
            ClosedBinOp::LtLtLt => "<<<",
            ClosedBinOp::Minus => "-",
            ClosedBinOp::Or => "or",
            ClosedBinOp::Percent => "%",
            ClosedBinOp::Plus => "+",
            ClosedBinOp::Slash => "/",
            ClosedBinOp::Star => "*",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub(crate) enum StringToken {
    Normal,
    Interpolated(InterpolatedStart, InterpolatedEnding),
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

    fn next_if_eq(&mut self, single: Single) -> bool {
        self.tokens
            .next_if(|t| t.kind == raw::TokenKind::Single(single))
            .is_some()
    }

    fn expect(&mut self, single: Single, complete: TokenKind) -> Result<(), Error> {
        if self.next_if_eq(single) {
            Ok(())
        } else if let Some(&raw::Token { kind, offset }) = self.tokens.peek() {
            let mut tokens = self.tokens.clone();
            let hi = tokens.nth(1).map_or_else(|| self.len, |t| t.offset);
            let span = Span { lo: offset, hi };
            Err(Error::Incomplete(single, complete, kind, span))
        } else {
            let lo = self.len;
            let span = Span { lo, hi: lo };
            Err(Error::IncompleteEof(single, complete, span))
        }
    }

    fn cook(&mut self, token: &raw::Token) -> Result<Option<Token>, Error> {
        let kind = match token.kind {
            raw::TokenKind::Comment(raw::CommentKind::Normal) | raw::TokenKind::Whitespace => {
                Ok(None)
            }
            raw::TokenKind::Comment(raw::CommentKind::Doc) => Ok(Some(TokenKind::DocComment)),
            raw::TokenKind::Ident => {
                let ident = &self.input[(token.offset as usize)..(self.offset() as usize)];
                Ok(Some(self.ident(ident)))
            }
            raw::TokenKind::Number(number) => Ok(Some(number.into())),
            raw::TokenKind::Single(single) => self.single(single).map(Some),
            raw::TokenKind::String(raw::StringToken::Normal { terminated: true }) => {
                Ok(Some(TokenKind::String(StringToken::Normal)))
            }
            raw::TokenKind::String(raw::StringToken::Interpolated(start, Some(ending))) => Ok(
                Some(TokenKind::String(StringToken::Interpolated(start, ending))),
            ),
            raw::TokenKind::String(
                raw::StringToken::Normal { terminated: false }
                | raw::StringToken::Interpolated(_, None),
            ) => Err(Error::UnterminatedString(Span {
                lo: token.offset,
                hi: token.offset,
            })),
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

    #[allow(clippy::too_many_lines)]
    fn single(&mut self, single: Single) -> Result<TokenKind, Error> {
        match single {
            Single::Amp => {
                let op = ClosedBinOp::AmpAmpAmp;
                self.expect(Single::Amp, TokenKind::ClosedBinOp(op))?;
                self.expect(Single::Amp, TokenKind::ClosedBinOp(op))?;
                Ok(self.closed_bin_op(op))
            }
            Single::Apos => Ok(TokenKind::Apos),
            Single::At => Ok(TokenKind::At),
            Single::Bang => {
                if self.next_if_eq(Single::Eq) {
                    Ok(TokenKind::Ne)
                } else {
                    Ok(TokenKind::Bang)
                }
            }
            Single::Bar => {
                if self.next_if_eq(Single::Bar) {
                    let op = ClosedBinOp::BarBarBar;
                    self.expect(Single::Bar, TokenKind::ClosedBinOp(op))?;
                    Ok(self.closed_bin_op(op))
                } else {
                    Ok(TokenKind::Bar)
                }
            }
            Single::Caret => {
                if self.next_if_eq(Single::Caret) {
                    let op = ClosedBinOp::CaretCaretCaret;
                    self.expect(Single::Caret, TokenKind::ClosedBinOp(op))?;
                    Ok(self.closed_bin_op(op))
                } else {
                    Ok(self.closed_bin_op(ClosedBinOp::Caret))
                }
            }
            Single::Close(delim) => Ok(TokenKind::Close(delim)),
            Single::Colon => {
                if self.next_if_eq(Single::Colon) {
                    Ok(TokenKind::ColonColon)
                } else {
                    Ok(TokenKind::Colon)
                }
            }
            Single::Comma => Ok(TokenKind::Comma),
            Single::Dot => {
                if self.next_if_eq(Single::Dot) {
                    if self.next_if_eq(Single::Dot) {
                        Ok(TokenKind::DotDotDot)
                    } else {
                        Ok(TokenKind::DotDot)
                    }
                } else {
                    Ok(TokenKind::Dot)
                }
            }
            Single::Eq => {
                if self.next_if_eq(Single::Eq) {
                    Ok(TokenKind::EqEq)
                } else if self.next_if_eq(Single::Gt) {
                    Ok(TokenKind::FatArrow)
                } else {
                    Ok(TokenKind::Eq)
                }
            }
            Single::Gt => {
                if self.next_if_eq(Single::Eq) {
                    Ok(TokenKind::Gte)
                } else if self.next_if_eq(Single::Gt) {
                    let op = ClosedBinOp::GtGtGt;
                    self.expect(Single::Gt, TokenKind::ClosedBinOp(op))?;
                    Ok(self.closed_bin_op(op))
                } else {
                    Ok(TokenKind::Gt)
                }
            }
            Single::Lt => {
                if self.next_if_eq(Single::Eq) {
                    Ok(TokenKind::Lte)
                } else if self.next_if_eq(Single::Minus) {
                    Ok(TokenKind::LArrow)
                } else if self.next_if_eq(Single::Lt) {
                    let op = ClosedBinOp::LtLtLt;
                    self.expect(Single::Lt, TokenKind::ClosedBinOp(op))?;
                    Ok(self.closed_bin_op(op))
                } else {
                    Ok(TokenKind::Lt)
                }
            }
            Single::Minus => {
                if self.next_if_eq(Single::Gt) {
                    Ok(TokenKind::RArrow)
                } else {
                    Ok(self.closed_bin_op(ClosedBinOp::Minus))
                }
            }
            Single::Open(delim) => Ok(TokenKind::Open(delim)),
            Single::Percent => Ok(self.closed_bin_op(ClosedBinOp::Percent)),
            Single::Plus => Ok(self.closed_bin_op(ClosedBinOp::Plus)),
            Single::Question => Ok(TokenKind::Question),
            Single::Semi => Ok(TokenKind::Semi),
            Single::Slash => Ok(self.closed_bin_op(ClosedBinOp::Slash)),
            Single::Star => Ok(self.closed_bin_op(ClosedBinOp::Star)),
            Single::Tilde => {
                let complete = TokenKind::TildeTildeTilde;
                self.expect(Single::Tilde, complete)?;
                self.expect(Single::Tilde, complete)?;
                Ok(complete)
            }
        }
    }

    fn closed_bin_op(&mut self, op: ClosedBinOp) -> TokenKind {
        if self.next_if_eq(Single::Eq) {
            TokenKind::BinOpEq(op)
        } else {
            TokenKind::ClosedBinOp(op)
        }
    }

    fn ident(&mut self, ident: &str) -> TokenKind {
        match ident {
            "and" => self.closed_bin_op(ClosedBinOp::And),
            "or" => self.closed_bin_op(ClosedBinOp::Or),
            "w" if self.next_if_eq(Single::Slash) => {
                if self.next_if_eq(Single::Eq) {
                    TokenKind::WSlashEq
                } else {
                    TokenKind::WSlash
                }
            }
            ident => ident
                .parse()
                .map(TokenKind::Keyword)
                .unwrap_or(TokenKind::Ident),
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
