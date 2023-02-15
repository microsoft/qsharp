// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    keyword::Keyword,
    prim::{dot_ident, ident, keyword, many, opt, pat, seq, token},
    scan::Scanner,
    stmt::{self, stmt},
    ty::{self, ty},
    ErrorKind, Result,
};
use crate::lex::{Delim, TokenKind};
use qsc_ast::ast::{
    Block, CallableBody, CallableDecl, CallableKind, DeclMeta, Item, ItemKind, Namespace, NodeId,
    Package, Spec, SpecBody, SpecDecl, SpecGen,
};

pub(super) fn package(s: &mut Scanner) -> Result<Package> {
    let namespaces = many(s, namespace)?;
    token(s, TokenKind::Eof)?;
    Ok(Package {
        id: NodeId::PLACEHOLDER,
        namespaces,
    })
}

fn namespace(s: &mut Scanner) -> Result<Namespace> {
    let lo = s.peek().span.lo;
    keyword(s, Keyword::Namespace)?;
    let name = dot_ident(s)?;
    token(s, TokenKind::Open(Delim::Brace))?;
    let items = many(s, item)?;
    token(s, TokenKind::Close(Delim::Brace))?;
    Ok(Namespace {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        name,
        items,
    })
}

fn item(s: &mut Scanner) -> Result<Item> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Open).is_ok() {
        let name = dot_ident(s)?;
        let alias = if keyword(s, Keyword::As).is_ok() {
            Some(dot_ident(s)?)
        } else {
            None
        };
        Ok(ItemKind::Open(name, alias))
    } else if let Some(decl) = opt(s, callable_decl)? {
        let meta = DeclMeta {
            attrs: Vec::new(),
            visibility: None,
        };
        Ok(ItemKind::Callable(meta, decl))
    } else {
        Err(s.error(ErrorKind::Rule("namespace item")))
    }?;

    Ok(Item {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        kind,
    })
}

fn callable_decl(s: &mut Scanner) -> Result<CallableDecl> {
    let lo = s.peek().span.lo;
    let kind = if keyword(s, Keyword::Function).is_ok() {
        Ok(CallableKind::Function)
    } else if keyword(s, Keyword::Operation).is_ok() {
        Ok(CallableKind::Operation)
    } else {
        Err(s.error(ErrorKind::Rule("callable declaration")))
    }?;

    let name = ident(s)?;
    let ty_params = if token(s, TokenKind::Lt).is_ok() {
        let vars = seq(s, ty::var)?.0;
        token(s, TokenKind::Gt)?;
        vars
    } else {
        Vec::new()
    };

    let input = pat(s)?;
    token(s, TokenKind::Colon)?;
    let output = ty(s)?;
    let body = callable_body(s)?;

    Ok(CallableDecl {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
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
    let lo = s.peek().span.lo;
    token(s, TokenKind::Open(Delim::Brace))?;
    let specs = many(s, spec_decl)?;
    if specs.is_empty() {
        let stmts = many(s, stmt)?;
        token(s, TokenKind::Close(Delim::Brace))?;
        Ok(CallableBody::Block(Block {
            id: NodeId::PLACEHOLDER,
            span: s.span(lo),
            stmts,
        }))
    } else {
        token(s, TokenKind::Close(Delim::Brace))?;
        Ok(CallableBody::Specs(specs))
    }
}

fn spec_decl(s: &mut Scanner) -> Result<SpecDecl> {
    let lo = s.peek().span.lo;
    let spec = if keyword(s, Keyword::Body).is_ok() {
        Ok(Spec::Body)
    } else if keyword(s, Keyword::Adjoint).is_ok() {
        Ok(Spec::Adj)
    } else if keyword(s, Keyword::Controlled).is_ok() {
        if keyword(s, Keyword::Adjoint).is_ok() {
            Ok(Spec::CtlAdj)
        } else {
            Ok(Spec::Ctl)
        }
    } else {
        Err(s.error(ErrorKind::Rule("specialization")))
    }?;

    let body = if let Some(gen) = opt(s, spec_gen)? {
        token(s, TokenKind::Semi)?;
        SpecBody::Gen(gen)
    } else {
        SpecBody::Impl(pat(s)?, stmt::block(s)?)
    };

    Ok(SpecDecl {
        id: NodeId::PLACEHOLDER,
        span: s.span(lo),
        spec,
        body,
    })
}

fn spec_gen(s: &mut Scanner) -> Result<SpecGen> {
    if keyword(s, Keyword::Auto).is_ok() {
        Ok(SpecGen::Auto)
    } else if keyword(s, Keyword::Distribute).is_ok() {
        Ok(SpecGen::Distribute)
    } else if keyword(s, Keyword::Intrinsic).is_ok() {
        Ok(SpecGen::Intrinsic)
    } else if keyword(s, Keyword::Invert).is_ok() {
        Ok(SpecGen::Invert)
    } else if keyword(s, Keyword::Slf).is_ok() {
        Ok(SpecGen::Slf)
    } else {
        Err(s.error(ErrorKind::Rule("specialization generator")))
    }
}

#[cfg(test)]
mod tests {
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
}
