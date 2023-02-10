// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    raw::{self, Single},
    Delim,
};
use qsc_ast::ast::Span;
use std::iter::Peekable;

// TODO: This will be used via the parser.
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Token {
    kind: TokenKind,
    span: Span,
}

// TODO: This will be used via the parser.
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Error {
    message: &'static str,
    span: Span,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Apos,
    At,
    Bang,
    Bar,
    BigInt,
    BinOpEq(ClosedBinOp),
    Close(Delim),
    ClosedBinOp(ClosedBinOp),
    Colon,
    ColonColon,
    Comma,
    Dollar,
    Dot,
    DotDot,
    DotDotDot,
    Eof,
    Eq,
    EqEq,
    FatArrow,
    Float,
    Gt,
    Gte,
    Ident,
    Int,
    LArrow,
    Lt,
    Lte,
    Ne,
    Open(Delim),
    Question,
    RArrow,
    Semi,
    String,
    TildeTildeTilde,
    WSlash,
    WSlashEq,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ClosedBinOp {
    AmpAmpAmp,
    And,
    BarBarBar,
    Caret,
    CaretCaretCaret,
    GtGtGt,
    LtLtLt,
    Minus,
    Or,
    Percent,
    Plus,
    Slash,
    Star,
}

pub(crate) struct Lexer<'a> {
    tokens: Peekable<raw::Lexer<'a>>,
    input: &'a str,
    eof: bool,
}

impl<'a> Lexer<'a> {
    // TODO: This will be used via the parser.
    #[allow(dead_code)]
    fn new(input: &'a str) -> Self {
        Self {
            tokens: raw::Lexer::new(input).peekable(),
            input,
            eof: false,
        }
    }

    fn offset(&mut self) -> usize {
        self.tokens
            .peek()
            .map_or_else(|| self.input.len(), |t| t.offset)
    }

    fn next_if_eq(&mut self, single: Single) -> bool {
        self.tokens
            .next_if(|t| t.kind == raw::TokenKind::Single(single))
            .is_some()
    }

