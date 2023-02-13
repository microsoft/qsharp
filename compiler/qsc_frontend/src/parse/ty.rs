// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    kw,
    prim::{comma_sep, ident, opt, path},
    scan::Scanner,
    Result,
};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{CallableKind, Ident, NodeId, Span, Ty, TyKind, TyPrim, TyVar};

pub(super) fn ty(s: &mut Scanner) -> Result<Ty> {
    let lo = s.span().lo;
    let mut acc = base(s)?;
    loop {
        if let Some(array) = opt(s, array)? {
            let hi = s.span().hi;
            acc = Ty {
                id: NodeId::PLACEHOLDER,
                span: Span { lo, hi },
                kind: TyKind::App(Box::new(array), vec![acc]),
            }
        } else if s.expect(TokenKind::RArrow).is_ok() {
            let output = ty(s)?;
            let hi = s.span().hi;
            acc = Ty {
                id: NodeId::PLACEHOLDER,
                span: Span { lo, hi },
                kind: TyKind::Arrow(
                    CallableKind::Function,
                    Box::new(acc),
                    Box::new(output),
                    None,
                ),
            }
        } else if s.expect(TokenKind::FatArrow).is_ok() {
            let output = ty(s)?;
            let hi = s.span().hi;
            acc = Ty {
                id: NodeId::PLACEHOLDER,
                span: Span { lo, hi },
                kind: TyKind::Arrow(
                    CallableKind::Operation,
                    Box::new(acc),
                    Box::new(output),
                    None,
                ),
            }
        } else {
            return Ok(acc);
        }
    }
}

pub(super) fn var(s: &mut Scanner) -> Result<Ident> {
    s.expect(TokenKind::Apos)?;
    ident(s)
}

fn base(s: &mut Scanner) -> Result<Ty> {
    let lo = s.peek().span.lo;
    let kind = if s.expect(TokenKind::Open(Delim::Paren)).is_ok() {
        let tys = comma_sep(s, ty)?;
        s.expect(TokenKind::Close(Delim::Paren))?;
        Ok(TyKind::Tuple(tys))
    } else if s.keyword(kw::UNDERSCORE).is_ok() {
        Ok(TyKind::Hole)
    } else if s.keyword(kw::BIG_INT).is_ok() {
        Ok(TyKind::Prim(TyPrim::BigInt))
    } else if s.keyword(kw::BOOL).is_ok() {
        Ok(TyKind::Prim(TyPrim::Bool))
    } else if s.keyword(kw::DOUBLE).is_ok() {
        Ok(TyKind::Prim(TyPrim::Double))
    } else if s.keyword(kw::INT).is_ok() {
        Ok(TyKind::Prim(TyPrim::Int))
    } else if s.keyword(kw::PAULI).is_ok() {
        Ok(TyKind::Prim(TyPrim::Pauli))
    } else if s.keyword(kw::QUBIT).is_ok() {
        Ok(TyKind::Prim(TyPrim::Qubit))
    } else if s.keyword(kw::RANGE).is_ok() {
        Ok(TyKind::Prim(TyPrim::Range))
    } else if s.keyword(kw::RESULT).is_ok() {
        Ok(TyKind::Prim(TyPrim::Result))
    } else if s.keyword(kw::STRING).is_ok() {
        Ok(TyKind::Prim(TyPrim::String))
    } else if s.keyword(kw::UNIT).is_ok() {
        Ok(TyKind::Tuple(Vec::new()))
    } else if let Some(var) = opt(s, var)? {
        Ok(TyKind::Var(TyVar::Name(var.name)))
    } else if let Some(path) = opt(s, path)? {
        Ok(TyKind::Path(path))
    } else {
        Err(s.error("Expecting type.".to_string()))
    }?;

    let hi = s.span().hi;
    Ok(Ty {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
    })
}

fn array(s: &mut Scanner) -> Result<Ty> {
    s.expect(TokenKind::Open(Delim::Bracket))?;
    let lo = s.span().lo;
    s.expect(TokenKind::Close(Delim::Bracket))?;
    let hi = s.span().hi;
    Ok(Ty {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind: TyKind::Prim(TyPrim::Array),
    })
}

