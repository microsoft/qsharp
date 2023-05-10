// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::replace_qubit_allocation::replace_qubit_allocation;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    let errors = replace_qubit_allocation(&mut unit, store.core());
    if errors.is_empty() {
        expect.assert_eq(&unit.package.to_string());
    } else {
        expect.assert_debug_eq(&errors);
    }
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
        &expect![[r#"
            Namespace 1 [0-99] (Ident 2 [10-15] "input"):
                Item 3 [22-97]:
                    Callable 4 [22-97] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-97]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60]: Bind:
                                    Ident 11 [59-60] "q"
                                Expr _id_ [59-60]: Call:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_allocate_array")
                                    Expr _id_ [59-60]: Tuple:
                                        Expr 13 [69-70]: Lit: Int(3)
                            Stmt 14 [81-91]: Local (Immutable):
                                Pat 15 [85-86]: Bind:
                                    Ident 16 [85-86] "x"
                                Expr 17 [89-90]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release_array")
                                Expr _id_ [59-60]: Tuple:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "q")"#]],
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
        &expect![[r#"
            Namespace 1 [0-109] (Ident 2 [10-15] "input"):
                Item 3 [22-107]:
                    Callable 4 [22-107] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-107]:
                            Stmt _id_ [64-71]: Local (Immutable):
                                Pat _id_ [64-71]: Bind:
                                    Ident _id_ [64-71] "__generated_ident_0__"
                                Expr _id_ [64-71]: Call:
                                    Expr _id_ [64-71]: Path: Path _id_ [64-71] (Ident _id_ [64-71] "QIR.Runtime") (Ident _id_ [64-71] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [64-71]: Unit
                            Stmt _id_ [73-80]: Local (Immutable):
                                Pat _id_ [73-80]: Bind:
                                    Ident _id_ [73-80] "__generated_ident_1__"
                                Expr _id_ [73-80]: Call:
                                    Expr _id_ [73-80]: Path: Path _id_ [73-80] (Ident _id_ [73-80] "QIR.Runtime") (Ident _id_ [73-80] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [73-80]: Unit
                            Stmt _id_ [55-82]: Local (Immutable):
                                Pat 10 [59-60]: Bind:
                                    Ident 11 [59-60] "q"
                                Expr _id_ [63-81]: Tuple:
                                    Expr _id_ [64-71]: Path: Path _id_ [64-71] (Ident _id_ [64-71] "__generated_ident_0__")
                                    Expr _id_ [73-80]: Path: Path _id_ [73-80] (Ident _id_ [73-80] "__generated_ident_1__")
                            Stmt 15 [91-101]: Local (Immutable):
                                Pat 16 [95-96]: Bind:
                                    Ident 17 [95-96] "x"
                                Expr 18 [99-100]: Lit: Int(3)
                            Stmt _id_ [73-80]: Semi: Expr _id_ [73-80]: Call:
                                Expr _id_ [73-80]: Path: Path _id_ [73-80] (Ident _id_ [73-80] "QIR.Runtime") (Ident _id_ [73-80] "__quantum__rt__qubit_release")
                                Expr _id_ [73-80]: Tuple:
                                    Expr _id_ [73-80]: Path: Path _id_ [73-80] (Ident _id_ [73-80] "__generated_ident_1__")
                            Stmt _id_ [64-71]: Semi: Expr _id_ [64-71]: Call:
                                Expr _id_ [64-71]: Path: Path _id_ [64-71] (Ident _id_ [64-71] "QIR.Runtime") (Ident _id_ [64-71] "__quantum__rt__qubit_release")
                                Expr _id_ [64-71]: Tuple:
                                    Expr _id_ [64-71]: Path: Path _id_ [64-71] (Ident _id_ [64-71] "__generated_ident_0__")"#]],
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
        &expect![[r#"
            Namespace 1 [0-115] (Ident 2 [10-15] "input"):
                Item 3 [22-113]:
                    Callable 4 [22-113] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-113]:
                            Stmt _id_ [69-76]: Local (Immutable):
                                Pat _id_ [69-76]: Bind:
                                    Ident _id_ [69-76] "__generated_ident_0__"
                                Expr _id_ [69-76]: Call:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "QIR.Runtime") (Ident _id_ [69-76] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [69-76]: Unit
                            Stmt _id_ [78-86]: Local (Immutable):
                                Pat _id_ [78-86]: Bind:
                                    Ident _id_ [78-86] "__generated_ident_1__"
                                Expr _id_ [78-86]: Call:
                                    Expr _id_ [78-86]: Path: Path _id_ [78-86] (Ident _id_ [78-86] "QIR.Runtime") (Ident _id_ [78-86] "__quantum__rt__qubit_allocate_array")
                                    Expr _id_ [78-86]: Tuple:
                                        Expr 18 [84-85]: Lit: Int(3)
                            Stmt _id_ [55-88]: Local (Immutable):
                                Pat 10 [59-65]: Tuple:
                                    Pat 11 [60-61]: Bind:
                                        Ident 12 [60-61] "a"
                                    Pat 13 [63-64]: Bind:
                                        Ident 14 [63-64] "b"
                                Expr _id_ [68-87]: Tuple:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "__generated_ident_0__")
                                    Expr _id_ [78-86]: Path: Path _id_ [78-86] (Ident _id_ [78-86] "__generated_ident_1__")
                            Stmt 19 [97-107]: Local (Immutable):
                                Pat 20 [101-102]: Bind:
                                    Ident 21 [101-102] "x"
                                Expr 22 [105-106]: Lit: Int(3)
                            Stmt _id_ [78-86]: Semi: Expr _id_ [78-86]: Call:
                                Expr _id_ [78-86]: Path: Path _id_ [78-86] (Ident _id_ [78-86] "QIR.Runtime") (Ident _id_ [78-86] "__quantum__rt__qubit_release_array")
                                Expr _id_ [78-86]: Tuple:
                                    Expr _id_ [78-86]: Path: Path _id_ [78-86] (Ident _id_ [78-86] "__generated_ident_1__")
                            Stmt _id_ [69-76]: Semi: Expr _id_ [69-76]: Call:
                                Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "QIR.Runtime") (Ident _id_ [69-76] "__quantum__rt__qubit_release")
                                Expr _id_ [69-76]: Tuple:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "__generated_ident_0__")"#]],
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
        &expect![[r#"
            Namespace 1 [0-214] (Ident 2 [10-15] "input"):
                Item 3 [22-112]:
                    Callable 4 [22-112] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-112]:
                            Stmt _id_ [69-76]: Local (Immutable):
                                Pat _id_ [69-76]: Bind:
                                    Ident _id_ [69-76] "__generated_ident_0__"
                                Expr _id_ [69-76]: Call:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "QIR.Runtime") (Ident _id_ [69-76] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [69-76]: Unit
                            Stmt _id_ [78-85]: Local (Immutable):
                                Pat _id_ [78-85]: Bind:
                                    Ident _id_ [78-85] "__generated_ident_1__"
                                Expr _id_ [78-85]: Call:
                                    Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "QIR.Runtime") (Ident _id_ [78-85] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [78-85]: Unit
                            Stmt _id_ [55-87]: Local (Immutable):
                                Pat 10 [59-65]: Tuple:
                                    Pat 11 [60-61]: Bind:
                                        Ident 12 [60-61] "a"
                                    Pat 13 [63-64]: Bind:
                                        Ident 14 [63-64] "b"
                                Expr _id_ [68-86]: Tuple:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "__generated_ident_0__")
                                    Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "__generated_ident_1__")
                            Stmt 18 [96-106]: Local (Immutable):
                                Pat 19 [100-101]: Bind:
                                    Ident 20 [100-101] "x"
                                Expr 21 [104-105]: Lit: Int(3)
                            Stmt _id_ [78-85]: Semi: Expr _id_ [78-85]: Call:
                                Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "QIR.Runtime") (Ident _id_ [78-85] "__quantum__rt__qubit_release")
                                Expr _id_ [78-85]: Tuple:
                                    Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "__generated_ident_1__")
                            Stmt _id_ [69-76]: Semi: Expr _id_ [69-76]: Call:
                                Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "QIR.Runtime") (Ident _id_ [69-76] "__quantum__rt__qubit_release")
                                Expr _id_ [69-76]: Tuple:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "__generated_ident_0__")
                Item 22 [122-212]:
                    Callable 23 [122-212] (Operation):
                        name: Ident 24 [132-135] "Bar"
                        input: Pat 25 [135-137]: Unit
                        output: Type 26 [140-144]: Unit
                        body: Block: Block 27 [145-212]:
                            Stmt _id_ [169-176]: Local (Immutable):
                                Pat _id_ [169-176]: Bind:
                                    Ident _id_ [169-176] "__generated_ident_2__"
                                Expr _id_ [169-176]: Call:
                                    Expr _id_ [169-176]: Path: Path _id_ [169-176] (Ident _id_ [169-176] "QIR.Runtime") (Ident _id_ [169-176] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [169-176]: Unit
                            Stmt _id_ [178-185]: Local (Immutable):
                                Pat _id_ [178-185]: Bind:
                                    Ident _id_ [178-185] "__generated_ident_3__"
                                Expr _id_ [178-185]: Call:
                                    Expr _id_ [178-185]: Path: Path _id_ [178-185] (Ident _id_ [178-185] "QIR.Runtime") (Ident _id_ [178-185] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [178-185]: Unit
                            Stmt _id_ [155-187]: Local (Immutable):
                                Pat 29 [159-165]: Tuple:
                                    Pat 30 [160-161]: Bind:
                                        Ident 31 [160-161] "c"
                                    Pat 32 [163-164]: Bind:
                                        Ident 33 [163-164] "d"
                                Expr _id_ [168-186]: Tuple:
                                    Expr _id_ [169-176]: Path: Path _id_ [169-176] (Ident _id_ [169-176] "__generated_ident_2__")
                                    Expr _id_ [178-185]: Path: Path _id_ [178-185] (Ident _id_ [178-185] "__generated_ident_3__")
                            Stmt 37 [196-206]: Local (Immutable):
                                Pat 38 [200-201]: Bind:
                                    Ident 39 [200-201] "x"
                                Expr 40 [204-205]: Lit: Int(3)
                            Stmt _id_ [178-185]: Semi: Expr _id_ [178-185]: Call:
                                Expr _id_ [178-185]: Path: Path _id_ [178-185] (Ident _id_ [178-185] "QIR.Runtime") (Ident _id_ [178-185] "__quantum__rt__qubit_release")
                                Expr _id_ [178-185]: Tuple:
                                    Expr _id_ [178-185]: Path: Path _id_ [178-185] (Ident _id_ [178-185] "__generated_ident_3__")
                            Stmt _id_ [169-176]: Semi: Expr _id_ [169-176]: Call:
                                Expr _id_ [169-176]: Path: Path _id_ [169-176] (Ident _id_ [169-176] "QIR.Runtime") (Ident _id_ [169-176] "__quantum__rt__qubit_release")
                                Expr _id_ [169-176]: Tuple:
                                    Expr _id_ [169-176]: Path: Path _id_ [169-176] (Ident _id_ [169-176] "__generated_ident_2__")"#]],
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
                    let y = 3;
                }
                let z = 3;
            }
        }" },
        &expect![[r#"
            Namespace 1 [0-200] (Ident 2 [10-15] "input"):
                Item 3 [22-198]:
                    Callable 4 [22-198] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-198]:
                            Stmt _id_ [55-173]: Expr: Expr _id_ [55-173]: Expr Block: Block 18 [87-173]:
                                Stmt _id_ [69-76]: Local (Immutable):
                                    Pat _id_ [69-76]: Bind:
                                        Ident _id_ [69-76] "__generated_ident_0__"
                                    Expr _id_ [69-76]: Call:
                                        Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "QIR.Runtime") (Ident _id_ [69-76] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [69-76]: Unit
                                Stmt _id_ [78-85]: Local (Immutable):
                                    Pat _id_ [78-85]: Bind:
                                        Ident _id_ [78-85] "__generated_ident_1__"
                                    Expr _id_ [78-85]: Call:
                                        Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "QIR.Runtime") (Ident _id_ [78-85] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [78-85]: Unit
                                Stmt _id_ [55-173]: Local (Immutable):
                                    Pat 10 [59-65]: Tuple:
                                        Pat 11 [60-61]: Bind:
                                            Ident 12 [60-61] "a"
                                        Pat 13 [63-64]: Bind:
                                            Ident 14 [63-64] "b"
                                    Expr _id_ [68-86]: Tuple:
                                        Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "__generated_ident_0__")
                                        Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "__generated_ident_1__")
                                Stmt 19 [101-111]: Local (Immutable):
                                    Pat 20 [105-106]: Bind:
                                        Ident 21 [105-106] "x"
                                    Expr 22 [109-110]: Lit: Int(3)
                                Stmt _id_ [128-129]: Local (Immutable):
                                    Pat _id_ [128-129]: Bind:
                                        Ident 25 [128-129] "c"
                                    Expr _id_ [128-129]: Call:
                                        Expr _id_ [128-129]: Path: Path _id_ [128-129] (Ident _id_ [128-129] "QIR.Runtime") (Ident _id_ [128-129] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [128-129]: Unit
                                Stmt 27 [153-163]: Local (Immutable):
                                    Pat 28 [157-158]: Bind:
                                        Ident 29 [157-158] "y"
                                    Expr 30 [161-162]: Lit: Int(3)
                                Stmt _id_ [128-129]: Semi: Expr _id_ [128-129]: Call:
                                    Expr _id_ [128-129]: Path: Path _id_ [128-129] (Ident _id_ [128-129] "QIR.Runtime") (Ident _id_ [128-129] "__quantum__rt__qubit_release")
                                    Expr _id_ [128-129]: Tuple:
                                        Expr _id_ [128-129]: Path: Path _id_ [128-129] (Ident 25 [128-129] "c")
                                Stmt _id_ [78-85]: Semi: Expr _id_ [78-85]: Call:
                                    Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "QIR.Runtime") (Ident _id_ [78-85] "__quantum__rt__qubit_release")
                                    Expr _id_ [78-85]: Tuple:
                                        Expr _id_ [78-85]: Path: Path _id_ [78-85] (Ident _id_ [78-85] "__generated_ident_1__")
                                Stmt _id_ [69-76]: Semi: Expr _id_ [69-76]: Call:
                                    Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "QIR.Runtime") (Ident _id_ [69-76] "__quantum__rt__qubit_release")
                                    Expr _id_ [69-76]: Tuple:
                                        Expr _id_ [69-76]: Path: Path _id_ [69-76] (Ident _id_ [69-76] "__generated_ident_0__")
                            Stmt 31 [182-192]: Local (Immutable):
                                Pat 32 [186-187]: Bind:
                                    Ident 33 [186-187] "z"
                                Expr 34 [190-191]: Lit: Int(3)"#]],
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
        &expect![[r#"
            Namespace 1 [0-157] (Ident 2 [10-15] "input"):
                Item 3 [22-155]:
                    Callable 4 [22-155] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-155]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60]: Bind:
                                    Ident 11 [59-60] "a"
                                Expr _id_ [59-60]: Call:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [59-60]: Unit
                            Stmt _id_ [80-130]: Expr: Expr _id_ [80-130]: Expr Block: Block 17 [96-130]:
                                Stmt _id_ [84-85]: Local (Immutable):
                                    Pat _id_ [84-85]: Bind:
                                        Ident 15 [84-85] "b"
                                    Expr _id_ [84-85]: Call:
                                        Expr _id_ [84-85]: Path: Path _id_ [84-85] (Ident _id_ [84-85] "QIR.Runtime") (Ident _id_ [84-85] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [84-85]: Unit
                                Stmt 18 [110-120]: Local (Immutable):
                                    Pat 19 [114-115]: Bind:
                                        Ident 20 [114-115] "x"
                                    Expr 21 [118-119]: Lit: Int(3)
                                Stmt _id_ [84-85]: Semi: Expr _id_ [84-85]: Call:
                                    Expr _id_ [84-85]: Path: Path _id_ [84-85] (Ident _id_ [84-85] "QIR.Runtime") (Ident _id_ [84-85] "__quantum__rt__qubit_release")
                                    Expr _id_ [84-85]: Tuple:
                                        Expr _id_ [84-85]: Path: Path _id_ [84-85] (Ident 15 [84-85] "b")
                            Stmt 22 [139-149]: Local (Immutable):
                                Pat 23 [143-144]: Bind:
                                    Ident 24 [143-144] "y"
                                Expr 25 [147-148]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release")
                                Expr _id_ [59-60]: Tuple:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "a")"#]],
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
                            Stmt _id_ [79-80]: Local (Immutable):
                                Pat _id_ [79-80]: Bind:
                                    Ident 15 [79-80] "a"
                                Expr _id_ [79-80]: Call:
                                    Expr _id_ [79-80]: Path: Path _id_ [79-80] (Ident _id_ [79-80] "QIR.Runtime") (Ident _id_ [79-80] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [79-80]: Unit
                            Stmt 17 [100-111]: Local (Immutable):
                                Pat 18 [104-106]: Bind:
                                    Ident 19 [104-106] "x2"
                                Expr 20 [109-110]: Lit: Int(3)
                            Stmt 21 [120-208]: Expr: Expr 22 [120-208]: Expr Block: Block 23 [120-208]:
                                Stmt 24 [134-145]: Local (Immutable):
                                    Pat 25 [138-140]: Bind:
                                        Ident 26 [138-140] "y1"
                                    Expr 27 [143-144]: Lit: Int(3)
                                Stmt _id_ [162-163]: Local (Immutable):
                                    Pat _id_ [162-163]: Bind:
                                        Ident 30 [162-163] "b"
                                    Expr _id_ [162-163]: Call:
                                        Expr _id_ [162-163]: Path: Path _id_ [162-163] (Ident _id_ [162-163] "QIR.Runtime") (Ident _id_ [162-163] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [162-163]: Unit
                                Stmt 32 [187-198]: Local (Immutable):
                                    Pat 33 [191-193]: Bind:
                                        Ident 34 [191-193] "y2"
                                    Expr 35 [196-197]: Lit: Int(3)
                                Stmt _id_ [162-163]: Semi: Expr _id_ [162-163]: Call:
                                    Expr _id_ [162-163]: Path: Path _id_ [162-163] (Ident _id_ [162-163] "QIR.Runtime") (Ident _id_ [162-163] "__quantum__rt__qubit_release")
                                    Expr _id_ [162-163]: Tuple:
                                        Expr _id_ [162-163]: Path: Path _id_ [162-163] (Ident 30 [162-163] "b")
                            Stmt 36 [217-228]: Local (Immutable):
                                Pat 37 [221-223]: Bind:
                                    Ident 38 [221-223] "x3"
                                Expr 39 [226-227]: Lit: Int(3)
                            Stmt 40 [237-325]: Expr: Expr 41 [237-325]: Expr Block: Block 42 [237-325]:
                                Stmt 43 [251-262]: Local (Immutable):
                                    Pat 44 [255-257]: Bind:
                                        Ident 45 [255-257] "z1"
                                    Expr 46 [260-261]: Lit: Int(3)
                                Stmt _id_ [279-280]: Local (Immutable):
                                    Pat _id_ [279-280]: Bind:
                                        Ident 49 [279-280] "c"
                                    Expr _id_ [279-280]: Call:
                                        Expr _id_ [279-280]: Path: Path _id_ [279-280] (Ident _id_ [279-280] "QIR.Runtime") (Ident _id_ [279-280] "__quantum__rt__qubit_allocate")
                                        Expr _id_ [279-280]: Unit
                                Stmt 51 [304-315]: Local (Immutable):
                                    Pat 52 [308-310]: Bind:
                                        Ident 53 [308-310] "z2"
                                    Expr 54 [313-314]: Lit: Int(3)
                                Stmt _id_ [279-280]: Semi: Expr _id_ [279-280]: Call:
                                    Expr _id_ [279-280]: Path: Path _id_ [279-280] (Ident _id_ [279-280] "QIR.Runtime") (Ident _id_ [279-280] "__quantum__rt__qubit_release")
                                    Expr _id_ [279-280]: Tuple:
                                        Expr _id_ [279-280]: Path: Path _id_ [279-280] (Ident 49 [279-280] "c")
                            Stmt 55 [334-345]: Local (Immutable):
                                Pat 56 [338-340]: Bind:
                                    Ident 57 [338-340] "x4"
                                Expr 58 [343-344]: Lit: Int(3)
                            Stmt _id_ [79-80]: Semi: Expr _id_ [79-80]: Call:
                                Expr _id_ [79-80]: Path: Path _id_ [79-80] (Ident _id_ [79-80] "QIR.Runtime") (Ident _id_ [79-80] "__quantum__rt__qubit_release")
                                Expr _id_ [79-80]: Tuple:
                                    Expr _id_ [79-80]: Path: Path _id_ [79-80] (Ident 15 [79-80] "a")"#]],
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
        &expect![[r#"
            Namespace 1 [0-241] (Ident 2 [10-15] "input"):
                Item 3 [22-239]:
                    Callable 4 [22-239] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-239]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60]: Bind:
                                    Ident 11 [59-60] "a"
                                Expr _id_ [59-60]: Call:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [59-60]: Unit
                            Stmt 13 [80-151]: Expr: Expr 14 [80-151]: If:
                                Expr 15 [83-87]: Lit: Bool(true)
                                Block 16 [88-151]:
                                    Stmt _id_ [106-107]: Local (Immutable):
                                        Pat _id_ [106-107]: Bind:
                                            Ident 19 [106-107] "b"
                                        Expr _id_ [106-107]: Call:
                                            Expr _id_ [106-107]: Path: Path _id_ [106-107] (Ident _id_ [106-107] "QIR.Runtime") (Ident _id_ [106-107] "__quantum__rt__qubit_allocate")
                                            Expr _id_ [106-107]: Unit
                                    Stmt 21 [131-141]: Semi: Expr _id_ [131-140]: Expr Block: Block _id_ [131-140]:
                                        Stmt _id_ [138-140]: Local (Immutable):
                                            Pat _id_ [138-140]: Bind:
                                                Ident _id_ [138-140] "__generated_ident_0__"
                                            Expr 23 [138-140]: Unit
                                        Stmt _id_ [106-107]: Semi: Expr _id_ [106-107]: Call:
                                            Expr _id_ [106-107]: Path: Path _id_ [106-107] (Ident _id_ [106-107] "QIR.Runtime") (Ident _id_ [106-107] "__quantum__rt__qubit_release")
                                            Expr _id_ [106-107]: Tuple:
                                                Expr _id_ [106-107]: Path: Path _id_ [106-107] (Ident 19 [106-107] "b")
                                        Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                            Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release")
                                            Expr _id_ [59-60]: Tuple:
                                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "a")
                                        Stmt _id_ [131-140]: Semi: Expr _id_ [131-140]: Return: Expr _id_ [138-140]: Path: Path _id_ [138-140] (Ident _id_ [138-140] "__generated_ident_0__")
                                    Stmt _id_ [106-107]: Semi: Expr _id_ [106-107]: Call:
                                        Expr _id_ [106-107]: Path: Path _id_ [106-107] (Ident _id_ [106-107] "QIR.Runtime") (Ident _id_ [106-107] "__quantum__rt__qubit_release")
                                        Expr _id_ [106-107]: Tuple:
                                            Expr _id_ [106-107]: Path: Path _id_ [106-107] (Ident 19 [106-107] "b")
                            Stmt _id_ [161-233]: Local (Immutable):
                                Pat _id_ [161-233]: Bind:
                                    Ident _id_ [161-233] "__generated_ident_2__"
                                Expr 25 [161-233]: If:
                                    Expr 26 [164-169]: Lit: Bool(false)
                                    Block 27 [170-233]:
                                        Stmt _id_ [188-189]: Local (Immutable):
                                            Pat _id_ [188-189]: Bind:
                                                Ident 30 [188-189] "c"
                                            Expr _id_ [188-189]: Call:
                                                Expr _id_ [188-189]: Path: Path _id_ [188-189] (Ident _id_ [188-189] "QIR.Runtime") (Ident _id_ [188-189] "__quantum__rt__qubit_allocate")
                                                Expr _id_ [188-189]: Unit
                                        Stmt 32 [213-223]: Semi: Expr _id_ [213-222]: Expr Block: Block _id_ [213-222]:
                                            Stmt _id_ [220-222]: Local (Immutable):
                                                Pat _id_ [220-222]: Bind:
                                                    Ident _id_ [220-222] "__generated_ident_1__"
                                                Expr 34 [220-222]: Unit
                                            Stmt _id_ [188-189]: Semi: Expr _id_ [188-189]: Call:
                                                Expr _id_ [188-189]: Path: Path _id_ [188-189] (Ident _id_ [188-189] "QIR.Runtime") (Ident _id_ [188-189] "__quantum__rt__qubit_release")
                                                Expr _id_ [188-189]: Tuple:
                                                    Expr _id_ [188-189]: Path: Path _id_ [188-189] (Ident 30 [188-189] "c")
                                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release")
                                                Expr _id_ [59-60]: Tuple:
                                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "a")
                                            Stmt _id_ [213-222]: Semi: Expr _id_ [213-222]: Return: Expr _id_ [220-222]: Path: Path _id_ [220-222] (Ident _id_ [220-222] "__generated_ident_1__")
                                        Stmt _id_ [188-189]: Semi: Expr _id_ [188-189]: Call:
                                            Expr _id_ [188-189]: Path: Path _id_ [188-189] (Ident _id_ [188-189] "QIR.Runtime") (Ident _id_ [188-189] "__quantum__rt__qubit_release")
                                            Expr _id_ [188-189]: Tuple:
                                                Expr _id_ [188-189]: Path: Path _id_ [188-189] (Ident 30 [188-189] "c")
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release")
                                Expr _id_ [59-60]: Tuple:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "a")
                            Stmt _id_ [161-233]: Expr: Expr _id_ [161-233]: Path: Path _id_ [161-233] (Ident _id_ [161-233] "__generated_ident_2__")"#]],
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
                };
            }
        }" },
        &expect![[r#"
            Namespace 1 [0-172] (Ident 2 [10-15] "input"):
                Item 3 [22-170]:
                    Callable 4 [22-170] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-170]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60]: Bind:
                                    Ident 11 [59-60] "a"
                                Expr _id_ [59-60]: Call:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [59-60]: Unit
                            Stmt 13 [80-92]: Local (Immutable):
                                Pat 14 [84-85]: Bind:
                                    Ident 15 [84-85] "x"
                                Expr 16 [88-91]: Expr Block: Block 17 [88-91]:
                                    Stmt 18 [89-90]: Expr: Expr 19 [89-90]: Lit: Int(3)
                            Stmt 20 [101-164]: Local (Immutable):
                                Pat 21 [105-106]: Bind:
                                    Ident 22 [105-106] "y"
                                Expr 23 [109-163]: Expr Block: Block 24 [109-163]:
                                    Stmt _id_ [127-128]: Local (Immutable):
                                        Pat _id_ [127-128]: Bind:
                                            Ident 27 [127-128] "b"
                                        Expr _id_ [127-128]: Call:
                                            Expr _id_ [127-128]: Path: Path _id_ [127-128] (Ident _id_ [127-128] "QIR.Runtime") (Ident _id_ [127-128] "__quantum__rt__qubit_allocate")
                                            Expr _id_ [127-128]: Unit
                                    Stmt _id_ [152-153]: Local (Immutable):
                                        Pat _id_ [152-153]: Bind:
                                            Ident _id_ [152-153] "__generated_ident_0__"
                                        Expr 30 [152-153]: Lit: Int(3)
                                    Stmt _id_ [127-128]: Semi: Expr _id_ [127-128]: Call:
                                        Expr _id_ [127-128]: Path: Path _id_ [127-128] (Ident _id_ [127-128] "QIR.Runtime") (Ident _id_ [127-128] "__quantum__rt__qubit_release")
                                        Expr _id_ [127-128]: Tuple:
                                            Expr _id_ [127-128]: Path: Path _id_ [127-128] (Ident 27 [127-128] "b")
                                    Stmt _id_ [152-153]: Expr: Expr _id_ [152-153]: Path: Path _id_ [152-153] (Ident _id_ [152-153] "__generated_ident_0__")
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release")
                                Expr _id_ [59-60]: Tuple:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "a")"#]],
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
        &expect![[r#"
            Namespace 1 [0-152] (Ident 2 [10-15] "input"):
                Item 3 [22-150]:
                    Callable 4 [22-150] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-44]: Unit
                        body: Block: Block 8 [45-150]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60]: Bind:
                                    Ident 11 [59-60] "a"
                                Expr _id_ [59-60]: Call:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_allocate_array")
                                    Expr _id_ [59-60]: Tuple:
                                        Expr 13 [69-123]: Expr Block: Block 14 [69-123]:
                                            Stmt _id_ [87-88]: Local (Immutable):
                                                Pat _id_ [87-88]: Bind:
                                                    Ident 17 [87-88] "b"
                                                Expr _id_ [87-88]: Call:
                                                    Expr _id_ [87-88]: Path: Path _id_ [87-88] (Ident _id_ [87-88] "QIR.Runtime") (Ident _id_ [87-88] "__quantum__rt__qubit_allocate")
                                                    Expr _id_ [87-88]: Unit
                                            Stmt _id_ [112-113]: Local (Immutable):
                                                Pat _id_ [112-113]: Bind:
                                                    Ident _id_ [112-113] "__generated_ident_0__"
                                                Expr 20 [112-113]: Lit: Int(3)
                                            Stmt _id_ [87-88]: Semi: Expr _id_ [87-88]: Call:
                                                Expr _id_ [87-88]: Path: Path _id_ [87-88] (Ident _id_ [87-88] "QIR.Runtime") (Ident _id_ [87-88] "__quantum__rt__qubit_release")
                                                Expr _id_ [87-88]: Tuple:
                                                    Expr _id_ [87-88]: Path: Path _id_ [87-88] (Ident 17 [87-88] "b")
                                            Stmt _id_ [112-113]: Expr: Expr _id_ [112-113]: Path: Path _id_ [112-113] (Ident _id_ [112-113] "__generated_ident_0__")
                            Stmt 21 [134-144]: Local (Immutable):
                                Pat 22 [138-139]: Bind:
                                    Ident 23 [138-139] "x"
                                Expr 24 [142-143]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60]: Call:
                                Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident _id_ [59-60] "QIR.Runtime") (Ident _id_ [59-60] "__quantum__rt__qubit_release_array")
                                Expr _id_ [59-60]: Tuple:
                                    Expr _id_ [59-60]: Path: Path _id_ [59-60] (Ident 11 [59-60] "a")"#]],
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
        &expect![[r#"
            Namespace 1 [0-149] (Ident 2 [10-15] "input"):
                Item 3 [22-147]:
                    Callable 4 [22-147] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-43]: Prim (Int)
                        body: Block: Block 8 [44-147]:
                            Stmt _id_ [58-59]: Local (Immutable):
                                Pat _id_ [58-59]: Bind:
                                    Ident 11 [58-59] "a"
                                Expr _id_ [58-59]: Call:
                                    Expr _id_ [58-59]: Path: Path _id_ [58-59] (Ident _id_ [58-59] "QIR.Runtime") (Ident _id_ [58-59] "__quantum__rt__qubit_allocate")
                                    Expr _id_ [58-59]: Unit
                            Stmt 13 [79-141]: Semi: Expr _id_ [79-140]: Expr Block: Block _id_ [79-140]:
                                Stmt _id_ [86-140]: Local (Immutable):
                                    Pat _id_ [86-140]: Bind:
                                        Ident _id_ [86-140] "__generated_ident_0__"
                                    Expr 15 [86-140]: Expr Block: Block 16 [86-140]:
                                        Stmt _id_ [104-105]: Local (Immutable):
                                            Pat _id_ [104-105]: Bind:
                                                Ident 19 [104-105] "b"
                                            Expr _id_ [104-105]: Call:
                                                Expr _id_ [104-105]: Path: Path _id_ [104-105] (Ident _id_ [104-105] "QIR.Runtime") (Ident _id_ [104-105] "__quantum__rt__qubit_allocate")
                                                Expr _id_ [104-105]: Unit
                                        Stmt _id_ [129-130]: Local (Immutable):
                                            Pat _id_ [129-130]: Bind:
                                                Ident _id_ [129-130] "__generated_ident_1__"
                                            Expr 22 [129-130]: Lit: Int(3)
                                        Stmt _id_ [104-105]: Semi: Expr _id_ [104-105]: Call:
                                            Expr _id_ [104-105]: Path: Path _id_ [104-105] (Ident _id_ [104-105] "QIR.Runtime") (Ident _id_ [104-105] "__quantum__rt__qubit_release")
                                            Expr _id_ [104-105]: Tuple:
                                                Expr _id_ [104-105]: Path: Path _id_ [104-105] (Ident 19 [104-105] "b")
                                        Stmt _id_ [129-130]: Expr: Expr _id_ [129-130]: Path: Path _id_ [129-130] (Ident _id_ [129-130] "__generated_ident_1__")
                                Stmt _id_ [58-59]: Semi: Expr _id_ [58-59]: Call:
                                    Expr _id_ [58-59]: Path: Path _id_ [58-59] (Ident _id_ [58-59] "QIR.Runtime") (Ident _id_ [58-59] "__quantum__rt__qubit_release")
                                    Expr _id_ [58-59]: Tuple:
                                        Expr _id_ [58-59]: Path: Path _id_ [58-59] (Ident 11 [58-59] "a")
                                Stmt _id_ [79-140]: Semi: Expr _id_ [79-140]: Return: Expr _id_ [86-140]: Path: Path _id_ [86-140] (Ident _id_ [86-140] "__generated_ident_0__")
                            Stmt _id_ [58-59]: Semi: Expr _id_ [58-59]: Call:
                                Expr _id_ [58-59]: Path: Path _id_ [58-59] (Ident _id_ [58-59] "QIR.Runtime") (Ident _id_ [58-59] "__quantum__rt__qubit_release")
                                Expr _id_ [58-59]: Tuple:
                                    Expr _id_ [58-59]: Path: Path _id_ [58-59] (Ident 11 [58-59] "a")"#]],
    );
}

