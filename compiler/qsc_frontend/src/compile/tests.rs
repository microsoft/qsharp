// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{compile, Error, PackageStore, SourceMap};
use expect_test::expect;
use indoc::indoc;
use miette::Diagnostic;
use qsc_data_structures::span::Span;
use qsc_hir::{
    global,
    hir::{
        Block, Expr, ExprKind, ItemId, ItemKind, Lit, LocalItemId, NodeId, Res, SpecBody, Stmt,
        StmtKind,
    },
    mut_visit::MutVisitor,
    ty::{Prim, Ty},
};

fn error_span(error: &Error) -> Span {
    let label = error
        .labels()
        .and_then(|mut ls| ls.next())
        .expect("error should have at least one label");

    let span = label.inner();
    let offset = span
        .offset()
        .try_into()
        .expect("span offset should fit into u32");
    let len: u32 = span.len().try_into().expect("span len should fit into u32");
    Span {
        lo: offset,
        hi: offset + len,
    }
}

fn source_span<'a>(sources: &'a SourceMap, error: &Error) -> (&'a str, Span) {
    let span = error_span(error);
    let source = sources
        .find_by_offset(span.lo)
        .expect("offset should match at least one source");
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
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    function A() : Unit {}
                }
            "}
            .into(),
        )],
        None,
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);

    let entry = unit.package.entry.as_ref();
    assert!(entry.is_none(), "{entry:#?}");
}

#[test]
fn one_file_error() {
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    function A() : Unit {
                        x
                    }
                }
            "}
            .into(),
        )],
        None,
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    let errors: Vec<_> = unit
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(vec![("test", Span { lo: 50, hi: 51 })], errors);
}

