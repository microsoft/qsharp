// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::{block, stmt};
use crate::parse::tests::check;
use expect_test::expect;

#[test]
fn let_stmt() {
    check(
        stmt,
        "let x = 2;",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 10,
                    },
                    kind: Local(
                        Immutable,
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
                                lo: 8,
                                hi: 9,
                            },
                            kind: Lit(
                                Int(
                                    2,
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
fn let_pat_match() {
    check(
        stmt,
        "let (x, (y, z)) = foo;",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 22,
                    },
                    kind: Local(
                        Immutable,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 4,
                                hi: 15,
                            },
                            kind: Tuple(
                                [
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 5,
                                                    hi: 6,
                                                },
                                                name: "x",
                                            },
                                            None,
                                        ),
                                    },
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 14,
                                        },
                                        kind: Tuple(
                                            [
                                                Pat {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 9,
                                                        hi: 10,
                                                    },
                                                    kind: Bind(
                                                        Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 9,
                                                                hi: 10,
                                                            },
                                                            name: "y",
                                                        },
                                                        None,
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
                                                    kind: Bind(
                                                        Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 12,
                                                                hi: 13,
                                                            },
                                                            name: "z",
                                                        },
                                                        None,
                                                    ),
                                                },
                                            ],
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
                                lo: 18,
                                hi: 21,
                            },
                            kind: Path(
                                Path {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 18,
                                        hi: 21,
                                    },
                                    namespace: None,
                                    name: Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 18,
                                            hi: 21,
                                        },
                                        name: "foo",
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
fn mutable_stmt() {
    check(
        stmt,
        "mutable x = 2;",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 14,
                    },
                    kind: Local(
                        Mutable,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 9,
                            },
                            kind: Bind(
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 8,
                                        hi: 9,
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
                                lo: 12,
                                hi: 13,
                            },
                            kind: Lit(
                                Int(
                                    2,
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
fn use_stmt() {
    check(
        stmt,
        "use q = Qubit();",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 16,
                    },
                    kind: Qubit(
                        Fresh,
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
                                    name: "q",
                                },
                                None,
                            ),
                        },
                        QubitInit {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 15,
                            },
                            kind: Single,
                        },
                        None,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn use_qubit_array() {
    check(
        stmt,
        "use qs = Qubit[5];",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 18,
                    },
                    kind: Qubit(
                        Fresh,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 4,
                                hi: 6,
                            },
                            kind: Bind(
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 4,
                                        hi: 6,
                                    },
                                    name: "qs",
                                },
                                None,
                            ),
                        },
                        QubitInit {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 17,
                            },
                            kind: Array(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 15,
                                        hi: 16,
                                    },
                                    kind: Lit(
                                        Int(
                                            5,
                                        ),
                                    ),
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
fn use_pat_match() {
    check(
        stmt,
        "use (q1, q2) = (Qubit(), Qubit());",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 34,
                    },
                    kind: Qubit(
                        Fresh,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 4,
                                hi: 12,
                            },
                            kind: Tuple(
                                [
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 7,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 5,
                                                    hi: 7,
                                                },
                                                name: "q1",
                                            },
                                            None,
                                        ),
                                    },
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 9,
                                            hi: 11,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 9,
                                                    hi: 11,
                                                },
                                                name: "q2",
                                            },
                                            None,
                                        ),
                                    },
                                ],
                            ),
                        },
                        QubitInit {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 15,
                                hi: 33,
                            },
                            kind: Tuple(
                                [
                                    QubitInit {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 16,
                                            hi: 23,
                                        },
                                        kind: Single,
                                    },
                                    QubitInit {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 25,
                                            hi: 32,
                                        },
                                        kind: Single,
                                    },
                                ],
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
fn use_paren() {
    check(
        stmt,
        "use q = (Qubit());",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 18,
                    },
                    kind: Qubit(
                        Fresh,
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
                                    name: "q",
                                },
                                None,
                            ),
                        },
                        QubitInit {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 17,
                            },
                            kind: Paren(
                                QubitInit {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 9,
                                        hi: 16,
                                    },
                                    kind: Single,
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
fn use_single_tuple() {
    check(
        stmt,
        "use (q,) = (Qubit(),);",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 22,
                    },
                    kind: Qubit(
                        Fresh,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 4,
                                hi: 8,
                            },
                            kind: Tuple(
                                [
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 5,
                                                    hi: 6,
                                                },
                                                name: "q",
                                            },
                                            None,
                                        ),
                                    },
                                ],
                            ),
                        },
                        QubitInit {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 11,
                                hi: 21,
                            },
                            kind: Tuple(
                                [
                                    QubitInit {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 12,
                                            hi: 19,
                                        },
                                        kind: Single,
                                    },
                                ],
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
fn borrow_stmt() {
    check(
        stmt,
        "borrow q = Qubit();",
        &expect![[r#"
            Ok(
                Stmt {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 19,
                    },
                    kind: Qubit(
                        Dirty,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 7,
                                hi: 8,
                            },
                            kind: Bind(
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 7,
                                        hi: 8,
                                    },
                                    name: "q",
                                },
                                None,
                            ),
                        },
                        QubitInit {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 11,
                                hi: 18,
                            },
                            kind: Single,
                        },
                        None,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn let_in_block() {
    check(
        block,
        "{ let x = 2; x }",
        &expect![[r#"
            Ok(
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
                            kind: Local(
                                Immutable,
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
                                            2,
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
            )
        "#]],
    );
}

#[test]
fn exprs_in_block() {
    check(
        block,
        "{ x; y; z }",
        &expect![[r#"
            Ok(
                Block {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 11,
                    },
                    stmts: [
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 2,
                                hi: 4,
                            },
                            kind: Semi(
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
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                            ),
                        },
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 7,
                            },
                            kind: Semi(
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
                                                name: "z",
                                            },
                                        },
                                    ),
                                },
                            ),
                        },
                    ],
                },
            )
        "#]],
    );
}

