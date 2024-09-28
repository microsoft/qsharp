use std::sync::Arc;

use super::{compile, PackageStore, SourceMap};
use crate::compile::TargetCapabilityFlags;

use crate::compile::core;
use expect_test::expect;
use expect_test::Expect;
use indoc::indoc;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_hir::hir::PackageId;

/// Runs a test where each successive package relies solely on the package before it
/// useful for testing chains of reexports
fn multiple_package_check(packages: Vec<(&str, &str)>) {
    multiple_package_check_inner(packages, None);
}

fn multiple_package_check_expect_err(packages: Vec<(&str, &str)>, expect: &Expect) {
    multiple_package_check_inner(packages, Some(expect));
}

fn multiple_package_check_inner(packages: Vec<(&str, &str)>, expect: Option<&Expect>) {
    let mut store = PackageStore::new(core());
    let mut prev_id_and_name: Option<(PackageId, &str)> = None;
    let num_packages = packages.len();
    for (ix, (package_name, package_source)) in packages.into_iter().enumerate() {
        let is_last = ix == num_packages - 1;
        let deps = if let Some((prev_id, prev_name)) = prev_id_and_name {
            vec![(prev_id, Some(Arc::from(prev_name)))]
        } else {
            vec![]
        };

        let sources = SourceMap::new(
            [(
                Arc::from(format!("{package_name}.qs")),
                Arc::from(package_source),
            )],
            None,
        );

        let unit = compile(
            &store,
            &deps[..],
            sources,
            TargetCapabilityFlags::all(),
            LanguageFeatures::default(),
        );

        match (is_last, &expect) {
            (true, Some(expect)) => {
                expect.assert_eq(&format!("{:#?}", unit.errors));
            }
            _ => {
                assert!(unit.errors.is_empty(), "{:#?}", unit.errors);
            }
        }

        prev_id_and_name = Some((store.insert(unit), package_name));
    }
}

