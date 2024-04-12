// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]
#![allow(clippy::needless_raw_string_hashes)]

use expect_test::expect;
use indoc::indoc;

use super::test_utils::check;

#[test]
fn body_with_implicit_return() {
    check(
        indoc! {r#"
            namespace A {
                operation B() : Int {
                    let x = 5;
                    x
                }
            }"#},
        None,
        &expect![[r#"
            namespace A {
                operation B() : Int {
                    let x = 5;
                    x
                }
            }"#]],
    );
}

#[test]
fn attributes() {
    check(
        indoc! {r#"
            namespace Sample {
                @EntryPoint()
                @Config(Base)
                operation Entry() : Unit {}
            }"#},
        None,
        &expect![[r#"
            namespace Sample {
                @EntryPoint()
                @Config(Base)
                operation Entry() : Unit {}
            }"#]],
    );
}

#[test]
fn comments_are_omitted() {
    check(
        indoc! {r#"
            // NS comment
            namespace A {
                // op comment here
                operation B() : Unit {
                    // comment here
                    // another comment
                } // trailing comment
            }"#},
        None,
        &expect![[r#"
            namespace A {
                operation B() : Unit {}
            }"#]],
    );
}

#[test]
fn visibility() {
    check(
        indoc! {r#"
            // NS comment
            namespace A {
                // op comment here
                internal operation B() : Unit {
                    // comment here
                    // another comment
                } // trailing comment
            }"#},
        None,
        &expect![[r#"
            namespace A {
                internal operation B() : Unit {}
            }"#]],
    );
}

#[test]
fn callable_specs() {
    check(
        indoc! {r#"
            namespace Sample {
                @EntryPoint()
                operation Entry() : Result {
                    use q = Qubit();
                    // comment here
                    H(q);
                    // implicit return
                    M(q)
                }
                operation Op1(q: Qubit[]) : Unit is Ctl + Adj {
                    body ... {
                        Microsoft.Quantum.Intrinsic.H(q[0]);
                    }
                    adjoint invert;
                    controlled distribute;
                    controlled adjoint auto;
                }
                operation op2(q: Qubit) : Unit is Adj + Ctl {
                    body ... {
                        H(q);
                    }
                    adjoint self;
                    controlled auto;
                    controlled adjoint invert;
                }
                operation op3(q: Qubit) : Unit is Ctl + Adj {
                    body ... {
                        H(q);
                    }
                    adjoint auto;
                    controlled adjoint self;
                }
                operation op4() : Unit {
                    body intrinsic;
                }
                operation op5(q: Qubit) : Unit is Ctl {
                    body ... {
                        H(q);
                    }
                    controlled auto;
                }
                operation op6(q: Qubit) : Unit is Adj {
                    body ... {
                        H(q);
                    }
                    adjoint auto;
                }
                operation op7() : Unit is Adj * Adj {
                    body ... {}
                }
            }"#},
        None,
        &expect![[r#"
            namespace Sample {
                @EntryPoint()
                operation Entry() : Result {
                    use q = Qubit();
                    H(q);
                    M(q)
                }
                operation Op1(q : Qubit[]) : Unit is Ctl + Adj {
                    body ... {
                        Microsoft.Quantum.Intrinsic.H(q[0]);
                    }
                    adjoint invert;
                    controlled distribute;
                    controlled adjoint auto;
                }
                operation op2(q : Qubit) : Unit is Adj + Ctl {
                    body ... {
                        H(q);
                    }
                    adjoint self;
                    controlled auto;
                    controlled adjoint invert;
                }
                operation op3(q : Qubit) : Unit is Ctl + Adj {
                    body ... {
                        H(q);
                    }
                    adjoint auto;
                    controlled adjoint self;
                }
                operation op4() : Unit {
                    body intrinsic;
                }
                operation op5(q : Qubit) : Unit is Ctl {
                    body ... {
                        H(q);
                    }
                    controlled auto;
                }
                operation op6(q : Qubit) : Unit is Adj {
                    body ... {
                        H(q);
                    }
                    adjoint auto;
                }
                operation op7() : Unit is Adj * Adj {
                    body ... {}
                }
            }"#]],
    );
}

#[test]
fn callable_core_types() {
    check(
        indoc! {r#"
            namespace A {
                operation B() : Int {
                    let x = 5;
                    x
                }
                function C() : Int {
                    let x = 42;
                    x
                }
            }"#},
        None,
        &expect![[r#"
            namespace A {
                operation B() : Int {
                    let x = 5;
                    x
                }
                function C() : Int {
                    let x = 42;
                    x
                }
            }"#]],
    );
}