    fn expect(&mut self, single: Single, err: &'static str) -> Result<(), &'static str> {
        if self.next_if_eq(single) {
            Ok(())
        } else {
            Err(err)
        }
    }

    fn cook(&mut self, token: &raw::Token) -> Result<Option<Token>, Error> {
        let lo = token.offset.try_into().unwrap();
        let kind = match token.kind {
            raw::TokenKind::Comment | raw::TokenKind::Whitespace => Ok(None),
            raw::TokenKind::Ident => {
                let ident = &self.input[token.offset..self.offset()];
                Ok(Some(self.ident(ident)))
            }
            raw::TokenKind::Number(raw::Number::BigInt) => Ok(Some(TokenKind::BigInt)),
            raw::TokenKind::Number(raw::Number::Float) => Ok(Some(TokenKind::Float)),
            raw::TokenKind::Number(raw::Number::Int) => Ok(Some(TokenKind::Int)),
            raw::TokenKind::Single(single) => self.single(single).map(Some),
            raw::TokenKind::String => Ok(Some(TokenKind::String)),
            raw::TokenKind::Unknown => Err("Unknown token."),
        };

        let span = Span {
            lo,
            hi: self.offset().try_into().unwrap(),
        };

        match kind {
            Ok(None) => Ok(None),
            Ok(Some(kind)) => Ok(Some(Token { kind, span })),
            Err(message) => Err(Error { message, span }),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn single(&mut self, single: Single) -> Result<TokenKind, &'static str> {
        match single {
            Single::Amp => {
                self.expect(Single::Amp, "Expecting `&&&`.")?;
                self.expect(Single::Amp, "Expecting `&&&`.")?;
                Ok(self.closed_bin_op(ClosedBinOp::AmpAmpAmp))
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
                    self.expect(Single::Bar, "Expecting `|||`.")?;
                    Ok(self.closed_bin_op(ClosedBinOp::BarBarBar))
                } else {
                    Ok(TokenKind::Bar)
                }
            }
            Single::Caret => {
                if self.next_if_eq(Single::Caret) {
                    self.expect(Single::Caret, "Expecting `^^^`.")?;
                    Ok(self.closed_bin_op(ClosedBinOp::CaretCaretCaret))
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
            Single::Dollar => Ok(TokenKind::Dollar),
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
                    self.expect(Single::Gt, "Expecting `>>>`.")?;
                    Ok(self.closed_bin_op(ClosedBinOp::GtGtGt))
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
                    self.expect(Single::Lt, "Expecting `<<<`.")?;
                    Ok(self.closed_bin_op(ClosedBinOp::LtLtLt))
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
                self.expect(Single::Tilde, "Expecting `~~~`.")?;
                self.expect(Single::Tilde, "Expecting `~~~`.")?;
                Ok(TokenKind::TildeTildeTilde)
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
            _ => TokenKind::Ident,
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

        if self.eof {
            None
        } else {
            self.eof = true;
            let offset = self.offset().try_into().unwrap();
            Some(Ok(Token {
                kind: TokenKind::Eof,
                span: Span {
                    lo: offset,
                    hi: offset,
                },
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClosedBinOp, Lexer, Token, TokenKind};
    use crate::lex::Delim;
    use expect_test::{expect, Expect};
    use qsc_ast::ast::Span;

    fn check(input: &str, expect: &Expect) {
        let actual: Vec<_> = Lexer::new(input).collect();
        expect.assert_debug_eq(&actual);
    }

    #[test]
    fn basic() {
        let cases = [
            ("'", TokenKind::Apos),
            ("@", TokenKind::At),
            ("!", TokenKind::Bang),
            ("|", TokenKind::Bar),
            (":", TokenKind::Colon),
            ("::", TokenKind::ColonColon),
            (",", TokenKind::Comma),
            ("$", TokenKind::Dollar),
            (".", TokenKind::Dot),
            ("..", TokenKind::DotDot),
            ("...", TokenKind::DotDotDot),
            ("=", TokenKind::Eq),
            ("==", TokenKind::EqEq),
            ("=>", TokenKind::FatArrow),
            (">", TokenKind::Gt),
            (">=", TokenKind::Gte),
            ("<-", TokenKind::LArrow),
            ("<", TokenKind::Lt),
            ("<=", TokenKind::Lte),
            ("!=", TokenKind::Ne),
            ("?", TokenKind::Question),
            ("->", TokenKind::RArrow),
            (";", TokenKind::Semi),
            ("~~~", TokenKind::TildeTildeTilde),
            ("w/", TokenKind::WSlash),
            ("w/=", TokenKind::WSlashEq),
            ("{", TokenKind::Open(Delim::Brace)),
            ("}", TokenKind::Close(Delim::Brace)),
            ("[", TokenKind::Open(Delim::Bracket)),
            ("]", TokenKind::Close(Delim::Bracket)),
            ("(", TokenKind::Open(Delim::Paren)),
            (")", TokenKind::Close(Delim::Paren)),
            ("&&&", TokenKind::ClosedBinOp(ClosedBinOp::AmpAmpAmp)),
            ("&&&=", TokenKind::BinOpEq(ClosedBinOp::AmpAmpAmp)),
            ("|||", TokenKind::ClosedBinOp(ClosedBinOp::BarBarBar)),
            ("|||=", TokenKind::BinOpEq(ClosedBinOp::BarBarBar)),
            ("^", TokenKind::ClosedBinOp(ClosedBinOp::Caret)),
            ("^=", TokenKind::BinOpEq(ClosedBinOp::Caret)),
            ("^^^", TokenKind::ClosedBinOp(ClosedBinOp::CaretCaretCaret)),
            ("^^^=", TokenKind::BinOpEq(ClosedBinOp::CaretCaretCaret)),
            (">>>", TokenKind::ClosedBinOp(ClosedBinOp::GtGtGt)),
            (">>>=", TokenKind::BinOpEq(ClosedBinOp::GtGtGt)),
            ("<<<", TokenKind::ClosedBinOp(ClosedBinOp::LtLtLt)),
            ("<<<=", TokenKind::BinOpEq(ClosedBinOp::LtLtLt)),
            ("-", TokenKind::ClosedBinOp(ClosedBinOp::Minus)),
            ("-=", TokenKind::BinOpEq(ClosedBinOp::Minus)),
            ("%", TokenKind::ClosedBinOp(ClosedBinOp::Percent)),
            ("%=", TokenKind::BinOpEq(ClosedBinOp::Percent)),
            ("+", TokenKind::ClosedBinOp(ClosedBinOp::Plus)),
            ("+=", TokenKind::BinOpEq(ClosedBinOp::Plus)),
            ("/", TokenKind::ClosedBinOp(ClosedBinOp::Slash)),
            ("/=", TokenKind::BinOpEq(ClosedBinOp::Slash)),
            ("*", TokenKind::ClosedBinOp(ClosedBinOp::Star)),
            ("*=", TokenKind::BinOpEq(ClosedBinOp::Star)),
            ("and", TokenKind::ClosedBinOp(ClosedBinOp::And)),
            ("and=", TokenKind::BinOpEq(ClosedBinOp::And)),
            ("or", TokenKind::ClosedBinOp(ClosedBinOp::Or)),
            ("or=", TokenKind::BinOpEq(ClosedBinOp::Or)),
        ];

        for (input, kind) in cases {
            let actual: Vec<_> = Lexer::new(input).collect();
            let len = input.len().try_into().unwrap();
            assert_eq!(
                actual,
                vec![
                    Ok(Token {
                        kind,
                        span: Span { lo: 0, hi: len }
                    }),
                    Ok(Token {
                        kind: TokenKind::Eof,
                        span: Span { lo: len, hi: len }
                    })
                ]
            );
        }
    }

    #[test]
    fn empty() {
        check(
            "",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 0,
                                hi: 0,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn amp() {
        check(
            "&",
            &expect![[r#"
                [
                    Err(
                        Error {
                            message: "Expecting `&&&`.",
                            span: Span {
                                lo: 0,
                                hi: 1,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 1,
                                hi: 1,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn amp_amp() {
        check(
            "&&",
            &expect![[r#"
                [
                    Err(
                        Error {
                            message: "Expecting `&&&`.",
                            span: Span {
                                lo: 0,
                                hi: 2,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 2,
                                hi: 2,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn amp_amp_amp_amp_amp_amp() {
        check(
            "&&&&&&",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: ClosedBinOp(
                                AmpAmpAmp,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 3,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: ClosedBinOp(
                                AmpAmpAmp,
                            ),
                            span: Span {
                                lo: 3,
                                hi: 6,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 6,
                                hi: 6,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn caret_caret() {
        check(
            "^^",
            &expect![[r#"
                [
                    Err(
                        Error {
                            message: "Expecting `^^^`.",
                            span: Span {
                                lo: 0,
                                hi: 2,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 2,
                                hi: 2,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn and_ws_eq() {
        check(
            "and =",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: ClosedBinOp(
                                And,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 3,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eq,
                            span: Span {
                                lo: 4,
                                hi: 5,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 5,
                                hi: 5,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn w() {
        check(
            "w",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: Ident,
                            span: Span {
                                lo: 0,
                                hi: 1,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 1,
                                hi: 1,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn w_slash_eq_ident() {
        check(
            "w/=foo",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: WSlashEq,
                            span: Span {
                                lo: 0,
                                hi: 3,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Ident,
                            span: Span {
                                lo: 3,
                                hi: 6,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 6,
                                hi: 6,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn int() {
        check(
            "123",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: Int,
                            span: Span {
                                lo: 0,
                                hi: 3,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 3,
                                hi: 3,
                            },
                        },
                    ),
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
                    Ok(
                        Token {
                            kind: BigInt,
                            span: Span {
                                lo: 0,
                                hi: 4,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 4,
                                hi: 4,
                            },
                        },
                    ),
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
                    Ok(
                        Token {
                            kind: Float,
                            span: Span {
                                lo: 0,
                                hi: 4,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 4,
                                hi: 4,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }

    #[test]
    fn ident() {
        check(
            "foo",
            &expect![[r#"
                [
                    Ok(
                        Token {
                            kind: Ident,
                            span: Span {
                                lo: 0,
                                hi: 3,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 3,
                                hi: 3,
                            },
                        },
                    ),
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
                    Ok(
                        Token {
                            kind: String,
                            span: Span {
                                lo: 0,
                                hi: 8,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 8,
                                hi: 8,
                            },
                        },
                    ),
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
                    Err(
                        Error {
                            message: "Unknown token.",
                            span: Span {
                                lo: 0,
                                hi: 1,
                            },
                        },
                    ),
                    Err(
                        Error {
                            message: "Unknown token.",
                            span: Span {
                                lo: 1,
                                hi: 2,
                            },
                        },
                    ),
                    Ok(
                        Token {
                            kind: Eof,
                            span: Span {
                                lo: 2,
                                hi: 2,
                            },
                        },
                    ),
                ]
            "#]],
        );
    }
}