#[test]
fn namespace_named_main_doesnt_create_main_namespace() {
    multiple_package_check_expect_err(
        vec![
            (
                "Main",
                "operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;",
            ),
            (
                "C",
                r#"
            // this fails because `Main` is considered the "root"
            import Main.Main.Foo;
                    @EntryPoint()
                    operation Main() : Unit {
                        Foo(10, true);
                    }"#,
            ),
        ],
        &expect!([r#"
            [
                Error(
                    Resolve(
                        NotFound(
                            "Main.Main.Foo",
                            Span {
                                lo: 86,
                                hi: 99,
                            },
                        ),
                    ),
                ),
                Error(
                    Resolve(
                        NotFound(
                            "Foo",
                            Span {
                                lo: 205,
                                hi: 208,
                            },
                        ),
                    ),
                ),
                Error(
                    Type(
                        Error(
                            AmbiguousTy(
                                Span {
                                    lo: 205,
                                    hi: 218,
                                },
                            ),
                        ),
                    ),
                ),
            ]"#]),
    );
}

#[test]
fn namespaces_named_main_treated_as_root() {
    multiple_package_check(vec![
        (
            "Main",
            "operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;",
        ),
        (
            "C",
            "
            // note that this is not Main.Main
            // and that  the namespace `Main` is omitted here
            import Main.Foo;
                    @EntryPoint()
                    operation Main() : Unit {
                        Foo(10, true);
                    }",
        ),
    ]);
}

#[test]
fn multiple_packages_reference_exports() {
    multiple_package_check(vec![
        (
            "PackageA",
            indoc! {"
                    operation Foo(x: Int, y: Bool) : Int {
                        x
                    }
                    export Foo;
                "},
        ),
        (
            "PackageB",
            indoc! {"
                    import PackageA.PackageA.Foo;
                    export Foo;
                "},
        ),
        (
            "PackageC",
            indoc! {"
                    import PackageB.PackageB.Foo;
                    @EntryPoint()
                    operation Main() : Unit {
                        Foo(10, true);
                    }
                "},
        ),
    ]);
}

#[test]
#[allow(clippy::too_many_lines)]
fn multiple_packages_disallow_unexported_imports() {
    multiple_package_check_expect_err(
        vec![
            (
                "PackageA",
                indoc! {"
                    function FunctionA() : Int {
                        1
                    }
                "},
            ),
            (
                "PackageB",
                indoc! {"
                    import PackageA.PackageA.FunctionA;
                    @EntryPoint()
                    function Main() : Unit {
                       FunctionA();
                    }
                "},
            ),
        ],
        &expect![[r#"
            [
                Error(
                    Resolve(
                        NotFound(
                            "PackageA.PackageA.FunctionA",
                            Span {
                                lo: 7,
                                hi: 34,
                            },
                        ),
                    ),
                ),
                Error(
                    Resolve(
                        NotFound(
                            "FunctionA",
                            Span {
                                lo: 78,
                                hi: 87,
                            },
                        ),
                    ),
                ),
                Error(
                    Type(
                        Error(
                            AmbiguousTy(
                                Span {
                                    lo: 78,
                                    hi: 89,
                                },
                            ),
                        ),
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn reexport() {
    multiple_package_check(vec![
        (
            "PackageA",
            indoc! {"
                    export Std.Core.Length as Foo;
                "},
        ),
        (
            "PackageB",
            indoc! {"

                    import PackageA.PackageA.Foo;
                    @EntryPoint()
                    function Main() : Unit {
                        use qs = Qubit[2];
                        let len = Foo(qs);
                    }
                "},
        ),
    ]);
}

#[test]
fn reexport_export_has_alias() {
    multiple_package_check(vec![
        (
            "PackageA",
            indoc! {"
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo as Bar;
                "},
        ),
        (
            "PackageB",
            indoc! {"
                import PackageA.PackageA.Bar;
                "},
        ),
    ]);
}

#[test]
fn reexport_import_has_alias() {
    multiple_package_check(vec![
        (
            "PackageA",
            "operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            ",
        ),
        (
            "PackageB",
            "
                import PackageA.PackageA.Foo as Bar;

                export Bar;
            ",
        ),
    ]);
}

#[test]
fn reexport_reexport_has_alias() {
    multiple_package_check(vec![
        (
            "PackageA",
            "
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            ",
        ),
        (
            "PackageB",
            "
                import PackageA.PackageA.Foo;
                export Foo as Bar;
            ",
        ),
        (
            "PackageC",
            "
                import PackageB.PackageB.Bar;
                @EntryPoint()
                operation Main() : Unit {
                    Bar(10, true);
                }
            ",
        ),
    ]);
}

#[test]
fn reexport_callable_combined_aliases() {
    multiple_package_check(vec![
        (
            "PackageA",
            "
                operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;
            ",
        ),
        (
            "PackageB",
            "
                import PackageA.PackageA.Foo;
                import PackageA.PackageA.Foo as Foo2;
                export Foo, Foo as Bar, Foo2, Foo2 as Bar2;
            ",
        ),
        (
            "PackageC",
            "
                import PackageB.PackageB.Foo, PackageB.PackageB.Bar, PackageB.PackageB.Foo2, PackageB.PackageB.Bar2;
                @EntryPoint()
                operation Main() : Unit {
                    Foo(10, true);
                    Foo2(10, true);
                    Bar(10, true);
                    Bar2(10, true);
                }
            ",
        ),
    ]);
}

#[test]
fn direct_reexport() {
    multiple_package_check(vec![
        (
            "A",
            "operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo as Bar;",
        ),
        ("B", "export A.A.Bar as Baz;"),
        (
            "C",
            "import B.B.Baz as Quux;
                    @EntryPoint()
                    operation Main() : Unit {
                        Quux(10, true);
                    }",
        ),
    ]);
}

#[test]
fn reexports_still_type_check() {
    multiple_package_check_expect_err(
        vec![
            (
                "A",
                "operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo as Bar;",
            ),
            (
                "B",
                "
                 export A.A.Bar as Baz;",
            ),
            (
                "C",
                "import B.B.Baz as Quux;
                    @EntryPoint()
                    operation Main() : Unit {
                        Quux(10, 10);
                    }",
            ),
        ],
        &expect![[r#"
            [
                Error(
                    Type(
                        Error(
                            TyMismatch(
                                "Bool",
                                "Int",
                                Span {
                                    lo: 128,
                                    hi: 140,
                                },
                            ),
                        ),
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn namespaces_named_lowercase_main_not_treated_as_root() {
    multiple_package_check(vec![
        (
            "main",
            "operation Foo(x: Int, y: Bool) : Int {
                    x
                }
                export Foo;",
        ),
        (
            "C",
            "
            import main.main.Foo;
                    @EntryPoint()
                    operation Main() : Unit {
                        Foo(10, true);
                    }",
        ),
    ]);
}

#[test]
fn aliased_export_via_aliased_import() {
    multiple_package_check(vec![
        (
            "MyGithubLibrary",
            r#"
        namespace TestPackage {

            import Subpackage.Subpackage.Hello as SubHello;

            export HelloFromGithub;
            export SubHello;

            /// This is a Doc String!
            function HelloFromGithub() : Unit {
                SubHello();
            }
        }

        namespace Subpackage.Subpackage {
            function Hello() : Unit {}
            export Hello;
        }

        "#,
        ),
        (
            "UserCode",
            r#"
           import MyGithubLibrary.TestPackage.SubHello;
           import MyGithubLibrary.TestPackage.HelloFromGithub;
           import MyGithubLibrary.Subpackage.Subpackage as P;

            function Main() : Unit {
                HelloFromGithub();
                SubHello();
                P.Hello();
            }
         "#,
        ),
    ]);
}