#[cfg(test)]
mod tests {
    use super::ty;
    use crate::parse::{scan::Scanner, Parser};
    use expect_test::{expect, Expect};
    use qsc_ast::ast::Ty;

    fn check(mut parser: impl Parser<Ty>, input: &str, expect: &Expect) {
        let mut scanner = Scanner::new(input);
        let actual = parser(&mut scanner);
        expect.assert_debug_eq(&actual);
    }

    #[test]
    fn ty_big_int() {
        check(
            ty,
            "BigInt",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Prim(
                            BigInt,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_bool() {
        check(
            ty,
            "Bool",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        kind: Prim(
                            Bool,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_double() {
        check(
            ty,
            "Double",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Prim(
                            Double,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_int() {
        check(
            ty,
            "Int",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Prim(
                            Int,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_pauli() {
        check(
            ty,
            "Pauli",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: Prim(
                            Pauli,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_qubit() {
        check(
            ty,
            "Qubit",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: Prim(
                            Qubit,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_range() {
        check(
            ty,
            "Range",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: Prim(
                            Range,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_result() {
        check(
            ty,
            "Result",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Prim(
                            Result,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_string() {
        check(
            ty,
            "String",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 6,
                        },
                        kind: Prim(
                            String,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_unit() {
        check(
            ty,
            "Unit",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
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
    fn ty_var() {
        check(
            ty,
            "'T",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 2,
                        },
                        kind: Var(
                            Name(
                                "T",
                            ),
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_hole() {
        check(
            ty,
            "_",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 1,
                        },
                        kind: Hole,
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_path() {
        check(
            ty,
            "Foo",
            &expect![[r#"
                Ok(
                    Ty {
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
                )
            "#]],
        );
    }

    #[test]
    fn ty_path2() {
        check(
            ty,
            "Foo.Bar",
            &expect![[r#"
                Ok(
                    Ty {
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
                                        name: "Foo",
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
                                    name: "Bar",
                                },
                            },
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ty_tuple() {
        check(
            ty,
            "(Int, Bool)",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 11,
                        },
                        kind: Tuple(
                            [
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 4,
                                    },
                                    kind: Prim(
                                        Int,
                                    ),
                                },
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 6,
                                        hi: 10,
                                    },
                                    kind: Prim(
                                        Bool,
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
    fn ty_tuple2() {
        check(
            ty,
            "((Int, Bool), Double)",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 21,
                        },
                        kind: Tuple(
                            [
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 12,
                                    },
                                    kind: Tuple(
                                        [
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 2,
                                                    hi: 5,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 7,
                                                    hi: 11,
                                                },
                                                kind: Prim(
                                                    Bool,
                                                ),
                                            },
                                        ],
                                    ),
                                },
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 14,
                                        hi: 20,
                                    },
                                    kind: Prim(
                                        Double,
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
    fn ty_array() {
        check(
            ty,
            "Int[]",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 5,
                        },
                        kind: App(
                            Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 3,
                                    hi: 5,
                                },
                                kind: Prim(
                                    Array,
                                ),
                            },
                            [
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 3,
                                    },
                                    kind: Prim(
                                        Int,
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
    fn ty_array2() {
        check(
            ty,
            "Int[][]",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: App(
                            Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 7,
                                },
                                kind: Prim(
                                    Array,
                                ),
                            },
                            [
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 5,
                                    },
                                    kind: App(
                                        Ty {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 3,
                                                hi: 5,
                                            },
                                            kind: Prim(
                                                Array,
                                            ),
                                        },
                                        [
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 0,
                                                    hi: 3,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                        ],
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
    fn ty_tuple_array() {
        check(
            ty,
            "(Int, Bool)[]",
            &expect![[r#"
                Ok(
                    Ty {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 13,
                        },
                        kind: App(
                            Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 11,
                                    hi: 13,
                                },
                                kind: Prim(
                                    Array,
                                ),
                            },
                            [
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 11,
                                    },
                                    kind: Tuple(
                                        [
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 4,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 6,
                                                    hi: 10,
                                                },
                                                kind: Prim(
                                                    Bool,
                                                ),
                                            },
                                        ],
                                    ),
                                },
                            ],
                        ),
                    },
                )
            "#]],
        );
    }
}
