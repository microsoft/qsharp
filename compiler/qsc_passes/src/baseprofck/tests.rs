// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

use crate::baseprofck::check_base_profile_compliance;

fn check(expr: &str, expect: &Expect) {
    let mut store = PackageStore::new(compile::core());
    let std = store.insert(compile::std(&store));
    let lib_src = SourceMap::new(
        [(
            "lib".into(),
            indoc! {"
        namespace Lib {
            operation Foo() : Unit {
                body intrinsic;
            }
            operation Bar() : Result {
                body intrinsic;
            }
            operation Baz() : Int {
                body intrinsic;
            }
            operation MeasAreEq(q1 : Qubit, q2 : Qubit) : Bool {
                M(q1) == M(q2)
            }
        }
    "}
            .into(),
        )],
        None,
    );
    let lib_unit = compile(&store, &[std], lib_src);
    assert!(lib_unit.errors.is_empty(), "{:?}", lib_unit.errors);
    let lib = store.insert(lib_unit);
    let sources = SourceMap::new([("test".into(), "".into())], Some(expr.into()));
    let unit = compile(&store, &[std, lib], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let errors = check_base_profile_compliance(&store, &unit.package);
    expect.assert_debug_eq(&errors);
}

#[test]
fn simple_program_is_valid() {
    check(
        indoc! {"{
            use q = Qubit();
            H(q);
            M(q)
        }"},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn intrinsic_lib_calls_with_supported_returns_are_valid() {
    check(
        indoc! {"{
            Lib.Foo();
            Lib.Bar()
        }"},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn intrinsic_lib_calls_with_unsupported_returns_are_invalid() {
    check(
        indoc! {"{
            Lib.Baz()
        }"},
        &expect![[r#"
            [
                ReturnNonResult(
                    Span {
                        lo: 0,
                        hi: 17,
                    },
                ),
                UnsupportedIntrinsic(
                    Span {
                        lo: 150,
                        hi: 153,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn result_comparison_error() {
    check(
        indoc! {"{
            use q = Qubit();
            H(q);
            if (M(q) == M(q)) {
                X(q);
            }
        }"},
        &expect![[r#"
            [
                ResultComparison(
                    Span {
                        lo: 41,
                        hi: 53,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn result_comparison_in_lib_error() {
    check(
        indoc! {"{
            use q = Qubit();
            Lib.MeasAreEq(q, q);
        }"},
        &expect![[r#"
            [
                ResultComparison(
                    Span {
                        lo: 259,
                        hi: 273,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn result_literal_error() {
    check(
        indoc! {"(One, Zero)"},
        &expect![[r#"
            [
                ResultLiteral(
                    Span {
                        lo: 1,
                        hi: 4,
                    },
                ),
                ResultLiteral(
                    Span {
                        lo: 6,
                        hi: 10,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn non_result_return_error() {
    check(
        indoc! {"{
            use q = Qubit();
            H(q);
            M(q);
            3 + 1
        }"},
        &expect![[r#"
            [
                ReturnNonResult(
                    Span {
                        lo: 0,
                        hi: 54,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn unsupported_intrsinsic_error() {
    check(
        indoc! {"{
            operation Rand() : Int {
                body intrinsic;
            }
        }"},
        &expect![[r#"
            [
                UnsupportedIntrinsic(
                    Span {
                        lo: 16,
                        hi: 20,
                    },
                ),
            ]
        "#]],
    );
}