#[test]
fn trailing_semi_expr() {
    check(
        block,
        "{ x; y; z; }",
        &expect![[r#"
            Ok(
                Block {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 12,
                    },
                    stmts: [
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 2,
                                hi: 4,
                            },
                            kind: Semi(
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
                                                name: "x",
                                            },
                                        },
                                    ),
                                },
                            ),
                        },
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 7,
                            },
                            kind: Semi(
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
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 10,
                            },
                            kind: Semi(
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
                    ],
                },
            )
        "#]],
    );
}

#[test]
fn stmt_missing_semi() {
    check(
        stmt,
        "let x = 2",
        &expect![[r#"
            Err(
                Token(
                    Semi,
                    Eof,
                    Span {
                        lo: 9,
                        hi: 9,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn if_followed_by() {
    check(
        block,
        "{ if c { x } return x; }",
        &expect![[r#"
            Ok(
                Block {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 24,
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
                            kind: Expr(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 2,
                                        hi: 12,
                                    },
                                    kind: If(
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
                                        None,
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
                                hi: 22,
                            },
                            kind: Semi(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 13,
                                        hi: 21,
                                    },
                                    kind: Return(
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 20,
                                                hi: 21,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 20,
                                                        hi: 21,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 20,
                                                            hi: 21,
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
                    ],
                },
            )
        "#]],
    );
}

#[test]
fn let_if() {
    check(
        block,
        "{ let x = if c { true } else { false }; x }",
        &expect![[r#"
            Ok(
                Block {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 43,
                    },
                    stmts: [
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 2,
                                hi: 39,
                            },
                            kind: Local(
                                Immutable,
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
                                        hi: 38,
                                    },
                                    kind: If(
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
                                                lo: 15,
                                                hi: 23,
                                            },
                                            stmts: [
                                                Stmt {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 17,
                                                        hi: 21,
                                                    },
                                                    kind: Expr(
                                                        Expr {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 17,
                                                                hi: 21,
                                                            },
                                                            kind: Lit(
                                                                Bool(
                                                                    true,
                                                                ),
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
                                                    lo: 24,
                                                    hi: 38,
                                                },
                                                kind: Block(
                                                    Block {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 29,
                                                            hi: 38,
                                                        },
                                                        stmts: [
                                                            Stmt {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 31,
                                                                    hi: 36,
                                                                },
                                                                kind: Expr(
                                                                    Expr {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 31,
                                                                            hi: 36,
                                                                        },
                                                                        kind: Lit(
                                                                            Bool(
                                                                                false,
                                                                            ),
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
                        },
                        Stmt {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 40,
                                hi: 41,
                            },
                            kind: Expr(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 40,
                                        hi: 41,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 40,
                                                hi: 41,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 40,
                                                    hi: 41,
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
            )
        "#]],
    );
}