#[test]
#[ignore = "lambdas are not supported yet"]
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

#[test]
fn test_unrelated_unaffected() {
    check(
        indoc! { "namespace input {
            operation Foo() : Int {
                if true {
                    return 3;
                }

                {
                    let x = 4;
                    x
                }
            }
        }" },
        &expect![[r#"
            Namespace 1 [0-161] (Ident 2 [10-15] "input"):
                Item 3 [22-159]:
                    Callable 4 [22-159] (Operation):
                        name: Ident 5 [32-35] "Foo"
                        input: Pat 6 [35-37]: Unit
                        output: Type 7 [40-43]: Prim (Int)
                        body: Block: Block 8 [44-159]:
                            Stmt 9 [54-95]: Expr: Expr 10 [54-95]: If:
                                Expr 11 [57-61]: Lit: Bool(true)
                                Block 12 [62-95]:
                                    Stmt 13 [76-85]: Semi: Expr 14 [76-84]: Return: Expr 15 [83-84]: Lit: Int(3)
                            Stmt 16 [105-153]: Expr: Expr 17 [105-153]: Expr Block: Block 18 [105-153]:
                                Stmt 19 [119-129]: Local (Immutable):
                                    Pat 20 [123-124]: Bind:
                                        Ident 21 [123-124] "x"
                                    Expr 22 [127-128]: Lit: Int(4)
                                Stmt 23 [142-143]: Expr: Expr 24 [142-143]: Path: Path 25 [142-143] (Ident 26 [142-143] "x")"#]],
    );
}
