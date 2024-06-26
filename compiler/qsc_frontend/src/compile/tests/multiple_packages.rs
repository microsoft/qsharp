use std::sync::Arc;

use super::{compile, PackageStore, SourceMap};
use crate::compile::TargetCapabilityFlags;

use crate::compile::core;
use expect_test::expect;
use indoc::indoc;
use qsc_data_structures::language_features::LanguageFeatures;

#[test]
fn multiple_packages_reference_exports() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                function FunctionA() : Int {
                    1
                }
                export FunctionA;
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_b = SourceMap::new(
        [(
            "PackageB".into(),
            indoc! {"
                function FunctionB() : Int {
                    1
                }
                export FunctionB;
            "}
            .into(),
        )],
        None,
    );

    let package_b = compile(
        &store,
        &[],
        package_b,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    assert!(package_b.errors.is_empty(), "{:#?}", package_b.errors);

    let package_a = store.insert(package_a);
    let package_b = store.insert(package_b);

    let user_code = SourceMap::new(
        [(
            "UserCode".into(),
            indoc! {"
                    import A.PackageA.FunctionA;
                    import B.PackageB.FunctionB;
                    @EntryPoint()
                    function Main() : Unit {
                       FunctionA();
                       FunctionB();
                    }
                "}
            .into(),
        )],
        None,
    );

    let user_code = compile(
        &store,
        &[
            (package_a, Some(Arc::from("A"))),
            (package_b, Some(Arc::from("B"))),
        ],
        user_code,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    assert!(user_code.errors.is_empty(), "{:#?}", user_code.errors);
}

#[test]
#[allow(clippy::too_many_lines)]
fn multiple_packages_disallow_unexported_imports() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                function FunctionA() : Int {
                    1
                }
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_a = store.insert(package_a);

    let user_code = SourceMap::new(
        [(
            "UserCode".into(),
            indoc! {"
                    import A.PackageA.FunctionA;
                    @EntryPoint()
                    function Main() : Unit {
                       FunctionA();
                    }
                "}
            .into(),
        )],
        None,
    );

    let user_code = compile(
        &store,
        &[(package_a, Some(Arc::from("A")))],
        user_code,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    expect![[r#"
        [
            Error(
                Resolve(
                    NotFound(
                        "A.PackageA.FunctionA",
                        Span {
                            lo: 7,
                            hi: 27,
                        },
                    ),
                ),
            ),
            Error(
                Resolve(
                    NotFound(
                        "FunctionA",
                        Span {
                            lo: 71,
                            hi: 80,
                        },
                    ),
                ),
            ),
            Error(
                Type(
                    Error(
                        AmbiguousTy(
                            Span {
                                lo: 71,
                                hi: 82,
                            },
                        ),
                    ),
                ),
            ),
        ]"#]]
    .assert_eq(&format!("{:#?}", user_code.errors));
}

#[test]
fn reexport() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                export Microsoft.Quantum.Core.Length as Foo;
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_a = store.insert(package_a);

    let user_code = SourceMap::new(
        [(
            "UserCode".into(),
            indoc! {"
                    import A.PackageA.Foo;
                    @EntryPoint()
                    function Main() : Unit {
                        use qs = Qubit[2];
                        let len = Foo(qs);
                    }
                "}
            .into(),
        )],
        None,
    );

    let user_code = compile(
        &store,
        &[(package_a, Some(Arc::from("A")))],
        user_code,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    expect!["[]"].assert_eq(&format!("{:#?}", user_code.errors));
}

#[test]
fn reexport_export_has_alias() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo as Bar;
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_a = store.insert(package_a);

    let package_b = SourceMap::new(
        [(
            "PackageB.qs".into(),
            indoc! {"
                import A.PackageA.Bar;
            "}
            .into(),
        )],
        None,
    );

    let package_b = compile(
        &store,
        &[(package_a, Some(Arc::from("A")))],
        package_b,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    expect!["[]"].assert_eq(&format!("{:#?}", package_b.errors));
}

#[test]
fn reexport_import_has_alias() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_a = store.insert(package_a);

    let package_b = SourceMap::new(
        [(
            "PackageB.qs".into(),
            indoc! {"
                import A.PackageA.Foo as Bar;

                export Bar;
            "}
            .into(),
        )],
        None,
    );

    let package_b = compile(
        &store,
        &[(package_a, Some(Arc::from("A")))],
        package_b,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    expect!["[]"].assert_eq(&format!("{:#?}", package_b.errors));
}
#[test]
fn reexport_reexport_has_alias() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_a = store.insert(package_a);

    let package_b = SourceMap::new(
        [(
            "PackageB.qs".into(),
            indoc! {"
                import A.PackageA.Foo;

                export Foo as Bar;
            "}
            .into(),
        )],
        None,
    );

    let package_b = compile(
        &store,
        &[(package_a, Some(Arc::from("A")))],
        package_b,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_b.errors.is_empty(), "{:#?}", package_b.errors);

    let package_b = store.insert(package_b);

    let user_code = SourceMap::new(
        [(
            "UserCode".into(),
            indoc! {"
                    import B.PackageB.Bar;
                    @EntryPoint()
                    function Main() : Unit {
                        Bar(10, true);
                    }
                "}
            .into(),
        )],
        None,
    );

    let user_code = compile(
        &store,
        &[(package_b, Some(Arc::from("B")))],
        user_code,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    expect!["[]"].assert_eq(&format!("{:#?}", user_code.errors));
}

#[test]
fn reexport_callable_combined_aliases() {
    let mut store = PackageStore::new(core());

    let package_a = SourceMap::new(
        [(
            "PackageA.qs".into(),
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            "}
            .into(),
        )],
        None,
    );

    let package_a = compile(
        &store,
        &[],
        package_a,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_a.errors.is_empty(), "{:#?}", package_a.errors);

    let package_a = store.insert(package_a);

    let package_b = SourceMap::new(
        [(
            "PackageB.qs".into(),
            indoc! {"
                import A.PackageA.Foo;
                import A.PackageA.Foo as Foo2;
                export Foo, Foo as Bar, Foo2, Foo2 as Bar2;

            "}
            .into(),
        )],
        None,
    );

    let package_b = compile(
        &store,
        &[(package_a, Some(Arc::from("A")))],
        package_b,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );
    assert!(package_b.errors.is_empty(), "{:#?}", package_b.errors);

    let package_b = store.insert(package_b);

    let user_code = SourceMap::new(
        [(
            "UserCode".into(),
            indoc! {"
                    import B.PackageB.Foo, B.PackageB.Bar, B.PackageB.Foo2, B.PackageB.Bar2;
                    @EntryPoint()
                    operation Main() : Unit {
                        Foo(10, true);
                        Foo2(10, true);
                        Bar(10, true);
                        Bar2(10, true);
                    }
                "}
            .into(),
        )],
        None,
    );

    let user_code = compile(
        &store,
        &[(package_b, Some(Arc::from("B")))],
        user_code,
        TargetCapabilityFlags::all(),
        LanguageFeatures::default(),
    );

    expect!["[]"].assert_eq(&format!("{:#?}", user_code.errors));
}
