// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! The first lexing phase transforms an input string into literals, single-character operators,
//! whitespace, and comments. Keywords are treated as identifiers. The raw token stream is
//! contiguous: there are no gaps between tokens.
//!
//! These are "raw" tokens because single-character operators don't always correspond to `OpenQASM`
//! operators, and whitespace and comments will later be discarded. Raw tokens are the ingredients
//! that are "cooked" into compound tokens before they can be consumed by the parser.
//!
//! Tokens never contain substrings from the original input, but are simply labels that refer back
//! to offsets in the input. Lexing never fails, but may produce unknown tokens.

#[cfg(test)]
mod tests;

use super::{Delim, Radix};
use enum_iterator::Sequence;
use std::{
    fmt::{self, Display, Formatter, Write},
    iter::Peekable,
    str::CharIndices,
};

/// A raw token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    /// The token kind.
    pub kind: TokenKind,
    /// The byte offset of the token starting character.
    pub offset: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum TokenKind {
    Bitstring { terminated: bool },
    Comment(CommentKind),
    HardwareQubit,
    Ident,
    LiteralFragment(LiteralFragmentKind),
    Newline,
    Number(Number),
    Single(Single),
    String { terminated: bool },
    Unknown,
    Whitespace,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TokenKind::Bitstring { .. } => f.write_str("bitstring"),
            TokenKind::Comment(CommentKind::Block) => f.write_str("block comment"),
            TokenKind::Comment(CommentKind::Normal) => f.write_str("comment"),
            TokenKind::HardwareQubit => f.write_str("hardware qubit"),
            TokenKind::Ident => f.write_str("identifier"),
            TokenKind::LiteralFragment(_) => f.write_str("literal fragment"),
            TokenKind::Newline => f.write_str("newline"),
            TokenKind::Number(Number::Float) => f.write_str("float"),
            TokenKind::Number(Number::Int(_)) => f.write_str("integer"),
            TokenKind::Single(single) => write!(f, "`{single}`"),
            TokenKind::String { .. } => f.write_str("string"),
            TokenKind::Unknown => f.write_str("unknown"),
            TokenKind::Whitespace => f.write_str("whitespace"),
        }
    }
}

