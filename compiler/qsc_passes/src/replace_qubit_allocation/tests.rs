// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::replace_qubit_allocation::ReplaceQubitAllocation;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::mut_visit::MutVisitor;
use qsc_frontend::compile::{compile, PackageStore};

fn check(input: &str, expected: &Expect) {
    let store = PackageStore::new();
    let mut unit = compile(&store, [], [input], "");

    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );

    let mut transforamtion = ReplaceQubitAllocation::new();
    transforamtion.visit_package(&mut unit.package);

    let ns = unit
        .package
        .namespaces
        .first()
        .expect("Didn't find a namespace");

    expected.assert_eq(&ns.to_string());
}

#[test]
fn test_single_qubit() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use q = Qubit();
                let x = 3;
            }
        }" },
        &expect![[r#"
            Namespace 1 [0-98] (Ident 2 [10-15] "input"):
                Item 3 [22-96]:
                    Callable 4 [22-96] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-96]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60]: Bind:
                                    Ident 11 [59-60] "q"
                                Expr _id_ [59-60]: Call:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [59-60]: Unit
                            Stmt 13 [80-90]: Local (Immutable):
                                Pat 14 [84-85]: Bind:
                                    Ident 15 [84-85] "x"
                                Expr 16 [88-89]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release")
                                Expr _id_ [59-60]: Tuple:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "q")"#]],
    );
}

