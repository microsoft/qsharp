// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{scan::Scanner, ty::ty, Parser, Result};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{Ident, NodeId, Pat, PatKind, Path, Span};

pub(super) fn ident(s: &mut Scanner) -> Result<Ident> {
    let name = s.ident()?.to_string();
    Ok(Ident {
        id: NodeId::PLACEHOLDER,
        span: s.span(),
        name,
    })
}

pub(super) fn path(s: &mut Scanner) -> Result<Path> {
    let lo = s.span().lo;
    let mut parts = vec![ident(s)?];
    while s.expect(TokenKind::Dot).is_ok() {
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

    let hi = s.span().hi;
    Ok(Path {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        namespace,
        name,
    })
}

pub(super) fn pat(s: &mut Scanner) -> Result<Pat> {
    let lo = s.span().lo;
    let kind = if let Some(name) = opt(s, ident)? {
        let ty = if s.expect(TokenKind::Colon).is_ok() {
            Some(ty(s)?)
        } else {
            None
        };
        if name.name == "_" {
            Ok(PatKind::Discard(ty))
        } else {
            Ok(PatKind::Bind(name, ty))
        }
    } else if s.expect(TokenKind::DotDotDot).is_ok() {
        Ok(PatKind::Elided)
    } else if s.expect(TokenKind::Open(Delim::Paren)).is_ok() {
        let pats = comma_sep(s, pat);
        s.expect(TokenKind::Close(Delim::Paren))?;
        Ok(PatKind::Tuple(pats))
    } else {
        Err(s.error("Expecting pattern.".to_string()))
    }?;

    let hi = s.span().hi;
    Ok(Pat {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
    })
}

pub(super) fn opt<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Result<Option<T>> {
    let span = s.span();
    match p(s) {
        Ok(x) => Ok(Some(x)),
        Err(_) if span == s.span() => Ok(None),
        Err(err) => Err(err),
    }
}

pub(super) fn comma_sep<T>(s: &mut Scanner, mut p: impl Parser<T>) -> Vec<T> {
    let mut items = Vec::new();
    while let Ok(item) = p(s) {
        items.push(item);
        if s.expect(TokenKind::Comma).is_err() {
            break;
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::{comma_sep, ident, opt, pat, path};
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
                                        lo: 0,
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
                                        lo: 4,
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
                                        lo: 0,
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
                                        lo: 10,
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
    fn comma_sep_empty() {
        check(
            |s| Ok(comma_sep(s, ident)),
            "",
            &expect![[r#"
                Ok(
                    [],
                )
            "#]],
        );
    }

    #[test]
    fn comma_sep_single() {
        check(
            |s| Ok(comma_sep(s, ident)),
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
    fn comma_sep_double() {
        check(
            |s| Ok(comma_sep(s, ident)),
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
    fn comma_sep_trailing() {
        check(
            |s| Ok(comma_sep(s, ident)),
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
    fn comma_sep_item_fail() {
        check(
            |s| Ok(comma_sep(s, ident)),
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
}
