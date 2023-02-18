// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    keyword::Keyword,
    prim::{ident, keyword, opt, pat, path, seq, token},
    scan::Scanner,
    stmt, ErrorKind, Result,
};
use crate::lex::{ClosedBinOp, Delim, TokenKind};
use qsc_ast::ast::{self, BinOp, Expr, ExprKind, Functor, Lit, NodeId, Pauli, TernOp, UnOp};
use std::str::FromStr;

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
    Ternary(TernOp, TokenKind, Assoc),
    Rich(fn(&mut Scanner, Expr) -> Result<ExprKind>),
}

#[derive(Clone, Copy)]
enum Assoc {
    Left,
    Right,
}

#[derive(Clone, Copy)]
enum OpName {
    Token(TokenKind),
    Keyword(Keyword),
}

const RANGE_PRECEDENCE: u8 = 1;

pub(super) fn expr(s: &mut Scanner) -> Result<Expr> {
    expr_op(s, 0)
}

fn expr_op(s: &mut Scanner, min_precedence: u8) -> Result<Expr> {
    let lo = s.peek().span.lo;
    let mut lhs = if let Some(op) = prefix_op(op_name(s)) {
        s.advance();
        let rhs = expr_op(s, op.precedence)?;
        Expr {
            id: NodeId::PLACEHOLDER,
            span: s.span(lo),
            kind: ExprKind::UnOp(op.kind, Box::new(rhs)),
        }
    } else {
        expr_base(s)?
    };

    while let Some(op) = mixfix_op(op_name(s)) {
        if op.precedence < min_precedence {
            break;
        }

        s.advance();
        let kind = match op.kind {
            OpKind::Postfix(kind) => ExprKind::UnOp(kind, Box::new(lhs)),
            OpKind::Binary(kind, assoc) => {
                let rhs = expr_op(s, next_precedence(op.precedence, assoc))?;
                ExprKind::BinOp(kind, Box::new(lhs), Box::new(rhs))
            }
            OpKind::Ternary(kind, delim, assoc) => {
                let middle = expr(s)?;
                token(s, delim)?;
                let rhs = expr_op(s, next_precedence(op.precedence, assoc))?;
                ExprKind::TernOp(kind, Box::new(lhs), Box::new(middle), Box::new(rhs))
            }
            OpKind::Rich(f) => f(s, lhs)?,
        };

        lhs = Expr {
            id: NodeId::PLACEHOLDER,
            span: s.span(lo),
            kind,
        };
    }

    Ok(lhs)
}

fn expr_base(s: &mut Scanner) -> Result<Expr> {
    let lo = s.peek().span.lo;
    let kind = if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let (exprs, final_sep) = seq(s, expr)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(final_sep.reify(exprs, |e| ExprKind::Paren(Box::new(e)), ExprKind::Tuple))
    } else if token(s, TokenKind::Open(Delim::Bracket)).is_ok() {
        let exprs = seq(s, expr)?.0;
        token(s, TokenKind::Close(Delim::Bracket))?;
        Ok(ExprKind::Array(exprs))
    } else if token(s, TokenKind::DotDotDot).is_ok() {
        let e = opt(s, |s| expr_op(s, RANGE_PRECEDENCE + 1))?.map(Box::new);
        if token(s, TokenKind::DotDotDot).is_ok() {
            Ok(ExprKind::Range(None, e, None))
        } else {
            Ok(ExprKind::Range(None, None, e))
        }
    } else if keyword(s, Keyword::Fail).is_ok() {
        Ok(ExprKind::Fail(Box::new(expr(s)?)))
    } else if keyword(s, Keyword::For).is_ok() {
        let vars = pat(s)?;
        keyword(s, Keyword::In)?;
        let iter = expr(s)?;
        let body = stmt::block(s)?;
        Ok(ExprKind::For(vars, Box::new(iter), body))
    } else if keyword(s, Keyword::If).is_ok() {
        if_kind(s)
    } else if keyword(s, Keyword::Repeat).is_ok() {
        let body = stmt::block(s)?;
        keyword(s, Keyword::Until)?;
        let cond = expr(s)?;
        let fixup = if keyword(s, Keyword::Fixup).is_ok() {
            Some(stmt::block(s)?)
        } else {
            None
        };
        Ok(ExprKind::Repeat(body, Box::new(cond), fixup))
    } else if keyword(s, Keyword::Return).is_ok() {
        Ok(ExprKind::Return(Box::new(expr(s)?)))
    } else if keyword(s, Keyword::Set).is_ok() {
        let lhs = expr(s)?;
        token(s, TokenKind::Eq)?;
        let rhs = expr(s)?;
        Ok(ExprKind::Assign(Box::new(lhs), Box::new(rhs)))
    } else if keyword(s, Keyword::While).is_ok() {
        Ok(ExprKind::While(Box::new(expr(s)?), stmt::block(s)?))
    } else if keyword(s, Keyword::Within).is_ok() {
        let outer = stmt::block(s)?;
        keyword(s, Keyword::Apply)?;
        let inner = stmt::block(s)?;
        Ok(ExprKind::Conjugate(outer, inner))
    } else if let Some(b) = opt(s, stmt::block)? {
        Ok(ExprKind::Block(b))
    } else if let Some(l) = opt(s, lit)? {
        Ok(ExprKind::Lit(l))
    } else if let Some(p) = opt(s, path)? {
        Ok(ExprKind::Path(p))
    } else {
        Err(s.error(ErrorKind::Rule("expression")))
    }?;

    Ok(Expr {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
    })
}

