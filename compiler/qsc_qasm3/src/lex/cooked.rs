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
    str::FromStr,
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
    #[diagnostic(code("Qasm3.Lex.Incomplete"))]
    Incomplete(raw::TokenKind, TokenKind, raw::TokenKind, #[label] Span),

    #[error("expected {0} to complete {1}, found EOF")]
    #[diagnostic(code("Qasm3.Lex.IncompleteEof"))]
    IncompleteEof(raw::TokenKind, TokenKind, #[label] Span),

    #[error("unterminated string literal")]
    #[diagnostic(code("Qasm3.Lex.UnterminatedString"))]
    UnterminatedString(#[label] Span),

    #[error("unrecognized character `{0}`")]
    #[diagnostic(code("Qasm3.Lex.UnknownChar"))]
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
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TokenKind::Keyword(keyword) => write!(f, "keyword `{keyword}`"),
            TokenKind::Type(type_) => write!(f, "keyword `{type_}`"),
            TokenKind::GPhase => write!(f, "gphase"),
            TokenKind::Inv => write!(f, "inv"),
            TokenKind::Pow => write!(f, "pow"),
            TokenKind::Ctrl => write!(f, "ctrl"),
            TokenKind::NegCtrl => write!(f, "negctrl"),
            TokenKind::Dim => write!(f, "dim"),
            TokenKind::DurationOf => write!(f, "durationof"),
            TokenKind::Delay => write!(f, "delay"),
            TokenKind::Reset => write!(f, "reset"),
            TokenKind::Measure => write!(f, "measure"),
            TokenKind::Barrier => write!(f, "barrier"),
            TokenKind::Literal(literal) => write!(f, "literal `{literal}`"),
            TokenKind::Open(Delim::Brace) => write!(f, "`{{`"),
            TokenKind::Open(Delim::Bracket) => write!(f, "`[`"),
            TokenKind::Open(Delim::Paren) => write!(f, "`(`"),
            TokenKind::Close(Delim::Brace) => write!(f, "`}}`"),
            TokenKind::Close(Delim::Bracket) => write!(f, "`]`"),
            TokenKind::Close(Delim::Paren) => write!(f, "`)`"),
            TokenKind::Colon => write!(f, "`:`"),
            TokenKind::Semicolon => write!(f, "`;`"),
            TokenKind::Dot => write!(f, "`.`"),
            TokenKind::Comma => write!(f, "`,`"),
            TokenKind::PlusPlus => write!(f, "`++`"),
            TokenKind::Arrow => write!(f, "`->`"),
            TokenKind::UnaryOperator(op) => write!(f, "`{op}`"),
            TokenKind::BinaryOperator(op) => write!(f, "`{op}`"),
            TokenKind::BinaryOperatorEq(op) => write!(f, "`{op}=`"),
            TokenKind::ComparisonOperator(op) => write!(f, "`{op}`"),
            TokenKind::Eq => write!(f, "`=`"),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::HardwareQubit => write!(f, "hardware bit"),
        }
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

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Type::Input => "input",
            Type::Output => "output",
            Type::Const => "const",
            Type::Readonly => "readonly",
            Type::Mutable => "mutable",
            Type::QReg => "qreg",
            Type::Qubit => "qubit",
            Type::CReg => "creg",
            Type::Bool => "bool",
            Type::Bit => "bit",
            Type::Int => "int",
            Type::UInt => "uint",
            Type::Float => "float",
            Type::Angle => "angle",
            Type::Complex => "complex",
            Type::Array => "array",
            Type::Void => "void",
            Type::Duration => "duration",
            Type::Stretch => "stretch",
        })
    }
}

impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "input" => Ok(Type::Input),
            "output" => Ok(Type::Output),
            "const" => Ok(Type::Const),
            "readonly" => Ok(Type::Readonly),
            "mutable" => Ok(Type::Mutable),
            "qreg" => Ok(Type::QReg),
            "qubit" => Ok(Type::Qubit),
            "creg" => Ok(Type::CReg),
            "bool" => Ok(Type::Bool),
            "bit" => Ok(Type::Bit),
            "int" => Ok(Type::Int),
            "uint" => Ok(Type::UInt),
            "float" => Ok(Type::Float),
            "angle" => Ok(Type::Angle),
            "complex" => Ok(Type::Complex),
            "array" => Ok(Type::Array),
            "void" => Ok(Type::Void),
            "duration" => Ok(Type::Duration),
            "stretch" => Ok(Type::Stretch),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Literal {
    Bitstring,
    Boolean,
    Float,
    Imaginary,
    Integer(Radix),
    String,
    Timing(TimingLiteralKind),
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Literal::Bitstring => "bitstring",
            Literal::Boolean => "boolean",
            Literal::Float => "float",
            Literal::Imaginary => "imaginary",
            Literal::Integer(_) => "integer",
            Literal::String => "string",
            Literal::Timing(_) => "timing",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum TimingLiteralKind {
    /// Timing literal: Backend-dependent unit.
    /// Equivalent to the duration of one waveform sample on the backend.
    Dt,
    /// Timing literal: Nanoseconds.
    Ns,
    /// Timing literal: Microseconds.
    Us,
    /// Timing literal: Milliseconds.
    Ms,
    /// Timing literal: Seconds.
    S,
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

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            UnaryOperator::Bang => "!",
            UnaryOperator::Minus => "-",
            UnaryOperator::Tilde => "~",
        })
    }
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
    // Note: Missing Tilde according to qasm3Lexer.g4 to be able to express ~=
    //       But this is this a bug in the official qasm lexer?
}

