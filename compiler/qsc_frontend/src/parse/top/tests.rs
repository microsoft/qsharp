// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::{attr, item, namespaces, spec_decl};
use crate::parse::tests::check;
use expect_test::expect;

#[test]
fn body_intrinsic() {
    check(
        spec_decl,
        "body intrinsic;",
        &expect![[r#"
            Ok(
                SpecDecl {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 15,
                    },
                    spec: Body,
                    body: Gen(
                        Intrinsic,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn adjoint_self() {
    check(
        spec_decl,
        "adjoint self;",
        &expect![[r#"
            Ok(
                SpecDecl {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 13,
                    },
                    spec: Adj,
                    body: Gen(
                        Slf,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn adjoint_invert() {
    check(
        spec_decl,
        "adjoint invert;",
        &expect![[r#"
            Ok(
                SpecDecl {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 15,
                    },
                    spec: Adj,
                    body: Gen(
                        Invert,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn controlled_distribute() {
    check(
        spec_decl,
        "controlled distribute;",
        &expect![[r#"
            Ok(
                SpecDecl {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 22,
                    },
                    spec: Ctl,
                    body: Gen(
                        Distribute,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn controlled_adjoint_auto() {
    check(
        spec_decl,
        "controlled adjoint auto;",
        &expect![[r#"
            Ok(
                SpecDecl {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 24,
                    },
                    spec: CtlAdj,
                    body: Gen(
                        Auto,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn spec_gen_missing_semi() {
    check(
        spec_decl,
        "body intrinsic",
        &expect![[r#"
            Err(
                Error {
                    kind: Token(
                        Semi,
                    ),
                    span: Span {
                        lo: 14,
                        hi: 14,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn spec_invalid_gen() {
    check(
        spec_decl,
        "adjoint foo;",
        &expect![[r#"
            Err(
                Error {
                    kind: Token(
                        Open(
                            Brace,
                        ),
                    ),
                    span: Span {
                        lo: 11,
                        hi: 12,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn open_no_alias() {
    check(
        item,
        "open Foo.Bar.Baz;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 17,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Open(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 16,
                            },
                            name: "Foo.Bar.Baz",
                        },
                        None,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn open_alias() {
    check(
        item,
        "open Foo.Bar.Baz as Baz;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 24,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Open(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 16,
                            },
                            name: "Foo.Bar.Baz",
                        },
                        Some(
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 20,
                                    hi: 23,
                                },
                                name: "Baz",
                            },
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn open_alias_dot() {
    check(
        item,
        "open Foo.Bar.Baz as Bar.Baz;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 28,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Open(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 5,
                                hi: 16,
                            },
                            name: "Foo.Bar.Baz",
                        },
                        Some(
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 20,
                                    hi: 27,
                                },
                                name: "Bar.Baz",
                            },
                        ),
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn ty_decl() {
    check(
        item,
        "newtype Foo = Unit;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 19,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 18,
                            },
                            kind: Field(
                                None,
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 14,
                                        hi: 18,
                                    },
                                    kind: Tuple(
                                        [],
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
fn ty_decl_field_name() {
    check(
        item,
        "newtype Foo = Bar : Int;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 24,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 23,
                            },
                            kind: Field(
                                Some(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 17,
                                        },
                                        name: "Bar",
                                    },
                                ),
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 20,
                                        hi: 23,
                                    },
                                    kind: Prim(
                                        Int,
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
fn ty_def_invalid_field_name() {
    check(
        item,
        "newtype Foo = Bar.Baz : Int[];",
        &expect![[r#"
            Err(
                Error {
                    kind: Rule(
                        "identifier",
                    ),
                    span: Span {
                        lo: 14,
                        hi: 21,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn ty_def_tuple() {
    check(
        item,
        "newtype Foo = (Int, Int);",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 25,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 24,
                            },
                            kind: Tuple(
                                [
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 15,
                                            hi: 18,
                                        },
                                        kind: Field(
                                            None,
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 15,
                                                    hi: 18,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                        ),
                                    },
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 20,
                                            hi: 23,
                                        },
                                        kind: Field(
                                            None,
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 20,
                                                    hi: 23,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
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
fn ty_def_tuple_one_named() {
    check(
        item,
        "newtype Foo = (X : Int, Int);",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 29,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 28,
                            },
                            kind: Tuple(
                                [
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 15,
                                            hi: 22,
                                        },
                                        kind: Field(
                                            Some(
                                                Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 15,
                                                        hi: 16,
                                                    },
                                                    name: "X",
                                                },
                                            ),
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 19,
                                                    hi: 22,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                        ),
                                    },
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 24,
                                            hi: 27,
                                        },
                                        kind: Field(
                                            None,
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 24,
                                                    hi: 27,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
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
fn ty_def_tuple_both_named() {
    check(
        item,
        "newtype Foo = (X : Int, Y : Int);",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 33,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 32,
                            },
                            kind: Tuple(
                                [
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 15,
                                            hi: 22,
                                        },
                                        kind: Field(
                                            Some(
                                                Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 15,
                                                        hi: 16,
                                                    },
                                                    name: "X",
                                                },
                                            ),
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 19,
                                                    hi: 22,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                        ),
                                    },
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 24,
                                            hi: 31,
                                        },
                                        kind: Field(
                                            Some(
                                                Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 24,
                                                        hi: 25,
                                                    },
                                                    name: "Y",
                                                },
                                            ),
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 28,
                                                    hi: 31,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
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
fn ty_def_nested_tuple() {
    check(
        item,
        "newtype Foo = ((X : Int, Y : Int), Z : Int);",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 44,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 43,
                            },
                            kind: Tuple(
                                [
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 15,
                                            hi: 33,
                                        },
                                        kind: Tuple(
                                            [
                                                TyDef {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 16,
                                                        hi: 23,
                                                    },
                                                    kind: Field(
                                                        Some(
                                                            Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 16,
                                                                    hi: 17,
                                                                },
                                                                name: "X",
                                                            },
                                                        ),
                                                        Ty {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 20,
                                                                hi: 23,
                                                            },
                                                            kind: Prim(
                                                                Int,
                                                            ),
                                                        },
                                                    ),
                                                },
                                                TyDef {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 25,
                                                        hi: 32,
                                                    },
                                                    kind: Field(
                                                        Some(
                                                            Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 25,
                                                                    hi: 26,
                                                                },
                                                                name: "Y",
                                                            },
                                                        ),
                                                        Ty {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 29,
                                                                hi: 32,
                                                            },
                                                            kind: Prim(
                                                                Int,
                                                            ),
                                                        },
                                                    ),
                                                },
                                            ],
                                        ),
                                    },
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 35,
                                            hi: 42,
                                        },
                                        kind: Field(
                                            Some(
                                                Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 35,
                                                        hi: 36,
                                                    },
                                                    name: "Z",
                                                },
                                            ),
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 39,
                                                    hi: 42,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
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
fn ty_def_tuple_with_name() {
    check(
        item,
        "newtype Foo = Pair : (Int, Int);",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 32,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 8,
                                hi: 11,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 31,
                            },
                            kind: Field(
                                Some(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 18,
                                        },
                                        name: "Pair",
                                    },
                                ),
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 21,
                                        hi: 31,
                                    },
                                    kind: Tuple(
                                        [
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 22,
                                                    hi: 25,
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
                                                    lo: 27,
                                                    hi: 30,
                                                },
                                                kind: Prim(
                                                    Int,
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
fn function_decl() {
    check(
        item,
        "function Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 41,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 41,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 12,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 12,
                                    hi: 14,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 21,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 24,
                                            hi: 39,
                                        },
                                        spec: Body,
                                        body: Gen(
                                            Intrinsic,
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
fn operation_decl() {
    check(
        item,
        "operation Foo() : Unit { body intrinsic; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 42,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 42,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 13,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 13,
                                    hi: 15,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
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
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 25,
                                            hi: 40,
                                        },
                                        spec: Body,
                                        body: Gen(
                                            Intrinsic,
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
fn function_one_param() {
    check(
        item,
        "function Foo(x : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 48,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 48,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 12,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 12,
                                    hi: 21,
                                },
                                kind: Paren(
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 13,
                                            hi: 20,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 13,
                                                    hi: 14,
                                                },
                                                name: "x",
                                            },
                                            Some(
                                                Ty {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 17,
                                                        hi: 20,
                                                    },
                                                    kind: Prim(
                                                        Int,
                                                    ),
                                                },
                                            ),
                                        ),
                                    },
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 24,
                                    hi: 28,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 31,
                                            hi: 46,
                                        },
                                        spec: Body,
                                        body: Gen(
                                            Intrinsic,
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
fn function_two_params() {
    check(
        item,
        "function Foo(x : Int, y : Int) : Unit { body intrinsic; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 57,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 57,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 12,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 12,
                                    hi: 30,
                                },
                                kind: Tuple(
                                    [
                                        Pat {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 13,
                                                hi: 20,
                                            },
                                            kind: Bind(
                                                Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 13,
                                                        hi: 14,
                                                    },
                                                    name: "x",
                                                },
                                                Some(
                                                    Ty {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 17,
                                                            hi: 20,
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
                                                lo: 22,
                                                hi: 29,
                                            },
                                            kind: Bind(
                                                Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 22,
                                                        hi: 23,
                                                    },
                                                    name: "y",
                                                },
                                                Some(
                                                    Ty {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 26,
                                                            hi: 29,
                                                        },
                                                        kind: Prim(
                                                            Int,
                                                        ),
                                                    },
                                                ),
                                            ),
                                        },
                                    ],
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 33,
                                    hi: 37,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 40,
                                            hi: 55,
                                        },
                                        spec: Body,
                                        body: Gen(
                                            Intrinsic,
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
fn function_one_ty_param() {
    check(
        item,
        "function Foo<'T>() : Unit { body intrinsic; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 45,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 45,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 12,
                                },
                                name: "Foo",
                            },
                            ty_params: [
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 14,
                                        hi: 15,
                                    },
                                    name: "T",
                                },
                            ],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 16,
                                    hi: 18,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 21,
                                    hi: 25,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 28,
                                            hi: 43,
                                        },
                                        spec: Body,
                                        body: Gen(
                                            Intrinsic,
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
fn function_two_ty_params() {
    check(
        item,
        "function Foo<'T, 'U>() : Unit { body intrinsic; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 49,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 49,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 12,
                                },
                                name: "Foo",
                            },
                            ty_params: [
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 14,
                                        hi: 15,
                                    },
                                    name: "T",
                                },
                                Ident {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 18,
                                        hi: 19,
                                    },
                                    name: "U",
                                },
                            ],
                            input: Pat {
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
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 25,
                                    hi: 29,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 32,
                                            hi: 47,
                                        },
                                        spec: Body,
                                        body: Gen(
                                            Intrinsic,
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
fn function_single_impl() {
    check(
        item,
        "function Foo(x : Int) : Int { let y = x; y }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 44,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 44,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 9,
                                    hi: 12,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 12,
                                    hi: 21,
                                },
                                kind: Paren(
                                    Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 13,
                                            hi: 20,
                                        },
                                        kind: Bind(
                                            Ident {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 13,
                                                    hi: 14,
                                                },
                                                name: "x",
                                            },
                                            Some(
                                                Ty {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 17,
                                                        hi: 20,
                                                    },
                                                    kind: Prim(
                                                        Int,
                                                    ),
                                                },
                                            ),
                                        ),
                                    },
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 24,
                                    hi: 27,
                                },
                                kind: Prim(
                                    Int,
                                ),
                            },
                            functors: None,
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 28,
                                        hi: 44,
                                    },
                                    stmts: [
                                        Stmt {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 30,
                                                hi: 40,
                                            },
                                            kind: Let(
                                                Pat {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 34,
                                                        hi: 35,
                                                    },
                                                    kind: Bind(
                                                        Ident {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 34,
                                                                hi: 35,
                                                            },
                                                            name: "y",
                                                        },
                                                        None,
                                                    ),
                                                },
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 38,
                                                        hi: 39,
                                                    },
                                                    kind: Path(
                                                        Path {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 38,
                                                                hi: 39,
                                                            },
                                                            namespace: None,
                                                            name: Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 38,
                                                                    hi: 39,
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
                                                lo: 41,
                                                hi: 42,
                                            },
                                            kind: Expr(
                                                Expr {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 41,
                                                        hi: 42,
                                                    },
                                                    kind: Path(
                                                        Path {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 41,
                                                                hi: 42,
                                                            },
                                                            namespace: None,
                                                            name: Ident {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 41,
                                                                    hi: 42,
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
                },
            )
        "#]],
    );
}

#[test]
fn operation_body_impl() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 43,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 43,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 13,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 13,
                                    hi: 15,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
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
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 25,
                                            hi: 41,
                                        },
                                        spec: Body,
                                        body: Impl(
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 30,
                                                    hi: 35,
                                                },
                                                kind: Paren(
                                                    Pat {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 31,
                                                            hi: 34,
                                                        },
                                                        kind: Elided,
                                                    },
                                                ),
                                            },
                                            Block {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 36,
                                                    hi: 41,
                                                },
                                                stmts: [
                                                    Stmt {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 38,
                                                            hi: 39,
                                                        },
                                                        kind: Expr(
                                                            Expr {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 38,
                                                                    hi: 39,
                                                                },
                                                                kind: Path(
                                                                    Path {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 38,
                                                                            hi: 39,
                                                                        },
                                                                        namespace: None,
                                                                        name: Ident {
                                                                            id: NodeId(
                                                                                4294967295,
                                                                            ),
                                                                            span: Span {
                                                                                lo: 38,
                                                                                hi: 39,
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
fn operation_body_ctl_impl() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } controlled (cs, ...) { y } }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 70,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 70,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 13,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 13,
                                    hi: 15,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
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
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 25,
                                            hi: 41,
                                        },
                                        spec: Body,
                                        body: Impl(
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 30,
                                                    hi: 35,
                                                },
                                                kind: Paren(
                                                    Pat {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 31,
                                                            hi: 34,
                                                        },
                                                        kind: Elided,
                                                    },
                                                ),
                                            },
                                            Block {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 36,
                                                    hi: 41,
                                                },
                                                stmts: [
                                                    Stmt {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 38,
                                                            hi: 39,
                                                        },
                                                        kind: Expr(
                                                            Expr {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 38,
                                                                    hi: 39,
                                                                },
                                                                kind: Path(
                                                                    Path {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 38,
                                                                            hi: 39,
                                                                        },
                                                                        namespace: None,
                                                                        name: Ident {
                                                                            id: NodeId(
                                                                                4294967295,
                                                                            ),
                                                                            span: Span {
                                                                                lo: 38,
                                                                                hi: 39,
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
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 42,
                                            hi: 68,
                                        },
                                        spec: Ctl,
                                        body: Impl(
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 53,
                                                    hi: 62,
                                                },
                                                kind: Tuple(
                                                    [
                                                        Pat {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 54,
                                                                hi: 56,
                                                            },
                                                            kind: Bind(
                                                                Ident {
                                                                    id: NodeId(
                                                                        4294967295,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 54,
                                                                        hi: 56,
                                                                    },
                                                                    name: "cs",
                                                                },
                                                                None,
                                                            ),
                                                        },
                                                        Pat {
                                                            id: NodeId(
                                                                4294967295,
                                                            ),
                                                            span: Span {
                                                                lo: 58,
                                                                hi: 61,
                                                            },
                                                            kind: Elided,
                                                        },
                                                    ],
                                                ),
                                            },
                                            Block {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 63,
                                                    hi: 68,
                                                },
                                                stmts: [
                                                    Stmt {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 65,
                                                            hi: 66,
                                                        },
                                                        kind: Expr(
                                                            Expr {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 65,
                                                                    hi: 66,
                                                                },
                                                                kind: Path(
                                                                    Path {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 65,
                                                                            hi: 66,
                                                                        },
                                                                        namespace: None,
                                                                        name: Ident {
                                                                            id: NodeId(
                                                                                4294967295,
                                                                            ),
                                                                            span: Span {
                                                                                lo: 65,
                                                                                hi: 66,
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
fn operation_impl_and_gen() {
    check(
        item,
        "operation Foo() : Unit { body (...) { x } adjoint self; }",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 57,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 57,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 13,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 13,
                                    hi: 15,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
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
                            functors: None,
                            body: Specs(
                                [
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 25,
                                            hi: 41,
                                        },
                                        spec: Body,
                                        body: Impl(
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 30,
                                                    hi: 35,
                                                },
                                                kind: Paren(
                                                    Pat {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 31,
                                                            hi: 34,
                                                        },
                                                        kind: Elided,
                                                    },
                                                ),
                                            },
                                            Block {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 36,
                                                    hi: 41,
                                                },
                                                stmts: [
                                                    Stmt {
                                                        id: NodeId(
                                                            4294967295,
                                                        ),
                                                        span: Span {
                                                            lo: 38,
                                                            hi: 39,
                                                        },
                                                        kind: Expr(
                                                            Expr {
                                                                id: NodeId(
                                                                    4294967295,
                                                                ),
                                                                span: Span {
                                                                    lo: 38,
                                                                    hi: 39,
                                                                },
                                                                kind: Path(
                                                                    Path {
                                                                        id: NodeId(
                                                                            4294967295,
                                                                        ),
                                                                        span: Span {
                                                                            lo: 38,
                                                                            hi: 39,
                                                                        },
                                                                        namespace: None,
                                                                        name: Ident {
                                                                            id: NodeId(
                                                                                4294967295,
                                                                            ),
                                                                            span: Span {
                                                                                lo: 38,
                                                                                hi: 39,
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
                                    SpecDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 42,
                                            hi: 55,
                                        },
                                        spec: Adj,
                                        body: Gen(
                                            Slf,
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
fn operation_is_adj() {
    check(
        item,
        "operation Foo() : Unit is Adj {}",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 32,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 32,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 13,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 13,
                                    hi: 15,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
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
                            functors: Some(
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
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 30,
                                        hi: 32,
                                    },
                                    stmts: [],
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
fn operation_is_adj_ctl() {
    check(
        item,
        "operation Foo() : Unit is Adj + Ctl {}",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 38,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 38,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 13,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 13,
                                    hi: 15,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
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
                            functors: Some(
                                FunctorExpr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 26,
                                        hi: 35,
                                    },
                                    kind: BinOp(
                                        Union,
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
                                        FunctorExpr {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 32,
                                                hi: 35,
                                            },
                                            kind: Lit(
                                                Ctl,
                                            ),
                                        },
                                    ),
                                },
                            ),
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 36,
                                        hi: 38,
                                    },
                                    stmts: [],
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
fn function_missing_output_ty() {
    check(
        item,
        "function Foo() { body intrinsic; }",
        &expect![[r#"
            Err(
                Error {
                    kind: Token(
                        Colon,
                    ),
                    span: Span {
                        lo: 15,
                        hi: 16,
                    },
                },
            )
        "#]],
    );
}

#[test]
fn internal_ty() {
    check(
        item,
        "internal newtype Foo = Unit;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 28,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: Some(
                            Visibility {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 8,
                                },
                                kind: Internal,
                            },
                        ),
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 17,
                                hi: 20,
                            },
                            name: "Foo",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 23,
                                hi: 27,
                            },
                            kind: Field(
                                None,
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 23,
                                        hi: 27,
                                    },
                                    kind: Tuple(
                                        [],
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
fn internal_function() {
    check(
        item,
        "internal function Foo() : Unit {}",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 33,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: Some(
                            Visibility {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 8,
                                },
                                kind: Internal,
                            },
                        ),
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 33,
                            },
                            kind: Function,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 18,
                                    hi: 21,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 21,
                                    hi: 23,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 26,
                                    hi: 30,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 31,
                                        hi: 33,
                                    },
                                    stmts: [],
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
fn internal_operation() {
    check(
        item,
        "internal operation Foo() : Unit {}",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 34,
                    },
                    meta: ItemMeta {
                        attrs: [],
                        visibility: Some(
                            Visibility {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 8,
                                },
                                kind: Internal,
                            },
                        ),
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 9,
                                hi: 34,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 19,
                                    hi: 22,
                                },
                                name: "Foo",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 22,
                                    hi: 24,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 27,
                                    hi: 31,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 32,
                                        hi: 34,
                                    },
                                    stmts: [],
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
fn attr_no_args() {
    check(
        attr,
        "@Foo()",
        &expect![[r#"
            Ok(
                Attr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 6,
                    },
                    name: Path {
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
                    arg: Expr {
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
                },
            )
        "#]],
    );
}

#[test]
fn attr_single_arg() {
    check(
        attr,
        "@Foo(123)",
        &expect![[r#"
            Ok(
                Attr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 9,
                    },
                    name: Path {
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
                    arg: Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 4,
                            hi: 9,
                        },
                        kind: Paren(
                            Expr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 5,
                                    hi: 8,
                                },
                                kind: Lit(
                                    Int(
                                        123,
                                    ),
                                ),
                            },
                        ),
                    },
                },
            )
        "#]],
    );
}

#[test]
fn attr_two_args() {
    check(
        attr,
        "@Foo(123, \"bar\")",
        &expect![[r#"
            Ok(
                Attr {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 16,
                    },
                    name: Path {
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
                    arg: Expr {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 4,
                            hi: 16,
                        },
                        kind: Tuple(
                            [
                                Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 5,
                                        hi: 8,
                                    },
                                    kind: Lit(
                                        Int(
                                            123,
                                        ),
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
                                    kind: Lit(
                                        String(
                                            "bar",
                                        ),
                                    ),
                                },
                            ],
                        ),
                    },
                },
            )
        "#]],
    );
}

#[test]
fn open_attr() {
    check(
        item,
        "@Foo() open Bar;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 16,
                    },
                    meta: ItemMeta {
                        attrs: [
                            Attr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 6,
                                },
                                name: Path {
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
                                arg: Expr {
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
                            },
                        ],
                        visibility: None,
                    },
                    kind: Open(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 12,
                                hi: 15,
                            },
                            name: "Bar",
                        },
                        None,
                    ),
                },
            )
        "#]],
    );
}

#[test]
fn newtype_attr() {
    check(
        item,
        "@Foo() newtype Bar = Unit;",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 26,
                    },
                    meta: ItemMeta {
                        attrs: [
                            Attr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 6,
                                },
                                name: Path {
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
                                arg: Expr {
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
                            },
                        ],
                        visibility: None,
                    },
                    kind: Ty(
                        Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 15,
                                hi: 18,
                            },
                            name: "Bar",
                        },
                        TyDef {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 21,
                                hi: 25,
                            },
                            kind: Field(
                                None,
                                Ty {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 21,
                                        hi: 25,
                                    },
                                    kind: Tuple(
                                        [],
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
fn operation_one_attr() {
    check(
        item,
        "@Foo() operation Bar() : Unit {}",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 32,
                    },
                    meta: ItemMeta {
                        attrs: [
                            Attr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 6,
                                },
                                name: Path {
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
                                arg: Expr {
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
                            },
                        ],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 7,
                                hi: 32,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 17,
                                    hi: 20,
                                },
                                name: "Bar",
                            },
                            ty_params: [],
                            input: Pat {
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
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 25,
                                    hi: 29,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 30,
                                        hi: 32,
                                    },
                                    stmts: [],
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
fn operation_two_attrs() {
    check(
        item,
        "@Foo() @Bar() operation Baz() : Unit {}",
        &expect![[r#"
            Ok(
                Item {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 39,
                    },
                    meta: ItemMeta {
                        attrs: [
                            Attr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 6,
                                },
                                name: Path {
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
                                arg: Expr {
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
                            },
                            Attr {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 7,
                                    hi: 13,
                                },
                                name: Path {
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
                                        name: "Bar",
                                    },
                                },
                                arg: Expr {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 11,
                                        hi: 13,
                                    },
                                    kind: Tuple(
                                        [],
                                    ),
                                },
                            },
                        ],
                        visibility: None,
                    },
                    kind: Callable(
                        CallableDecl {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 14,
                                hi: 39,
                            },
                            kind: Operation,
                            name: Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 24,
                                    hi: 27,
                                },
                                name: "Baz",
                            },
                            ty_params: [],
                            input: Pat {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 27,
                                    hi: 29,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            output: Ty {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 32,
                                    hi: 36,
                                },
                                kind: Tuple(
                                    [],
                                ),
                            },
                            functors: None,
                            body: Block(
                                Block {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 37,
                                        hi: 39,
                                    },
                                    stmts: [],
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
fn namespace_function() {
    check(
        namespaces,
        "namespace A { function Foo() : Unit { body intrinsic; } }",
        &expect![[r#"
            Ok(
                [
                    Namespace {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 57,
                        },
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 10,
                                hi: 11,
                            },
                            name: "A",
                        },
                        items: [
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 14,
                                    hi: 55,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Callable(
                                    CallableDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 55,
                                        },
                                        kind: Function,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 23,
                                                hi: 26,
                                            },
                                            name: "Foo",
                                        },
                                        ty_params: [],
                                        input: Pat {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 26,
                                                hi: 28,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        output: Ty {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 31,
                                                hi: 35,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        functors: None,
                                        body: Specs(
                                            [
                                                SpecDecl {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 38,
                                                        hi: 53,
                                                    },
                                                    spec: Body,
                                                    body: Gen(
                                                        Intrinsic,
                                                    ),
                                                },
                                            ],
                                        ),
                                    },
                                ),
                            },
                        ],
                    },
                ],
            )
        "#]],
    );
}

#[test]
fn two_namespaces() {
    check(
        namespaces,
        "namespace A {} namespace B {}",
        &expect![[r#"
            Ok(
                [
                    Namespace {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 14,
                        },
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 10,
                                hi: 11,
                            },
                            name: "A",
                        },
                        items: [],
                    },
                    Namespace {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 15,
                            hi: 29,
                        },
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 25,
                                hi: 26,
                            },
                            name: "B",
                        },
                        items: [],
                    },
                ],
            )
        "#]],
    );
}

#[test]
fn two_open_items() {
    check(
        namespaces,
        "namespace A { open B; open C; }",
        &expect![[r#"
            Ok(
                [
                    Namespace {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 31,
                        },
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 10,
                                hi: 11,
                            },
                            name: "A",
                        },
                        items: [
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 14,
                                    hi: 21,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Open(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 19,
                                            hi: 20,
                                        },
                                        name: "B",
                                    },
                                    None,
                                ),
                            },
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 22,
                                    hi: 29,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Open(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 27,
                                            hi: 28,
                                        },
                                        name: "C",
                                    },
                                    None,
                                ),
                            },
                        ],
                    },
                ],
            )
        "#]],
    );
}

#[test]
fn two_ty_items() {
    check(
        namespaces,
        "namespace A { newtype B = Unit; newtype C = Unit; }",
        &expect![[r#"
            Ok(
                [
                    Namespace {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 51,
                        },
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 10,
                                hi: 11,
                            },
                            name: "A",
                        },
                        items: [
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 14,
                                    hi: 31,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Ty(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 22,
                                            hi: 23,
                                        },
                                        name: "B",
                                    },
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 26,
                                            hi: 30,
                                        },
                                        kind: Field(
                                            None,
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 26,
                                                    hi: 30,
                                                },
                                                kind: Tuple(
                                                    [],
                                                ),
                                            },
                                        ),
                                    },
                                ),
                            },
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 32,
                                    hi: 49,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Ty(
                                    Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 40,
                                            hi: 41,
                                        },
                                        name: "C",
                                    },
                                    TyDef {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 44,
                                            hi: 48,
                                        },
                                        kind: Field(
                                            None,
                                            Ty {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 44,
                                                    hi: 48,
                                                },
                                                kind: Tuple(
                                                    [],
                                                ),
                                            },
                                        ),
                                    },
                                ),
                            },
                        ],
                    },
                ],
            )
        "#]],
    );
}

#[test]
fn two_callable_items() {
    check(
        namespaces,
        "namespace A { operation B() : Unit {} function C() : Unit {} }",
        &expect![[r#"
            Ok(
                [
                    Namespace {
                        id: NodeId(
                            4294967295,
                        ),
                        span: Span {
                            lo: 0,
                            hi: 62,
                        },
                        name: Ident {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 10,
                                hi: 11,
                            },
                            name: "A",
                        },
                        items: [
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 14,
                                    hi: 37,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Callable(
                                    CallableDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 37,
                                        },
                                        kind: Operation,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 24,
                                                hi: 25,
                                            },
                                            name: "B",
                                        },
                                        ty_params: [],
                                        input: Pat {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 25,
                                                hi: 27,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        output: Ty {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 30,
                                                hi: 34,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        functors: None,
                                        body: Block(
                                            Block {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 35,
                                                    hi: 37,
                                                },
                                                stmts: [],
                                            },
                                        ),
                                    },
                                ),
                            },
                            Item {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 38,
                                    hi: 60,
                                },
                                meta: ItemMeta {
                                    attrs: [],
                                    visibility: None,
                                },
                                kind: Callable(
                                    CallableDecl {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 38,
                                            hi: 60,
                                        },
                                        kind: Function,
                                        name: Ident {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 47,
                                                hi: 48,
                                            },
                                            name: "C",
                                        },
                                        ty_params: [],
                                        input: Pat {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 48,
                                                hi: 50,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        output: Ty {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 53,
                                                hi: 57,
                                            },
                                            kind: Tuple(
                                                [],
                                            ),
                                        },
                                        functors: None,
                                        body: Block(
                                            Block {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 58,
                                                    hi: 60,
                                                },
                                                stmts: [],
                                            },
                                        ),
                                    },
                                ),
                            },
                        ],
                    },
                ],
            )
        "#]],
    );
}