fn if_kind(s: &mut Scanner) -> Result<ExprKind> {
    let cond = expr(s)?;
    let body = stmt::block(s)?;

    let lo = s.peek().span.lo;
    let otherwise = if keyword(s, Keyword::Elif).is_ok() {
        Some(if_kind(s)?)
    } else if keyword(s, Keyword::Else).is_ok() {
        Some(ExprKind::Block(stmt::block(s)?))
    } else {
        None
    }
    .map(|kind| {
        Box::new(Expr {
            id: NodeId::PLACEHOLDER,
            span: s.span(lo),
            kind,
        })
    });

    Ok(ExprKind::If(Box::new(cond), body, otherwise))
}

fn lit(s: &mut Scanner) -> Result<Lit> {
    let lexeme = s.read();
    if token(s, TokenKind::BigInt).is_ok() {
        let lexeme = &lexeme[..lexeme.len() - 1]; // Slice off suffix.
        let value = lexeme.parse().expect("BigInt token can't be parsed.");
        Ok(Lit::BigInt(value))
    } else if token(s, TokenKind::Float).is_ok() {
        let lexeme = lexeme.replace('_', "");
        let value = lexeme.parse().expect("Float token can't be parsed.");
        Ok(Lit::Double(value))
    } else if token(s, TokenKind::Int).is_ok() {
        let lexeme = lexeme.replace('_', "");
        let value = lexeme.parse().expect("Int token can't be parsed.");
        Ok(Lit::Int(value))
    } else if token(s, TokenKind::String).is_ok() {
        let lexeme = &lexeme[1..lexeme.len() - 1]; // Slice off quotation marks.
        Ok(Lit::String(lexeme.to_string()))
    } else if keyword(s, Keyword::False).is_ok() {
        Ok(Lit::Bool(false))
    } else if keyword(s, Keyword::True).is_ok() {
        Ok(Lit::Bool(true))
    } else if keyword(s, Keyword::Zero).is_ok() {
        Ok(Lit::Result(ast::Result::Zero))
    } else if keyword(s, Keyword::One).is_ok() {
        Ok(Lit::Result(ast::Result::One))
    } else if keyword(s, Keyword::PauliI).is_ok() {
        Ok(Lit::Pauli(Pauli::I))
    } else if keyword(s, Keyword::PauliX).is_ok() {
        Ok(Lit::Pauli(Pauli::X))
    } else if keyword(s, Keyword::PauliY).is_ok() {
        Ok(Lit::Pauli(Pauli::Y))
    } else if keyword(s, Keyword::PauliZ).is_ok() {
        Ok(Lit::Pauli(Pauli::Z))
    } else {
        Err(s.error(ErrorKind::Rule("literal")))
    }
}

fn prefix_op(name: OpName) -> Option<PrefixOp> {
    match name {
        OpName::Keyword(Keyword::Not) => Some(PrefixOp {
            kind: UnOp::NotL,
            precedence: 11,
        }),
        OpName::Token(TokenKind::TildeTildeTilde) => Some(PrefixOp {
            kind: UnOp::NotB,
            precedence: 11,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Plus)) => Some(PrefixOp {
            kind: UnOp::Pos,
            precedence: 11,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Minus)) => Some(PrefixOp {
            kind: UnOp::Neg,
            precedence: 11,
        }),
        OpName::Keyword(Keyword::AdjointUpper) => Some(PrefixOp {
            kind: UnOp::Functor(Functor::Adj),
            precedence: 13,
        }),
        OpName::Keyword(Keyword::ControlledUpper) => Some(PrefixOp {
            kind: UnOp::Functor(Functor::Ctl),
            precedence: 13,
        }),
        _ => None,
    }
}

#[allow(clippy::too_many_lines)]
fn mixfix_op(name: OpName) -> Option<MixfixOp> {
    match name {
        OpName::Token(TokenKind::DotDot) => Some(MixfixOp {
            kind: OpKind::Rich(closed_range_op),
            precedence: RANGE_PRECEDENCE,
        }),
        OpName::Token(TokenKind::DotDotDot) => Some(MixfixOp {
            kind: OpKind::Rich(|_, start| Ok(ExprKind::Range(Some(Box::new(start)), None, None))),
            precedence: RANGE_PRECEDENCE,
        }),
        OpName::Token(TokenKind::WSlash) => Some(MixfixOp {
            kind: OpKind::Ternary(TernOp::Update, TokenKind::LArrow, Assoc::Left),
            precedence: 1,
        }),
        OpName::Token(TokenKind::Question) => Some(MixfixOp {
            kind: OpKind::Ternary(TernOp::Cond, TokenKind::Bar, Assoc::Right),
            precedence: 1,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Or)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::OrL, Assoc::Left),
            precedence: 2,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::And)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::AndL, Assoc::Left),
            precedence: 3,
        }),
        OpName::Token(TokenKind::EqEq) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Eq, Assoc::Left),
            precedence: 4,
        }),
        OpName::Token(TokenKind::Ne) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Neq, Assoc::Left),
            precedence: 4,
        }),
        OpName::Token(TokenKind::Gt) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Gt, Assoc::Left),
            precedence: 4,
        }),
        OpName::Token(TokenKind::Gte) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Gte, Assoc::Left),
            precedence: 4,
        }),
        OpName::Token(TokenKind::Lt) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Lt, Assoc::Left),
            precedence: 4,
        }),
        OpName::Token(TokenKind::Lte) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Lte, Assoc::Left),
            precedence: 4,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::BarBarBar)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::OrB, Assoc::Left),
            precedence: 5,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::CaretCaretCaret)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::XorB, Assoc::Left),
            precedence: 6,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::AmpAmpAmp)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::AndB, Assoc::Left),
            precedence: 7,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::LtLtLt)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Shl, Assoc::Left),
            precedence: 8,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::GtGtGt)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Shr, Assoc::Left),
            precedence: 8,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Plus)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Add, Assoc::Left),
            precedence: 9,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Minus)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Sub, Assoc::Left),
            precedence: 9,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Star)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Mul, Assoc::Left),
            precedence: 10,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Slash)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Div, Assoc::Left),
            precedence: 10,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Percent)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Mod, Assoc::Left),
            precedence: 10,
        }),
        OpName::Token(TokenKind::ClosedBinOp(ClosedBinOp::Caret)) => Some(MixfixOp {
            kind: OpKind::Binary(BinOp::Exp, Assoc::Right),
            precedence: 12,
        }),
        OpName::Token(TokenKind::Bang) => Some(MixfixOp {
            kind: OpKind::Postfix(UnOp::Unwrap),
            precedence: 14,
        }),
        OpName::Token(TokenKind::ColonColon) => Some(MixfixOp {
            kind: OpKind::Rich(field_op),
            precedence: 14,
        }),
        OpName::Token(TokenKind::Open(Delim::Bracket)) => Some(MixfixOp {
            kind: OpKind::Rich(index_op),
            precedence: 14,
        }),
        OpName::Token(TokenKind::Open(Delim::Paren)) => Some(MixfixOp {
            kind: OpKind::Rich(call_op),
            precedence: 14,
        }),
        _ => None,
    }
}

