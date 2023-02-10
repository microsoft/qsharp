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
    /// `$`
    Dollar,
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
            Single::Dollar => '$',
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
        if self.next_if_eq('.') {
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
        '$' => Some(Single::Dollar),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use crate::lex::raw::{Single, Token, TokenKind};
    use expect_test::{expect, Expect};

    fn check(input: &str, expect: &Expect) {
        let actual: Vec<_> = Lexer::new(input).collect();
        expect.assert_debug_eq(&actual);
    }

    #[test]
    fn singles() {
        for single in enum_iterator::all::<Single>() {
            let actual: Vec<_> = Lexer::new(&single.to_string()).collect();
            let kind = TokenKind::Single(single);
            assert_eq!(actual, vec![Token { kind, offset: 0 }]);
        }
    }

    #[test]
    fn braces() {
        check(
            "{}",
            &expect![[r#"
                [
                    Token {
                        kind: Single(
                            Open(
                                Brace,
                            ),
                        ),
                        offset: 0,
                    },
                    Token {
                        kind: Single(
                            Close(
                                Brace,
                            ),
                        ),
                        offset: 1,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn negate() {
        check(
            "-x",
            &expect![[r#"
                [
                    Token {
                        kind: Single(
                            Minus,
                        ),
                        offset: 0,
                    },
                    Token {
                        kind: Ident,
                        offset: 1,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn whitespace() {
        check(
            "-   x",
            &expect![[r#"
                [
                    Token {
                        kind: Single(
                            Minus,
                        ),
                        offset: 0,
                    },
                    Token {
                        kind: Whitespace,
                        offset: 1,
                    },
                    Token {
                        kind: Ident,
                        offset: 4,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn comment() {
        check(
            "//comment\nx",
            &expect![[r#"
                [
                    Token {
                        kind: Comment,
                        offset: 0,
                    },
                    Token {
                        kind: Whitespace,
                        offset: 9,
                    },
                    Token {
                        kind: Ident,
                        offset: 10,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn string() {
        check(
            r#""string""#,
            &expect![[r#"
                [
                    Token {
                        kind: String,
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn string_escape_quote() {
        check(
            r#""str\"ing""#,
            &expect![[r#"
                [
                    Token {
                        kind: String,
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn binary() {
        check(
            "0b10110",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn octal() {
        check(
            "0o70351",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn decimal() {
        check(
            "123",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn number_seps() {
        check(
            "123_456",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn number_underscore_prefix() {
        check(
            "_123_456",
            &expect![[r#"
                [
                    Token {
                        kind: Ident,
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn hexadecimal() {
        check(
            "0x123abc",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn bigint() {
        check(
            "123L",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            BigInt,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn negative() {
        check(
            "-4",
            &expect![[r#"
                [
                    Token {
                        kind: Single(
                            Minus,
                        ),
                        offset: 0,
                    },
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 1,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn bigint_hexadecimal() {
        check(
            "0x123abcL",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            BigInt,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn float() {
        check(
            "1.23",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Float,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn leading_zero() {
        check(
            "0123",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn leading_point() {
        check(
            ".123",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Float,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn exp() {
        check(
            "1e23",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Float,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn exp_plus() {
        check(
            "1e+23",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Float,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn exp_minus() {
        check(
            "1e-23",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Float,
                        ),
                        offset: 0,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn unknown() {
        check(
            "##",
            &expect![[r#"
                [
                    Token {
                        kind: Unknown,
                        offset: 0,
                    },
                    Token {
                        kind: Unknown,
                        offset: 1,
                    },
                ]
            "#]],
        );
    }

    #[test]
    fn float_hexadecimal() {
        check(
            "0x123.45",
            &expect![[r#"
                [
                    Token {
                        kind: Number(
                            Int,
                        ),
                        offset: 0,
                    },
                    Token {
                        kind: Number(
                            Float,
                        ),
                        offset: 5,
                    },
                ]
            "#]],
        );
    }
}