/// A single-character operator token.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Single {
    /// `&`
    Amp,
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
            Single::Semi => ';',
            Single::Slash => '/',
            Single::Star => '*',
            Single::Tilde => '~',
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum Number {
    Float,
    Int(Radix),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub struct StringToken {
    pub terminated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum CommentKind {
    Block,
    Normal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Sequence)]
pub enum LiteralFragmentKind {
    /// Imaginary literal fragment.
    Imag,
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

#[derive(Clone)]
pub struct Lexer<'a> {
    chars: Peekable<CharIndices<'a>>,
    starting_offset: u32,
}

impl<'a> Lexer<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            starting_offset: 0,
        }
    }

    #[must_use]
    pub fn new_with_starting_offset(input: &'a str, starting_offset: u32) -> Self {
        Self {
            chars: input.char_indices().peekable(),
            starting_offset,
        }
    }

    fn next_if_eq(&mut self, c: char) -> bool {
        self.chars.next_if(|i| i.1 == c).is_some()
    }

    /// Consumes the characters while they satisfy `f`. Returns the last character eaten, if any.
    fn eat_while(&mut self, mut f: impl FnMut(char) -> bool) -> Option<char> {
        let mut last_eaten = None;
        loop {
            let c = self.chars.next_if(|i| f(i.1));
            if c.is_none() {
                return last_eaten.map(|(_, c)| c);
            }
            last_eaten = c;
        }
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

    fn newline(&mut self, c: char) -> bool {
        if is_newline(c) {
            self.eat_while(is_newline);
            true
        } else {
            false
        }
    }

    fn whitespace(&mut self, c: char) -> bool {
        if is_whitespace(c) {
            self.eat_while(is_whitespace);
            true
        } else {
            false
        }
    }

    fn comment(&mut self, c: char) -> Option<CommentKind> {
        if c == '/' && self.next_if_eq('/') {
            self.eat_while(|c| !is_newline(c));
            Some(CommentKind::Normal)
        } else if c == '/' && self.next_if_eq('*') {
            loop {
                let (_, c) = self.chars.next()?;
                if c == '*' && self.next_if_eq('/') {
                    return Some(CommentKind::Block);
                }
            }
        } else {
            None
        }
    }

    fn ident(&mut self, c: char) -> Option<TokenKind> {
        let first = self.first();
        let second = self.second();

        // Check for some special literal fragments.
        if c == 's'
            && (first.is_none() || first.is_some_and(|c1| c1 != '_' && !c1.is_alphanumeric()))
        {
            return Some(TokenKind::LiteralFragment(LiteralFragmentKind::S));
        }

        if let Some(c1) = first {
            if second.is_none() || second.is_some_and(|c1| c1 != '_' && !c1.is_alphanumeric()) {
                let fragment = match (c, c1) {
                    ('i', 'm') => Some(TokenKind::LiteralFragment(LiteralFragmentKind::Imag)),
                    ('d', 't') => Some(TokenKind::LiteralFragment(LiteralFragmentKind::Dt)),
                    ('n', 's') => Some(TokenKind::LiteralFragment(LiteralFragmentKind::Ns)),
                    ('u' | 'µ', 's') => Some(TokenKind::LiteralFragment(LiteralFragmentKind::Us)),
                    ('m', 's') => Some(TokenKind::LiteralFragment(LiteralFragmentKind::Ms)),
                    _ => None,
                };

                if fragment.is_some() {
                    // consume `first` before returning.
                    self.next();
                    return fragment;
                }
            }
        }

        if c == '_' || c.is_alphabetic() {
            self.eat_while(|c| c == '_' || c.is_alphanumeric());
            Some(TokenKind::Ident)
        } else {
            None
        }
    }

    fn number(&mut self, c: char) -> Option<Number> {
        self.leading_zero(c).or_else(|| self.decimal(c))
    }

    fn leading_dot(&mut self, c: char) -> bool {
        if c == '.' && self.first().is_some_and(|c| char::is_ascii_digit(&c)) {
            self.next();
            self.eat_while(|c| c == '_' || c.is_ascii_digit());
            self.exp();
            true
        } else {
            false
        }
    }

    fn leading_zero(&mut self, c: char) -> Option<Number> {
        if c != '0' {
            return None;
        }

        let radix = if self.next_if_eq('b') || self.next_if_eq('B') {
            Radix::Binary
        } else if self.next_if_eq('o') || self.next_if_eq('O') {
            Radix::Octal
        } else if self.next_if_eq('x') || self.next_if_eq('X') {
            Radix::Hexadecimal
        } else {
            Radix::Decimal
        };

        self.eat_while(|c| c == '_' || c.is_digit(radix.into()));
        if radix == Radix::Decimal && self.float() {
            Some(Number::Float)
        } else {
            Some(Number::Int(radix))
        }
    }

    fn decimal(&mut self, c: char) -> Option<Number> {
        if !c.is_ascii_digit() {
            return None;
        }

        self.eat_while(|c| c == '_' || c.is_ascii_digit());

        if self.float() {
            Some(Number::Float)
        } else {
            Some(Number::Int(Radix::Decimal))
        }
    }

    fn float(&mut self) -> bool {
        // Watch out for ranges: `0..` should be an integer followed by two dots.
        if self.first() == Some('.') {
            self.chars.next();
            self.eat_while(|c| c == '_' || c.is_ascii_digit());
            self.exp();
            true
        } else {
            self.exp()
        }
    }

    fn exp(&mut self) -> bool {
        if self.next_if_eq('e') || self.next_if_eq('E') {
            self.chars.next_if(|i| i.1 == '+' || i.1 == '-');
            self.eat_while(|c| c.is_ascii_digit());
            true
        } else {
            false
        }
    }

    fn string(&mut self, string_start: char) -> Option<TokenKind> {
        if string_start != '"' && string_start != '\'' {
            return None;
        }

        if let Some(bitstring) = self.bitstring() {
            // consume the closing '"'
            self.next();
            return Some(bitstring);
        }

        while self.first().is_some_and(|c| c != string_start) {
            self.eat_while(|c| c != '\\' && c != string_start);
            if self.next_if_eq('\\') {
                self.chars.next();
            }
        }

        Some(TokenKind::String {
            terminated: self.next_if_eq(string_start),
        })
    }

    fn bitstring(&mut self) -> Option<TokenKind> {
        const STRING_START: char = '"';

        // A bitstring must have at least one character.
        if matches!(self.first(), None | Some(STRING_START)) {
            return None;
        }

        // A bitstring must end in a 0 or a 1.
        if let Some('_') = self.eat_while(is_bitstring_char) {
            return None;
        }

        match self.first() {
            None => Some(TokenKind::Bitstring { terminated: false }),
            Some(STRING_START) => Some(TokenKind::Bitstring { terminated: true }),
            _ => None,
        }
    }

    fn hardware_qubit(&mut self, c: char) -> bool {
        if c == '$' {
            self.eat_while(|c| c.is_ascii_digit());
            true
        } else {
            false
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let (offset, c) = self.chars.next()?;
        let kind = if let Some(kind) = self.comment(c) {
            TokenKind::Comment(kind)
        } else if self.whitespace(c) {
            TokenKind::Whitespace
        } else if self.newline(c) {
            TokenKind::Newline
        } else if let Some(ident) = self.ident(c) {
            ident
        } else if self.hardware_qubit(c) {
            TokenKind::HardwareQubit
        } else if self.leading_dot(c) {
            TokenKind::Number(Number::Float)
        } else {
            self.number(c)
                .map(TokenKind::Number)
                .or_else(|| self.string(c))
                .or_else(|| single(c).map(TokenKind::Single))
                .unwrap_or(TokenKind::Unknown)
        };
        let offset: u32 = offset.try_into().expect("offset should fit into u32");
        Some(Token {
            kind,
            offset: offset + self.starting_offset,
        })
    }
}

fn single(c: char) -> Option<Single> {
    match c {
        '-' => Some(Single::Minus),
        ',' => Some(Single::Comma),
        ';' => Some(Single::Semi),
        ':' => Some(Single::Colon),
        '!' => Some(Single::Bang),
        '.' => Some(Single::Dot),
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

fn is_bitstring_char(c: char) -> bool {
    c == '0' || c == '1' || c == '_'
}

fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn is_whitespace(c: char) -> bool {
    !is_newline(c) && c.is_whitespace()
}
