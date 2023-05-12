// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, PackageStore, SourceMap};
use expect_test::{expect, Expect};
use indoc::indoc;

fn check(file: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let unit = compile(&PackageStore::new(compile::core()), &[], sources);

    let semantic_errors: Vec<_> = unit
        .errors
        .into_iter()
        .filter_map(try_into_funop_error)
        .collect();

    expect.assert_debug_eq(&semantic_errors);
}

fn try_into_funop_error(error: compile::Error) -> Option<super::Error> {
    if let compile::ErrorKind::FunOp(error) = error.0 {
        Some(error)
    } else {
        None
    }
}

#[test]
fn funcs_cannot_use_conj() {
    check(
        indoc! {"
            namespace Test {
                function A() : Unit {
                    within {} apply {}
                }
            }
        "},
        &expect![[r#"
            [
                ConjInFunc(
                    Span {
                        lo: 51,
                        hi: 69,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn funcs_cannot_have_functors() {
    check(
        indoc! {"
            namespace Test {
                function A() : Unit is Adj {}
            }
        "},
        &expect![[r#"
            [
                FunctorInFunc(
                    Span {
                        lo: 44,
                        hi: 47,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn funcs_cannot_call_ops() {
    check(
        indoc! {"
            namespace Test {
                operation A() : Unit {}
                function B() : Unit {
                    A();
                }
            }
        "},
        &expect![[r#"
            [
                OpCallInFunc(
                    Span {
                        lo: 79,
                        hi: 82,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn funcs_cannot_allocate_qubits() {
    check(
        indoc! {"
            namespace Test {
                function A() : Unit {
                    use q = Qubit();
                }
            }
        "},
        &expect![[r#"
            [
                QubitAllocInFunc(
                    Span {
                        lo: 51,
                        hi: 67,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn funcs_cannot_use_repeat() {
    check(
        indoc! {"
            namespace Test {
                function A() : Unit {
                    repeat {} until true;
                }
            }
        "},
        &expect![[r#"
            [
                RepeatInFunc(
                    Span {
                        lo: 51,
                        hi: 71,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn funcs_cannot_have_specs() {
    check(
        indoc! {"
            namespace Test {
                function A() : Unit {
                    body ... {}
                    adjoint self;
                }
            }
        "},
        &expect![[r#"
            [
                SpecInFunc(
                    Span {
                        lo: 21,
                        hi: 90,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn ops_cannot_use_while() {
    check(
        indoc! {"
            namespace Test {
                operation B() : Unit {
                    while true {}
                }
            }
        "},
        &expect![[r#"
            [
                WhileInOp(
                    Span {
                        lo: 52,
                        hi: 65,
                    },
                ),
            ]
        "#]],
    );
}
