// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::Span;
use std::{iter::Peekable, str::CharIndices};

pub(crate) struct Token {
    kind: TokenKind,
    span: Span,
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

    fn tokenize(&mut self, c: char) -> TokenKind {
        match c {
            '{' => TokenKind::OpenDelim(Delim::Brace),
            '}' => TokenKind::CloseDelim(Delim::Brace),
            '[' => TokenKind::OpenDelim(Delim::Bracket),
            ']' => TokenKind::CloseDelim(Delim::Bracket),
            '(' => TokenKind::OpenDelim(Delim::Paren),
            ')' => TokenKind::CloseDelim(Delim::Paren),
            '@' => TokenKind::At,
            '!' if self.attempt('=') => TokenKind::Ne,
            '!' => TokenKind::Bang,
            '|' if !self.followed_by('|') => TokenKind::Bar,
            ':' if self.attempt(':') => TokenKind::ColonColon,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '$' => {
                self.require('"');
                TokenKind::DollarQuote
            }
            '.' if !self.followed_by_match(|c| c.is_ascii_digit()) => {
                if self.attempt('.') {
                    if self.attempt('.') {
                        TokenKind::DotDotDot
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '=' if self.attempt('=') => TokenKind::EqEq,
            '=' if self.attempt('>') => TokenKind::FatArrow,
            '=' => TokenKind::Eq,
            '>' if self.attempt('=') => TokenKind::Gte,
            '>' if !self.followed_by('>') => TokenKind::Gt,
            '<' if self.attempt('-') => TokenKind::LArrow,
            '<' if self.attempt('=') => TokenKind::Lte,
            '<' if !self.followed_by('<') => TokenKind::Lt,
            '?' => TokenKind::Question,
            '-' if self.attempt('>') => TokenKind::RArrow,
            ';' => TokenKind::Semi,
            '\'' => TokenKind::SingleQuote,
            '~' => {
                self.require('~');
                self.require('~');
                TokenKind::TildeTildeTilde
            }
            'w' if self.attempt('/') => {
                if self.attempt('=') {
                    TokenKind::WSlashEq
                } else {
                    TokenKind::WSlash
                }
            }
            _ => self
                .number(c)
                .map(TokenKind::Lit)
                .or_else(|| self.closed_bin_op(c))
                .or_else(|| self.ident(c).then_some(TokenKind::Ident))
                .unwrap_or_else(|| panic!("Unexpected character: '{c}'")),
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

    fn and_or_hack(&mut self, mut token: Token) -> Token {
        if let TokenKind::Ident = token.kind {
            let lo = token.span.lo.try_into().unwrap();
            let hi = token.span.hi.try_into().unwrap();
            let sym = &self.input[lo..hi];
            match sym {
                "and" if self.attempt('=') => {
                    token.kind = TokenKind::BinOpEq(ClosedBinOp::And);
                    token.span.hi += 1;
                }
                "and" => token.kind = TokenKind::ClosedBinOp(ClosedBinOp::And),
                "or" if self.attempt('=') => {
                    token.kind = TokenKind::BinOpEq(ClosedBinOp::Or);
                    token.span.hi += 1;
                }
                "or" => token.kind = TokenKind::ClosedBinOp(ClosedBinOp::Or),
                _ => {}
            }
        }

        token
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.next() {
            Some((lo, c)) => {
                let kind = self.tokenize(c);
                let lo = lo.try_into().unwrap();
                let hi = self.offset().try_into().unwrap();
                let span = Span { lo, hi };
                Some(self.and_or_hack(Token { kind, span }))
            }
            None if self.eof => None,
            None => {
                self.eof = true;
                let len = self.input.len().try_into().unwrap();
                Some(Token {
                    kind: TokenKind::Eof,
                    span: Span { lo: len, hi: len },
                })
            }
        }
    }
}
