// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::expect;
use qsc_data_structures::language_features::LanguageFeatures;

#[test]
fn explicit_namespace_overrides_implicit() {
    let result = format!(
        "{:#?}",
        crate::namespaces(
            "namespace Explicit {}",
            Some("code/src/Implicit.qs"),
            LanguageFeatures::default()
        )
    );
    expect![[r#"
        (
            [
                Namespace {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 21,
                    },
                    doc: "",
                    name: Idents(
                        [
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 10,
                                    hi: 18,
                                },
                                name: "Explicit",
                            },
                        ],
                    ),
                    items: [],
                },
            ],
            [],
        )"#]]
    .assert_eq(&result);
}

#[test]
fn fixup_bad_namespace_name_with_dash() {
    let result = format!(
        "{:#?}",
        crate::namespaces(
            "operation Main() : Unit {}",
            Some("code/src/Foo-Bar.qs"),
            LanguageFeatures::default()
        )
    );
    expect![[r#"
        (
            [
                Namespace {
                    id: NodeId(
                        4294967295,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 26,
                    },
                    doc: "",
                    name: Idents(
                        [
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 26,
                                },
                                name: "code",
                            },
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 26,
                                },
                                name: "src",
                            },
                            Ident {
                                id: NodeId(
                                    4294967295,
                                ),
                                span: Span {
                                    lo: 0,
                                    hi: 26,
                                },
                                name: "Foo_Bar",
                            },
                        ],
                    ),
                    items: [
                        Item {
                            id: NodeId(
                                4294967295,
                            ),
                            span: Span {
                                lo: 0,
                                hi: 26,
                            },
                            doc: "",
                            attrs: [],
                            kind: Callable(
                                CallableDecl {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
                                        hi: 26,
                                    },
                                    kind: Operation,
                                    name: Ident {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 10,
                                            hi: 14,
                                        },
                                        name: "Main",
                                    },
                                    generics: [],
                                    input: Pat {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 14,
                                            hi: 16,
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
                                            lo: 19,
                                            hi: 23,
                                        },
                                        kind: Path(
                                            Path {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 19,
                                                    hi: 23,
                                                },
                                                segments: None,
                                                name: Ident {
                                                    id: NodeId(
                                                        4294967295,
                                                    ),
                                                    span: Span {
                                                        lo: 19,
                                                        hi: 23,
                                                    },
                                                    name: "Unit",
                                                },
                                            },
                                        ),
                                    },
                                    functors: None,
                                    body: Block(
                                        Block {
                                            id: NodeId(
                                                4294967295,
                                            ),
                                            span: Span {
                                                lo: 24,
                                                hi: 26,
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
            [],
        )"#]]
    .assert_eq(&result);
}

#[test]
fn reject_bad_namespace_name_starts_with_number() {
    let result = format!(
        "{:#?}",
        crate::namespaces(
            "operation Main() : Unit {}",
            Some("code/src/123Bar.qs"),
            LanguageFeatures::default()
        )
    );
    expect![[r#"
        (
            [],
            [
                Error(
                    InvalidFileName(
                        Span {
                            lo: 0,
                            hi: 26,
                        },
                        "123Bar",
                    ),
                ),
            ],
        )"#]]
    .assert_eq(&result);
}
