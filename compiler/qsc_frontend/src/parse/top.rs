// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    kw,
    prim::{comma_sep, ident, opt, pat, path},
    scan::Scanner,
    ty::{self, ty},
    Result,
};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{
    CallableBody, CallableDecl, CallableKind, DeclMeta, Item, ItemKind, Namespace, NodeId, Package,
    Span, Spec, SpecBody, SpecDecl, SpecGen,
};

pub(super) fn package(s: &mut Scanner) -> Result<Package> {
    let mut namespaces = Vec::new();
    while s.keyword(kw::NAMESPACE).is_ok() {
        namespaces.push(namespace(s)?);
    }

    s.expect(TokenKind::Eof)?;
    Ok(Package {
        id: NodeId::PLACEHOLDER,
        namespaces,
    })
}

fn namespace(s: &mut Scanner) -> Result<Namespace> {
    let lo = s.span().lo;
    let name = path(s)?;
    s.expect(TokenKind::Open(Delim::Brace))?;

    let mut items = Vec::new();
    while let Some(item) = opt(s, item)? {
        items.push(item);
    }

    s.expect(TokenKind::Close(Delim::Brace))?;
    let hi = s.span().hi;
    Ok(Namespace {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        name,
        items,
    })
}

fn item(s: &mut Scanner) -> Result<Item> {
    let lo = s.span().lo;
    let meta = DeclMeta {
        attrs: Vec::new(),
        visibility: None,
    };

    let kind = if s.keyword(kw::FUNCTION).is_ok() {
        let decl = callable_decl(s, CallableKind::Function)?;
        Ok(ItemKind::Callable(meta, decl))
    } else if s.keyword(kw::OPERATION).is_ok() {
        let decl = callable_decl(s, CallableKind::Operation)?;
        Ok(ItemKind::Callable(meta, decl))
    } else {
        Err(s.error("Expecting namespace item.".to_string()))
    }?;

    let hi = s.span().hi;
    Ok(Item {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
    })
}

fn callable_decl(s: &mut Scanner, kind: CallableKind) -> Result<CallableDecl> {
    let lo = s.span().lo;
    let name = ident(s)?;

    let ty_params = if s.expect(TokenKind::Lt).is_ok() {
        let ty_params = comma_sep(s, ty::var)?;
        s.expect(TokenKind::Gt)?;
        ty_params
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    s.expect(TokenKind::Colon)?;
    let output = ty(s)?;
    let body = callable_body(s)?;
    let hi = s.span().hi;
    Ok(CallableDecl {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        kind,
        name,
        ty_params,
        input,
        output,
        functors: None,
        body,
    })
}

fn callable_body(s: &mut Scanner) -> Result<CallableBody> {
    s.expect(TokenKind::Open(Delim::Brace))?;
    let mut specs = Vec::new();
    while let Some(spec) = opt(s, spec_decl)? {
        specs.push(spec);
    }

    s.expect(TokenKind::Close(Delim::Brace))?;
    Ok(CallableBody::Specs(specs))
}

fn spec_decl(s: &mut Scanner) -> Result<SpecDecl> {
    let spec = if s.keyword(kw::BODY).is_ok() {
        Spec::Body
    } else if s.keyword(kw::ADJOINT).is_ok() {
        Spec::Adj
    } else if s.keyword(kw::CONTROLLED).is_ok() {
        if s.keyword(kw::ADJOINT).is_ok() {
            Spec::CtlAdj
        } else {
            Spec::Ctl
        }
    } else {
        return Err(s.error("Expecting specialization.".to_string()));
    };

    let lo = s.span().lo;
    let gen = if s.keyword(kw::AUTO).is_ok() {
        SpecGen::Auto
    } else if s.keyword(kw::DISTRIBUTE).is_ok() {
        SpecGen::Distribute
    } else if s.keyword(kw::INTRINSIC).is_ok() {
        SpecGen::Intrinsic
    } else if s.keyword(kw::INVERT).is_ok() {
        SpecGen::Invert
    } else if s.keyword(kw::SELF).is_ok() {
        SpecGen::Slf
    } else {
        return Err(s.error("Expecting specialization generator.".to_string()));
    };

    s.expect(TokenKind::Semi)?;
    let hi = s.span().hi;
    Ok(SpecDecl {
        id: NodeId::PLACEHOLDER,
        span: Span { lo, hi },
        spec,
        body: SpecBody::Gen(gen),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::too_many_lines)]

    use super::{item, package, spec_decl};
    use crate::parse::{scan::Scanner, Parser};
    use expect_test::{expect, Expect};
    use std::fmt::Debug;

    fn check<T: Debug>(mut parser: impl Parser<T>, input: &str, expect: &Expect) {
        let mut scanner = Scanner::new(input);
        let actual = parser(&mut scanner);
        expect.assert_debug_eq(&actual);
    }

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
                            lo: 11,
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
                        message: "Expecting Semi.",
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
                        message: "Expecting specialization generator.",
                        span: Span {
                            lo: 8,
                            hi: 11,
                        },
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
                                        lo: 9,
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
                                        lo: 10,
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
                                        lo: 9,
                                        hi: 21,
                                    },
                                    kind: Tuple(
                                        [
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 12,
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
                                        ],
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
                                        lo: 9,
                                        hi: 30,
                                    },
                                    kind: Tuple(
                                        [
                                            Pat {
                                                id: NodeId(
                                                    4294967295,
                                                ),
                                                span: Span {
                                                    lo: 12,
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
                                                    lo: 20,
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
                                        lo: 15,
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
                                        lo: 19,
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
    fn function_missing_output_ty() {
        check(
            item,
            "function Foo() { body intrinsic; }",
            &expect![[r#"
                Err(
                    Error {
                        message: "Expecting Colon.",
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
                                name: Path {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
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
                                        name: "A",
                                    },
                                },
                                items: [
                                    Item {
                                        id: NodeId(
                                            4294967295,
                                        ),
                                        span: Span {
                                            lo: 12,
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
                                                        lo: 23,
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
                                name: Path {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 0,
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
                                        name: "A",
                                    },
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
                                name: Path {
                                    id: NodeId(
                                        4294967295,
                                    ),
                                    span: Span {
                                        lo: 15,
                                        hi: 26,
                                    },
                                    namespace: None,
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
                                },
                                items: [],
                            },
                        ],
                    },
                )
            "#]],
        );
    }
}
