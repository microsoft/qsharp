// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore};
use qsc_passes::globals::extract_callables;

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
    let unit = store
        .get(id)
        .expect("Compile unit should be in package store");
    let globals = extract_callables(&store);
    match Evaluator::eval(
        unit.package
            .entry
            .as_ref()
            .expect("Entry statement should be provided."),
        &store,
        &globals,
        unit.context.resolutions(),
        id,
        Evaluator::empty_scope(),
    ) {
        Ok((result, _)) => expect.assert_eq(&result.to_string()),
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
            Type(
                "Array",
                "Tuple",
                Span {
                    lo: 6,
                    hi: 17,
                },
            )
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
            Type(
                "Int",
                "Bool",
                Span {
                    lo: 37,
                    hi: 44,
                },
            )
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
            UnknownIntrinsic(
                Span {
                    lo: 76,
                    hi: 84,
                },
            )
        "#]],
    );
}
