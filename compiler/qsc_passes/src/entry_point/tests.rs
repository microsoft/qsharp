// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore};

use crate::entry_point::extract_entry;

fn check(file: &str, expr: &str, expect: &Expect) {
    let unit = compile(&PackageStore::new(), [], [file], expr);
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
    let res = extract_entry(&unit.package);
    match res {
        Ok(result) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
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
            Expr _id_ [0-0]: Expr Block: Block 7 [62-72]:
                Stmt 8 [64-70]: Expr: Expr 9 [64-70]: BinOp (Add):
                    Expr 10 [64-66]: Lit: Int(41)
                    Expr 11 [69-70]: Lit: Int(1)"#]],
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
                    EntryPointMissing,
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
                    DuplicateEntryPoint(
                        "Main",
                        Span {
                            lo: 49,
                            hi: 53,
                        },
                    ),
                ),
                EntryPoint(
                    DuplicateEntryPoint(
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