impl Display for ClosedBinaryOperator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(match self {
            ClosedBinaryOperator::Amp => "&",
            ClosedBinaryOperator::Bar => "|",
            ClosedBinaryOperator::Caret => "^",
            ClosedBinaryOperator::GtGt => ">>",
            ClosedBinaryOperator::LtLt => "<<",
            ClosedBinaryOperator::Minus => "-",
            ClosedBinaryOperator::Percent => "%",
            ClosedBinaryOperator::Plus => "+",
            ClosedBinaryOperator::Slash => "/",
            ClosedBinaryOperator::Star => "*",
            ClosedBinaryOperator::StarStar => "**",
        })
    }
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

impl Display for ComparisonOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ComparisonOperator::AmpAmp => "&&",
            ComparisonOperator::BangEq => "!=",
            ComparisonOperator::BarBar => "||",
            ComparisonOperator::EqEq => "==",
            ComparisonOperator::Gt => ">",
            ComparisonOperator::GtEq => ">=",
            ComparisonOperator::Lt => "<",
            ComparisonOperator::LtEq => "<=",
        })
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

    /// Returns the first token ahead of the cursor without consuming it. This operation is fast,
    /// but if you know you want to consume the token if it matches, use [`next_if_eq`] instead.
    fn first(&mut self) -> Option<raw::TokenKind> {
        self.tokens.peek().map(|i| i.kind)
    }

    /// Returns the second token ahead of the cursor without consuming it. This is slower
    /// than [`first`] and should be avoided when possible.
    fn second(&self) -> Option<raw::TokenKind> {
        let mut tokens = self.tokens.clone();
        tokens.next();
        tokens.next().map(|i| i.kind)
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

    fn cook(&mut self, token: &raw::Token) -> Result<Option<Token>, Error> {
        let kind = match token.kind {
            raw::TokenKind::Bitstring { terminated: true } => {
                Ok(Some(TokenKind::Literal(Literal::Bitstring)))
            }
            raw::TokenKind::Bitstring { terminated: false } => {
                Err(Error::UnterminatedString(Span {
                    lo: token.offset,
                    hi: token.offset,
                }))
            }
            raw::TokenKind::Comment(_) | raw::TokenKind::Newline | raw::TokenKind::Whitespace => {
                Ok(None)
            }
            raw::TokenKind::Ident => {
                let ident = &self.input[(token.offset as usize)..(self.offset() as usize)];
                Ok(Some(self.ident(ident)))
            }
            raw::TokenKind::HardwareQubit => Ok(Some(TokenKind::HardwareQubit)),
            raw::TokenKind::LiteralFragment(_) => {
                // if a literal fragment does not appear after a decimal
                // or a float, treat it as an identifier.
                Ok(Some(TokenKind::Identifier))
            }
            raw::TokenKind::Number(number) => {
                // after reading a decimal number or a float there could be a whitespace
                // followed by a fragment, which will change the type of the literal.
                if let (
                    Some(raw::TokenKind::Whitespace),
                    Some(raw::TokenKind::LiteralFragment(fragment)),
                ) = (self.first(), self.second())
                {
                    use self::Literal::{Imaginary, Timing};
                    use TokenKind::Literal;
                    Ok(Some(match fragment {
                        raw::LiteralFragmentKind::Imag => Literal(Imaginary),
                        raw::LiteralFragmentKind::Dt => Literal(Timing(TimingLiteralKind::Dt)),
                        raw::LiteralFragmentKind::Ns => Literal(Timing(TimingLiteralKind::Ns)),
                        raw::LiteralFragmentKind::Us => Literal(Timing(TimingLiteralKind::Us)),
                        raw::LiteralFragmentKind::Ms => Literal(Timing(TimingLiteralKind::Ms)),
                        raw::LiteralFragmentKind::S => Literal(Timing(TimingLiteralKind::S)),
                    }))
                } else {
                    Ok(Some(number.into()))
                }
            }
            raw::TokenKind::Single(single) => self.single(single).map(Some),
            raw::TokenKind::String { terminated: true } => {
                Ok(Some(TokenKind::Literal(Literal::String)))
            }
            raw::TokenKind::String { terminated: false } => Err(Error::UnterminatedString(Span {
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
                Ok(complete)
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
            ident => {
                if let Ok(keyword) = ident.parse::<Keyword>() {
                    TokenKind::Keyword(keyword)
                } else if let Ok(type_) = ident.parse::<Type>() {
                    TokenKind::Type(type_)
                } else {
                    TokenKind::Identifier
                }
            }
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
