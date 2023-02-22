// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The first lexing phase transforms an input string into literals, single-character operators,
//! whitespace, and comments. Keywords are treated as identifiers. The raw token stream is
//! contiguous: there are no gaps between tokens.
//!
//! These are "raw" tokens because single-character operators don't always correspond to Q#
//! operators, and whitespace and comments will later be discarded. Raw tokens are the ingredients
//! that are "cooked" into compound tokens before they can be consumed by the parser.
//!
//! Tokens never contain substrings from the original input, but are simply labels that refer back
//! to offsets in the input. Lexing never fails, but may produce unknown tokens.

#[cfg(test)]
mod tests;

use super::Delim;
use enum_iterator::Sequence;
use std::{
    fmt::{self, Display, Formatter, Write},
    iter::Peekable,
    str::CharIndices,
};

/// A raw token.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Token {
    /// The token kind.
    pub(super) kind: TokenKind,
    /// The byte offset of the token starting character.
    pub(super) offset: usize,
}

#[derive(Debug, Eq, PartialEq, Sequence)]
pub(super) enum TokenKind {
    Comment,
    Ident,
    Number(Number),
    Single(Single),
    String,
    Unknown,
    Whitespace,
}

/// A single-character operator token.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub(super) enum Single {
    /// `&`
    Amp,
    /// `'`
    Apos,
    /// `@`
    At,
    /// `!`
    Bang,
    /// `|`
    Bar,
    /// `^`
    Caret,
    /// A closing delimiter.
    Close(Delim),
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `=`
    Eq,
    /// `>`
    Gt,
    /// `<`
    Lt,
    /// `-`
    Minus,
    /// An opening delimiter.
    Open(Delim),
    /// `%`
    Percent,
    /// `+`
    Plus,
    /// `?`
    Question,
    /// `;`
    Semi,
    /// `/`
    Slash,
    /// `*`
    Star,
    /// `~`
    Tilde,
}

impl Display for Single {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_char(match self {
            Single::Amp => '&',
            Single::Apos => '\'',
            Single::At => '@',
            Single::Bang => '!',
            Single::Bar => '|',
            Single::Caret => '^',
            Single::Close(Delim::Brace) => '}',
            Single::Close(Delim::Bracket) => ']',
            Single::Close(Delim::Paren) => ')',
            Single::Colon => ':',
            Single::Comma => ',',
            Single::Dot => '.',
            Single::Eq => '=',
            Single::Gt => '>',
            Single::Lt => '<',
            Single::Minus => '-',
            Single::Open(Delim::Brace) => '{',
            Single::Open(Delim::Bracket) => '[',
            Single::Open(Delim::Paren) => '(',
            Single::Percent => '%',
            Single::Plus => '+',
            Single::Question => '?',
            Single::Semi => ';',
            Single::Slash => '/',
            Single::Star => '*',
            Single::Tilde => '~',
        })
    }
}

#[derive(Debug, Eq, PartialEq, Sequence)]
pub(super) enum Number {
    BigInt,
    Float,
    Int,
}