#[test]
fn test_qubit_array() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use q = Qubit[3];
                let x = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_qubit_tuple() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use q = (Qubit(), Qubit());
                let x = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_multiple_qubits_tuple() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use (a, b) = (Qubit(), Qubit[3]);
                let x = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_multiple_callables() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use (a, b) = (Qubit(), Qubit());
                let x = 3;
            }
            
            operation Bar() : Unit {
                use (c, d) = (Qubit(), Qubit());
                let x = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_qubit_block() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use (a, b) = (Qubit(), Qubit()) {
                    let x = 3;
                    use c = Qubit();
                    let y = 3
                }
                let z = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_qubit_nested_block() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use a = Qubit();
                use b = Qubit() {
                    let x = 3;
                }
                let y = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_qubit_multiple_nested_blocks() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                let x1 = 3;
                use a = Qubit();
                let x2 = 3;
                {
                    let y1 = 3;
                    use b = Qubit();
                    let y2 = 3;
                }
                let x3 = 3;
                {
                    let z1 = 3;
                    use c = Qubit();
                    let z2 = 3;
                }
                let x4 = 3;
            }
        }" },
        &expect![[r#"
            Namespace 1 [0-353] (Ident 2 [10-15] "input"):
                Item 3 [22-351]:
                    Callable 4 [22-351] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-351]:
                            Stmt 9 [55-66]: Local (Immutable):
                                Pat 10 [59-61]: Bind:
                                    Ident 11 [59-61] "x1"
                                Expr 12 [64-65]: Lit: Int(3)
                            Stmt _id_ [83-90]: Local (Immutable):
                                Pat _id_ [83-90]: Bind:
                                    Ident _id_ [83-90] "__generated_ident_0__"
                                Expr _id_ [83-90]: Call:
                                    Expr _id_ [83-90]: Path: Path _id_ [83-90] (Ident _id_ [83-90] "QIR.Runtime") (Ident _id_ [83-90] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [83-90]: Unit
                            Stmt _id_ [75-91]: Local (Immutable):
                                Pat 14 [79-80]: Bind:
                                    Ident 15 [79-80] "a"
                                Expr _id_ [83-90]: Path: Path _id_ [83-90] (Ident _id_ [83-90] "__generated_ident_0__")
                            Stmt 17 [100-111]: Local (Immutable):
                                Pat 18 [104-106]: Bind:
                                    Ident 19 [104-106] "x2"
                                Expr 20 [109-110]: Lit: Int(3)
                            Stmt 21 [120-208]: Expr: Expr 22 [120-208]: Expr Block: Block 23 [120-208]:
                                Stmt 24 [134-145]: Local (Immutable):
                                    Pat 25 [138-140]: Bind:
                                        Ident 26 [138-140] "y1"
                                    Expr 27 [143-144]: Lit: Int(3)
                                Stmt _id_ [166-173]: Local (Immutable):
                                    Pat _id_ [166-173]: Bind:
                                        Ident _id_ [166-173] "__generated_ident_1__"
                                    Expr _id_ [166-173]: Call:
                                        Expr _id_ [166-173]: Path: Path _id_ [166-173] (Ident _id_ [166-173] "QIR.Runtime") (Ident _id_ [166-173] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [166-173]: Unit
                                Stmt _id_ [158-174]: Local (Immutable):
                                    Pat 29 [162-163]: Bind:
                                        Ident 30 [162-163] "b"
                                    Expr _id_ [166-173]: Path: Path _id_ [166-173] (Ident _id_ [166-173] "__generated_ident_1__")
                                Stmt 32 [187-198]: Local (Immutable):
                                    Pat 33 [191-193]: Bind:
                                        Ident 34 [191-193] "y2"
                                    Expr 35 [196-197]: Lit: Int(3)
                                Stmt _id_ [166-173]: Semi: Expr _id_ [166-173]: Call:
                                    Expr _id_ [166-173]: Path: Path _id_ [166-173] (Ident _id_ [166-173] "QIR.Runtime") (Ident _id_ [166-173] "__quantum__rt__qubit_release")
                                    Expr _id_ [166-173]: Tuple:
                                        Expr _id_ [166-173]: Path: Path _id_ [166-173] (Ident _id_ [166-173] "__generated_ident_1__")
                            Stmt 36 [217-228]: Local (Immutable):
                                Pat 37 [221-223]: Bind:
                                    Ident 38 [221-223] "x3"
                                Expr 39 [226-227]: Lit: Int(3)
                            Stmt 40 [237-325]: Expr: Expr 41 [237-325]: Expr Block: Block 42 [237-325]:
                                Stmt 43 [251-262]: Local (Immutable):
                                    Pat 44 [255-257]: Bind:
                                        Ident 45 [255-257] "z1"
                                    Expr 46 [260-261]: Lit: Int(3)
                                Stmt _id_ [283-290]: Local (Immutable):
                                    Pat _id_ [283-290]: Bind:
                                        Ident _id_ [283-290] "__generated_ident_2__"
                                    Expr _id_ [283-290]: Call:
                                        Expr _id_ [283-290]: Path: Path _id_ [283-290] (Ident _id_ [283-290] "QIR.Runtime") (Ident _id_ [283-290] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [283-290]: Unit
                                Stmt _id_ [275-291]: Local (Immutable):
                                    Pat 48 [279-280]: Bind:
                                        Ident 49 [279-280] "c"
                                    Expr _id_ [283-290]: Path: Path _id_ [283-290] (Ident _id_ [283-290] "__generated_ident_2__")
                                Stmt 51 [304-315]: Local (Immutable):
                                    Pat 52 [308-310]: Bind:
                                        Ident 53 [308-310] "z2"
                                    Expr 54 [313-314]: Lit: Int(3)
                                Stmt _id_ [283-290]: Semi: Expr _id_ [283-290]: Call:
                                    Expr _id_ [283-290]: Path: Path _id_ [283-290] (Ident _id_ [283-290] "QIR.Runtime") (Ident _id_ [283-290] "__quantum__rt__qubit_release")
                                    Expr _id_ [283-290]: Tuple:
                                        Expr _id_ [283-290]: Path: Path _id_ [283-290] (Ident _id_ [283-290] "__generated_ident_2__")
                            Stmt 55 [334-345]: Local (Immutable):
                                Pat 56 [338-340]: Bind:
                                    Ident 57 [338-340] "x4"
                                Expr 58 [343-344]: Lit: Int(3)
                            Stmt _id_ [83-90]: Semi: Expr _id_ [83-90]: Call:
                                Expr _id_ [83-90]: Path: Path _id_ [83-90] (Ident _id_ [83-90] "QIR.Runtime") (Ident _id_ [83-90] "__quantum__rt__qubit_release")
                                Expr _id_ [83-90]: Tuple:
                                    Expr _id_ [83-90]: Path: Path _id_ [83-90] (Ident _id_ [83-90] "__generated_ident_0__")"#]],
    );
}

#[test]
fn test_early_returns() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use a = Qubit();
                if true {
                    use b = Qubit();
                    return ();
                }

                if false {
                    use c = Qubit();
                    return ();
                }
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_end_exprs() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use a = Qubit();
                let x = {3};
                let y = {
                    use b = Qubit();
                    3
                }
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_array_expr() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use a = Qubit[{
                    use b = Qubit();
                    3
                }];
                let x = 3;
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_rtrn_expr() {
    check(
        indoc! { "namespace input {
            operation Foo() : Int {
                use a = Qubit();
                return {
                    use b = Qubit();
                    3
                };
            }
        }" },
        &expect![[""]],
    );
}

#[test]
fn test_lambdas() {
    check(
        indoc! { "namespace input {
            operation Foo() : Unit {
                use a = Qubit();
                let lambda = x => {
                    use b = Qubit();
                    return x;
                };
            }
        }" },
        &expect![[""]],
    );
}
