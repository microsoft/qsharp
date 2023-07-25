// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_hir::{
    assigner::Assigner,
    hir::{Expr, Package},
};

/// Extracts a single entry point callable declaration, if found.
/// # Errors
/// Returns an error if a single entry point with no parameters cannot be found.
fn extract_entry(assigner: &mut Assigner, package: &Package) -> Result<Expr, Vec<crate::Error>> {
    let callables = super::get_callables(package);
    super::create_entry_from_callables(assigner, callables)
}

fn check(file: &str, expr: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), file.into())], Some(expr.into()));
    let mut unit = compile(&PackageStore::new(compile::core()), &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    match extract_entry(&mut unit.assigner, &unit.package) {
        Ok(entry) => expect.assert_eq(&entry.to_string()),
        Err(errors) => expect.assert_debug_eq(&errors),
    }
}

#[test]
fn test_entry_point_attr_to_expr() {
    check(
        indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Int { 41 + 1 }
            }"},
        "",
        &expect![[r#"
            Expr 12 [39-72] [Type Int]: Call:
                Expr 11 [39-72] [Type Int]: Var: Item 1
                Expr 10 [39-72] [Type Unit]: Unit"#]],
    );
}

#[test]
fn test_entry_point_attr_missing() {
    check(
        indoc! {"
            namespace Test {
                operation Main() : Int { 41 + 1 }
            }"},
        "",
        &expect![[r#"
            [
                EntryPoint(
                    NotFound,
                ),
            ]
        "#]],
    );
}

#[test]
fn test_entry_point_attr_multiple() {
    check(
        indoc! {"
            namespace Test {
                @EntryPoint()
                operation Main() : Int { 41 + 1 }

                @EntryPoint()
                operation Main2() : Int { 40 + 1 }
            }"},
        "",
        &expect![[r#"
            [
                EntryPoint(
                    Duplicate(
                        "Main",
                        Span {
                            lo: 49,
                            hi: 53,
                        },
                    ),
                ),
                EntryPoint(
                    Duplicate(
                        "Main2",
                        Span {
                            lo: 106,
                            hi: 111,
                        },
                    ),
                ),
            ]
        "#]],
    );
}
