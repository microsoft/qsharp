// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// while we work through the conversion, allow dead code to avoid warnings
#![allow(dead_code)]

//! Expression parsing makes use of Pratt parsing (or “top-down operator-precedence parsing”) to handle
//! relative precedence of operators.

#[cfg(test)]
pub(crate) mod tests;

use num_bigint::BigInt;
use num_traits::Num;
use qsc_data_structures::span::Span;

use crate::{
    ast::{BinOp, Expr, ExprKind, ExprStmt, Lit, LiteralKind, UnOp, Version},
    keyword::Keyword,
    lex::{cooked::Literal, ClosedBinOp, Delim, Radix, Token, TokenKind},
    parser::{
        completion::WordKinds,
        prim::{shorten, token},
        scan::ParserContext,
    },
};

use crate::parser::Result;

use super::error::{Error, ErrorKind};

struct PrefixOp {
    kind: UnOp,
    precedence: u8,
}

struct MixfixOp {
    kind: OpKind,
    precedence: u8,
}

enum OpKind {
    Postfix(UnOp),
    Binary(BinOp, Assoc),
    Assign,
    AssignUpdate,
    AssignBinary(BinOp),
}

#[derive(Clone, Copy)]
enum OpName {
    Token(TokenKind),
    Keyword(Keyword),
}

#[derive(Clone, Copy)]
enum OpContext {
    Precedence(u8),
    Stmt,
}

#[derive(Clone, Copy)]
enum Assoc {
    Left,
    Right,
}

const RANGE_PRECEDENCE: u8 = 1;

pub(super) fn expr(s: &mut ParserContext) -> Result<Box<Expr>> {
    expr_op(s, OpContext::Precedence(0))
}

pub(super) fn expr_stmt(s: &mut ParserContext) -> Result<Box<Expr>> {
    expr_op(s, OpContext::Stmt)
}

fn expr_op(s: &mut ParserContext, _context: OpContext) -> Result<Box<Expr>> {
    let lhs = expr_base(s)?;
    Ok(lhs)
}

fn expr_base(s: &mut ParserContext) -> Result<Box<Expr>> {
    let lo = s.peek().span.lo;
    let kind = if let Some(l) = lit(s)? {
        Ok(Box::new(ExprKind::Lit(l)))
    } else {
        Err(Error::new(ErrorKind::Rule(
            "expression",
            s.peek().kind,
            s.peek().span,
        )))
    }?;

    Ok(Box::new(Expr {
        span: s.span(lo),
        kind,
    }))
}

pub(super) fn lit(s: &mut ParserContext) -> Result<Option<Lit>> {
    let lexeme = s.read();

    s.expect(WordKinds::True | WordKinds::False);

    let token = s.peek();
    match lit_token(lexeme, token) {
        Ok(Some(lit)) => {
            s.advance();
            Ok(Some(lit))
        }
        Ok(None) => Ok(None),
        Err(err) => {
            s.advance();
            Err(err)
        }
    }
}

