// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::entry_point::generate_entry_expr;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap, TargetProfile};

fn check(file: &str, expr: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), file.into())], Some(expr.into()));
    let mut unit = compile(
        &PackageStore::new(compile::core()),
        &[],
        sources,
        TargetProfile::Full,
    );
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let errors = generate_entry_expr(&mut unit.package, &mut unit.assigner);
    if errors.is_empty() {
        expect.assert_eq(
            &unit
                .package
                .entry
                .expect("entry should be present in success case")
                .to_string(),
        );
    } else {
        expect.assert_debug_eq(&errors);
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
            Expr 12 [40-73] [Type Int]: Call:
                Expr 11 [40-73] [Type Int]: Var: Item 1
                Expr 10 [40-73] [Type Unit]: Unit"#]],
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
                            lo: 50,
                            hi: 54,
                        },
                    ),
                ),
                EntryPoint(
                    Duplicate(
                        "Main2",
                        Span {
                            lo: 107,
                            hi: 112,
                        },
                    ),
                ),
            ]
        "#]],
    );
}
