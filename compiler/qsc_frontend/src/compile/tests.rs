// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, Error, PackageStore, SourceMap};
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
use std::path::Path;

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

fn source_span<'a>(sources: &'a SourceMap, error: &Error) -> (&'a Path, Span) {
    let span = error_span(error);
    let source = sources.find_by_offset(span.lo);
    (
        &source.name,
        Span {
            lo: span.lo - source.offset,
            hi: span.hi - source.offset,
        },
    )
}

#[test]
fn one_file_no_entry() {
    let unit = compile(
        &PackageStore::new(),
        [],
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Foo {
                        function A() : Unit {}
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
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
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Foo {
                        function A() : Unit {
                            x
                        }
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
    );

    let errors: Vec<_> = unit
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(vec![("source1".as_ref(), Span { lo: 50, hi: 51 })], errors);
}

#[test]
fn two_files_dependency() {
    let unit = compile(
        &PackageStore::new(),
        [],
        SourceMap::new(
            [
                (
                    "source1".into(),
                    indoc! {"
                        namespace Foo {
                            function A() : Unit {}
                        }
                    "}
                    .to_string(),
                ),
                (
                    "source2".into(),
                    indoc! {"
                        namespace Foo {
                            function B() : Unit {
                                A();
                            }
                        }
                    "}
                    .to_string(),
                ),
            ],
            String::new(),
        ),
    );

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}

#[test]
fn two_files_mutual_dependency() {
    let unit = compile(
        &PackageStore::new(),
        [],
        SourceMap::new(
            [
                (
                    "source1".into(),
                    indoc! {"
                        namespace Foo {
                            function A() : Unit {
                                B();
                            }
                        }
                    "}
                    .to_string(),
                ),
                (
                    "source2".into(),
                    indoc! {"
                        namespace Foo {
                            function B() : Unit {
                                A();
                            }
                        }    
                    "}
                    .to_string(),
                ),
            ],
            String::new(),
        ),
    );

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}

#[test]
fn two_files_error() {
    let unit = compile(
        &PackageStore::new(),
        [],
        SourceMap::new(
            [
                (
                    "source1".into(),
                    indoc! {"
                        namespace Foo {
                            function A() : Unit {}
                        }
                    "}
                    .to_string(),
                ),
                (
                    "source2".into(),
                    indoc! {"
                        namespace Foo {
                            function B() : Unit {
                                C();
                            }
                        }
                    "}
                    .to_string(),
                ),
            ],
            String::new(),
        ),
    );

    let errors: Vec<_> = unit
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(vec![("source2".as_ref(), Span { lo: 50, hi: 51 })], errors);
}

#[test]
fn entry_call_operation() {
    let unit = compile(
        &PackageStore::new(),
        [],
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Foo {
                        operation A() : Unit {}
                    }
                "}
                .to_string(),
            )],
            "Foo.A()".to_string(),
        ),
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
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Foo {
                        operation A() : Unit {}
                    }
                "}
                .to_string(),
            )],
            "Foo.B()".to_string(),
        ),
    );

    assert_eq!(
        ("<entry>".as_ref(), Span { lo: 0, hi: 5 }),
        source_span(&unit.sources, &unit.errors[0])
    );
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
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace A {
                        function Foo() : Int {
                            1
                        }
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
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
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Package1 {
                        function Foo() : Int {
                            1
                        }
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
    );

    let package1 = store.insert(unit1);
    let unit2 = compile(
        &store,
        [package1],
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Package2 {
                        function Bar() : Int {
                            Package1.Foo()
                        }
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
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
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Package1 {
                        internal function Foo() : Int {
                            1
                        }
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
    );

    let package1 = store.insert(unit1);
    let unit2 = compile(
        &store,
        [package1],
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Package2 {
                        function Bar() : Int {
                            Package1.Foo()
                        }
                    }
                "}
                .to_string(),
            )],
            String::new(),
        ),
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
        SourceMap::new(
            [(
                "source1".into(),
                indoc! {"
                    namespace Foo {
                        open Microsoft.Quantum.Intrinsic;

                        operation Main() : Unit {
                            use q = Qubit();
                            X(q);
                        }
                    }
                "}
                .to_string(),
            )],
            "Foo.Main()".to_string(),
        ),
    );

    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}
