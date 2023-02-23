// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use super::{GlobalTable, Id};
use crate::{id, parse};
use expect_test::{expect, Expect};
use qsc_ast::{ast::NodeId, mut_visit::MutVisitor, visit::Visitor};

fn check(input: &str, expect: &Expect) {
    let (mut package, errors) = parse::package(input);
    assert!(errors.is_empty());

    let mut assigner = id::Assigner::new();
    assigner.visit_package(&mut package);
    let mut globals = GlobalTable::new();
    globals.visit_package(&package);
    let mut resolver = globals.into_resolver();
    resolver.visit_package(&package);
    let (symbols, errors) = resolver.into_table();
    let mut symbols: Vec<(NodeId, Id)> = symbols.nodes.into_iter().collect();
    symbols.sort();

    expect.assert_debug_eq(&(errors, symbols, package));
}

#[test]
fn local_var() {
    check(
        "namespace A { function B() : Int { let x = 0; x } }",
        &expect![[r#"
            (
                [],
                [
                    (
                        NodeId(
                            5,
                        ),
                        Id(
                            0,
                        ),
                    ),
                    (
                        NodeId(
                            11,
                        ),
                        Id(
                            1,
                        ),
                    ),
                    (
                        NodeId(
                            15,
                        ),
                        Id(
                            1,
                        ),
                    ),
                ],
                Package {
                    id: NodeId(
                        0,
                    ),
                    namespaces: [
                        Namespace {
                            id: NodeId(
                                1,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 51,
                            },
                            name: Ident {
                                id: NodeId(
                                    2,
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
                                        3,
                                    ),
                                    span: Span {
                                        lo: 14,
                                        hi: 49,
                                    },
                                    kind: Callable(
                                        DeclMeta {
                                            attrs: [],
                                            visibility: None,
                                        },
                                        CallableDecl {
                                            id: NodeId(
                                                4,
                                            ),
                                            span: Span {
                                                lo: 14,
                                                hi: 49,
                                            },
                                            kind: Function,
                                            name: Ident {
                                                id: NodeId(
                                                    5,
                                                ),
                                                span: Span {
                                                    lo: 23,
                                                    hi: 24,
                                                },
                                                name: "B",
                                            },
                                            ty_params: [],
                                            input: Pat {
                                                id: NodeId(
                                                    6,
                                                ),
                                                span: Span {
                                                    lo: 24,
                                                    hi: 26,
                                                },
                                                kind: Tuple(
                                                    [],
                                                ),
                                            },
                                            output: Ty {
                                                id: NodeId(
                                                    7,
                                                ),
                                                span: Span {
                                                    lo: 29,
                                                    hi: 32,
                                                },
                                                kind: Prim(
                                                    Int,
                                                ),
                                            },
                                            functors: None,
                                            body: Block(
                                                Block {
                                                    id: NodeId(
                                                        8,
                                                    ),
                                                    span: Span {
                                                        lo: 33,
                                                        hi: 49,
                                                    },
                                                    stmts: [
                                                        Stmt {
                                                            id: NodeId(
                                                                9,
                                                            ),
                                                            span: Span {
                                                                lo: 35,
                                                                hi: 45,
                                                            },
                                                            kind: Let(
                                                                Pat {
                                                                    id: NodeId(
                                                                        10,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 39,
                                                                        hi: 40,
                                                                    },
                                                                    kind: Bind(
                                                                        Ident {
                                                                            id: NodeId(
                                                                                11,
                                                                            ),
                                                                            span: Span {
                                                                                lo: 39,
                                                                                hi: 40,
                                                                            },
                                                                            name: "x",
                                                                        },
                                                                        None,
                                                                    ),
                                                                },
                                                                Expr {
                                                                    id: NodeId(
                                                                        12,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 43,
                                                                        hi: 44,
                                                                    },
                                                                    kind: Lit(
                                                                        Int(
                                                                            0,
                                                                        ),
                                                                    ),
                                                                },
                                                            ),
                                                        },
                                                        Stmt {
                                                            id: NodeId(
                                                                13,
                                                            ),
                                                            span: Span {
                                                                lo: 46,
                                                                hi: 47,
                                                            },
                                                            kind: Expr(
                                                                Expr {
                                                                    id: NodeId(
                                                                        14,
                                                                    ),
                                                                    span: Span {
                                                                        lo: 46,
                                                                        hi: 47,
                                                                    },
                                                                    kind: Path(
                                                                        Path {
                                                                            id: NodeId(
                                                                                15,
                                                                            ),
                                                                            span: Span {
                                                                                lo: 46,
                                                                                hi: 47,
                                                                            },
                                                                            namespace: None,
                                                                            name: Ident {
                                                                                id: NodeId(
                                                                                    4294967295,
                                                                                ),
                                                                                span: Span {
                                                                                    lo: 46,
                                                                                    hi: 47,
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