pub(super) struct Lexer<'a> {
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub(super) fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
        }
    }

    fn next_if_eq(&mut self, c: char) -> bool {
        self.chars.next_if(|i| i.1 == c).is_some()
    }

    fn eat_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while self.chars.next_if(|i| f(i.1)).is_some() {}
    }

    /// Returns the first character ahead of the cursor without consuming it. This operation is fast,
    /// but if you know you want to consume the character if it matches, use [`next_if_eq`] instead.
    fn first(&mut self) -> Option<char> {
        self.chars.peek().map(|i| i.1)
    }
    
    /// Returns the second character ahead of the cursor without consuming it. This is slower
    /// than [`first`] and should be avoided when possible.
    fn second(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().map(|i| i.1)
    }

    fn whitespace(&mut self, c: char) -> bool {
        if c.is_whitespace() {
            self.eat_while(char::is_whitespace);
            true
        } else {
            false
        }
    }

    fn comment(&mut self, c: char) -> bool {
        if c == '/' && self.next_if_eq('/') {
            self.eat_while(|c| c != '\n');
            true
        } else {
            false
        }
    }

    fn ident(&mut self, c: char) -> bool {
        if c == '_' || c.is_alphabetic() {
            self.eat_while(|c| c == '_' || c.is_alphanumeric());
            true
        } else {
            false
        }
    }

    fn number(&mut self, c: char) -> Option<Number> {
        self.leading_zero(c).or_else(|| self.decimal(c))
    }

    fn leading_zero(&mut self, c: char) -> Option<Number> {
        if c != '0' {
            return None;
        }

        let radix = if self.next_if_eq('b') {
            2
        } else if self.next_if_eq('o') {
            8
        } else if self.next_if_eq('x') {
            16
        } else {
            10
        };

        self.eat_while(|c| c == '_' || c.is_digit(radix));
        if self.next_if_eq('L') {
            Some(Number::BigInt)
        } else {
            Some(Number::Int)
        }
    }

    fn decimal(&mut self, c: char) -> Option<Number> {
        if !c.is_ascii_digit() {
            return None;
        }

        self.eat_while(|c| c == '_' || c.is_ascii_digit());

        // Watch out for ranges: `0..` should be an integer followed by two dots.
        if self.first() == Some('.') && self.second() != Some('.') {
            self.chars.next();
            self.eat_while(|c| c == '_' || c.is_ascii_digit());
            self.exp();
            Some(Number::Float)
        } else if self.exp() {
            Some(Number::Float)
        } else if self.next_if_eq('L') {
            Some(Number::BigInt)
        } else {
            Some(Number::Int)
        }
    }

    fn exp(&mut self) -> bool {
        if self.next_if_eq('e') {
            self.chars.next_if(|i| i.1 == '+' || i.1 == '-');
            self.eat_while(|c| c.is_ascii_digit());
            true
        } else {
            false
        }
    }

    fn string(&mut self, c: char) -> bool {
        if c != '"' {
            return false;
        }

        while !self.next_if_eq('"') {
            self.eat_while(|c| c != '\\' && c != '"');
            if self.next_if_eq('\\') {
                self.next_if_eq('"');
            }
        }

        true
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, c) = self.chars.next()?;
        let kind = if self.comment(c) {
            TokenKind::Comment
        } else if self.whitespace(c) {
            TokenKind::Whitespace
        } else if self.ident(c) {
            TokenKind::Ident
        } else if self.string(c) {
            TokenKind::String
        } else {
            self.number(c)
                .map(TokenKind::Number)
                .or_else(|| single(c).map(TokenKind::Single))
                .unwrap_or(TokenKind::Unknown)
        };
        Some(Token { kind, offset })
    }
}

fn single(c: char) -> Option<Single> {
    match c {
        '-' => Some(Single::Minus),
        ',' => Some(Single::Comma),
        ';' => Some(Single::Semi),
        ':' => Some(Single::Colon),
        '!' => Some(Single::Bang),
        '?' => Some(Single::Question),
        '.' => Some(Single::Dot),
        '\'' => Some(Single::Apos),
        '(' => Some(Single::Open(Delim::Paren)),
        ')' => Some(Single::Close(Delim::Paren)),
        '[' => Some(Single::Open(Delim::Bracket)),
        ']' => Some(Single::Close(Delim::Bracket)),
        '{' => Some(Single::Open(Delim::Brace)),
        '}' => Some(Single::Close(Delim::Brace)),
        '@' => Some(Single::At),
        '*' => Some(Single::Star),
        '/' => Some(Single::Slash),
        '&' => Some(Single::Amp),
        '%' => Some(Single::Percent),
        '^' => Some(Single::Caret),
        '+' => Some(Single::Plus),
        '<' => Some(Single::Lt),
        '=' => Some(Single::Eq),
        '>' => Some(Single::Gt),
        '|' => Some(Single::Bar),
        '~' => Some(Single::Tilde),
        _ => None,
    }
}
