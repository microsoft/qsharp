// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// while we work through the conversion, allow dead code to avoid warnings
#![allow(dead_code)]

//! Expression parsing makes use of Pratt parsing (or “top-down operator-precedence parsing”) to handle
//! relative precedence of operators.

use crate::{
    ast::{BinOp, Expr, Lit, LiteralKind, StmtKind, UnOp, Version},
    keyword::Keyword,
    lex::{cooked::Literal, ClosedBinOp, Radix, Token, TokenKind},
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
    Err(Error::new(ErrorKind::Rule(
        "expression",
        s.peek().kind,
        s.peek().span,
    )))
}

pub(super) fn expr_eof(s: &mut ParserContext) -> Result<Box<Expr>> {
    let expr = expr(s)?;
    token(s, TokenKind::Eof)?;
    Ok(expr)
}

/// Returns true if the expression kind is statement-final. When a statement-final expression occurs
/// at the top level of an expression statement, it indicates the end of the statement, and any
/// operators following it will not be parsed as part of the expression. Statement-final expressions
/// in a top level position also do not require a semicolon when they are followed by another
/// statement.
pub(super) fn is_stmt_final(kind: &StmtKind) -> bool {
    matches!(
        kind,
        StmtKind::Block(_)
            | StmtKind::Box(_)
            | StmtKind::Cal(_)
            | StmtKind::DefCal(_)
            | StmtKind::Def(_)
            | StmtKind::If(_)
            | StmtKind::For(_)
            | StmtKind::Switch(_)
            | StmtKind::WhileLoop(_)
    )
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
                let value = lit_int(&lexeme[offset..], radix.into())
                    .ok_or(Error::new(ErrorKind::Lit("integer", token.span)))?;
                Ok(Some(Lit {
                    kind: LiteralKind::Integer(value),
                    span: token.span,
                }))
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
                let lit = shorten(1, 1, lexeme);
                Ok(Some(Lit {
                    kind: LiteralKind::String(lit.into()),
                    span: token.span,
                }))
            }
            Literal::Bitstring => todo!("bitstring literal"),
            Literal::Boolean => todo!("boolean literal"),
            Literal::Imaginary => todo!("imaginary literal"),
            Literal::Timing(_timing_literal_kind) => todo!("timing literal"),
        },
        TokenKind::Keyword(Keyword::True) => Ok(Some(Lit {
            kind: LiteralKind::Boolean(true),
            span: token.span,
        })),
        TokenKind::Keyword(Keyword::False) => Ok(Some(Lit {
            kind: LiteralKind::Boolean(false),
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
