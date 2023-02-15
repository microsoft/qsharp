// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    keyword::Keyword,
    prim::{keyword, opt, pat, path, token},
    scan::Scanner,
    stmt, ErrorKind, Result,
};
use crate::lex::TokenKind;
use qsc_ast::ast::{self, Expr, ExprKind, Lit, NodeId, Pauli};

pub(super) fn expr(s: &mut Scanner) -> Result<Expr> {
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
}
