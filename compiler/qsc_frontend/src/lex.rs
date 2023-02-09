// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod raw;

use qsc_ast::ast::Span;
use std::{iter::Peekable, str::CharIndices};

pub(crate) struct Token {
    kind: TokenKind,
    span: Span,
}

pub(crate) struct RawToken {
    kind: RawTokenKind,
    offset: usize,
}

pub(crate) enum RawTokenKind {
    Comment,
    Cooked(TokenKind),
    Unknown,
    Whitespace,
}

pub(crate) enum TokenKind {
    /// `@`
    At,
    /// `!`
    Bang,
    /// `|`
    Bar,
    /// A closed binary operator followed by an equals sign.
    BinOpEq(ClosedBinOp),
    /// A closed binary operator.
    ClosedBinOp(ClosedBinOp),
    /// A closing delimiter.
    CloseDelim(Delim),
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `,`
    Comma,
    /// `$"`
    DollarQuote,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `...`
    DotDotDot,
    // End of file.
    Eof,
    /// `=`
    Eq,
    /// `==`
    EqEq,
    /// `=>`
    FatArrow,
    /// `>`
    Gt,
    /// `>=`
    Gte,
    /// An identifier.
    Ident,
    /// `<-`
    LArrow,
    /// A literal.
    Lit(Lit),
    /// `<`
    Lt,
    /// `<=`
    Lte,
    /// `!=`
    Ne,
    /// An opening delimiter.
    OpenDelim(Delim),
    /// `?`
    Question,
    /// `->`
    RArrow,
    /// `;`
    Semi,
    /// `'`
    SingleQuote,
    /// `~~~`
    TildeTildeTilde,
    /// `w/`
    WSlash,
    /// `w/=`
    WSlashEq,
}

/// Binary operators whose input type is closed under the operation. These are the only binary
/// operators that can be used in compound assignment, like `set x += y`.
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

pub(crate) enum Delim {
    /// `{` `}`
    Brace,
    /// `[` `]`
    Bracket,
    /// `(` `)`
    Paren,
}

pub(crate) enum Lit {
    BigInt,
    Float,
    Int,
    String,
}

