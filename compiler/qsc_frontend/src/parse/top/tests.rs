// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::{item, package, spec_decl};
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
                            attrs: [],
                            visibility: None,
                        },
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
                    kind: Callable(
                        DeclMeta {
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
                    kind: Callable(
                        DeclMeta {
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
fn namespace_function() {
    check(
        package,
        "namespace A { function Foo() : Unit { body intrinsic; } }",
        &expect![[r#"
            Ok(
                Package {
                    id: NodeId(
                        4294967295,
                    ),
                    namespaces: [
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
                                    kind: Callable(
                                        DeclMeta {
                                            attrs: [],
                                            visibility: None,
                                        },
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
                },
            )
        "#]],
    );
}

#[test]
fn two_namespaces() {
    check(
        package,
        "namespace A {} namespace B {}",
        &expect![[r#"
            Ok(
                Package {
                    id: NodeId(
                        4294967295,
                    ),
                    namespaces: [
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
                },
            )
        "#]],
    );
}

#[test]
fn two_open_items() {
    check(
        package,
        "namespace A { open B; open C; }",
        &expect![[r#"
            Ok(
                Package {
                    id: NodeId(
                        4294967295,
                    ),
                    namespaces: [
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
                },
            )
        "#]],
    );
}
