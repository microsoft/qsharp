// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::ty;
use crate::parse::tests::check;
use expect_test::expect;

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
fn ty_paren() {
    check(
        ty,
        "(Int)",
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
                    kind: Paren(
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
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn ty_singleton_tuple() {
    check(
        ty,
        "(Int,)",
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
                        ],
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

#[test]
fn ty_function() {
    check(
        ty,
        "Int -> Int",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 10,
                    },
                    kind: Arrow(
                        Function,
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
                        None,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn ty_operation() {
    check(
        ty,
        "Int => Int",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 10,
                    },
                    kind: Arrow(
                        Operation,
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
                        None,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn ty_curried_function() {
    check(
        ty,
        "Int -> Int -> Int",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 17,
                    },
                    kind: Arrow(
                        Function,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 7,
                                hi: 17,
                            },
                            kind: Arrow(
                                Function,
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
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 14,
                                        hi: 17,
                                    },
                                    kind: Prim(
                                        Int,
                                    ),
                                },
                                None,
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
fn ty_higher_order_function() {
    check(
        ty,
        "(Int -> Int) -> Int",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 19,
                    },
                    kind: Arrow(
                        Function,
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 12,
                            },
                            kind: Paren(
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 1,
                                        hi: 11,
                                    },
                                    kind: Arrow(
                                        Function,
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
                                                lo: 8,
                                                hi: 11,
                                            },
                                            kind: Prim(
                                                Int,
                                            ),
                                        },
                                        None,
                                    ),
                                },
                            ),
                        },
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 16,
                                hi: 19,
                            },
                            kind: Prim(
                                Int,
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
fn op_ty_is_adj() {
    check(
        ty,
        "Qubit => Unit is Adj",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 20,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 13,
                            },
                            kind: Tuple(
                                [],
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 20,
                                },
                                kind: Lit(
                                    Adj,
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
fn op_ty_is_adj_ctl() {
    check(
        ty,
        "Qubit => Unit is Adj + Ctl",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 26,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 13,
                            },
                            kind: Tuple(
                                [],
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 26,
                                },
                                kind: BinOp(
                                    Union,
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 17,
                                            hi: 20,
                                        },
                                        kind: Lit(
                                            Adj,
                                        ),
                                    },
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 23,
                                            hi: 26,
                                        },
                                        kind: Lit(
                                            Ctl,
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
fn op_ty_is_nested() {
    check(
        ty,
        "Qubit => Qubit => Unit is Adj is Ctl",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 36,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 29,
                            },
                            kind: Arrow(
                                Operation,
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 9,
                                        hi: 14,
                                    },
                                    kind: Prim(
                                        Qubit,
                                    ),
                                },
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 18,
                                        hi: 22,
                                    },
                                    kind: Tuple(
                                        [],
                                    ),
                                },
                                Some(
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 26,
                                            hi: 29,
                                        },
                                        kind: Lit(
                                            Adj,
                                        ),
                                    },
                                ),
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 33,
                                    hi: 36,
                                },
                                kind: Lit(
                                    Ctl,
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
fn op_ty_is_nested_paren() {
    check(
        ty,
        "Qubit => (Qubit => Unit) is Ctl",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 31,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 24,
                            },
                            kind: Paren(
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 10,
                                        hi: 23,
                                    },
                                    kind: Arrow(
                                        Operation,
                                        Ty {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 10,
                                                hi: 15,
                                            },
                                            kind: Prim(
                                                Qubit,
                                            ),
                                        },
                                        Ty {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 19,
                                                hi: 23,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        None,
                                    ),
                                },
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 28,
                                    hi: 31,
                                },
                                kind: Lit(
                                    Ctl,
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
fn op_ty_is_paren() {
    check(
        ty,
        "Qubit => Unit is (Adj)",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 22,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 13,
                            },
                            kind: Tuple(
                                [],
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 22,
                                },
                                kind: Paren(
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 18,
                                            hi: 21,
                                        },
                                        kind: Lit(
                                            Adj,
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
fn op_ty_union_assoc() {
    check(
        ty,
        "Qubit => Unit is Adj + Adj + Adj",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 32,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 13,
                            },
                            kind: Tuple(
                                [],
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 32,
                                },
                                kind: BinOp(
                                    Union,
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 17,
                                            hi: 26,
                                        },
                                        kind: BinOp(
                                            Union,
                                            FunctorExpr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 17,
                                                    hi: 20,
                                                },
                                                kind: Lit(
                                                    Adj,
                                                ),
                                            },
                                            FunctorExpr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 23,
                                                    hi: 26,
                                                },
                                                kind: Lit(
                                                    Adj,
                                                ),
                                            },
                                        ),
                                    },
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 29,
                                            hi: 32,
                                        },
                                        kind: Lit(
                                            Adj,
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
fn op_ty_intersect_assoc() {
    check(
        ty,
        "Qubit => Unit is Adj * Adj * Adj",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 32,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 13,
                            },
                            kind: Tuple(
                                [],
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 32,
                                },
                                kind: BinOp(
                                    Intersect,
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 17,
                                            hi: 26,
                                        },
                                        kind: BinOp(
                                            Intersect,
                                            FunctorExpr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 17,
                                                    hi: 20,
                                                },
                                                kind: Lit(
                                                    Adj,
                                                ),
                                            },
                                            FunctorExpr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 23,
                                                    hi: 26,
                                                },
                                                kind: Lit(
                                                    Adj,
                                                ),
                                            },
                                        ),
                                    },
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 29,
                                            hi: 32,
                                        },
                                        kind: Lit(
                                            Adj,
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
fn op_ty_is_prec() {
    check(
        ty,
        "Qubit => Unit is Adj + Adj * Ctl",
        &expect![[r#"
            Ok(
                Ty {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 32,
                    },
                    kind: Arrow(
                        Operation,
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
                        Ty {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 13,
                            },
                            kind: Tuple(
                                [],
                            ),
                        },
                        Some(
                            FunctorExpr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 32,
                                },
                                kind: BinOp(
                                    Union,
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 17,
                                            hi: 20,
                                        },
                                        kind: Lit(
                                            Adj,
                                        ),
                                    },
                                    FunctorExpr {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 23,
                                            hi: 32,
                                        },
                                        kind: BinOp(
                                            Intersect,
                                            FunctorExpr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 23,
                                                    hi: 26,
                                                },
                                                kind: Lit(
                                                    Adj,
                                                ),
                                            },
                                            FunctorExpr {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 29,
                                                    hi: 32,
                                                },
                                                kind: Lit(
                                                    Ctl,
                                                ),
                                            },
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