#[test]
fn two_files_dependency() {
    let sources = SourceMap::new(
        [
            (
                "test1".into(),
                indoc! {"
                    namespace Foo {
                        function A() : Unit {}
                    }
                "}
                .into(),
            ),
            (
                "test2".into(),
                indoc! {"
                    namespace Foo {
                        function B() : Unit {
                            A();
                        }
                    }
                "}
                .into(),
            ),
        ],
        None,
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}

#[test]
fn two_files_mutual_dependency() {
    let sources = SourceMap::new(
        [
            (
                "test1".into(),
                indoc! {"
                    namespace Foo {
                        function A() : Unit {
                            B();
                        }
                    }
                "}
                .into(),
            ),
            (
                "test2".into(),
                indoc! {"
                    namespace Foo {
                        function B() : Unit {
                            A();
                        }
                    }
                "}
                .into(),
            ),
        ],
        None,
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}

#[test]
fn two_files_error() {
    let sources = SourceMap::new(
        [
            (
                "test1".into(),
                indoc! {"
                    namespace Foo {
                        function A() : Unit {}
                    }
                "}
                .into(),
            ),
            (
                "test2".into(),
                indoc! {"
                    namespace Foo {
                        function B() : Unit {
                            C();
                        }
                    }
                "}
                .into(),
            ),
        ],
        None,
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    let errors: Vec<_> = unit
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(
        vec![
            ("test2", Span { lo: 50, hi: 51 }),
            ("test2", Span { lo: 50, hi: 53 }),
        ],
        errors
    );
}

#[test]
fn entry_call_operation() {
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}
            .into(),
        )],
        Some("Foo.A()".into()),
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);

    let entry = &unit.package.entry.expect("package should have entry");
    let ExprKind::Call(callee, _) = &entry.kind else { panic!("entry should be a call") };
    let ExprKind::Var(res, _) = &callee.kind else { panic!("callee should be a variable") };
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
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    operation A() : Unit {}
                }
            "}
            .into(),
        )],
        Some("Foo.B()".into()),
    );

    let unit = compile(&PackageStore::new(super::core()), &[], sources);
    assert_eq!(
        ("<entry>", Span { lo: 4, hi: 5 }),
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
                ty: Ty::Prim(Prim::Int),
                kind: ExprKind::Lit(Lit::Int(2)),
            };
        }
    }

    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace A {
                    function Foo() : Int {
                        1
                    }
                }
            "}
            .into(),
        )],
        None,
    );

    let mut unit = compile(&PackageStore::new(super::core()), &[], sources);
    Replacer.visit_package(&mut unit.package);
    unit.assigner.visit_package(&mut unit.package);

    let ItemKind::Callable(callable) = &unit
        .package
        .items
        .get(LocalItemId::from(1))
        .expect("package should have item")
        .kind else { panic!("item should be a callable"); };
    let SpecBody::Impl(_, block) = &callable.body.body else { panic!("callable body have a block") };
    expect![[r#"
        Block 4 [39-56] [Type Int]:
            Stmt 5 [49-50]: Expr: Expr 8 [49-50] [Type Int]: Lit: Int(2)"#]]
    .assert_eq(&block.to_string());
}

#[test]
fn insert_core_call() {
    struct Inserter<'a> {
        core: &'a global::Table,
    }

    impl MutVisitor for Inserter<'_> {
        fn visit_block(&mut self, block: &mut Block) {
            let allocate = self
                .core
                .resolve_term("QIR.Runtime", "__quantum__rt__qubit_allocate")
                .expect("qubit allocation should be in core");
            let allocate_ty = allocate
                .scheme
                .instantiate(&[])
                .expect("qubit allocation scheme should instantiate");
            let callee = Expr {
                id: NodeId::default(),
                span: Span::default(),
                ty: Ty::Arrow(Box::new(allocate_ty)),
                kind: ExprKind::Var(Res::Item(allocate.id), Vec::new()),
            };

            let arg = Expr {
                id: NodeId::default(),
                span: Span::default(),
                ty: Ty::UNIT,
                kind: ExprKind::Tuple(Vec::new()),
            };

            let call = Expr {
                id: NodeId::default(),
                span: Span::default(),
                ty: Ty::Prim(Prim::Qubit),
                kind: ExprKind::Call(Box::new(callee), Box::new(arg)),
            };

            block.stmts.push(Stmt {
                id: NodeId::default(),
                span: Span::default(),
                kind: StmtKind::Semi(call),
            });
        }
    }

    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace A {
                    operation Foo() : () {}
                }
            "}
            .into(),
        )],
        None,
    );

    let store = PackageStore::new(super::core());
    let mut unit = compile(&store, &[], sources);
    let mut inserter = Inserter { core: store.core() };
    inserter.visit_package(&mut unit.package);
    unit.assigner.visit_package(&mut unit.package);

    expect![[r#"
        Package:
            Item 0 [0-43] (Public):
                Namespace (Ident 5 [10-11] "A"): Item 1
            Item 1 [18-41] (Public):
                Parent: 0
                Callable 0 [18-41] (operation):
                    name: Ident 1 [28-31] "Foo"
                    input: Pat 2 [31-33] [Type Unit]: Unit
                    output: Unit
                    functors: empty set
                    body: SpecDecl 3 [18-41]: Impl:
                        Block 4 [39-41] [Type Unit]:
                            Stmt 6 [0-0]: Semi: Expr 7 [0-0] [Type Qubit]: Call:
                                Expr 8 [0-0] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                Expr 9 [0-0] [Type Unit]: Unit
                    adj: <none>
                    ctl: <none>
                    ctl-adj: <none>"#]]
    .assert_eq(&unit.package.to_string());
}

#[test]
fn package_dependency() {
    let mut store = PackageStore::new(super::core());

    let sources1 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package1 {
                    function Foo() : Int {
                        1
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let package1 = store.insert(compile(&store, &[], sources1));

    let sources2 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package2 {
                    function Bar() : Int {
                        Package1.Foo()
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit2 = compile(&store, &[package1], sources2);

    let foo_id = LocalItemId::from(1);
    let ItemKind::Callable(callable) = &unit2
        .package
        .items
        .get(foo_id)
        .expect("package should have item")
        .kind else { panic!("item should be a callable"); };
    let SpecBody::Impl(_, block) = &callable.body.body else { panic!("callable body have a block") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("statement should be an expression") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("expression should be a call") };
    let ExprKind::Var(res, _) = &callee.kind else { panic!("callee should be a variable") };
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
    let mut store = PackageStore::new(super::core());

    let sources1 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package1 {
                    internal function Foo() : Int {
                        1
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let package1 = store.insert(compile(&store, &[], sources1));

    let sources2 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package2 {
                    function Bar() : Int {
                        Package1.Foo()
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit2 = compile(&store, &[package1], sources2);

    let ItemKind::Callable(callable) = &unit2
        .package
        .items
        .get(LocalItemId::from(1))
        .expect("package should have item")
        .kind else { panic!("item should be a callable"); };
    let SpecBody::Impl(_, block) = &callable.body.body else { panic!("callable body have a block") };
    let StmtKind::Expr(expr) = &block.stmts[0].kind else { panic!("statement should be an expression") };
    let ExprKind::Call(callee, _) = &expr.kind else { panic!("expression should be a call") };
    let ExprKind::Var(res, _) = &callee.kind else { panic!("callee should be a variable") };
    assert_eq!(&Res::Err, res);
}

#[test]
fn std_dependency() {
    let mut store = PackageStore::new(super::core());
    let std = store.insert(super::std(&store));
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    open Microsoft.Quantum.Intrinsic;

                    operation Main() : Unit {
                        use q = Qubit();
                        X(q);
                    }
                }
            "}
            .into(),
        )],
        Some("Foo.Main()".into()),
    );

    let unit = compile(&store, &[std], sources);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}
#[test]
fn introduce_prelude_ambiguity() {
    let mut store = PackageStore::new(super::core());
    let std = store.insert(super::std(&store));
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"namespace Microsoft.Quantum.Canon {
                function Length () : () { }
            }
                namespace Foo {
                    function Main (): () { Length }
                }"}
            .into(),
        )],
        Some("Foo.Main()".into()),
    );

    let unit = compile(&store, &[std], sources);
    let errors: Vec<Error> = unit.errors;
    assert!(
        errors.len() == 1
            && matches!(
                errors[0],
                Error(super::ErrorKind::Resolve(
                    super::resolve::Error::AmbiguousPrelude { .. }
                ))
            )
    );
}
