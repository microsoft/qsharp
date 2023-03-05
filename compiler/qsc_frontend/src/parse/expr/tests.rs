// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

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
fn lit_int_overflow() {
    check(
        expr,
        "9_223_372_036_854_775_808",
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
                    kind: Lit(
                        Int(
                            -9223372036854775808,
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn lit_int_min() {
    check(
        expr,
        "-9_223_372_036_854_775_808",
        &expect![[r#"
            Ok(
                Expr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 26,
                    },
                    kind: UnOp(
                        Neg,
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 1,
                                hi: 26,
                            },
                            kind: Lit(
                                Int(
                                    -9223372036854775808,
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
fn lit_int_hexadecimal() {
    check(
        expr,
        "0x1a2b3c",
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
                        Int(
                            1715004,
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn lit_int_octal() {
    check(
        expr,
        "0o1234567",
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
                    kind: Lit(
                        Int(
                            342391,
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn lit_int_binary() {
    check(
        expr,
        "0b10110",
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
                            22,
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn lit_bigint() {
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
fn lit_bigint_underscore() {
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
fn lit_bigint_hexadecimal() {
    check(
        expr,
        "0x1a2b3cL",
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
                    kind: Lit(
                        BigInt(
                            1715004,
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn lit_bigint_octal() {
    check(
        expr,
        "0o1234567L",
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
                        BigInt(
                            342391,
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn lit_bigint_binary() {
    check(
        expr,
        "0b10110L",
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
                            22,
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
                Rule(
                    "expression",
                    Dot,
                    Span {
                        lo: 0,
                        hi: 1,
                    },
                ),
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
fn hole() {
    check(
        expr,
        "_",
        &expect![[r#"
            Ok(
                Expr {
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
fn set_hole() {
    check(
        expr,
        "set _ = 1",
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
                            kind: Hole,
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
                                    1,
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
fn set_hole_tuple() {
    check(
        expr,
        "set (x, _) = (1, 2)",
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
                    kind: Assign(
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 4,
                                hi: 10,
                            },
                            kind: Tuple(
                                [
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
                                        kind: Hole,
                                    },
                                ],
                            ),
                        },
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 13,
                                hi: 19,
                            },
                            kind: Tuple(
                                [
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 15,
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
                                            lo: 17,
                                            hi: 18,
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
            )
        "#]],
    );
}

#[test]
fn set_hole_tuple_nested() {
    check(
        expr,
        "set (_, (x, _)) = (1, (2, 3))",
        &expect![[r#"
            Ok(
                Expr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 29,
                    },
                    kind: Assign(
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 4,
                                hi: 15,
                            },
                            kind: Tuple(
                                [
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 5,
                                            hi: 6,
                                        },
                                        kind: Hole,
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 8,
                                            hi: 14,
                                        },
                                        kind: Tuple(
                                            [
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
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 12,
                                                        hi: 13,
                                                    },
                                                    kind: Hole,
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
                                hi: 29,
                            },
                            kind: Tuple(
                                [
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 19,
                                            hi: 20,
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
                                            lo: 22,
                                            hi: 28,
                                        },
                                        kind: Tuple(
                                            [
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 23,
                                                        hi: 24,
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
                                                        lo: 26,
                                                        hi: 27,
                                                    },
                                                    kind: Lit(
                                                        Int(
                                                            3,
                                                        ),
                                                    ),
                                                },
                                            ],
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
fn set_bitwise_and() {
    check(
        expr,
        "set x &&&= y",
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
                    kind: AssignOp(
                        AndB,
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
fn set_logical_and() {
    check(
        expr,
        "set x and= y",
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
                    kind: AssignOp(
                        AndL,
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
fn set_bitwise_or() {
    check(
        expr,
        "set x |||= y",
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
                    kind: AssignOp(
                        OrB,
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
fn set_exp() {
    check(
        expr,
        "set x ^= y",
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
                    kind: AssignOp(
                        Exp,
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
fn set_bitwise_xor() {
    check(
        expr,
        "set x ^^^= y",
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
                    kind: AssignOp(
                        XorB,
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
fn set_shr() {
    check(
        expr,
        "set x >>>= y",
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
                    kind: AssignOp(
                        Shr,
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
fn set_shl() {
    check(
        expr,
        "set x <<<= y",
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
                    kind: AssignOp(
                        Shl,
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
fn set_sub() {
    check(
        expr,
        "set x -= y",
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
                    kind: AssignOp(
                        Sub,
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
fn set_logical_or() {
    check(
        expr,
        "set x or= y",
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
                    kind: AssignOp(
                        OrL,
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
fn set_mod() {
    check(
        expr,
        "set x %= y",
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
                    kind: AssignOp(
                        Mod,
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
fn set_add() {
    check(
        expr,
        "set x += y",
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
                    kind: AssignOp(
                        Add,
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
fn set_div() {
    check(
        expr,
        "set x /= y",
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
                    kind: AssignOp(
                        Div,
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
fn set_mul() {
    check(
        expr,
        "set x *= y",
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
                    kind: AssignOp(
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
fn set_with_update() {
    check(
        expr,
        "set x w/= i <- y",
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
                    kind: AssignUpdate(
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
                                lo: 15,
                                hi: 16,
                            },
                            kind: Path(
                                Path {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 15,
                                        hi: 16,
                                    },
                                    namespace: None,
                                    name: Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 15,
                                            hi: 16,
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
fn array_repeat() {
    check(
        expr,
        "[0, size = 3]",
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
                    kind: ArrayRepeat(
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
                                    0,
                                ),
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
                            kind: Lit(
                                Int(
                                    3,
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
fn array_repeat_complex() {
    check(
        expr,
        "[Foo(), size = Count() + 1]",
        &expect![[r#"
            Ok(
                Expr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 27,
                    },
                    kind: ArrayRepeat(
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 1,
                                hi: 6,
                            },
                            kind: Call(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 4,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 4,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 4,
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
                                        lo: 4,
                                        hi: 6,
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
                                lo: 15,
                                hi: 26,
                            },
                            kind: BinOp(
                                Add,
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 15,
                                        hi: 22,
                                    },
                                    kind: Call(
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 15,
                                                hi: 20,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 15,
                                                        hi: 20,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 15,
                                                            hi: 20,
                                                        },
                                                        name: "Count",
                                                    },
                                                },
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 20,
                                                hi: 22,
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
                                        lo: 25,
                                        hi: 26,
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
                },
            )
        "#]],
    );
}

#[test]
fn array_size_last_item() {
    check(
        expr,
        "[foo, size]",
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
                    kind: Array(
                        [
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 4,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 4,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 4,
                                            },
                                            name: "foo",
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
                                    hi: 10,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 10,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 10,
                                            },
                                            name: "size",
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
fn array_size_middle_item() {
    check(
        expr,
        "[foo, size, bar]",
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
                    kind: Array(
                        [
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 1,
                                    hi: 4,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 1,
                                            hi: 4,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 1,
                                                hi: 4,
                                            },
                                            name: "foo",
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
                                    hi: 10,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 6,
                                            hi: 10,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 6,
                                                hi: 10,
                                            },
                                            name: "size",
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
                                    hi: 15,
                                },
                                kind: Path(
                                    Path {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 12,
                                            hi: 15,
                                        },
                                        namespace: None,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 12,
                                                hi: 15,
                                            },
                                            name: "bar",
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
fn array_repeat_no_items() {
    check(
        expr,
        "[size = 3]",
        &expect![[r#"
            Err(
                Token(
                    Close(
                        Bracket,
                    ),
                    Eq,
                    Span {
                        lo: 6,
                        hi: 7,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn array_repeat_two_items() {
    check(
        expr,
        "[1, 2, size = 3]",
        &expect![[r#"
            Err(
                Token(
                    Close(
                        Bracket,
                    ),
                    Eq,
                    Span {
                        lo: 12,
                        hi: 13,
                    },
                ),
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
fn adjoint_call_ops() {
    check(
        expr,
        "Adjoint X(q)",
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
                    kind: Call(
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
                                                name: "X",
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
                                lo: 9,
                                hi: 12,
                            },
                            kind: Paren(
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
                                                name: "q",
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
fn adjoint_index_call_ops() {
    check(
        expr,
        "Adjoint ops[i](q)",
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
                    kind: Call(
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 14,
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
                                        hi: 14,
                                    },
                                    kind: Index(
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 8,
                                                hi: 11,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 8,
                                                        hi: 11,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 8,
                                                            hi: 11,
                                                        },
                                                        name: "ops",
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
                                                        name: "i",
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
                                lo: 14,
                                hi: 17,
                            },
                            kind: Paren(
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 15,
                                        hi: 16,
                                    },
                                    kind: Path(
                                        Path {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 15,
                                                hi: 16,
                                            },
                                            namespace: None,
                                            name: Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 15,
                                                    hi: 16,
                                                },
                                                name: "q",
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
fn controlled_call_ops() {
    check(
        expr,
        "Controlled X([q1], q2)",
        &expect![[r#"
            Ok(
                Expr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 22,
                    },
                    kind: Call(
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
                                                name: "X",
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
                                lo: 12,
                                hi: 22,
                            },
                            kind: Tuple(
                                [
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 13,
                                            hi: 17,
                                        },
                                        kind: Array(
                                            [
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 14,
                                                        hi: 16,
                                                    },
                                                    kind: Path(
                                                        Path {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 14,
                                                                hi: 16,
                                                            },
                                                            namespace: None,
                                                            name: Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 14,
                                                                    hi: 16,
                                                                },
                                                                name: "q1",
                                                            },
                                                        },
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
                                            lo: 19,
                                            hi: 21,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 19,
                                                    hi: 21,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 19,
                                                        hi: 21,
                                                    },
                                                    name: "q2",
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
fn controlled_index_call_ops() {
    check(
        expr,
        "Controlled ops[i]([q1], q2)",
        &expect![[r#"
            Ok(
                Expr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 27,
                    },
                    kind: Call(
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 17,
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
                                        hi: 17,
                                    },
                                    kind: Index(
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 11,
                                                hi: 14,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 11,
                                                        hi: 14,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 11,
                                                            hi: 14,
                                                        },
                                                        name: "ops",
                                                    },
                                                },
                                            ),
                                        },
                                        Expr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 15,
                                                hi: 16,
                                            },
                                            kind: Path(
                                                Path {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 15,
                                                        hi: 16,
                                                    },
                                                    namespace: None,
                                                    name: Ident {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 15,
                                                            hi: 16,
                                                        },
                                                        name: "i",
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
                                lo: 17,
                                hi: 27,
                            },
                            kind: Tuple(
                                [
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 18,
                                            hi: 22,
                                        },
                                        kind: Array(
                                            [
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 19,
                                                        hi: 21,
                                                    },
                                                    kind: Path(
                                                        Path {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 19,
                                                                hi: 21,
                                                            },
                                                            namespace: None,
                                                            name: Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 19,
                                                                    hi: 21,
                                                                },
                                                                name: "q1",
                                                            },
                                                        },
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
                                            lo: 24,
                                            hi: 26,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 24,
                                                    hi: 26,
                                                },
                                                namespace: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 24,
                                                        hi: 26,
                                                    },
                                                    name: "q2",
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
fn call_partial_app() {
    check(
        expr,
        "Foo(1, _, 3)",
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
                                hi: 12,
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
                                            lo: 7,
                                            hi: 8,
                                        },
                                        kind: Hole,
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
                                                3,
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
fn call_partial_app_nested() {
    check(
        expr,
        "Foo(1, _, (_, 4))",
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
                                hi: 17,
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
                                            lo: 7,
                                            hi: 8,
                                        },
                                        kind: Hole,
                                    },
                                    Expr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 10,
                                            hi: 16,
                                        },
                                        kind: Tuple(
                                            [
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 11,
                                                        hi: 12,
                                                    },
                                                    kind: Hole,
                                                },
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 14,
                                                        hi: 15,
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
fn range_start_step_open() {
    check(
        expr,
        "3..2...",
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
                                kind: Lit(
                                    Int(
                                        3,
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

#[test]
fn range_open_stop() {
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
fn range_open_step_stop() {
    check(
        expr,
        "...2..3",
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
                        Some(
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
                                        3,
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
fn range_open_step_open() {
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

#[test]
fn function_lambda() {
    check(
        expr,
        "x -> x + 1",
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
                    kind: Lambda(
                        Function,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 1,
                            },
                            kind: Bind(
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
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
                                lo: 5,
                                hi: 10,
                            },
                            kind: BinOp(
                                Add,
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
                                        lo: 9,
                                        hi: 10,
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
                },
            )
        "#]],
    );
}

#[test]
fn operation_lambda() {
    check(
        expr,
        "q => X(q)",
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
                    kind: Lambda(
                        Operation,
                        Pat {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 1,
                            },
                            kind: Bind(
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 1,
                                    },
                                    name: "q",
                                },
                                None,
                            ),
                        },
                        Expr {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 9,
                            },
                            kind: Call(
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
                                                name: "X",
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
                                        hi: 9,
                                    },
                                    kind: Paren(
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
                                                        name: "q",
                                                    },
                                                },
                                            ),
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
fn lambda_tuple_input() {
    check(
        expr,
        "(x, y) -> x + y",
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
                    kind: Lambda(
                        Function,
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
                                            hi: 2,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 1,
                                                    hi: 2,
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
                                                name: "y",
                                            },
                                            None,
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
                                lo: 10,
                                hi: 15,
                            },
                            kind: BinOp(
                                Add,
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
                                                name: "y",
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
fn lambda_invalid_input() {
    check(
        expr,
        "x + 1 -> x",
        &expect![[r#"
            Err(
                Convert(
                    "pattern",
                    "expression",
                    Span {
                        lo: 0,
                        hi: 5,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lambda_invalid_tuple_input() {
    check(
        expr,
        "(x, y + 1) -> x + y",
        &expect![[r#"
            Err(
                Convert(
                    "pattern",
                    "expression",
                    Span {
                        lo: 4,
                        hi: 9,
                    },
                ),
            )
        "#]],
    );
}