pub(crate) struct Lexer<'a> {
    input: &'a str,
    chars: Peekable<CharIndices<'a>>,
    eof: bool,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
            eof: false,
        }
    }

    fn require(&mut self, c: char) {
        match self.chars.next() {
            Some((_, actual)) => assert!(c == actual, "Expected '{c}' but got '{actual}'."),
            None => panic!("Expected '{c}' but got EOF."),
        }
    }

    fn attempt(&mut self, c: char) -> bool {
        self.chars.next_if(|(_, next_c)| *next_c == c).is_some()
    }

    fn followed_by(&mut self, c: char) -> bool {
        self.chars.peek().iter().any(|(_, next_c)| *next_c == c)
    }

    fn followed_by_match(&mut self, f: impl FnOnce(char) -> bool) -> bool {
        if let Some(&(_, c)) = self.chars.peek() {
            f(c)
        } else {
            false
        }
    }

    fn take_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while self.chars.next_if(|(_, c)| f(*c)).is_some() {}
    }

    fn offset(&mut self) -> usize {
        self.chars
            .peek()
            .map_or_else(|| self.input.len(), |(offset, _)| *offset)
    }

    fn tokenize(&mut self, c: char) -> RawTokenKind {
        match c {
            '/' if self.attempt('/') => {
                self.take_while(|c| c != '\n');
                RawTokenKind::Comment
            }
            '{' => RawTokenKind::Cooked(TokenKind::OpenDelim(Delim::Brace)),
            '}' => RawTokenKind::Cooked(TokenKind::CloseDelim(Delim::Brace)),
            '[' => RawTokenKind::Cooked(TokenKind::OpenDelim(Delim::Bracket)),
            ']' => RawTokenKind::Cooked(TokenKind::CloseDelim(Delim::Bracket)),
            '(' => RawTokenKind::Cooked(TokenKind::OpenDelim(Delim::Paren)),
            ')' => RawTokenKind::Cooked(TokenKind::CloseDelim(Delim::Paren)),
            '@' => RawTokenKind::Cooked(TokenKind::At),
            '!' if self.attempt('=') => RawTokenKind::Cooked(TokenKind::Ne),
            '!' => RawTokenKind::Cooked(TokenKind::Bang),
            '|' if !self.followed_by('|') => RawTokenKind::Cooked(TokenKind::Bar),
            ':' if self.attempt(':') => RawTokenKind::Cooked(TokenKind::ColonColon),
            ':' => RawTokenKind::Cooked(TokenKind::Colon),
            ',' => RawTokenKind::Cooked(TokenKind::Comma),
            '$' => {
                self.require('"');
                RawTokenKind::Cooked(TokenKind::DollarQuote)
            }
            '.' if !self.followed_by_match(|c| c.is_ascii_digit()) => {
                if self.attempt('.') {
                    if self.attempt('.') {
                        RawTokenKind::Cooked(TokenKind::DotDotDot)
                    } else {
                        RawTokenKind::Cooked(TokenKind::DotDot)
                    }
                } else {
                    RawTokenKind::Cooked(TokenKind::Dot)
                }
            }
            '=' if self.attempt('=') => RawTokenKind::Cooked(TokenKind::EqEq),
            '=' if self.attempt('>') => RawTokenKind::Cooked(TokenKind::FatArrow),
            '=' => RawTokenKind::Cooked(TokenKind::Eq),
            '>' if self.attempt('=') => RawTokenKind::Cooked(TokenKind::Gte),
            '>' if !self.followed_by('>') => RawTokenKind::Cooked(TokenKind::Gt),
            '<' if self.attempt('-') => RawTokenKind::Cooked(TokenKind::LArrow),
            '<' if self.attempt('=') => RawTokenKind::Cooked(TokenKind::Lte),
            '<' if !self.followed_by('<') => RawTokenKind::Cooked(TokenKind::Lt),
            '?' => RawTokenKind::Cooked(TokenKind::Question),
            '-' if self.attempt('>') => RawTokenKind::Cooked(TokenKind::RArrow),
            ';' => RawTokenKind::Cooked(TokenKind::Semi),
            '\'' => RawTokenKind::Cooked(TokenKind::SingleQuote),
            '~' => {
                self.require('~');
                self.require('~');
                RawTokenKind::Cooked(TokenKind::TildeTildeTilde)
            }
            'w' if self.attempt('/') => {
                if self.attempt('=') {
                    RawTokenKind::Cooked(TokenKind::WSlashEq)
                } else {
                    RawTokenKind::Cooked(TokenKind::WSlash)
                }
            }
            _ if c.is_whitespace() => {
                self.take_while(char::is_whitespace);
                RawTokenKind::Whitespace
            }
            _ => self
                .number(c)
                .map(TokenKind::Lit)
                .or_else(|| self.closed_bin_op(c))
                .or_else(|| self.ident(c).then_some(TokenKind::Ident))
                .map_or(RawTokenKind::Unknown, RawTokenKind::Cooked),
        }
    }

    fn closed_bin_op(&mut self, c: char) -> Option<TokenKind> {
        let op = match c {
            '&' => {
                self.require('&');
                self.require('&');
                Some(ClosedBinOp::AmpAmpAmp)
            }
            '|' => {
                self.require('|');
                self.require('|');
                Some(ClosedBinOp::BarBarBar)
            }
            '^' => {
                if self.attempt('^') {
                    self.require('^');
                    Some(ClosedBinOp::CaretCaretCaret)
                } else {
                    Some(ClosedBinOp::Caret)
                }
            }
            '>' => {
                self.require('>');
                self.require('>');
                Some(ClosedBinOp::GtGtGt)
            }
            '<' => {
                self.require('<');
                self.require('<');
                Some(ClosedBinOp::LtLtLt)
            }
            '-' => Some(ClosedBinOp::Minus),
            '%' => Some(ClosedBinOp::Percent),
            '+' => Some(ClosedBinOp::Plus),
            '/' => Some(ClosedBinOp::Slash),
            '*' => Some(ClosedBinOp::Star),
            _ => None,
        }?;

        if self.attempt('=') {
            Some(TokenKind::BinOpEq(op))
        } else {
            Some(TokenKind::ClosedBinOp(op))
        }
    }

    fn ident(&mut self, c: char) -> bool {
        if c == '_' || c.is_alphabetic() {
            self.take_while(|c| c == '_' || c.is_alphanumeric());
            true
        } else {
            false
        }
    }

    fn digits(&mut self, c: char) -> bool {
        if c.is_ascii_digit() {
            self.take_while(|c| c == '_' || c.is_ascii_digit());
            true
        } else {
            false
        }
    }

    fn number(&mut self, c: char) -> Option<Lit> {
        let c = if c == '+' || c == '-' && self.followed_by_match(|c| c.is_ascii_digit()) {
            self.chars.next().unwrap().1
        } else {
            c
        };

        match c {
            '0' if self.followed_by('b') => {
                self.take_while(|c| c == '_' || c.is_digit(2));
                Some(self.try_int_suffix())
            }
            '0' if self.followed_by('o') => {
                self.take_while(|c| c == '_' || c.is_digit(8));
                Some(self.try_int_suffix())
            }
            '0' if self.followed_by('x') => {
                self.take_while(|c| c == '_' || c.is_ascii_hexdigit());
                Some(self.try_int_suffix())
            }
            '.' => {
                let (_, c) = self.chars.next().unwrap();
                assert!(self.digits(c), "Expected digit.");
                self.try_float_exp();
                Some(Lit::Float)
            }
            _ if self.digits(c) => {
                if self.attempt('.') {
                    let (_, c) = self.chars.next().unwrap();
                    assert!(self.digits(c), "Expected digit.");
                    self.try_float_exp();
                    Some(Lit::Float)
                } else if self.try_float_exp() {
                    Some(Lit::Float)
                } else {
                    Some(self.try_int_suffix())
                }
            }
            _ => None,
        }
    }

    fn try_float_exp(&mut self) -> bool {
        if self.attempt('e') {
            self.chars.next_if(|&(_, c)| c == '+' || c == '-');
            let (_, c) = self.chars.next().unwrap();
            assert!(self.digits(c), "Expected digit.");
            true
        } else {
            false
        }
    }

    fn try_int_suffix(&mut self) -> Lit {
        if self.attempt('L') {
            Lit::BigInt
        } else {
            Lit::Int
        }
    }

    fn and_or_hack(&mut self, mut token: RawToken) -> RawToken {
        if let RawTokenKind::Cooked(TokenKind::Ident) = token.kind {
            let sym = &self.input[token.offset..self.offset()];
            match sym {
                "and" if self.attempt('=') => {
                    token.kind = RawTokenKind::Cooked(TokenKind::BinOpEq(ClosedBinOp::And));
                }
                "and" => {
                    token.kind = RawTokenKind::Cooked(TokenKind::ClosedBinOp(ClosedBinOp::And));
                }
                "or" if self.attempt('=') => {
                    token.kind = RawTokenKind::Cooked(TokenKind::BinOpEq(ClosedBinOp::Or));
                }
                "or" => token.kind = RawTokenKind::Cooked(TokenKind::ClosedBinOp(ClosedBinOp::Or)),
                _ => {}
            }
        }

        token
    }
}

impl Iterator for Lexer<'_> {
    type Item = RawToken;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.next() {
            Some((offset, c)) => {
                let kind = self.tokenize(c);
                Some(self.and_or_hack(RawToken { kind, offset }))
            }
            None if self.eof => None,
            None => {
                self.eof = true;
                Some(RawToken {
                    kind: RawTokenKind::Cooked(TokenKind::Eof),
                    offset: self.input.len(),
                })
            }
        }
    }
}
