// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, SourceId};
use crate::{id::Assigner, symbol, Error};
use expect_test::expect;
use indoc::indoc;
use qsc_ast::{
    ast::{CallableBody, CallableDecl, Expr, ExprKind, ItemKind, Lit, Path, Span},
    mut_visit::MutVisitor,
};

fn error_span(error: &Error) -> Span {
    match error {
        Error::Parse(error) => error.span,
        Error::Symbol(
            symbol::Error::NotFound(_, span) | symbol::Error::Ambiguous(_, span, _, _),
        ) => *span,
    }
}

#[test]
fn one_file_no_entry() {
    let (package, context) = compile(
        [indoc! {"
            namespace Foo {
                function A() : Unit {}
            }
        "}],
        "",
    );
    assert!(context.errors().is_empty(), "{:#?}", context.errors());
    assert!(package.entry.is_none(), "{:#?}", package.entry);
}

#[test]
fn one_file_error() {
    let (_, context) = compile(
        [indoc! {"
            namespace Foo {
                function A() : Unit {
                    x
                }
            }
        "}],
        "",
    );

    assert_eq!(context.errors().len(), 1, "{:#?}", context.errors());
    let error = &context.errors()[0];
    let (file, span) = context.source_span(error_span(error));
    assert_eq!(file, SourceId(0));
    assert_eq!(span.lo, 50);
    assert_eq!(span.hi, 51);
}

#[test]
fn two_files_dependency() {
    let (_, context) = compile(
        [
            indoc! {"
                namespace Foo {
                    function A() : Unit {}
                }
            "},
            indoc! {"
                namespace Foo {
                    function B() : Unit {
                        A();
                    }
                }
            "},
        ],
        "",
    );
    assert!(context.errors().is_empty(), "{:#?}", context.errors());
}

#[test]
fn two_files_mutual_dependency() {
    let (_, context) = compile(
        [
            indoc! {"
                namespace Foo {
                    function A() : Unit {
                        B();
                    }
                }
            "},
            indoc! {"
                namespace Foo {
                    function B() : Unit {
                        A();
                    }
                }    
            "},
        ],
        "",
    );
    assert!(context.errors().is_empty(), "{:#?}", context.errors());
}

#[test]
fn two_files_error() {
    let (_, context) = compile(
        [
            indoc! {"
                namespace Foo {
                    function A() : Unit {}
                }
            "},
            indoc! {"
                namespace Foo {
                    function B() : Unit {
                        C();
                    }
                }
            "},
        ],
        "",
    );

    assert_eq!(context.errors.len(), 1, "{:#?}", context.errors());
    let error = &context.errors()[0];
    let (file, span) = context.source_span(error_span(error));
    assert_eq!(file, SourceId(1));
    assert_eq!(span.lo, 50);
    assert_eq!(span.hi, 51);
}

#[test]
fn entry_call_operation() {
    let (package, context) = compile(
        [indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.A()",
    );
    assert!(context.errors.is_empty(), "{:#?}", context.errors());

    let operation = if let ItemKind::Callable(callable) = &package.namespaces[0].items[0].kind {
        context
            .symbols
            .get(callable.name.id)
            .expect("Callable should have a symbol ID.")
    } else {
        panic!("First item should be a callable.")
    };

    if let Some(Expr {
        kind: ExprKind::Call(callee, _),
        ..
    }) = &package.entry
    {
        if let ExprKind::Path(Path { id, .. }) = callee.kind {
            assert_eq!(context.symbols.get(id), Some(operation));
        } else {
            panic!("Callee should be a path.");
        }
    } else {
        panic!("Entry should be a call expression.");
    }
}

#[test]
fn entry_error() {
    let (_, context) = compile(
        [indoc! {"
            namespace Foo {
                operation A() : Unit {}
            }
        "}],
        "Foo.B()",
    );

    assert_eq!(context.errors.len(), 1, "{:#?}", context.errors());
    let error = &context.errors()[0];
    let (file, span) = context.source_span(error_span(error));
    assert_eq!(file, SourceId(1));
    assert_eq!(span.lo, 0);
    assert_eq!(span.hi, 5);
}

#[test]
fn replace_node() {
    struct Replacer<'a>(&'a mut Assigner);

    impl MutVisitor for Replacer<'_> {
        fn visit_expr(&mut self, expr: &mut Expr) {
            *expr = Expr {
                id: self.0.next_id(),
                span: expr.span,
                kind: ExprKind::Lit(Lit::Int(2)),
            };
        }
    }

    let (mut package, mut context) = compile(
        [indoc! {"
            namespace Foo {
                function A() : Int {
                    1
                }
            }"}],
        "",
    );

    Replacer(context.assigner_mut()).visit_package(&mut package);

    let ItemKind::Callable(CallableDecl {
        body: CallableBody::Block(block),
        ..
    }) = &package.namespaces[0].items[0].kind else {
        panic!("Expected callable item.");
    };

    expect![[r#"
        Block {
            id: NodeId(
                8,
            ),
            span: Span {
                lo: 39,
                hi: 56,
            },
            stmts: [
                Stmt {
                    id: NodeId(
                        9,
                    ),
                    span: Span {
                        lo: 49,
                        hi: 50,
                    },
                    kind: Expr(
                        Expr {
                            id: NodeId(
                                11,
                            ),
                            span: Span {
                                lo: 49,
                                hi: 50,
                            },
                            kind: Lit(
                                Int(
                                    2,
                                ),
                            ),
                        },
                    ),
                },
            ],
        }
    "#]]
    .assert_debug_eq(&block);
}
