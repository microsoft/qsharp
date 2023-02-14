// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{kw, scan::Scanner, ty::ty, Parser, Result};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{Ident, NodeId, Pat, PatKind, Path, Span};

pub(super) fn token(s: &mut Scanner, kind: TokenKind) -> Result<()> {
    if s.peek().kind == kind {
        s.advance();
        Ok(())
    } else {
        Err(s.error(format!("Expecting {kind:?}.")))
    }
}

pub(super) fn keyword(s: &mut Scanner, kw: &str) -> Result<()> {
    if kw::is_keyword(kw) && s.peek().kind == TokenKind::Ident && s.read() == kw {
        s.advance();
        Ok(())
    } else {
        Err(s.error(format!("Expecting keyword `{kw}`.")))
    }
}

pub(super) fn ident(s: &mut Scanner) -> Result<Ident> {
    if s.peek().kind != TokenKind::Ident || kw::is_keyword(s.read()) {
        return Err(s.error("Expecting identifier.".to_string()));
    }

    let span = s.peek().span;
    let name = s.read().to_string();
    s.advance();
    Ok(Ident {
        id: NodeId::PLACEHOLDER,
        span,
        name,
    })
}

pub(super) fn path(s: &mut Scanner) -> Result<Path> {
    let lo = s.peek().span.lo;
    let mut parts = vec![ident(s)?];
    while token(s, TokenKind::Dot).is_ok() {
        parts.push(ident(s)?);
    }

    let name = parts.pop().unwrap();
    let namespace = if parts.is_empty() {
        None
    } else {
        let lo = parts.first().unwrap().span.lo;
        let hi = parts.last().unwrap().span.hi;
        let names: Vec<_> = parts.into_iter().map(|i| i.name).collect();
        Some(Ident {
            id: NodeId::PLACEHOLDER,
            span: Span { lo, hi },
            name: names.join("."),
        })
    };

    Ok(Path {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        namespace,
        name,
    })
}

pub(super) fn pat(s: &mut Scanner) -> Result<Pat> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, kw::UNDERSCORE).is_ok() {
        let ty = if token(s, TokenKind::Colon).is_ok() {
            Some(ty(s)?)
        } else {
            None
        };
        Ok(PatKind::Discard(ty))
    } else if token(s, TokenKind::DotDotDot).is_ok() {
        Ok(PatKind::Elided)
    } else if token(s, TokenKind::Open(Delim::Paren)).is_ok() {
        let pats = seq(s, pat)?;
        token(s, TokenKind::Close(Delim::Paren))?;
        Ok(PatKind::Tuple(pats))
    } else if let Some(name) = opt(s, ident)? {
        let ty = if token(s, TokenKind::Colon).is_ok() {
            Some(ty(s)?)
        } else {
            None
        };
        Ok(PatKind::Bind(name, ty))
    } else {
        Err(s.error("Expecting pattern.".to_string()))
    }?;

    Ok(Pat {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
    })
}

pub(super) fn opt<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Option<T>> {
    let offset = s.peek().span.lo;
    match p(s) {
        Ok(x) => Ok(Some(x)),
        Err(_) if offset == s.peek().span.lo => Ok(None),
        Err(err) => Err(err),
    }
}

pub(super) fn many<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Vec<T>> {
    let mut xs = Vec::new();
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
    }
    Ok(xs)
}

pub(super) fn seq<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Vec<T>> {
    let mut xs = Vec::new();
    while let Some(x) = opt(s, &mut p)? {
        xs.push(x);
        if token(s, TokenKind::Comma).is_err() {
            break;
        }
    }
    Ok(xs)
}

#[cfg(test)]
mod tests {
    use super::{ident, opt, pat, path, seq};
    use crate::parse::{scan::Scanner, Parser};
    use expect_test::{expect, Expect};
    use std::fmt::Debug;

    fn check<T: Debug>(mut parser: impl Parser<T>, input: &str, expect: &Expect) {
        let mut scanner = Scanner::new(input);
        let actual = parser(&mut scanner);
        expect.assert_debug_eq(&actual);
    }

