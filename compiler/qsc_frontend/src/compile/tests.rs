// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, Error, PackageStore, SourceIndex, SourceMap};
use expect_test::expect;
use indoc::indoc;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        CallableBody, Expr, ExprKind, ItemId, ItemKind, Lit, LocalItemId, NodeId, PrimTy, Res,
        StmtKind, Ty,
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

fn source_span(sources: &SourceMap, error: &Error) -> (SourceIndex, Span) {
    let span = error_span(error);
    let (index, offset) = sources.offset(span.lo);
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

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
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
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(vec![(SourceIndex(0), Span { lo: 50, hi: 51 })], errors);
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

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
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

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
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
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(vec![(SourceIndex(1), Span { lo: 50, hi: 51 })], errors);
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

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);

    let entry = &unit.package.entry.expect("package should have entry");
    let ExprKind::Call(callee, _) = &entry.kind else { panic!("entry should be a call") };
    let ExprKind::Name(res) = &callee.kind else { panic!("callee should be a name") };
    assert_eq!(
        &Res::Item(ItemId {
            package: None,
            item: LocalItemId::from(1),
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

    let errors = unit.errors;
    let (source, span) = source_span(&unit.sources, &errors[0]);
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
                ty: Ty::Prim(PrimTy::Int),
                kind: ExprKind::Lit(Lit::Int(2)),
            };
        }
    }

    let mut unit = compile(
        &PackageStore::new(),
        [],
        [indoc! {"
            namespace A {
                function Foo() : Int {
                    1
                }
            }
        "}],
        "",
    );

    Replacer.visit_package(&mut unit.package);
    unit.assigner.visit_package(&mut unit.package);

    let ItemKind::Callable(callable) = &unit
        .package
        .items
        .get(LocalItemId::from(1))
        .expect("package should have item")
        .kind else { panic!("item should be a callable"); };
    let CallableBody::Block(block) = &callable.body else { panic!("callable body should be a block") };
    expect![[r#"
        Block 3 [39-56] [Type Int]:
            Stmt 4 [49-50]: Expr: Expr 7 [49-50] [Type Int]: Lit: Int(2)"#]]
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

    let foo_id = LocalItemId::from(1);
    let ItemKind::Callable(callable) = &unit2
        .package
        .items
        .get(foo_id)
        .expect("package should have item")
        .kind else { panic!("item should be a callable"); };
    let CallableBody::Block(block) = &callable.body else { panic!("callable body should be a block") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("statement should be an expression") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("expression should be a call") };
    let ExprKind::Name(res) = &callee.kind else { panic!("callee should be a name") };
    assert_eq!(
        &Res::Item(ItemId {
            package: Some(package1),
            item: foo_id,
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

    let ItemKind::Callable(callable) = &unit2
        .package
        .items
        .get(LocalItemId::from(1))
        .expect("package should have item")
        .kind else { panic!("item should be a callable"); };
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

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}
