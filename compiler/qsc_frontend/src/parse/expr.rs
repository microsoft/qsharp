// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    keyword::Keyword,
    prim::{keyword, opt, pat, path, token},
    scan::Scanner,
    stmt, ErrorKind, Result,
};
use crate::lex::{ClosedBinOp, TokenKind};
use qsc_ast::ast::{self, BinOp, Expr, ExprKind, Functor, Lit, NodeId, Pauli, UnOp};

pub(super) fn expr(s: &mut Scanner) -> Result<Expr> {
    expr_bp(s, 0)
}

fn expr_bp(s: &mut Scanner, min_bp: u8) -> Result<Expr> {
    let lo = s.peek().span.lo;
    let mut lhs = if let Some((op, ((), r_bp))) = prefix_op(s.peek().kind, s.read()) {
        s.advance();
        let rhs = expr_bp(s, r_bp)?;
        Expr {
            id: NodeId::PLACEHOLDER,
            span: s.span(lo),
            kind: ExprKind::UnOp(op, Box::new(rhs)),
        }
    } else {
        expr_base(s)?
    };

    loop {
        if let Some((op, (l_bp, ()))) = postfix_op(s.peek().kind) {
            if l_bp < min_bp {
                break;
            }

            s.advance();
            lhs = Expr {
                id: NodeId::PLACEHOLDER,
                span: s.span(lo),
                kind: ExprKind::UnOp(op, Box::new(lhs)),
            };
            continue;
        }

        if let Some((op, (l_bp, r_bp))) = infix_op(s.peek().kind) {
            if l_bp < min_bp {
                break;
            }

            s.advance();
            let rhs = expr_bp(s, r_bp)?;
            lhs = Expr {
                id: NodeId::PLACEHOLDER,
                span: s.span(lo),
                kind: ExprKind::BinOp(op, Box::new(lhs), Box::new(rhs)),
            };
            continue;
        }

        break;
    }

    Ok(lhs)
}

fn prefix_op(token: TokenKind, lexeme: &str) -> Option<(UnOp, ((), u8))> {
    match token {
        TokenKind::Ident if lexeme == Keyword::Not.as_str() => Some((UnOp::NotB, ((), 13))),
        TokenKind::TildeTildeTilde => Some((UnOp::NotB, ((), 13))),
        TokenKind::ClosedBinOp(ClosedBinOp::Plus) => Some((UnOp::Pos, ((), 13))),
        TokenKind::ClosedBinOp(ClosedBinOp::Minus) => Some((UnOp::Neg, ((), 13))),
        TokenKind::Ident if lexeme == Keyword::Adjoint.as_str() => {
            Some((UnOp::Functor(Functor::Adj), ((), 16)))
        }
        TokenKind::Ident if lexeme == Keyword::Controlled.as_str() => {
            Some((UnOp::Functor(Functor::Ctl), ((), 16)))
        }
        _ => None,
    }
}

fn postfix_op(token: TokenKind) -> Option<(UnOp, (u8, ()))> {
    match token {
        TokenKind::Bang => Some((UnOp::Unwrap, (17, ()))),
        _ => None,
    }
}

fn infix_op(token: TokenKind) -> Option<(BinOp, (u8, u8))> {
    match token {
        TokenKind::ClosedBinOp(ClosedBinOp::Or) => Some((BinOp::OrL, (1, 2))),
        TokenKind::ClosedBinOp(ClosedBinOp::And) => Some((BinOp::AndL, (3, 4))),
        TokenKind::EqEq => Some((BinOp::Eq, (5, 6))),
        TokenKind::Ne => Some((BinOp::Neq, (5, 6))),
        TokenKind::Gt => Some((BinOp::Gt, (5, 6))),
        TokenKind::Gte => Some((BinOp::Gte, (5, 6))),
        TokenKind::Lt => Some((BinOp::Lt, (5, 6))),
        TokenKind::Lte => Some((BinOp::Lte, (5, 6))),
        TokenKind::ClosedBinOp(ClosedBinOp::AmpAmpAmp) => Some((BinOp::AndB, (7, 8))),
        TokenKind::ClosedBinOp(ClosedBinOp::BarBarBar) => Some((BinOp::OrB, (7, 8))),
        TokenKind::ClosedBinOp(ClosedBinOp::CaretCaretCaret) => Some((BinOp::XorB, (7, 8))),
        TokenKind::ClosedBinOp(ClosedBinOp::LtLtLt) => Some((BinOp::Shl, (7, 8))),
        TokenKind::ClosedBinOp(ClosedBinOp::GtGtGt) => Some((BinOp::Shr, (7, 8))),
        TokenKind::ClosedBinOp(ClosedBinOp::Plus) => Some((BinOp::Add, (9, 10))),
        TokenKind::ClosedBinOp(ClosedBinOp::Minus) => Some((BinOp::Sub, (9, 10))),
        TokenKind::ClosedBinOp(ClosedBinOp::Star) => Some((BinOp::Mul, (11, 12))),
        TokenKind::ClosedBinOp(ClosedBinOp::Slash) => Some((BinOp::Div, (11, 12))),
        TokenKind::ClosedBinOp(ClosedBinOp::Percent) => Some((BinOp::Mod, (11, 12))),
        TokenKind::ClosedBinOp(ClosedBinOp::Caret) => Some((BinOp::Exp, (15, 14))),
        _ => None,
    }
}

fn expr_base(s: &mut Scanner) -> Result<Expr> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Fail).is_ok() {
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
                            Double(
                                0.23,
                            ),
                        ),
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
}
