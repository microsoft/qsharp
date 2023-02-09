use super::raw::{self, Delim, Single};
use qsc_ast::ast::Span;
use std::iter::Peekable;

pub(crate) struct Token {
    kind: TokenKind,
    span: Span,
}

pub(crate) struct Error {
    message: &'static str,
    span: Span,
}

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
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            tokens: raw::Lexer::new(input).peekable(),
            input,
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
            raw::TokenKind::Comment | raw::TokenKind::Unknown | raw::TokenKind::Whitespace => {
                Ok(None)
            }
            raw::TokenKind::Ident => Ok(Some(TokenKind::Ident)),
            raw::TokenKind::Number(raw::Number::BigInt) => Ok(Some(TokenKind::BigInt)),
            raw::TokenKind::Number(raw::Number::Float) => Ok(Some(TokenKind::Float)),
            raw::TokenKind::Number(raw::Number::Int) => Ok(Some(TokenKind::Int)),
            raw::TokenKind::Single(single) => self.single(single).map(Some),
            raw::TokenKind::String => Ok(Some(TokenKind::String)),
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
                    self.expect(Single::Bar, "Expecting `|||`")?;
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
