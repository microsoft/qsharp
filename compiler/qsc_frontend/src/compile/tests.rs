// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, FileIndex};
use crate::{compile::PackageStore, id::Assigner};
use expect_test::expect;
use indoc::indoc;
use qsc_ast::{
    ast::{CallableBody, CallableDecl, Expr, ExprKind, ItemKind, Lit, Path},
    mut_visit::MutVisitor,
};

#[test]
fn one_file_no_entry() {
    let package = compile(
        &PackageStore::new(),
        &[indoc! {"
            namespace Foo {
                function A() : Unit {}
            }
        "}],
        "",
        Vec::new(),
    );
    assert!(
        package.context.errors().is_empty(),
        "{:#?}",
        package.context.errors()
    );
    assert!(
        package.package.entry.is_none(),
        "{:#?}",
        package.package.entry
    );
}

#[test]
fn one_file_error() {
    let package = compile(
        &PackageStore::new(),
        &[indoc! {"
            namespace Foo {
                function A() : Unit {
                    x
                }
            }
        "}],
        "",
        Vec::new(),
    );

    assert_eq!(
        package.context.errors().len(),
        1,
        "{:#?}",
        package.context.errors()
    );
    let error = &package.context.errors()[0];
    let (file, span) = package.context.file_span(error.span);
    assert_eq!(file, FileIndex(0));
    assert_eq!(span.lo, 50);
    assert_eq!(span.hi, 51);
}

#[test]
fn two_files_dependency() {
    let package = compile(
        &PackageStore::new(),
        &[
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
        Vec::new(),
    );
    assert!(
        package.context.errors().is_empty(),
        "{:#?}",
        package.context.errors()
    );
}

#[test]
fn two_files_mutual_dependency() {
    let package = compile(
        &PackageStore::new(),
        &[
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
        Vec::new(),
    );
    assert!(
        package.context.errors().is_empty(),
        "{:#?}",
        package.context.errors()
    );
}

#[test]
fn two_files_error() {
    let package = compile(
        &PackageStore::new(),
        &[
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
        Vec::new(),
    );

    assert_eq!(
        package.context.errors.len(),
        1,
        "{:#?}",
        package.context.errors()
    );
    let error = &package.context.errors()[0];
    let (file, span) = package.context.file_span(error.span);
    assert_eq!(file, FileIndex(1));
    assert_eq!(span.lo, 50);
    assert_eq!(span.hi, 51);
}

#[test]
fn entry_call_operation() {
    let package = compile(
        &PackageStore::new(),
        &[indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.A()",
        Vec::new(),
    );
    assert!(
        package.context.errors.is_empty(),
        "{:#?}",
        package.context.errors()
    );

    let operation =
        if let ItemKind::Callable(callable) = &package.package.namespaces[0].items[0].kind {
            package
                .context
                .symbols
                .get(callable.name.id)
                .expect("Callable should have a symbol ID.")
        } else {
            panic!("First item should be a callable.")
        };

    if let Some(Expr {
        kind: ExprKind::Call(callee, _),
        ..
    }) = &package.package.entry
    {
        if let ExprKind::Path(Path { id, .. }) = callee.kind {
            assert_eq!(package.context.symbols.get(id), Some(operation));
        } else {
            panic!("Callee should be a path.");
        }
    } else {
        panic!("Entry should be a call expression.");
    }
}

#[test]
fn entry_error() {
    let package = compile(
        &PackageStore::new(),
        &[indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.B()",
        Vec::new(),
    );

    assert_eq!(
        package.context.errors.len(),
        1,
        "{:#?}",
        package.context.errors()
    );
    let error = &package.context.errors()[0];
    let (file, span) = package.context.file_span(error.span);
    assert_eq!(file, FileIndex(1));
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

    let mut package = compile(
        &PackageStore::new(),
        &[indoc! {"
            namespace Foo {
                function A() : Int {
                    1
                }
            }"}],
        "",
        Vec::new(),
    );

    Replacer(package.context.assigner_mut()).visit_package(&mut package.package);

    let ItemKind::Callable(CallableDecl {
        body: CallableBody::Block(block),
        ..
    }) = &package.package.namespaces[0].items[0].kind else {
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
