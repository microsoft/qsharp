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

use super::{
    raw::{self, Single},
    Delim,
};
use enum_iterator::Sequence;
use qsc_ast::ast::Span;
use std::{
    fmt::{self, Display, Formatter},
    iter::Peekable,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) span: Span,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Error {
    pub(crate) message: &'static str,
    pub(crate) span: Span,
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
    BigInt,
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
    Int,
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
    String,
    /// `~~~`
    TildeTildeTilde,
    /// `w/`
    WSlash,
    /// `w/=`
    WSlashEq,
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

pub(crate) struct Lexer<'a> {
    input: &'a str,
    tokens: Peekable<raw::Lexer<'a>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            input,
            tokens: raw::Lexer::new(input).peekable(),
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
            lo: token.offset,
            hi: self.offset(),
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

        None
    }
}

#[cfg(test)]
mod tests {
    use super::{Lexer, Token, TokenKind};
    use crate::lex::Delim;
    use expect_test::{expect, Expect};
    use qsc_ast::ast::Span;

    fn check(input: &str, expect: &Expect) {
        let actual: Vec<_> = Lexer::new(input).collect();
        expect.assert_debug_eq(&actual);
    }

    fn op_string(kind: TokenKind) -> Option<String> {
        match kind {
            TokenKind::Apos => Some("'".to_string()),
            TokenKind::At => Some("@".to_string()),
            TokenKind::Bang => Some("!".to_string()),
            TokenKind::Bar => Some("|".to_string()),
            TokenKind::BinOpEq(op) => Some(format!("{op}=")),
            TokenKind::Close(Delim::Brace) => Some("}".to_string()),
            TokenKind::Close(Delim::Bracket) => Some("]".to_string()),
            TokenKind::Close(Delim::Paren) => Some(")".to_string()),
            TokenKind::ClosedBinOp(op) => Some(op.to_string()),
            TokenKind::Colon => Some(":".to_string()),
            TokenKind::ColonColon => Some("::".to_string()),
            TokenKind::Comma => Some(",".to_string()),
            TokenKind::Dot => Some(".".to_string()),
            TokenKind::DotDot => Some("..".to_string()),
            TokenKind::DotDotDot => Some("...".to_string()),
            TokenKind::Eq => Some("=".to_string()),
            TokenKind::EqEq => Some("==".to_string()),
            TokenKind::FatArrow => Some("=>".to_string()),
            TokenKind::Gt => Some(">".to_string()),
            TokenKind::Gte => Some(">=".to_string()),
            TokenKind::LArrow => Some("<-".to_string()),
            TokenKind::Lt => Some("<".to_string()),
            TokenKind::Lte => Some("<=".to_string()),
            TokenKind::Ne => Some("!=".to_string()),
            TokenKind::Open(Delim::Brace) => Some("{".to_string()),
            TokenKind::Open(Delim::Bracket) => Some("[".to_string()),
            TokenKind::Open(Delim::Paren) => Some("(".to_string()),
            TokenKind::Question => Some("?".to_string()),
            TokenKind::RArrow => Some("->".to_string()),
            TokenKind::Semi => Some(";".to_string()),
            TokenKind::TildeTildeTilde => Some("~~~".to_string()),
            TokenKind::WSlash => Some("w/".to_string()),
            TokenKind::WSlashEq => Some("w/=".to_string()),
            TokenKind::BigInt
            | TokenKind::Eof
            | TokenKind::Float
            | TokenKind::Ident
            | TokenKind::Int
            | TokenKind::String => None,
        }
    }

    #[test]
    fn basic_ops() {
        for kind in enum_iterator::all() {
            let Some(input) = op_string(kind) else { continue };
            let actual: Vec<_> = Lexer::new(&input).collect();
            let len = input.len();
            assert_eq!(
                actual,
                vec![Ok(Token {
                    kind,
                    span: Span { lo: 0, hi: len }
                }),]
            );
        }
    }

    #[test]
    fn empty() {
        check(
            "",
            &expect![[r#"
                []
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
                ]
            "#]],
        );
    }
}