fn field_op(s: &mut Scanner, lhs: Expr) -> Result<ExprKind> {
    Ok(ExprKind::Field(Box::new(lhs), ident(s)?))
}

fn index_op(s: &mut Scanner, lhs: Expr) -> Result<ExprKind> {
    let index = expr(s)?;
    token(s, TokenKind::Close(Delim::Bracket))?;
    Ok(ExprKind::Index(Box::new(lhs), Box::new(index)))
}

fn call_op(s: &mut Scanner, lhs: Expr) -> Result<ExprKind> {
    let lo = s.span(0).hi - 1;
    let (args, final_sep) = seq(s, expr)?;
    token(s, TokenKind::Close(Delim::Paren))?;
    let rhs = Expr {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind: final_sep.reify(args, |a| ExprKind::Paren(Box::new(a)), ExprKind::Tuple),
    };
    Ok(ExprKind::Call(Box::new(lhs), Box::new(rhs)))
}

fn closed_range_op(s: &mut Scanner, start: Expr) -> Result<ExprKind> {
    let e = expr_op(s, RANGE_PRECEDENCE + 1)?;
    let (step, end) = if token(s, TokenKind::DotDot).is_ok() {
        (Some(Box::new(e)), expr_op(s, RANGE_PRECEDENCE + 1)?)
    } else {
        (None, e)
    };
    Ok(ExprKind::Range(
        Some(Box::new(start)),
        step,
        Some(Box::new(end)),
    ))
}

fn op_name(s: &Scanner) -> OpName {
    match Keyword::from_str(s.read()) {
        Ok(Keyword::And | Keyword::Or) | Err(_) => OpName::Token(s.peek().kind),
        Ok(keyword) => OpName::Keyword(keyword),
    }
}