    #[test]
    fn ident_basic() {
        check(
            ident,
            "foo",
            &expect![[r#"
                Ok(
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
                )
            "#]],
        );
    }

    #[test]
    fn ident_num_suffix() {
        check(
            ident,
            "foo2",
            &expect![[r#"
                Ok(
                    Ident {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        name: "foo2",
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ident_underscore_prefix() {
        check(
            ident,
            "_foo",
            &expect![[r#"
                Ok(
                    Ident {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 4,
                        },
                        name: "_foo",
                    },
                )
            "#]],
        );
    }

    #[test]
    fn ident_num_prefix() {
        check(
            ident,
            "2foo",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting identifier.",
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
    fn path_single() {
        check(
            path,
            "Foo",
            &expect![[r#"
                Ok(
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
                )
            "#]],
        );
    }

    #[test]
    fn path_double() {
        check(
            path,
            "Foo.Bar",
            &expect![[r#"
                Ok(
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
                )
            "#]],
        );
    }

    #[test]
    fn path_triple() {
        check(
            path,
            "Foo.Bar.Baz",
            &expect![[r#"
                Ok(
                    Path {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 11,
                        },
                        namespace: Some(
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 7,
                                },
                                name: "Foo.Bar",
                            },
                        ),
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Baz",
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn path_trailing_dot() {
        check(
            path,
            "Foo.Bar.",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting identifier.",
                        span: Span {
                            lo: 8,
                            hi: 8,
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn pat_bind() {
        check(
            pat,
            "foo",
            &expect![[r#"
                Ok(
                    Pat {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 3,
                        },
                        kind: Bind(
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
                            None,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn pat_bind_ty() {
        check(
            pat,
            "foo : Int",
            &expect![[r#"
                Ok(
                    Pat {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 9,
                        },
                        kind: Bind(
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
                            Some(
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 6,
                                        hi: 9,
                                    },
                                    kind: Prim(
                                        Int,
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
    fn pat_bind_discard() {
        check(
            pat,
            "_",
            &expect![[r#"
                Ok(
                    Pat {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 1,
                        },
                        kind: Discard(
                            None,
                        ),
                    },
                )
            "#]],
        );
    }

    #[test]
    fn pat_discard_ty() {
        check(
            pat,
            "_ : Int",
            &expect![[r#"
                Ok(
                    Pat {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 7,
                        },
                        kind: Discard(
                            Some(
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 4,
                                        hi: 7,
                                    },
                                    kind: Prim(
                                        Int,
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
    fn pat_tuple() {
        check(
            pat,
            "(foo, bar)",
            &expect![[r#"
                Ok(
                    Pat {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 10,
                        },
                        kind: Tuple(
                            [
                                Pat {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 4,
                                    },
                                    kind: Bind(
                                        Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 4,
                                            },
                                            name: "foo",
                                        },
                                        None,
                                    ),
                                },
                                Pat {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 6,
                                        hi: 9,
                                    },
                                    kind: Bind(
                                        Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 9,
                                            },
                                            name: "bar",
                                        },
                                        None,
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
    fn pat_tuple_ty_discard() {
        check(
            pat,
            "(foo : Int, _)",
            &expect![[r#"
                Ok(
                    Pat {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 14,
                        },
                        kind: Tuple(
                            [
                                Pat {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 10,
                                    },
                                    kind: Bind(
                                        Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 4,
                                            },
                                            name: "foo",
                                        },
                                        Some(
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 7,
                                                    hi: 10,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                        ),
                                    ),
                                },
                                Pat {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 12,
                                        hi: 13,
                                    },
                                    kind: Discard(
                                        None,
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
    fn pat_invalid() {
        check(
            pat,
            "@",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting pattern.",
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
    fn pat_missing_ty() {
        check(
            pat,
            "foo :",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting type.",
                        span: Span {
                            lo: 5,
                            hi: 5,
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn opt_succeed() {
        check(
            |s| opt(s, path),
            "Foo.Bar",
            &expect![[r#"
                Ok(
                    Some(
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
                )
            "#]],
        );
    }

    #[test]
    fn opt_fail_no_consume() {
        check(
            |s| opt(s, path),
            "123",
            &expect![[r#"
                Ok(
                    None,
                )
            "#]],
        );
    }

    #[test]
    fn opt_fail_consume() {
        check(
            |s| opt(s, path),
            "Foo.#",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting identifier.",
                        span: Span {
                            lo: 5,
                            hi: 5,
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn seq_empty() {
        check(
            |s| seq(s, ident),
            "",
            &expect![[r#"
                Ok(
                    [],
                )
            "#]],
        );
    }

    #[test]
    fn seq_single() {
        check(
            |s| seq(s, ident),
            "foo",
            &expect![[r#"
                Ok(
                    [
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
                    ],
                )
            "#]],
        );
    }

    #[test]
    fn seq_double() {
        check(
            |s| seq(s, ident),
            "foo, bar",
            &expect![[r#"
                Ok(
                    [
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
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 8,
                            },
                            name: "bar",
                        },
                    ],
                )
            "#]],
        );
    }

    #[test]
    fn seq_trailing() {
        check(
            |s| seq(s, ident),
            "foo, bar,",
            &expect![[r#"
                Ok(
                    [
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
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 8,
                            },
                            name: "bar",
                        },
                    ],
                )
            "#]],
        );
    }

    #[test]
    fn seq_fail_no_consume() {
        check(
            |s| seq(s, ident),
            "foo, 2",
            &expect![[r#"
                Ok(
                    [
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
                    ],
                )
            "#]],
        );
    }

    #[test]
    fn seq_fail_consume() {
        check(
            |s| seq(s, path),
            "foo, bar.",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting identifier.",
                        span: Span {
                            lo: 9,
                            hi: 9,
                        },
                    },
                )
            "#]],
        );
    }
}
