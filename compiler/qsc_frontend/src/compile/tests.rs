// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, Context, Error, PackageStore, SourceIndex};
use expect_test::expect;
use indoc::indoc;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        CallableBody, Expr, ExprKind, ItemId, ItemKind, Lit, LocalItemId, NodeId, Res, StmtKind,
    },
    mut_visit::MutVisitor,
};

fn error_span(error: &Error) -> Span {
    let label = error
        .labels()
        .and_then(|mut ls| ls.next())
        .expect("error should have at least one label");

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

    let errors: Vec<_> = unit
        .context
        .errors()
        .iter()
        .map(|error| source_span(&unit.context, error))
        .collect();

    assert_eq!(
        vec![
            (SourceIndex(0), Span { lo: 50, hi: 51 }),
            (SourceIndex(0), Span { lo: 40, hi: 57 })
        ],
        errors,
    );
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

    let errors: Vec<_> = unit
        .context
        .errors()
        .iter()
        .map(|error| source_span(&unit.context, error))
        .collect();

    assert_eq!(
        vec![
            (SourceIndex(1), Span { lo: 50, hi: 51 }),
            (SourceIndex(1), Span { lo: 50, hi: 53 })
        ],
        errors,
    );
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
    let entry = unit.package.entry.expect("package should have entry");
    let ExprKind::Call(callee, _) = entry.kind else { panic!("entry should be a call") };
    let ExprKind::Name(res) = callee.kind else { panic!("callee should be a name") };
    assert_eq!(
        Res::Item(ItemId {
            package: None,
            item: LocalItemId::from(1)
        }),
        res
    );
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
    let (source, span) = source_span(&unit.context, &errors[0]);
    assert_eq!(source, SourceIndex(1));
    assert_eq!(span, Span { lo: 0, hi: 5 });
}

#[test]
fn replace_node() {
    struct Replacer;

    impl MutVisitor for Replacer {
        fn visit_expr(&mut self, expr: &mut Expr) {
            *expr = Expr {
                id: NodeId::default(),
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

    Replacer.visit_package(&mut unit.package);
    unit.context.assigner_mut().visit_package(&mut unit.package);
    let ItemKind::Callable(callable)= &unit.package.items.get(LocalItemId::from(1)).expect("").kind else {
        panic!("item should be a callable");
    };
    let CallableBody::Block(block) = &callable.body else { panic!("callable body should be a block") };

    expect![[r#"
        Block 4 [39-56]:
            Stmt 5 [49-50]: Expr: Expr 8 [49-50]: Lit: Int(2)"#]]
    .assert_eq(&block.to_string());
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

    let ItemKind::Callable(callable) = &unit2.package.items.get(LocalItemId::from(1)).expect("").kind else {
        panic!("item should be a callable");
    };
    let CallableBody::Block(block) = &callable.body else { panic!("callable body should be a block") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("statement should be an expression") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("expression should be a call") };
    let ExprKind::Name(res) = &callee.kind else { panic!("callee should be a name") };
    assert_eq!(
        &Res::Item(ItemId {
            package: Some(package1),
            item: LocalItemId::from(1)
        }),
        res
    );
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

    let ItemKind::Callable(callable) = &unit2.package.items.get(LocalItemId::from(1)).expect("").kind else {
        panic!("item should be a callable");
    };
    let CallableBody::Block(block) = &callable.body else { panic!("callable body should be a block") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("statement should be an expression") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("expression should be a call") };
    let ExprKind::Name(res) = &callee.kind else { panic!("callee should be a name") };
    assert_eq!(&Res::Err, res);
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
