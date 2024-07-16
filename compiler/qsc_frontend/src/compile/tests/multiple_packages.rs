use std::sync::Arc;

use super::{compile, PackageStore, SourceMap};
use crate::compile::TargetCapabilityFlags;

use crate::compile::core;
use expect_test::expect;
use expect_test::Expect;
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
