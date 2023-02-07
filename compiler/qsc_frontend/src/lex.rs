// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use qsc_ast::ast::{Lit, Span};
use std::{
    array,
    iter::{Chain, Enumerate, Peekable},
    str::Chars,
};

const EOF: char = '\0';

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
    Ident(String),
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
    /// `^`
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

pub(crate) struct Lexer<'a> {
    chars: Peekable<Enumerate<Chain<Chars<'a>, array::IntoIter<char, 1>>>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(s: &'a str) -> Self {
        Self {
            chars: s.chars().chain([EOF]).enumerate().peekable(),
        }
    }

    fn require(&mut self, c: char) {
        match self.chars.next() {
            Some((_, actual)) => assert!(c == actual, "Expected '{c}' but got '{actual}'."),
            None => panic!("Expected '{c}' but got EOF."),
        }
    }

    fn attempt(&mut self, c: char) -> bool {
        self.chars.next_if(|item| item.1 == c).is_some()
    }

    fn tokenize(&mut self, c: char) -> TokenKind {
        match c {
            EOF => TokenKind::Eof,
            '{' => TokenKind::OpenDelim(Delim::Brace),
            '}' => TokenKind::CloseDelim(Delim::Brace),
            '[' => TokenKind::OpenDelim(Delim::Bracket),
            ']' => TokenKind::CloseDelim(Delim::Bracket),
            '(' => TokenKind::OpenDelim(Delim::Paren),
            ')' => TokenKind::CloseDelim(Delim::Paren),
            '@' => TokenKind::At,
            '!' if self.attempt('=') => TokenKind::Ne,
            '!' => TokenKind::Bang,
            '|' => TokenKind::Bar,
            ':' if self.attempt(':') => TokenKind::ColonColon,
            ':' => TokenKind::Colon,
            ',' => TokenKind::Comma,
            '$' => {
                self.require('"');
                TokenKind::DollarQuote
            }
            '=' if self.attempt('=') => TokenKind::EqEq,
            '=' if self.attempt('>') => TokenKind::FatArrow,
            '=' => TokenKind::Eq,
            '>' if self.attempt('=') => TokenKind::Gte,
            '>' => TokenKind::Gt,
            '<' if self.attempt('-') => TokenKind::LArrow,
            '<' if self.attempt('=') => TokenKind::Lte,
            '<' => TokenKind::Lt,
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
            _ => panic!("Unexpected character: '{c}'"),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let (lo, c) = self.chars.next()?;
        let kind = self.tokenize(c);
        let lo = lo.try_into().unwrap();
        let hi = match kind {
            TokenKind::Eof => lo,
            _ => self
                .chars
                .peek()
                .expect("Non-EOF token has no following character.")
                .0
                .try_into()
                .unwrap(),
        };
        let span = Span { lo, hi };
        Some(Token { kind, span })
    }
}
