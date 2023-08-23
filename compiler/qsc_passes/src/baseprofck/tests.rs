// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap, TargetProfile};

use crate::baseprofck::check_base_profile_compliance;

fn check(expr: &str, expect: &Expect) {
    let mut store = PackageStore::new(compile::core());
    let std = store.insert(compile::std(&store, TargetProfile::Full));
    let sources = SourceMap::new([("test".into(), "".into())], Some(expr.into()));
    let unit = compile(&store, &[std], sources, TargetProfile::Full);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let errors = check_base_profile_compliance(&unit.package);
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
fn intrinsic_calls_with_supported_returns_are_valid() {
    check(
        indoc! {"{
            operation Foo() : Unit {
                body intrinsic;
            }
            operation Bar() : Result {
                body intrinsic;
            }
            Foo();
            Bar()
        }"},
        &expect![[r#"
            []
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
                        lo: 42,
                        hi: 54,
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
                        lo: 2,
                        hi: 5,
                    },
                ),
                ResultLiteral(
                    Span {
                        lo: 7,
                        hi: 11,
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
                        lo: 1,
                        hi: 55,
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
                        lo: 17,
                        hi: 21,
                    },
                ),
            ]
        "#]],
    );
}
