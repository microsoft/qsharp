// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

use crate::entry_point::extract_entry;

fn check(file: &str, expr: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), file.into())], Some(expr.into()));
    let unit = compile(&PackageStore::new(compile::core()), &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    match extract_entry(&unit.package) {
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
            Expr _id_ [0-0] [Type Int]: Expr Block: Block 4 [62-72] [Type Int]:
                Stmt 5 [64-70]: Expr: Expr 6 [64-70] [Type Int]: BinOp (Add):
                    Expr 7 [64-66] [Type Int]: Lit: Int(41)
                    Expr 8 [69-70] [Type Int]: Lit: Int(1)"#]],
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