pub(super) fn version(s: &mut ParserContext) -> Result<Option<Version>> {
    let lexeme = s.read();
    let token = s.peek();
    match version_token(lexeme, token) {
        Ok(Some(lit)) => {
            s.advance();
            Ok(Some(lit))
        }
        Ok(None) => Ok(None),
        Err(err) => {
            s.advance();
            Err(err)
        }
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
fn lit_token(lexeme: &str, token: Token) -> Result<Option<Lit>> {
    match token.kind {
        TokenKind::Literal(literal) => match literal {
            Literal::Integer(radix) => {
                let offset = if radix == Radix::Decimal { 0 } else { 2 };
                let value = lit_int(&lexeme[offset..], radix.into());
                if let Some(value) = value {
                    Ok(Some(Lit {
                        kind: LiteralKind::Int(value),
                        span: token.span,
                    }))
                } else if let Some(value) = lit_bigint(&lexeme[offset..], radix.into()) {
                    Ok(Some(Lit {
                        kind: LiteralKind::BigInt(value),
                        span: token.span,
                    }))
                } else {
                    Err(Error::new(ErrorKind::Lit("integer", token.span)))
                }
            }
            Literal::Float => {
                let lexeme = lexeme.replace('_', "");
                let value = lexeme
                    .parse()
                    .map_err(|_| Error::new(ErrorKind::Lit("floating-point", token.span)))?;
                Ok(Some(Lit {
                    kind: LiteralKind::Float(value),
                    span: token.span,
                }))
            }
            Literal::String => {
                let lexeme = shorten(1, 1, lexeme);
                let string = unescape(lexeme).map_err(|index| {
                    let ch = lexeme[index + 1..]
                        .chars()
                        .next()
                        .expect("character should be found at index");
                    let index: u32 = index.try_into().expect("index should fit into u32");
                    let lo = token.span.lo + index + 2;
                    let span = Span { lo, hi: lo + 1 };
                    Error::new(ErrorKind::Escape(ch, span))
                })?;
                Ok(Some(Lit {
                    kind: LiteralKind::String(string.into()),
                    span: token.span,
                }))
            }
            Literal::Bitstring => {
                let lexeme = shorten(1, 1, lexeme);
                let width = lexeme
                    .to_string()
                    .chars()
                    .filter(|c| *c == '0' || *c == '1')
                    .count();
                // parse it to validate the bitstring
                let value = BigInt::from_str_radix(lexeme, 2)
                    .map_err(|_| Error::new(ErrorKind::Lit("bitstring", token.span)))?;

                Ok(Some(Lit {
                    span: token.span,
                    kind: LiteralKind::Bitstring(value, width),
                }))
            }
            Literal::Imaginary => {
                let lexeme = lexeme
                    .chars()
                    .filter(|x| *x != '_')
                    .take_while(|x| x.is_numeric() || *x == '.')
                    .collect::<String>();

                let value = lexeme
                    .parse()
                    .map_err(|_| Error::new(ErrorKind::Lit("imaginary", token.span)))?;
                Ok(Some(Lit {
                    kind: LiteralKind::Imaginary(value),
                    span: token.span,
                }))
            }
            Literal::Timing(_timing_literal_kind) => Err(Error::new(ErrorKind::Lit(
                "unimplemented: timing literal",
                token.span,
            ))),
        },
        TokenKind::Keyword(Keyword::True) => Ok(Some(Lit {
            kind: LiteralKind::Bool(true),
            span: token.span,
        })),
        TokenKind::Keyword(Keyword::False) => Ok(Some(Lit {
            kind: LiteralKind::Bool(false),
            span: token.span,
        })),
        _ => Ok(None),
    }
}

pub(super) fn version_token(lexeme: &str, token: Token) -> Result<Option<Version>> {
    match token.kind {
        TokenKind::Literal(literal) => {
            if let Literal::Float = literal {
                // validate the version number is in the form of `x.y`
                let (major, minor) = split_and_parse_numbers(lexeme, token)?;
                Ok(Some(Version {
                    major,
                    minor: Some(minor),
                    span: token.span,
                }))
            } else if let Literal::Integer(radix) = literal {
                if radix != Radix::Decimal {
                    return Err(Error::new(ErrorKind::Lit("version", token.span)));
                }
                let major = lexeme
                    .parse::<u32>()
                    .map_err(|_| Error::new(ErrorKind::Lit("version", token.span)))?;

                Ok(Some(Version {
                    major,
                    minor: None,
                    span: token.span,
                }))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

fn split_and_parse_numbers(lexeme: &str, token: Token) -> Result<(u32, u32)> {
    let parts: Vec<&str> = lexeme.split('.').collect();
    if parts.len() != 2 {
        return Err(Error::new(ErrorKind::Lit("version", token.span)));
    }

    let left = parts[0]
        .parse::<u32>()
        .map_err(|_| Error::new(ErrorKind::Lit("version major", token.span)))?;
    let right = parts[1]
        .parse::<u32>()
        .map_err(|_| Error::new(ErrorKind::Lit("version minor", token.span)))?;

    Ok((left, right))
}

fn lit_int(lexeme: &str, radix: u32) -> Option<i64> {
    let multiplier = i64::from(radix);
    lexeme
        .chars()
        .filter(|&c| c != '_')
        .try_rfold((0i64, 1i64, false), |(value, place, mut overflow), c| {
            let (increment, over) = i64::from(c.to_digit(radix)?).overflowing_mul(place);
            overflow |= over;

            let (new_value, over) = value.overflowing_add(increment);
            overflow |= over;

            // Only treat as overflow if the value is not i64::MIN, since we need to allow once special
            // case of overflow to allow for minimum value literals.
            if overflow && new_value != i64::MIN {
                return None;
            }

            let (new_place, over) = place.overflowing_mul(multiplier);
            overflow |= over;

            // If the place overflows, we can still accept the value as long as it's the last digit.
            // Pass the overflow forward so that it fails if there are more digits.
            Some((new_value, new_place, overflow))
        })
        .map(|(value, _, _)| value)
}

fn lit_bigint(lexeme: &str, radix: u32) -> Option<BigInt> {
    // from_str_radix does removes underscores as long as the lexeme
    // doesn't start with an underscore.
    match BigInt::from_str_radix(lexeme, radix) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

fn prefix_op(name: OpName) -> Option<PrefixOp> {
    match name {
        OpName::Token(TokenKind::Bang) => Some(PrefixOp {
            kind: UnOp::NotL,
            precedence: 11,
        }),
        OpName::Token(TokenKind::Tilde) => Some(PrefixOp {
            kind: UnOp::NotB,
            precedence: 11,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Minus)) => Some(PrefixOp {
            kind: UnOp::Neg,
            precedence: 11,
        }),

        _ => None,
    }
}

fn closed_bin_op(op: ClosedBinOp) -> BinOp {
    match op {
        ClosedBinOp::Amp => BinOp::AndB,
        ClosedBinOp::AmpAmp => BinOp::AndL,
        ClosedBinOp::Bar => BinOp::OrB,
        ClosedBinOp::StarStar => BinOp::Exp,
        ClosedBinOp::Caret => BinOp::XorB,
        ClosedBinOp::GtGt => BinOp::Shr,
        ClosedBinOp::LtLt => BinOp::Shl,
        ClosedBinOp::Minus => BinOp::Sub,
        ClosedBinOp::BarBar => BinOp::OrL,
        ClosedBinOp::Percent => BinOp::Mod,
        ClosedBinOp::Plus => BinOp::Add,
        ClosedBinOp::Slash => BinOp::Div,
        ClosedBinOp::Star => BinOp::Mul,
    }
}

fn unescape(s: &str) -> std::result::Result<String, usize> {
    let mut chars = s.char_indices();
    let mut buf = String::with_capacity(s.len());
    while let Some((index, ch)) = chars.next() {
        buf.push(if ch == '\\' {
            let escape = chars.next().expect("escape should not be empty").1;
            match escape {
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err(index),
            }
        } else {
            ch
        });
    }

    Ok(buf)
}

pub(super) fn designator(s: &mut ParserContext) -> Result<ExprStmt> {
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Bracket))?;
    let expr = expr(s)?;
    token(s, TokenKind::Close(Delim::Bracket))?;
    Ok(ExprStmt {
        span: s.span(lo),
        expr,
    })
}
