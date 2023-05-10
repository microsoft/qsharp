// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

use crate::semantics::validate_semantics;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let unit = compile(&store, &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let errors = validate_semantics(&unit);
    expect.assert_debug_eq(&errors);
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
