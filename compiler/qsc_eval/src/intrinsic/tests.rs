// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore};

use crate::Evaluator;

fn check_intrinsic(file: &str, expr: &str, expect: &Expect) {
    let mut store = PackageStore::new();
    let stdlib = store.insert(compile::std());
    let unit = compile(&store, [stdlib], [file], expr);
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
    let id = store.insert(unit);
    match Evaluator::new(&store, id).run() {
        Ok(result) => expect.assert_eq(&result.to_string()),
        Err(e) => expect.assert_debug_eq(&e),
    }
}

#[test]
fn length() {
    check_intrinsic("", "Length([1, 2, 3])", &expect!["3"]);
}

#[test]
fn length_type_err() {
    check_intrinsic(
        "",
        "Length((1, 2, 3))",
        &expect![[r#"
        Error {
            span: Span {
                lo: 6,
                hi: 17,
            },
            kind: Type(
                "Array",
                "Tuple",
            ),
        }
    "#]],
    );
}

#[test]
fn int_as_double() {
    check_intrinsic(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(2)",
        &expect!["2.0"],
    );
}

#[test]
fn int_as_double_type_error() {
    check_intrinsic(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(false)",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 37,
                    hi: 44,
                },
                kind: Type(
                    "Int",
                    "Bool",
                ),
            }
        "#]],
    );
}

#[test]
fn int_as_double_precision_loss() {
    check_intrinsic(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(9_223_372_036_854_775_807)",
        &expect!["9223372036854775808.0"],
    );
}

#[test]
fn unknown_intrinsic() {
    check_intrinsic(
        indoc! {"
            namespace Test {
                function Foo() : Int {
                    body intrinsic;
                }
            }
        "},
        "Test.Foo()",
        &expect![[r#"
            Error {
                span: Span {
                    lo: 30,
                    hi: 33,
                },
                kind: UnknownIntrinsic,
            }
        "#]],
    );
}