fn next_precedence(precedence: u8, assoc: Assoc) -> u8 {
    match assoc {
        Assoc::Left => precedence + 1,
        Assoc::Right => precedence,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]

    use super::expr;
    use crate::parse::tests::check;
    use expect_test::expect;

    #[test]
    fn lit_int() {
        check(
            expr,
            "123",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Lit(
                            Int(
                                123,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_int_underscore() {
        check(
            expr,
            "123_456",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Lit(
                            Int(
                                123456,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_int_leading_zero() {
        check(
            expr,
            "0123",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Lit(
                            Int(
                                123,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_big_int() {
        check(
            expr,
            "123L",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Lit(
                            BigInt(
                                123,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_big_int_underscore() {
        check(
            expr,
            "123_456L",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 8,
                        },
                        kind: Lit(
                            BigInt(
                                123456,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_double() {
        check(
            expr,
            "1.23",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Lit(
                            Double(
                                1.23,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_double_leading_dot() {
        check(
            expr,
            ".23",
            &expect![[r#"
                Err(
                    Error {
                        kind: Rule(
                            "expression",
                        ),
                        span: Span {
                            lo: 0,
                            hi: 1,
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_double_trailing_dot() {
        check(
            expr,
            "1.",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: Lit(
                            Double(
                                1.0,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_double_underscore() {
        check(
            expr,
            "123_456.78",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 10,
                        },
                        kind: Lit(
                            Double(
                                123456.78,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_string() {
        check(
            expr,
            r#""foo""#,
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: Lit(
                            String(
                                "foo",
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_string_escape_quote() {
        check(
            expr,
            r#""foo\"bar""#,
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 10,
                        },
                        kind: Lit(
                            String(
                                "foo\\\"bar",
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_false() {
        check(
            expr,
            "false",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: Lit(
                            Bool(
                                false,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_true() {
        check(
            expr,
            "true",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Lit(
                            Bool(
                                true,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_zero() {
        check(
            expr,
            "Zero",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Lit(
                            Result(
                                Zero,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_one() {
        check(
            expr,
            "One",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Lit(
                            Result(
                                One,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_pauli_i() {
        check(
            expr,
            "PauliI",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Lit(
                            Pauli(
                                I,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_pauli_x() {
        check(
            expr,
            "PauliX",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Lit(
                            Pauli(
                                X,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_pauli_y() {
        check(
            expr,
            "PauliY",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Lit(
                            Pauli(
                                Y,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lit_pauli_z() {
        check(
            expr,
            "PauliZ",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Lit(
                            Pauli(
                                Z,
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn single_path() {
        check(
            expr,
            "foo",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Path(
                            Path {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 3,
                                },
                                namespace: None,
                                name: Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 3,
                                    },
                                    name: "foo",
                                },
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn double_path() {
        check(
            expr,
            "foo.bar",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Path(
                            Path {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 7,
                                },
                                namespace: Some(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 3,
                                        },
                                        name: "foo",
                                    },
                                ),
                                name: Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 4,
                                        hi: 7,
                                    },
                                    name: "bar",
                                },
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn block() {
        check(
            expr,
            "{ let x = 1; x }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 16,
                        },
                        kind: Block(
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 16,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 2,
                                            hi: 12,
                                        },
                                        kind: Let(
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 6,
                                                    hi: 7,
                                                },
                                                kind: Bind(
                                                    Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 6,
                                                            hi: 7,
                                                        },
                                                        name: "x",
                                                    },
                                                    None,
                                                ),
                                            },
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 10,
                                                    hi: 11,
                                                },
                                                kind: Lit(
                                                    Int(
                                                        1,
                                                    ),
                                                ),
                                            },
                                        ),
                                    },
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 13,
                                            hi: 14,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 13,
                                                    hi: 14,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 13,
                                                            hi: 14,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 13,
                                                                hi: 14,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn fail() {
        check(
            expr,
            r#"fail "message""#,
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 14,
                        },
                        kind: Fail(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 14,
                                },
                                kind: Lit(
                                    String(
                                        "message",
                                    ),
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn for_in() {
        check(
            expr,
            "for x in xs { x }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 17,
                        },
                        kind: For(
                            Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Bind(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        name: "x",
                                    },
                                    None,
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 11,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 9,
                                            hi: 11,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 9,
                                                hi: 11,
                                            },
                                            name: "xs",
                                        },
                                    },
                                ),
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 12,
                                    hi: 17,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 15,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 14,
                                                    hi: 15,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 14,
                                                            hi: 15,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 14,
                                                                hi: 15,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn if_then() {
        check(
            expr,
            "if c { e }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 10,
                        },
                        kind: If(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 4,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 4,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 4,
                                            },
                                            name: "c",
                                        },
                                    },
                                ),
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 10,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 7,
                                            hi: 8,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 7,
                                                    hi: 8,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 7,
                                                            hi: 8,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 7,
                                                                hi: 8,
                                                            },
                                                            name: "e",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            None,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn if_else() {
        check(
            expr,
            "if c { x } else { y }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 21,
                        },
                        kind: If(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 4,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 4,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 4,
                                            },
                                            name: "c",
                                        },
                                    },
                                ),
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 10,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 7,
                                            hi: 8,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 7,
                                                    hi: 8,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 7,
                                                            hi: 8,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 7,
                                                                hi: 8,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 11,
                                        hi: 21,
                                    },
                                    kind: Block(
                                        Block {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 16,
                                                hi: 21,
                                            },
                                            stmts: [
                                                Stmt {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 18,
                                                        hi: 19,
                                                    },
                                                    kind: Expr(
                                                        Expr {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 18,
                                                                hi: 19,
                                                            },
                                                            kind: Path(
                                                                Path {
                                                                    id: NodeId(
                                                                        4294967295,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 18,
                                                                        hi: 19,
                                                                    },
                                                                    namespace: None,
                                                                    name: Ident {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 18,
                                                                            hi: 19,
                                                                        },
                                                                        name: "y",
                                                                    },
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                },
                                            ],
                                        },
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn if_elif() {
        check(
            expr,
            "if c1 { x } elif c2 { y }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 25,
                        },
                        kind: If(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 5,
                                            },
                                            name: "c1",
                                        },
                                    },
                                ),
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 11,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 8,
                                                    hi: 9,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 8,
                                                            hi: 9,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 8,
                                                                hi: 9,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 12,
                                        hi: 25,
                                    },
                                    kind: If(
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 17,
                                                hi: 19,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 17,
                                                        hi: 19,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 17,
                                                            hi: 19,
                                                        },
                                                        name: "c2",
                                                    },
                                                },
                                            ),
                                        },
                                        Block {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 20,
                                                hi: 25,
                                            },
                                            stmts: [
                                                Stmt {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 22,
                                                        hi: 23,
                                                    },
                                                    kind: Expr(
                                                        Expr {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 22,
                                                                hi: 23,
                                                            },
                                                            kind: Path(
                                                                Path {
                                                                    id: NodeId(
                                                                        4294967295,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 22,
                                                                        hi: 23,
                                                                    },
                                                                    namespace: None,
                                                                    name: Ident {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 22,
                                                                            hi: 23,
                                                                        },
                                                                        name: "y",
                                                                    },
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                },
                                            ],
                                        },
                                        None,
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn if_elif_else() {
        check(
            expr,
            "if c1 { x } elif c2 { y } else { z }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 36,
                        },
                        kind: If(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 5,
                                            },
                                            name: "c1",
                                        },
                                    },
                                ),
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 11,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 8,
                                                    hi: 9,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 8,
                                                            hi: 9,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 8,
                                                                hi: 9,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 12,
                                        hi: 36,
                                    },
                                    kind: If(
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 17,
                                                hi: 19,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 17,
                                                        hi: 19,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 17,
                                                            hi: 19,
                                                        },
                                                        name: "c2",
                                                    },
                                                },
                                            ),
                                        },
                                        Block {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 20,
                                                hi: 25,
                                            },
                                            stmts: [
                                                Stmt {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 22,
                                                        hi: 23,
                                                    },
                                                    kind: Expr(
                                                        Expr {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 22,
                                                                hi: 23,
                                                            },
                                                            kind: Path(
                                                                Path {
                                                                    id: NodeId(
                                                                        4294967295,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 22,
                                                                        hi: 23,
                                                                    },
                                                                    namespace: None,
                                                                    name: Ident {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 22,
                                                                            hi: 23,
                                                                        },
                                                                        name: "y",
                                                                    },
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                },
                                            ],
                                        },
                                        Some(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 26,
                                                    hi: 36,
                                                },
                                                kind: Block(
                                                    Block {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 31,
                                                            hi: 36,
                                                        },
                                                        stmts: [
                                                            Stmt {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 33,
                                                                    hi: 34,
                                                                },
                                                                kind: Expr(
                                                                    Expr {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 33,
                                                                            hi: 34,
                                                                        },
                                                                        kind: Path(
                                                                            Path {
                                                                                id: NodeId(
                                                                                    4294967295,
                                                                                ),
                                                                                span: Span {
                                                                                    lo: 33,
                                                                                    hi: 34,
                                                                                },
                                                                                namespace: None,
                                                                                name: Ident {
                                                                                    id: NodeId(
                                                                                        4294967295,
                                                                                    ),
                                                                                    span: Span {
                                                                                        lo: 33,
                                                                                        hi: 34,
                                                                                    },
                                                                                    name: "z",
                                                                                },
                                                                            },
                                                                        ),
                                                                    },
                                                                ),
                                                            },
                                                        ],
                                                    },
                                                ),
                                            },
                                        ),
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn repeat_until() {
        check(
            expr,
            "repeat { x } until c",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 20,
                        },
                        kind: Repeat(
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 7,
                                    hi: 12,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 9,
                                            hi: 10,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 9,
                                                    hi: 10,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 9,
                                                            hi: 10,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 9,
                                                                hi: 10,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 19,
                                    hi: 20,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 19,
                                            hi: 20,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 19,
                                                hi: 20,
                                            },
                                            name: "c",
                                        },
                                    },
                                ),
                            },
                            None,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn repeat_until_fixup() {
        check(
            expr,
            "repeat { x } until c fixup { y }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 32,
                        },
                        kind: Repeat(
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 7,
                                    hi: 12,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 9,
                                            hi: 10,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 9,
                                                    hi: 10,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 9,
                                                            hi: 10,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 9,
                                                                hi: 10,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 19,
                                    hi: 20,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 19,
                                            hi: 20,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 19,
                                                hi: 20,
                                            },
                                            name: "c",
                                        },
                                    },
                                ),
                            },
                            Some(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 27,
                                        hi: 32,
                                    },
                                    stmts: [
                                        Stmt {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 29,
                                                hi: 30,
                                            },
                                            kind: Expr(
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 29,
                                                        hi: 30,
                                                    },
                                                    kind: Path(
                                                        Path {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 29,
                                                                hi: 30,
                                                            },
                                                            namespace: None,
                                                            name: Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 29,
                                                                    hi: 30,
                                                                },
                                                                name: "y",
                                                            },
                                                        },
                                                    ),
                                                },
                                            ),
                                        },
                                    ],
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn return_expr() {
        check(
            expr,
            "return x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 8,
                        },
                        kind: Return(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 7,
                                    hi: 8,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 7,
                                            hi: 8,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 7,
                                                hi: 8,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn set() {
        check(
            expr,
            "set x = y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: Assign(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 8,
                                    hi: 9,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 8,
                                                hi: 9,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn while_expr() {
        check(
            expr,
            "while c { x }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 13,
                        },
                        kind: While(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "c",
                                        },
                                    },
                                ),
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 8,
                                    hi: 13,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 10,
                                            hi: 11,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 10,
                                                    hi: 11,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 10,
                                                            hi: 11,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 10,
                                                                hi: 11,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn within_apply() {
        check(
            expr,
            "within { x } apply { y }",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 24,
                        },
                        kind: Conjugate(
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 7,
                                    hi: 12,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 9,
                                            hi: 10,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 9,
                                                    hi: 10,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 9,
                                                            hi: 10,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 9,
                                                                hi: 10,
                                                            },
                                                            name: "x",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                            Block {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 19,
                                    hi: 24,
                                },
                                stmts: [
                                    Stmt {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 21,
                                            hi: 22,
                                        },
                                        kind: Expr(
                                            Expr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 21,
                                                    hi: 22,
                                                },
                                                kind: Path(
                                                    Path {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 21,
                                                            hi: 22,
                                                        },
                                                        namespace: None,
                                                        name: Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 21,
                                                                hi: 22,
                                                            },
                                                            name: "y",
                                                        },
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                ],
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn unit() {
        check(
            expr,
            "()",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: Tuple(
                            [],
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn paren() {
        check(
            expr,
            "(x)",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Paren(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 2,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 2,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn singleton_tuple() {
        check(
            expr,
            "(x,)",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Tuple(
                            [
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 2,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 2,
                                                },
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                            ],
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn pair() {
        check(
            expr,
            "(x, y)",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Tuple(
                            [
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 2,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 2,
                                                },
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 4,
                                        hi: 5,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 4,
                                                    hi: 5,
                                                },
                                                name: "y",
                                            },
                                        },
                                    ),
                                },
                            ],
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn array_empty() {
        check(
            expr,
            "[]",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: Array(
                            [],
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn array_single() {
        check(
            expr,
            "[x]",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Array(
                            [
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 2,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 2,
                                                },
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                            ],
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn array_pair() {
        check(
            expr,
            "[x, y]",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Array(
                            [
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 2,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 2,
                                                },
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 4,
                                        hi: 5,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 4,
                                                    hi: 5,
                                                },
                                                name: "y",
                                            },
                                        },
                                    ),
                                },
                            ],
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn array_concat() {
        check(
            expr,
            "[1, 2] + [3, 4]",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 15,
                        },
                        kind: BinOp(
                            Add,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 6,
                                },
                                kind: Array(
                                    [
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            kind: Lit(
                                                Int(
                                                    1,
                                                ),
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            kind: Lit(
                                                Int(
                                                    2,
                                                ),
                                            ),
                                        },
                                    ],
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 15,
                                },
                                kind: Array(
                                    [
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 10,
                                                hi: 11,
                                            },
                                            kind: Lit(
                                                Int(
                                                    3,
                                                ),
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 13,
                                                hi: 14,
                                            },
                                            kind: Lit(
                                                Int(
                                                    4,
                                                ),
                                            ),
                                        },
                                    ],
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn and_op() {
        check(
            expr,
            "x and y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: BinOp(
                            AndL,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn or_op() {
        check(
            expr,
            "x or y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: BinOp(
                            OrL,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn and_or_ops() {
        check(
            expr,
            "x or y and z",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 12,
                        },
                        kind: BinOp(
                            OrL,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 12,
                                },
                                kind: BinOp(
                                    AndL,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 5,
                                                    hi: 6,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 5,
                                                        hi: 6,
                                                    },
                                                    name: "y",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 11,
                                            hi: 12,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 11,
                                                    hi: 12,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 11,
                                                        hi: 12,
                                                    },
                                                    name: "z",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn eq_op() {
        check(
            expr,
            "x == y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: BinOp(
                            Eq,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ne_op() {
        check(
            expr,
            "x != y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: BinOp(
                            Neq,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn gt_op() {
        check(
            expr,
            "x > y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Gt,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn gte_op() {
        check(
            expr,
            "x >= y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: BinOp(
                            Gte,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lt_op() {
        check(
            expr,
            "x < y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Lt,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn lte_op() {
        check(
            expr,
            "x <= y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: BinOp(
                            Lte,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn bitwise_and_op() {
        check(
            expr,
            "x &&& y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: BinOp(
                            AndB,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn bitwise_or_op() {
        check(
            expr,
            "x ||| y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: BinOp(
                            OrB,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn bitwise_and_or_op() {
        check(
            expr,
            "x ||| y &&& z",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 13,
                        },
                        kind: BinOp(
                            OrB,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 13,
                                },
                                kind: BinOp(
                                    AndB,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 6,
                                                    hi: 7,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 6,
                                                        hi: 7,
                                                    },
                                                    name: "y",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 12,
                                            hi: 13,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 12,
                                                    hi: 13,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 12,
                                                        hi: 13,
                                                    },
                                                    name: "z",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn bitwise_xor_op() {
        check(
            expr,
            "x ^^^ y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: BinOp(
                            XorB,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn bitwise_or_xor_ops() {
        check(
            expr,
            "x ||| y ^^^ z",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 13,
                        },
                        kind: BinOp(
                            OrB,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 13,
                                },
                                kind: BinOp(
                                    XorB,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 6,
                                                    hi: 7,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 6,
                                                        hi: 7,
                                                    },
                                                    name: "y",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 12,
                                            hi: 13,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 12,
                                                    hi: 13,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 12,
                                                        hi: 13,
                                                    },
                                                    name: "z",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn shl_op() {
        check(
            expr,
            "x <<< y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: BinOp(
                            Shl,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn shr_op() {
        check(
            expr,
            "x >>> y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: BinOp(
                            Shr,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 6,
                                    hi: 7,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 7,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn add_op() {
        check(
            expr,
            "x + y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Add,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn add_left_assoc() {
        check(
            expr,
            "x + y + z",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: BinOp(
                            Add,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 5,
                                },
                                kind: BinOp(
                                    Add,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 1,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 0,
                                                        hi: 1,
                                                    },
                                                    name: "x",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 4,
                                                    hi: 5,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 4,
                                                        hi: 5,
                                                    },
                                                    name: "y",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 8,
                                    hi: 9,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 8,
                                                hi: 9,
                                            },
                                            name: "z",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn sub_op() {
        check(
            expr,
            "x - y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Sub,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn mul_op() {
        check(
            expr,
            "x * y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Mul,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn add_mul_ops() {
        check(
            expr,
            "x + y * z",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: BinOp(
                            Add,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 9,
                                },
                                kind: BinOp(
                                    Mul,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 4,
                                                    hi: 5,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 4,
                                                        hi: 5,
                                                    },
                                                    name: "y",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 8,
                                                    hi: 9,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 8,
                                                        hi: 9,
                                                    },
                                                    name: "z",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn div_op() {
        check(
            expr,
            "x / y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Div,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn mod_op() {
        check(
            expr,
            "x % y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Mod,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn two_plus_two_is_four() {
        check(
            expr,
            "2 + 2 == 4",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 10,
                        },
                        kind: BinOp(
                            Eq,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 5,
                                },
                                kind: BinOp(
                                    Add,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        kind: Lit(
                                            Int(
                                                2,
                                            ),
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        kind: Lit(
                                            Int(
                                                2,
                                            ),
                                        ),
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 10,
                                },
                                kind: Lit(
                                    Int(
                                        4,
                                    ),
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn exp_op() {
        check(
            expr,
            "x ^ y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: BinOp(
                            Exp,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn exp_right_assoc() {
        check(
            expr,
            "2 ^ 3 ^ 4",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: BinOp(
                            Exp,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Lit(
                                    Int(
                                        2,
                                    ),
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 9,
                                },
                                kind: BinOp(
                                    Exp,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        kind: Lit(
                                            Int(
                                                3,
                                            ),
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        kind: Lit(
                                            Int(
                                                4,
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn negate_exp() {
        check(
            expr,
            "-2^3",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: UnOp(
                            Neg,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 4,
                                },
                                kind: BinOp(
                                    Exp,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 2,
                                        },
                                        kind: Lit(
                                            Int(
                                                2,
                                            ),
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 4,
                                        },
                                        kind: Lit(
                                            Int(
                                                3,
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn unwrap_op() {
        check(
            expr,
            "x!",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: UnOp(
                            Unwrap,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn logical_not_op() {
        check(
            expr,
            "not x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: UnOp(
                            NotL,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn bitwise_not_op() {
        check(
            expr,
            "~~~x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: UnOp(
                            NotB,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 4,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 4,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 4,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn pos_op() {
        check(
            expr,
            "+x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: UnOp(
                            Pos,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 2,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 2,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn neg_op() {
        check(
            expr,
            "-x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: UnOp(
                            Neg,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 2,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 2,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 2,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn neg_minus_ops() {
        check(
            expr,
            "-x - y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: BinOp(
                            Sub,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 2,
                                },
                                kind: UnOp(
                                    Neg,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 2,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 2,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 1,
                                                        hi: 2,
                                                    },
                                                    name: "x",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "y",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn adjoint_op() {
        check(
            expr,
            "Adjoint x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: UnOp(
                            Functor(
                                Adj,
                            ),
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 8,
                                    hi: 9,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 8,
                                                hi: 9,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn controlled_op() {
        check(
            expr,
            "Controlled x",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 12,
                        },
                        kind: UnOp(
                            Functor(
                                Ctl,
                            ),
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 11,
                                    hi: 12,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 11,
                                            hi: 12,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 11,
                                                hi: 12,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn update_op() {
        check(
            expr,
            "x w/ i <- v",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 11,
                        },
                        kind: TernOp(
                            Update,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "i",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 11,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 10,
                                            hi: 11,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 10,
                                                hi: 11,
                                            },
                                            name: "v",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn update_op_left_assoc() {
        check(
            expr,
            "x w/ i1 <- v1 w/ i2 <- v2",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 25,
                        },
                        kind: TernOp(
                            Update,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 13,
                                },
                                kind: TernOp(
                                    Update,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 1,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 0,
                                                        hi: 1,
                                                    },
                                                    name: "x",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 7,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 5,
                                                    hi: 7,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 5,
                                                        hi: 7,
                                                    },
                                                    name: "i1",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 11,
                                            hi: 13,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 11,
                                                    hi: 13,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 11,
                                                        hi: 13,
                                                    },
                                                    name: "v1",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 19,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 17,
                                            hi: 19,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 17,
                                                hi: 19,
                                            },
                                            name: "i2",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 23,
                                    hi: 25,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 23,
                                            hi: 25,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 23,
                                                hi: 25,
                                            },
                                            name: "v2",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn cond_op() {
        check(
            expr,
            "c ? a | b",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: TernOp(
                            Cond,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "c",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "a",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 8,
                                    hi: 9,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 9,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 8,
                                                hi: 9,
                                            },
                                            name: "b",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn cond_op_right_assoc() {
        check(
            expr,
            "c1 ? a | c2 ? b | c",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 19,
                        },
                        kind: TernOp(
                            Cond,
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 2,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 2,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 2,
                                            },
                                            name: "c1",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 6,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 5,
                                                hi: 6,
                                            },
                                            name: "a",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 19,
                                },
                                kind: TernOp(
                                    Cond,
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 9,
                                            hi: 11,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 9,
                                                    hi: 11,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 9,
                                                        hi: 11,
                                                    },
                                                    name: "c2",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 15,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 14,
                                                    hi: 15,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 14,
                                                        hi: 15,
                                                    },
                                                    name: "b",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 18,
                                            hi: 19,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 18,
                                                    hi: 19,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 18,
                                                        hi: 19,
                                                    },
                                                    name: "c",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn field_op() {
        check(
            expr,
            "x::foo",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Field(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 6,
                                },
                                name: "foo",
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn index_op() {
        check(
            expr,
            "x[i]",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Index(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "x",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 2,
                                    hi: 3,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 2,
                                            hi: 3,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 2,
                                                hi: 3,
                                            },
                                            name: "i",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn call_op_unit() {
        check(
            expr,
            "Foo()",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: Call(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 3,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 3,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 3,
                                            },
                                            name: "Foo",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 5,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn call_op_one() {
        check(
            expr,
            "Foo(x)",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Call(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 3,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 3,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 3,
                                            },
                                            name: "Foo",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 6,
                                },
                                kind: Paren(
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 4,
                                                    hi: 5,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 4,
                                                        hi: 5,
                                                    },
                                                    name: "x",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn call_op_singleton_tuple() {
        check(
            expr,
            "Foo(x,)",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Call(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 3,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 3,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 3,
                                            },
                                            name: "Foo",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 7,
                                },
                                kind: Tuple(
                                    [
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 4,
                                                        hi: 5,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 4,
                                                            hi: 5,
                                                        },
                                                        name: "x",
                                                    },
                                                },
                                            ),
                                        },
                                    ],
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn call_op_pair() {
        check(
            expr,
            "Foo(x, y)",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: Call(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 3,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 3,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 3,
                                            },
                                            name: "Foo",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 9,
                                },
                                kind: Tuple(
                                    [
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 4,
                                                        hi: 5,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 4,
                                                            hi: 5,
                                                        },
                                                        name: "x",
                                                    },
                                                },
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 7,
                                                hi: 8,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 7,
                                                        hi: 8,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 7,
                                                            hi: 8,
                                                        },
                                                        name: "y",
                                                    },
                                                },
                                            ),
                                        },
                                    ],
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn call_with_array() {
        check(
            expr,
            "f([1, 2])",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: Call(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 1,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            name: "f",
                                        },
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 9,
                                },
                                kind: Paren(
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 2,
                                            hi: 8,
                                        },
                                        kind: Array(
                                            [
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 3,
                                                        hi: 4,
                                                    },
                                                    kind: Lit(
                                                        Int(
                                                            1,
                                                        ),
                                                    ),
                                                },
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 6,
                                                        hi: 7,
                                                    },
                                                    kind: Lit(
                                                        Int(
                                                            2,
                                                        ),
                                                    ),
                                                },
                                            ],
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn call_index_ops() {
        check(
            expr,
            "f()[i]",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Index(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 3,
                                },
                                kind: Call(
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 1,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 1,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 0,
                                                        hi: 1,
                                                    },
                                                    name: "f",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 3,
                                        },
                                        kind: Tuple(
                                            [],
                                        ),
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 4,
                                    hi: 5,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 4,
                                            hi: 5,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            name: "i",
                                        },
                                    },
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn index_call_ops() {
        check(
            expr,
            "fs[i]()",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Call(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 5,
                                },
                                kind: Index(
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 0,
                                            hi: 2,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 2,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 0,
                                                        hi: 2,
                                                    },
                                                    name: "fs",
                                                },
                                            },
                                        ),
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 3,
                                            hi: 4,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 3,
                                                    hi: 4,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 3,
                                                        hi: 4,
                                                    },
                                                    name: "i",
                                                },
                                            },
                                        ),
                                    },
                                ),
                            },
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 7,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_op() {
        check(
            expr,
            "x..y",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Range(
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 1,
                                                },
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                            ),
                            None,
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 3,
                                        hi: 4,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 4,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 3,
                                                    hi: 4,
                                                },
                                                name: "y",
                                            },
                                        },
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_op_with_step() {
        check(
            expr,
            "x..y..z",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Range(
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 1,
                                                },
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                            ),
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 3,
                                        hi: 4,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 4,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 3,
                                                    hi: 4,
                                                },
                                                name: "y",
                                            },
                                        },
                                    ),
                                },
                            ),
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 6,
                                        hi: 7,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 7,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 6,
                                                    hi: 7,
                                                },
                                                name: "z",
                                            },
                                        },
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_complex_stop() {
        check(
            expr,
            "0..Length(xs) - 1",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 17,
                        },
                        kind: Range(
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
                                    },
                                    kind: Lit(
                                        Int(
                                            0,
                                        ),
                                    ),
                                },
                            ),
                            None,
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 3,
                                        hi: 17,
                                    },
                                    kind: BinOp(
                                        Sub,
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 13,
                                            },
                                            kind: Call(
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 3,
                                                        hi: 9,
                                                    },
                                                    kind: Path(
                                                        Path {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 3,
                                                                hi: 9,
                                                            },
                                                            namespace: None,
                                                            name: Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 3,
                                                                    hi: 9,
                                                                },
                                                                name: "Length",
                                                            },
                                                        },
                                                    ),
                                                },
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 9,
                                                        hi: 13,
                                                    },
                                                    kind: Paren(
                                                        Expr {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 10,
                                                                hi: 12,
                                                            },
                                                            kind: Path(
                                                                Path {
                                                                    id: NodeId(
                                                                        4294967295,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 10,
                                                                        hi: 12,
                                                                    },
                                                                    namespace: None,
                                                                    name: Ident {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 10,
                                                                            hi: 12,
                                                                        },
                                                                        name: "xs",
                                                                    },
                                                                },
                                                            ),
                                                        },
                                                    ),
                                                },
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 16,
                                                hi: 17,
                                            },
                                            kind: Lit(
                                                Int(
                                                    1,
                                                ),
                                            ),
                                        },
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_complex_start() {
        check(
            expr,
            "i + 1..n",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 8,
                        },
                        kind: Range(
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 5,
                                    },
                                    kind: BinOp(
                                        Add,
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 0,
                                                hi: 1,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 0,
                                                        hi: 1,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 0,
                                                            hi: 1,
                                                        },
                                                        name: "i",
                                                    },
                                                },
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 4,
                                                hi: 5,
                                            },
                                            kind: Lit(
                                                Int(
                                                    1,
                                                ),
                                            ),
                                        },
                                    ),
                                },
                            ),
                            None,
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 7,
                                        hi: 8,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 7,
                                                hi: 8,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 7,
                                                    hi: 8,
                                                },
                                                name: "n",
                                            },
                                        },
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_complex_step() {
        check(
            expr,
            "0..s + 1..n",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 11,
                        },
                        kind: Range(
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
                                    },
                                    kind: Lit(
                                        Int(
                                            0,
                                        ),
                                    ),
                                },
                            ),
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 3,
                                        hi: 8,
                                    },
                                    kind: BinOp(
                                        Add,
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 4,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 3,
                                                        hi: 4,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 3,
                                                            hi: 4,
                                                        },
                                                        name: "s",
                                                    },
                                                },
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 7,
                                                hi: 8,
                                            },
                                            kind: Lit(
                                                Int(
                                                    1,
                                                ),
                                            ),
                                        },
                                    ),
                                },
                            ),
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 10,
                                        hi: 11,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 10,
                                                hi: 11,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 10,
                                                    hi: 11,
                                                },
                                                name: "n",
                                            },
                                        },
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_start_open() {
        check(
            expr,
            "2...",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Range(
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
                                    },
                                    kind: Lit(
                                        Int(
                                            2,
                                        ),
                                    ),
                                },
                            ),
                            None,
                            None,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_stop_open() {
        check(
            expr,
            "...2",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Range(
                            None,
                            None,
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 3,
                                        hi: 4,
                                    },
                                    kind: Lit(
                                        Int(
                                            2,
                                        ),
                                    ),
                                },
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn range_step_open() {
        check(
            expr,
            "...2...",
            &expect![[r#"
                Ok(
                    Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Range(
                            None,
                            Some(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 3,
                                        hi: 4,
                                    },
                                    kind: Lit(
                                        Int(
                                            2,
                                        ),
                                    ),
                                },
                            ),
                            None,
                        ),
                    },
                )
            "#]],
        );
    }
}
