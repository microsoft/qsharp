// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, Context, Error, PackageStore, SourceIndex};
use crate::{id::Assigner, resolve::PackageSrc};
use expect_test::expect;
use indoc::indoc;
use miette::Diagnostic;
use qsc_ast::{
    ast::{CallableBody, Expr, ExprKind, ItemKind, Lit, Span, StmtKind},
    mut_visit::MutVisitor,
};

fn error_span(error: &Error) -> Span {
    let label = error
        .labels()
        .and_then(|mut ls| ls.next())
        .expect("Error should have label.");

    let span = label.inner();
    Span {
        lo: span.offset(),
        hi: span.offset() + span.len(),
    }
}

fn source_span(context: &Context, error: &Error) -> (SourceIndex, Span) {
    let span = error_span(error);
    let (index, offset) = context.source(span.lo);
    (
        index,
        Span {
            lo: span.lo - offset,
            hi: span.hi - offset,
        },
    )
}

#[test]
fn one_file_no_entry() {
    let unit = compile(
        &PackageStore::new(),
        [],
        [indoc! {"
            namespace Foo {
                function A() : Unit {}
            }
        "}],
        "",
    );

    let errors = unit.context.errors();
    assert!(errors.is_empty(), "{errors:#?}");
    let entry = unit.package.entry.as_ref();
    assert!(entry.is_none(), "{entry:#?}");
}

#[test]
fn one_file_error() {
    let unit = compile(
        &PackageStore::new(),
        [],
        [indoc! {"
            namespace Foo {
                function A() : Unit {
                    x
                }
            }
        "}],
        "",
    );

    let errors = unit.context.errors();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let (source, span) = source_span(&unit.context, &errors[0]);
    assert_eq!(source, SourceIndex(0));
    assert_eq!(span, Span { lo: 50, hi: 51 });
}

#[test]
fn two_files_dependency() {
    let unit = compile(
        &PackageStore::new(),
        [],
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

    let errors = unit.context.errors();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn two_files_mutual_dependency() {
    let unit = compile(
        &PackageStore::new(),
        [],
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

    let errors = unit.context.errors();
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn two_files_error() {
    let unit = compile(
        &PackageStore::new(),
        [],
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

    let errors = unit.context.errors();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let (source, span) = source_span(&unit.context, &errors[0]);
    assert_eq!(source, SourceIndex(1));
    assert_eq!(span, Span { lo: 50, hi: 51 });
}

#[test]
fn entry_call_operation() {
    let unit = compile(
        &PackageStore::new(),
        [],
        [indoc! {"
            namespace Foo {
                operation A() : Unit {}
            }
        "}],
        "Foo.A()",
    );

    let errors = unit.context.errors();
    assert!(errors.is_empty(), "{errors:#?}");
    let resolutions = unit.context.resolutions();
    let ItemKind::Callable(callable) = &unit.package.namespaces[0].items[0].kind else {
        panic!("Expected callable item.");
    };
    let id = resolutions.get(&callable.name.id).expect("Should resolve.");
    let entry = unit.package.entry.expect("Should have entry expression.");
    let ExprKind::Call(callee, _) = entry.kind else { panic!("Expected call.") };
    let ExprKind::Path(path) = callee.kind else { panic!("Expected path.") };
    assert_eq!(unit.context.resolutions.get(&path.id), Some(id));
}

#[test]
fn entry_error() {
    let unit = compile(
        &PackageStore::new(),
        [],
        [indoc! {"
            namespace Foo {
                operation A() : Unit {}
            }
        "}],
        "Foo.B()",
    );

    let errors = unit.context.errors();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let (source, span) = source_span(&unit.context, &errors[0]);
    assert_eq!(source, SourceIndex(1));
    assert_eq!(span, Span { lo: 0, hi: 5 });
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

    let mut unit = compile(
        &PackageStore::new(),
        [],
        [indoc! {"
            namespace Foo {
                function A() : Int {
                    1
                }
            }
        "}],
        "",
    );

    Replacer(unit.context.assigner_mut()).visit_package(&mut unit.package);
    let ItemKind::Callable(callable)= &unit.package.namespaces[0].items[0].kind else {
        panic!("Expected callable.");
    };
    let CallableBody::Block(block) = &callable.body else { panic!("Expected block.") };

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
    let unit1 = compile(
        &store,
        [],
        [indoc! {"
            namespace Package1 {
                function Foo() : Int {
                    1
                }
            }
        "}],
        "",
    );

    let foo = if let ItemKind::Callable(foo) = &unit1.package.namespaces[0].items[0].kind {
        foo.name.id
    } else {
        panic!("Expected callable.");
    };

    let package1 = store.insert(unit1);
    let unit2 = compile(
        &store,
        [package1],
        [indoc! {"
            namespace Package2 {
                function Bar() : Int {
                    Package1.Foo()
                }
            }
        "}],
        "",
    );

    let ItemKind::Callable(callable) = &unit2.package.namespaces[0].items[0].kind else {
        panic!("Expected callable.");
    };
    let CallableBody::Block(block) = &callable.body else { panic!("Expected block.") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("Expected expression.") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("Expected call.") };
    let ExprKind::Path(path) = &callee.kind else { panic!("Expected path.") };
    let resolutions = unit2.context.resolutions();
    let id = resolutions.get(&path.id).expect("Should resolve.");
    assert_eq!(id.package, PackageSrc::Extern(package1));
    assert_eq!(id.node, foo);
}

#[test]
fn package_dependency_internal() {
    let mut store = PackageStore::new();
    let unit1 = compile(
        &store,
        [],
        [indoc! {"
            namespace Package1 {
                internal function Foo() : Int {
                    1
                }
            }
        "}],
        "",
    );

    let package1 = store.insert(unit1);
    let unit2 = compile(
        &store,
        [package1],
        [indoc! {"
            namespace Package2 {
                function Bar() : Int {
                    Package1.Foo()
                }
            }
        "}],
        "",
    );

    let ItemKind::Callable(callable) = &unit2.package.namespaces[0].items[0].kind else {
        panic!("Expected callable.");
    };
    let CallableBody::Block(block) = &callable.body else { panic!("Expected block.") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("Expected expression.") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("Expected call.") };
    let ExprKind::Path(path) = &callee.kind else { panic!("Expected path.") };
    assert!(unit2.context.resolutions.get(&path.id).is_none());
}

#[test]
fn std_dependency() {
    let mut store = PackageStore::new();
    let std = store.insert(super::std());
    let unit = compile(
        &store,
        [std],
        [indoc! {"
            namespace Foo {
                open Microsoft.Quantum.Intrinsic;

                operation Main() : Unit {
                    use q = Qubit();
                    X(q);
                }
            }
        "}],
        "Foo.Main()",
    );

    let errors = unit.context.errors();
    assert!(errors.is_empty(), "{errors:#?}");
}
