// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::TargetProfile;

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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);

    let entry = &unit.package.entry.expect("package should have entry");
    let ExprKind::Call(callee, _) = &entry.kind else {
        panic!("entry should be a call")
    };
    let ExprKind::Var(res, _) = &callee.kind else {
        panic!("callee should be a variable")
    };
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

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
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

    let mut unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
    Replacer.visit_package(&mut unit.package);
    unit.assigner.visit_package(&mut unit.package);

    let ItemKind::Callable(callable) = &unit
        .package
        .items
        .get(LocalItemId::from(1))
        .expect("package should have item")
        .kind
    else {
        panic!("item should be a callable");
    };
    let SpecBody::Impl(_, block) = &callable.body.body else {
        panic!("callable body have a block")
    };
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
    let mut unit = compile(&store, &[], sources, TargetProfile::Full);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
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
    let unit1 = compile(&store, &[], sources1, TargetProfile::Full);
    assert!(unit1.errors.is_empty(), "{:#?}", unit1.errors);
    let package1 = store.insert(unit1);

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
    let unit2 = compile(&store, &[package1], sources2, TargetProfile::Full);
    assert!(unit2.errors.is_empty(), "{:#?}", unit2.errors);

    expect![[r#"
        Package:
            Item 0 [0-78] (Public):
                Namespace (Ident 9 [10-18] "Package2"): Item 1
            Item 1 [25-76] (Public):
                Parent: 0
                Callable 0 [25-76] (function):
                    name: Ident 1 [34-37] "Bar"
                    input: Pat 2 [37-39] [Type Unit]: Unit
                    output: Int
                    functors: empty set
                    body: SpecDecl 3 [25-76]: Impl:
                        Block 4 [46-76] [Type Int]:
                            Stmt 5 [56-70]: Expr: Expr 6 [56-70] [Type Int]: Call:
                                Expr 7 [56-68] [Type (Unit -> Int)]: Var: Item 1 (Package 1)
                                Expr 8 [68-70] [Type Unit]: Unit
                    adj: <none>
                    ctl: <none>
                    ctl-adj: <none>"#]]
    .assert_eq(&unit2.package.to_string());
}

#[test]
fn package_dependency_internal_error() {
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
    let unit1 = compile(&store, &[], sources1, TargetProfile::Full);
    assert!(unit1.errors.is_empty(), "{:#?}", unit1.errors);
    let package1 = store.insert(unit1);

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
    let unit2 = compile(&store, &[package1], sources2, TargetProfile::Full);

    let errors: Vec<_> = unit2
        .errors
        .iter()
        .map(|error| source_span(&unit2.sources, error))
        .collect();
    assert_eq!(vec![("test", Span { lo: 65, hi: 68 }),], errors);

    expect![[r#"
        Package:
            Item 0 [0-78] (Public):
                Namespace (Ident 9 [10-18] "Package2"): Item 1
            Item 1 [25-76] (Public):
                Parent: 0
                Callable 0 [25-76] (function):
                    name: Ident 1 [34-37] "Bar"
                    input: Pat 2 [37-39] [Type Unit]: Unit
                    output: Int
                    functors: empty set
                    body: SpecDecl 3 [25-76]: Impl:
                        Block 4 [46-76] [Type Int]:
                            Stmt 5 [56-70]: Expr: Expr 6 [56-70] [Type Int]: Call:
                                Expr 7 [56-68] [Type ?]: Var: Err
                                Expr 8 [68-70] [Type Unit]: Unit
                    adj: <none>
                    ctl: <none>
                    ctl-adj: <none>"#]]
    .assert_eq(&unit2.package.to_string());
}

#[test]
fn package_dependency_udt() {
    let mut store = PackageStore::new(super::core());

    let sources1 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package1 {
                    newtype Bar = Int;
                    function Foo(bar : Bar) : Int {
                        bar!
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit1 = compile(&store, &[], sources1, TargetProfile::Full);
    assert!(unit1.errors.is_empty(), "{:#?}", unit1.errors);
    let package1 = store.insert(unit1);

    let sources2 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package2 {
                    function Baz() : Int {
                        Package1.Foo(Package1.Bar(1))
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit2 = compile(&store, &[package1], sources2, TargetProfile::Full);
    assert!(unit2.errors.is_empty(), "{:#?}", unit2.errors);

    expect![[r#"
        Package:
            Item 0 [0-93] (Public):
                Namespace (Ident 11 [10-18] "Package2"): Item 1
            Item 1 [25-91] (Public):
                Parent: 0
                Callable 0 [25-91] (function):
                    name: Ident 1 [34-37] "Baz"
                    input: Pat 2 [37-39] [Type Unit]: Unit
                    output: Int
                    functors: empty set
                    body: SpecDecl 3 [25-91]: Impl:
                        Block 4 [46-91] [Type Int]:
                            Stmt 5 [56-85]: Expr: Expr 6 [56-85] [Type Int]: Call:
                                Expr 7 [56-68] [Type (UDT<Item 1 (Package 1)> -> Int)]: Var: Item 2 (Package 1)
                                Expr 8 [69-84] [Type UDT<Item 1 (Package 1)>]: Call:
                                    Expr 9 [69-81] [Type (Int -> UDT<Item 1 (Package 1)>)]: Var: Item 1 (Package 1)
                                    Expr 10 [82-83] [Type Int]: Lit: Int(1)
                    adj: <none>
                    ctl: <none>
                    ctl-adj: <none>"#]]
    .assert_eq(&unit2.package.to_string());
}

#[test]
fn package_dependency_nested_udt() {
    let mut store = PackageStore::new(super::core());

    let sources1 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package1 {
                    newtype Bar = Int;
                    newtype Baz = Int;
                    newtype Foo = (bar : Bar, Baz);
                }
            "}
            .into(),
        )],
        None,
    );
    let unit1 = compile(&store, &[], sources1, TargetProfile::Full);
    assert!(unit1.errors.is_empty(), "{:#?}", unit1.errors);
    let package1 = store.insert(unit1);

    let sources2 = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Package2 {
                    function Test() : Int {
                        let bar = Package1.Bar(1);
                        let baz = Package1.Baz(2);
                        let foo = Package1.Foo(bar, baz);
                        let inner : Package1.Bar = foo::bar;
                        let (_, other : Package1.Baz) = foo!;
                        inner!
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit2 = compile(&store, &[package1], sources2, TargetProfile::Full);
    assert!(unit2.errors.is_empty(), "{:#?}", unit2.errors);

    expect![[r#"
        Package:
            Item 0 [0-274] (Public):
                Namespace (Ident 40 [10-18] "Package2"): Item 1
            Item 1 [25-272] (Public):
                Parent: 0
                Callable 0 [25-272] (function):
                    name: Ident 1 [34-38] "Test"
                    input: Pat 2 [38-40] [Type Unit]: Unit
                    output: Int
                    functors: empty set
                    body: SpecDecl 3 [25-272]: Impl:
                        Block 4 [47-272] [Type Int]:
                            Stmt 5 [57-83]: Local (Immutable):
                                Pat 6 [61-64] [Type UDT<Item 1 (Package 1)>]: Bind: Ident 7 [61-64] "bar"
                                Expr 8 [67-82] [Type UDT<Item 1 (Package 1)>]: Call:
                                    Expr 9 [67-79] [Type (Int -> UDT<Item 1 (Package 1)>)]: Var: Item 1 (Package 1)
                                    Expr 10 [80-81] [Type Int]: Lit: Int(1)
                            Stmt 11 [92-118]: Local (Immutable):
                                Pat 12 [96-99] [Type UDT<Item 2 (Package 1)>]: Bind: Ident 13 [96-99] "baz"
                                Expr 14 [102-117] [Type UDT<Item 2 (Package 1)>]: Call:
                                    Expr 15 [102-114] [Type (Int -> UDT<Item 2 (Package 1)>)]: Var: Item 2 (Package 1)
                                    Expr 16 [115-116] [Type Int]: Lit: Int(2)
                            Stmt 17 [127-160]: Local (Immutable):
                                Pat 18 [131-134] [Type UDT<Item 3 (Package 1)>]: Bind: Ident 19 [131-134] "foo"
                                Expr 20 [137-159] [Type UDT<Item 3 (Package 1)>]: Call:
                                    Expr 21 [137-149] [Type ((UDT<Item 1 (Package 1)>, UDT<Item 2 (Package 1)>) -> UDT<Item 3 (Package 1)>)]: Var: Item 3 (Package 1)
                                    Expr 22 [149-159] [Type (UDT<Item 1 (Package 1)>, UDT<Item 2 (Package 1)>)]: Tuple:
                                        Expr 23 [150-153] [Type UDT<Item 1 (Package 1)>]: Var: Local 7
                                        Expr 24 [155-158] [Type UDT<Item 2 (Package 1)>]: Var: Local 13
                            Stmt 25 [169-205]: Local (Immutable):
                                Pat 26 [173-193] [Type UDT<Item 1 (Package 1)>]: Bind: Ident 27 [173-178] "inner"
                                Expr 28 [196-204] [Type UDT<Item 1 (Package 1)>]: Field:
                                    Expr 29 [196-199] [Type UDT<Item 3 (Package 1)>]: Var: Local 19
                                    Path(FieldPath { indices: [0] })
                            Stmt 30 [214-251]: Local (Immutable):
                                Pat 31 [218-243] [Type (UDT<Item 1 (Package 1)>, UDT<Item 2 (Package 1)>)]: Tuple:
                                    Pat 32 [219-220] [Type UDT<Item 1 (Package 1)>]: Discard
                                    Pat 33 [222-242] [Type UDT<Item 2 (Package 1)>]: Bind: Ident 34 [222-227] "other"
                                Expr 35 [246-250] [Type (UDT<Item 1 (Package 1)>, UDT<Item 2 (Package 1)>)]: UnOp (Unwrap):
                                    Expr 36 [246-249] [Type UDT<Item 3 (Package 1)>]: Var: Local 19
                            Stmt 37 [260-266]: Expr: Expr 38 [260-266] [Type Int]: UnOp (Unwrap):
                                Expr 39 [260-265] [Type UDT<Item 1 (Package 1)>]: Var: Local 27
                    adj: <none>
                    ctl: <none>
                    ctl-adj: <none>"#]]
    .assert_eq(&unit2.package.to_string());
}

#[test]
fn std_dependency() {
    let mut store = PackageStore::new(super::core());
    let std = store.insert(super::std(&store, TargetProfile::Full));
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

    let unit = compile(&store, &[std], sources, TargetProfile::Full);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}

#[test]
fn std_dependency_base_profile() {
    let mut store = PackageStore::new(super::core());
    let std = store.insert(super::std(&store, TargetProfile::Base));
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

    let unit = compile(&store, &[std], sources, TargetProfile::Base);
    assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
}

#[test]
fn introduce_prelude_ambiguity() {
    let mut store = PackageStore::new(super::core());
    let std = store.insert(super::std(&store, TargetProfile::Full));
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

    let unit = compile(&store, &[std], sources, TargetProfile::Full);
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

#[test]
fn entry_parse_error() {
    let sources = SourceMap::new(
        [(
            "test".into(),
            "namespace Foo { operation B() : Unit {} }".into(),
        )],
        Some("Foo.B)".into()),
    );

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );

    assert_eq!(
        unit.errors[0]
            .code()
            .expect("expected error code")
            .to_string(),
        "Qsc.Parse.Token"
    );

    assert_eq!(
        ("<entry>", Span { lo: 5, hi: 6 }),
        source_span(&unit.sources, &unit.errors[0])
    );
}

#[test]
fn two_files_error_eof() {
    let sources = SourceMap::new(
        [
            ("test1".into(), "namespace Foo {".into()),
            ("test2".into(), "namespace Bar {}".into()),
        ],
        None,
    );

    let unit = compile(
        &PackageStore::new(super::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
    let errors: Vec<_> = unit
        .errors
        .iter()
        .map(|error| source_span(&unit.sources, error))
        .collect();

    assert_eq!(vec![("test1", Span { lo: 15, hi: 15 }),], errors);

    expect![[r#"
        Package:
            Item 0 [0-15] (Public):
                Namespace (Ident 0 [10-13] "Foo"): <empty>
            Item 1 [16-32] (Public):
                Namespace (Ident 1 [26-29] "Bar"): <empty>"#]]
    .assert_eq(&unit.package.to_string());
}

#[test]
fn unimplemented_call_from_dependency_produces_error() {
    let lib_sources = SourceMap::new(
        [(
            "lib".into(),
            indoc! {"
                namespace Foo {
                    @Unimplemented()
                    operation Bar() : Unit {}
                }
            "}
            .into(),
        )],
        None,
    );
    let mut store = PackageStore::new(super::core());
    let lib = compile(&store, &[], lib_sources, TargetProfile::Full);
    assert!(lib.errors.is_empty(), "{:#?}", lib.errors);
    let lib = store.insert(lib);

    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Test {
                    open Foo;
                    operation Main() : Unit {
                        Bar();
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit = compile(&store, &[lib], sources, TargetProfile::Full);
    expect![[r#"
        [
            Error(
                Resolve(
                    Unimplemented(
                        "Bar",
                        Span {
                            lo: 69,
                            hi: 72,
                        },
                    ),
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&unit.errors);
}

#[test]
fn unimplemented_attribute_call_within_unit_error() {
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    @Unimplemented()
                    operation Bar() : Unit {}
                    operation Baz() : Unit {
                        Bar();
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let store = PackageStore::new(super::core());
    let unit = compile(&store, &[], sources, TargetProfile::Full);
    expect![[r#"
        [
            Error(
                Resolve(
                    Unimplemented(
                        "Bar",
                        Span {
                            lo: 104,
                            hi: 107,
                        },
                    ),
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&unit.errors);
}

#[test]
fn unimplemented_attribute_with_non_unit_expr_error() {
    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Foo {
                    @Unimplemented(1)
                    operation Bar() : Unit {}
                }
            "}
            .into(),
        )],
        None,
    );
    let store = PackageStore::new(super::core());
    let unit = compile(&store, &[], sources, TargetProfile::Full);
    expect![[r#"
        [
            Error(
                Lower(
                    InvalidAttrArgs(
                        "()",
                        Span {
                            lo: 34,
                            hi: 37,
                        },
                    ),
                ),
            ),
        ]
    "#]]
    .assert_debug_eq(&unit.errors);
}

#[test]
fn unimplemented_attribute_avoids_ambiguous_error_with_duplicate_names_in_scope() {
    let lib_sources = SourceMap::new(
        [(
            "lib".into(),
            indoc! {"
                namespace Foo {
                    @Unimplemented()
                    operation Bar() : Unit {}
                }
            "}
            .into(),
        )],
        None,
    );
    let mut store = PackageStore::new(super::core());
    let lib = compile(&store, &[], lib_sources, TargetProfile::Full);
    assert!(lib.errors.is_empty(), "{:#?}", lib.errors);
    let lib = store.insert(lib);

    let sources = SourceMap::new(
        [(
            "test".into(),
            indoc! {"
                namespace Dependency {
                    operation Bar() : Unit {}
                }
                namespace Test {
                    open Foo;
                    open Dependency;
                    operation Main() : Unit {
                        Bar();
                    }
                }
            "}
            .into(),
        )],
        None,
    );
    let unit = compile(&store, &[lib], sources, TargetProfile::Full);
    expect![[r#"
        []
    "#]]
    .assert_debug_eq(&unit.errors);
}
