// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, FileIndex};
use crate::{
    compile::PackageStore,
    id::Assigner,
    resolve::{PackageRes, Res},
};
use expect_test::expect;
use indoc::indoc;
use qsc_ast::{
    ast::{CallableBody, CallableDecl, Expr, ExprKind, ItemKind, Lit, Path, StmtKind},
    mut_visit::MutVisitor,
};

#[test]
fn one_file_no_entry() {
    let package = compile(
        &PackageStore::new(),
        &[],
        &[indoc! {"
            namespace Foo {
                function A() : Unit {}
            }
        "}],
        "",
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
        &[],
        &[indoc! {"
            namespace Foo {
                function A() : Unit {
                    x
                }
            }
        "}],
        "",
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
        &[],
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
        &[],
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
        &[],
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
        &[],
        &[indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.A()",
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
                .resolutions
                .get(callable.name.id)
                .expect("Callable should resolve.")
        } else {
            panic!("First item should be a callable.")
        };

    if let Some(Expr {
        kind: ExprKind::Call(callee, _),
        ..
    }) = &package.package.entry
    {
        if let ExprKind::Path(Path { id, .. }) = callee.kind {
            assert_eq!(package.context.resolutions.get(id), Some(operation));
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
        &[],
        &[indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}],
        "Foo.B()",
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
        &[],
        &[indoc! {"
            namespace Foo {
                function A() : Int {
                    1
                }
            }"}],
        "",
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

#[test]
fn package_dependency() {
    let mut store = PackageStore::new();
    let package1 = compile(
        &store,
        &[],
        &[indoc! {"
            namespace Package1 {
                function Foo() : Int {
                    1
                }
            }"}],
        "",
    );

    let foo_node_id =
        if let ItemKind::Callable(callable) = &package1.package.namespaces[0].items[0].kind {
            package1
                .context
                .resolutions
                .get(callable.name.id)
                .expect("Callable should resolve.")
                .node
        } else {
            panic!("First item should be a callable.")
        };

    let package1_id = store.insert(package1);
    let package2 = compile(
        &store,
        &[package1_id],
        &[indoc! {"
            namespace Package2 {
                function Bar() : Int {
                    Package1.Foo()
                }
            }
        "}],
        "",
    );

    let foo_ref = if let ItemKind::Callable(CallableDecl {
        body: CallableBody::Block(block),
        ..
    }) = &package2.package.namespaces[0].items[0].kind
    {
        match &block.stmts[0].kind {
            StmtKind::Expr(Expr {
                kind: ExprKind::Call(callee, _),
                ..
            }) => match &callee.kind {
                ExprKind::Path(path) => package2
                    .context
                    .resolutions
                    .get(path.id)
                    .expect("Path should resolve."),
                _ => panic!("Expression is not a path."),
            },
            _ => panic!("Statement is not a call expression."),
        }
    } else {
        panic!("Expected callable not found.");
    };

    assert_eq!(
        foo_ref,
        Res {
            package: PackageRes::Extern(package1_id),
            node: foo_node_id
        }
    );
}

#[test]
fn package_dependency_internal() {
    let mut store = PackageStore::new();
    let package1 = compile(
        &store,
        &[],
        &[indoc! {"
            namespace Package1 {
                internal function Foo() : Int {
                    1
                }
            }"}],
        "",
    );
    let package1_id = store.insert(package1);
    let package2 = compile(
        &store,
        &[package1_id],
        &[indoc! {"
            namespace Package2 {
                function Bar() : Int {
                    Package1.Foo()
                }
            }
        "}],
        "",
    );

    if let ItemKind::Callable(CallableDecl {
        body: CallableBody::Block(block),
        ..
    }) = &package2.package.namespaces[0].items[0].kind
    {
        match &block.stmts[0].kind {
            StmtKind::Expr(Expr {
                kind: ExprKind::Call(callee, _),
                ..
            }) => match &callee.kind {
                ExprKind::Path(path) => assert!(
                    package2.context.resolutions.get(path.id).is_none(),
                    "Path resolved to internal function."
                ),
                _ => panic!("Expression is not a path."),
            },
            _ => panic!("Statement is not a call expression."),
        }
    } else {
        panic!("Expected callable not found.");
    };
}
