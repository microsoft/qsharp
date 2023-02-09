// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{iter::Peekable, str::CharIndices};

pub(crate) struct Token {
    kind: TokenKind,
    offset: usize,
}

pub(crate) enum TokenKind {
    Comment,
    Ident,
    Number(Number),
    Single(Single),
    String,
    Unknown,
    Whitespace,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Single {
    Amp,
    Apos,
    At,
    Bang,
    Bar,
    Caret,
    Close(Delim),
    Colon,
    Comma,
    Dollar,
    Dot,
    Eq,
    Gt,
    Lt,
    Minus,
    Open(Delim),
    Percent,
    Plus,
    Question,
    Semi,
    Slash,
    Star,
    Tilde,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Delim {
    Brace,
    Bracket,
    Paren,
}

pub(crate) enum Number {
    BigInt,
    Float,
    Int,
}

pub(crate) struct Lexer<'a> {
    chars: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
        }
    }

    fn next_if_eq(&mut self, c: char) -> bool {
        self.chars.next_if(|i| i.1 == c).is_some()
    }

    fn eat_while(&mut self, mut f: impl FnMut(char) -> bool) -> bool {
        let any = self.chars.next_if(|i| f(i.1)).is_some();
        while self.chars.next_if(|i| f(i.1)).is_some() {}
        any
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
        if let Some(n) = self.leading_zero(c) {
            Some(n)
        } else if self.leading_point(c) {
            Some(Number::Float)
        } else {
            self.decimal(c)
        }
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

    fn leading_point(&mut self, c: char) -> bool {
        if c == '.' && self.chars.next_if(|i| i.1.is_ascii_digit()).is_some() {
            self.eat_while(|c| c == '_' || c.is_ascii_digit());
            self.exp();
            true
        } else {
            false
        }
    }

    fn decimal(&mut self, c: char) -> Option<Number> {
        if !c.is_ascii_digit() {
            return None;
        }

        self.eat_while(|c| c == '_' || c.is_ascii_digit());
        let Some((_, c)) = self.chars.next() else { return Some(Number::Int) };

        if c == '.' {
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
        } else if self.eat_while(char::is_whitespace) {
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
        '$' => Some(Single::Dollar),
        _ => None,
    }
}
