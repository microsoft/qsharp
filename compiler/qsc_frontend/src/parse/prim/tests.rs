// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{ident, opt, pat, path, seq};
use crate::parse::{scan::Scanner, tests::check, Error, Keyword};
use expect_test::expect;
use qsc_ast::ast::Span;

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
                    kind: Rule(
                        "identifier",
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
fn ident_keyword() {
    for keyword in enum_iterator::all::<Keyword>() {
        let mut scanner = Scanner::new(keyword.as_str());
        let actual = ident(&mut scanner);
        let expected = Err(Error::RuleKeyword(
            "identifier",
            keyword,
            Span {
                lo: 0,
                hi: keyword.as_str().len(),
            },
        ));
        assert_eq!(actual, expected, "{keyword}");
    }
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
                    kind: Rule(
                        "identifier",
                    ),
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
fn pat_paren() {
    check(
        pat,
        "(foo)",
        &expect![[r#"
            Ok(
                Pat {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 5,
                    },
                    kind: Paren(
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
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn pat_singleton_tuple() {
    check(
        pat,
        "(foo,)",
        &expect![[r#"
            Ok(
                Pat {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 6,
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
                        ],
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
                    kind: Rule(
                        "pattern",
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
fn pat_missing_ty() {
    check(
        pat,
        "foo :",
        &expect![[r#"
            Err(
                Error {
                    kind: Rule(
                        "type",
                    ),
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
                    kind: Rule(
                        "identifier",
                    ),
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
                (
                    [],
                    Missing,
                ),
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
                (
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
                    Missing,
                ),
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
                (
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
                    Missing,
                ),
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
                (
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
                    Present,
                ),
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
                (
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
                    Present,
                ),
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
                    kind: Rule(
                        "identifier",
                    ),
                    span: Span {
                        lo: 9,
                        hi: 9,
                    },
                },
            )
        "#]],
    );
}
