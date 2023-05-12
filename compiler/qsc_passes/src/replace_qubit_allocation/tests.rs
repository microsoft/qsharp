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
    replace_qubit_allocation(&mut unit, store.core());
    expect.assert_eq(&unit.package.to_string());
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
            Package:
                Item 0 [0-98] (Public):
                    Namespace (Ident 12 [10-15] "input"): Item 1
                Item 1 [22-96] (Public):
                    Parent: 0
                    Callable 0 [22-96] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-96] [Type Unit]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60] [Type Qubit]: Bind: Ident 6 [59-60] "q"
                                Expr _id_ [59-60] [Type Qubit]: Call:
                                    Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [59-60] [Type Unit]: Unit
                            Stmt 8 [80-90]: Local (Immutable):
                                Pat 9 [84-85] [Type Int]: Bind: Ident 10 [84-85] "x"
                                Expr 11 [88-89] [Type Int]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [59-60] [Type Qubit]: Var: Local 6"#]],
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
            Package:
                Item 0 [0-99] (Public):
                    Namespace (Ident 13 [10-15] "input"): Item 1
                Item 1 [22-97] (Public):
                    Parent: 0
                    Callable 0 [22-97] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-97] [Type Unit]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60] [Type (Qubit)[]]: Bind: Ident 6 [59-60] "q"
                                Expr _id_ [59-60] [Type Qubit]: Call:
                                    Expr _id_ [59-60] [Type (Int => (Qubit)[])]: Var: Item 3 (Package 0)
                                    Expr 8 [69-70] [Type Int]: Lit: Int(3)
                            Stmt 9 [81-91]: Local (Immutable):
                                Pat 10 [85-86] [Type Int]: Bind: Ident 11 [85-86] "x"
                                Expr 12 [89-90] [Type Int]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                Expr _id_ [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 4 (Package 0)
                                Expr _id_ [59-60] [Type (Qubit)[]]: Var: Local 6"#]],
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
            Package:
                Item 0 [0-109] (Public):
                    Namespace (Ident 14 [10-15] "input"): Item 1
                Item 1 [22-107] (Public):
                    Parent: 0
                    Callable 0 [22-107] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-107] [Type Unit]:
                            Stmt _id_ [64-71]: Local (Immutable):
                                Pat _id_ [64-71] [Type Qubit]: Bind: Ident 15 [64-71] "generated_ident_15"
                                Expr _id_ [64-71] [Type Qubit]: Call:
                                    Expr _id_ [64-71] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [64-71] [Type Unit]: Unit
                            Stmt _id_ [73-80]: Local (Immutable):
                                Pat _id_ [73-80] [Type Qubit]: Bind: Ident 16 [73-80] "generated_ident_16"
                                Expr _id_ [73-80] [Type Qubit]: Call:
                                    Expr _id_ [73-80] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [73-80] [Type Unit]: Unit
                            Stmt _id_ [55-82]: Local (Immutable):
                                Pat 5 [59-60] [Type (Qubit, Qubit)]: Bind: Ident 6 [59-60] "q"
                                Expr _id_ [63-81] [Type (Qubit, Qubit)]: Tuple:
                                    Expr _id_ [64-71] [Type Qubit]: Var: Local 15
                                    Expr _id_ [73-80] [Type Qubit]: Var: Local 16
                            Stmt 10 [91-101]: Local (Immutable):
                                Pat 11 [95-96] [Type Int]: Bind: Ident 12 [95-96] "x"
                                Expr 13 [99-100] [Type Int]: Lit: Int(3)
                            Stmt _id_ [73-80]: Semi: Expr _id_ [73-80] [Type Unit]: Call:
                                Expr _id_ [73-80] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [73-80] [Type Qubit]: Var: Local 16
                            Stmt _id_ [64-71]: Semi: Expr _id_ [64-71] [Type Unit]: Call:
                                Expr _id_ [64-71] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [64-71] [Type Qubit]: Var: Local 15"#]],
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
            Package:
                Item 0 [0-115] (Public):
                    Namespace (Ident 18 [10-15] "input"): Item 1
                Item 1 [22-113] (Public):
                    Parent: 0
                    Callable 0 [22-113] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-113] [Type Unit]:
                            Stmt _id_ [69-76]: Local (Immutable):
                                Pat _id_ [69-76] [Type Qubit]: Bind: Ident 19 [69-76] "generated_ident_19"
                                Expr _id_ [69-76] [Type Qubit]: Call:
                                    Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [69-76] [Type Unit]: Unit
                            Stmt _id_ [78-86]: Local (Immutable):
                                Pat _id_ [78-86] [Type (Qubit)[]]: Bind: Ident 20 [78-86] "generated_ident_20"
                                Expr _id_ [78-86] [Type Qubit]: Call:
                                    Expr _id_ [78-86] [Type (Int => (Qubit)[])]: Var: Item 3 (Package 0)
                                    Expr 13 [84-85] [Type Int]: Lit: Int(3)
                            Stmt _id_ [55-88]: Local (Immutable):
                                Pat 5 [59-65] [Type (Qubit, (Qubit)[])]: Tuple:
                                    Pat 6 [60-61] [Type Qubit]: Bind: Ident 7 [60-61] "a"
                                    Pat 8 [63-64] [Type (Qubit)[]]: Bind: Ident 9 [63-64] "b"
                                Expr _id_ [68-87] [Type (Qubit, (Qubit)[])]: Tuple:
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 19
                                    Expr _id_ [78-86] [Type (Qubit)[]]: Var: Local 20
                            Stmt 14 [97-107]: Local (Immutable):
                                Pat 15 [101-102] [Type Int]: Bind: Ident 16 [101-102] "x"
                                Expr 17 [105-106] [Type Int]: Lit: Int(3)
                            Stmt _id_ [78-86]: Semi: Expr _id_ [78-86] [Type Unit]: Call:
                                Expr _id_ [78-86] [Type ((Qubit)[] => Unit)]: Var: Item 4 (Package 0)
                                Expr _id_ [78-86] [Type (Qubit)[]]: Var: Local 20
                            Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [69-76] [Type Qubit]: Var: Local 19"#]],
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
            Package:
                Item 0 [0-214] (Public):
                    Namespace (Ident 34 [10-15] "input"): Item 1, Item 2
                Item 1 [22-112] (Public):
                    Parent: 0
                    Callable 0 [22-112] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-112] [Type Unit]:
                            Stmt _id_ [69-76]: Local (Immutable):
                                Pat _id_ [69-76] [Type Qubit]: Bind: Ident 35 [69-76] "generated_ident_35"
                                Expr _id_ [69-76] [Type Qubit]: Call:
                                    Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [69-76] [Type Unit]: Unit
                            Stmt _id_ [78-85]: Local (Immutable):
                                Pat _id_ [78-85] [Type Qubit]: Bind: Ident 36 [78-85] "generated_ident_36"
                                Expr _id_ [78-85] [Type Qubit]: Call:
                                    Expr _id_ [78-85] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [78-85] [Type Unit]: Unit
                            Stmt _id_ [55-87]: Local (Immutable):
                                Pat 5 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                    Pat 6 [60-61] [Type Qubit]: Bind: Ident 7 [60-61] "a"
                                    Pat 8 [63-64] [Type Qubit]: Bind: Ident 9 [63-64] "b"
                                Expr _id_ [68-86] [Type (Qubit, Qubit)]: Tuple:
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 35
                                    Expr _id_ [78-85] [Type Qubit]: Var: Local 36
                            Stmt 13 [96-106]: Local (Immutable):
                                Pat 14 [100-101] [Type Int]: Bind: Ident 15 [100-101] "x"
                                Expr 16 [104-105] [Type Int]: Lit: Int(3)
                            Stmt _id_ [78-85]: Semi: Expr _id_ [78-85] [Type Unit]: Call:
                                Expr _id_ [78-85] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [78-85] [Type Qubit]: Var: Local 36
                            Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [69-76] [Type Qubit]: Var: Local 35
                Item 2 [122-212] (Public):
                    Parent: 0
                    Callable 17 [122-212] (Operation):
                        name: Ident 18 [132-135] "Bar"
                        input: Pat 19 [135-137] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 20 [145-212] [Type Unit]:
                            Stmt _id_ [169-176]: Local (Immutable):
                                Pat _id_ [169-176] [Type Qubit]: Bind: Ident 37 [169-176] "generated_ident_37"
                                Expr _id_ [169-176] [Type Qubit]: Call:
                                    Expr _id_ [169-176] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [169-176] [Type Unit]: Unit
                            Stmt _id_ [178-185]: Local (Immutable):
                                Pat _id_ [178-185] [Type Qubit]: Bind: Ident 38 [178-185] "generated_ident_38"
                                Expr _id_ [178-185] [Type Qubit]: Call:
                                    Expr _id_ [178-185] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [178-185] [Type Unit]: Unit
                            Stmt _id_ [155-187]: Local (Immutable):
                                Pat 22 [159-165] [Type (Qubit, Qubit)]: Tuple:
                                    Pat 23 [160-161] [Type Qubit]: Bind: Ident 24 [160-161] "c"
                                    Pat 25 [163-164] [Type Qubit]: Bind: Ident 26 [163-164] "d"
                                Expr _id_ [168-186] [Type (Qubit, Qubit)]: Tuple:
                                    Expr _id_ [169-176] [Type Qubit]: Var: Local 37
                                    Expr _id_ [178-185] [Type Qubit]: Var: Local 38
                            Stmt 30 [196-206]: Local (Immutable):
                                Pat 31 [200-201] [Type Int]: Bind: Ident 32 [200-201] "x"
                                Expr 33 [204-205] [Type Int]: Lit: Int(3)
                            Stmt _id_ [178-185]: Semi: Expr _id_ [178-185] [Type Unit]: Call:
                                Expr _id_ [178-185] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [178-185] [Type Qubit]: Var: Local 38
                            Stmt _id_ [169-176]: Semi: Expr _id_ [169-176] [Type Unit]: Call:
                                Expr _id_ [169-176] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [169-176] [Type Qubit]: Var: Local 37"#]],
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
            Package:
                Item 0 [0-200] (Public):
                    Namespace (Ident 30 [10-15] "input"): Item 1
                Item 1 [22-198] (Public):
                    Parent: 0
                    Callable 0 [22-198] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-198] [Type Unit]:
                            Stmt _id_ [55-173]: Expr: Expr _id_ [55-173] [Type Unit]: Expr Block: Block 13 [87-173] [Type Unit]:
                                Stmt _id_ [69-76]: Local (Immutable):
                                    Pat _id_ [69-76] [Type Qubit]: Bind: Ident 31 [69-76] "generated_ident_31"
                                    Expr _id_ [69-76] [Type Qubit]: Call:
                                        Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                        Expr _id_ [69-76] [Type Unit]: Unit
                                Stmt _id_ [78-85]: Local (Immutable):
                                    Pat _id_ [78-85] [Type Qubit]: Bind: Ident 32 [78-85] "generated_ident_32"
                                    Expr _id_ [78-85] [Type Qubit]: Call:
                                        Expr _id_ [78-85] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                        Expr _id_ [78-85] [Type Unit]: Unit
                                Stmt _id_ [55-173]: Local (Immutable):
                                    Pat 5 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 6 [60-61] [Type Qubit]: Bind: Ident 7 [60-61] "a"
                                        Pat 8 [63-64] [Type Qubit]: Bind: Ident 9 [63-64] "b"
                                    Expr _id_ [68-86] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 31
                                        Expr _id_ [78-85] [Type Qubit]: Var: Local 32
                                Stmt 14 [101-111]: Local (Immutable):
                                    Pat 15 [105-106] [Type Int]: Bind: Ident 16 [105-106] "x"
                                    Expr 17 [109-110] [Type Int]: Lit: Int(3)
                                Stmt _id_ [128-129]: Local (Immutable):
                                    Pat _id_ [128-129] [Type Qubit]: Bind: Ident 20 [128-129] "c"
                                    Expr _id_ [128-129] [Type Qubit]: Call:
                                        Expr _id_ [128-129] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                        Expr _id_ [128-129] [Type Unit]: Unit
                                Stmt 22 [153-163]: Local (Immutable):
                                    Pat 23 [157-158] [Type Int]: Bind: Ident 24 [157-158] "y"
                                    Expr 25 [161-162] [Type Int]: Lit: Int(3)
                                Stmt _id_ [128-129]: Semi: Expr _id_ [128-129] [Type Unit]: Call:
                                    Expr _id_ [128-129] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [128-129] [Type Qubit]: Var: Local 20
                                Stmt _id_ [78-85]: Semi: Expr _id_ [78-85] [Type Unit]: Call:
                                    Expr _id_ [78-85] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [78-85] [Type Qubit]: Var: Local 32
                                Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                    Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 31
                            Stmt 26 [182-192]: Local (Immutable):
                                Pat 27 [186-187] [Type Int]: Bind: Ident 28 [186-187] "z"
                                Expr 29 [190-191] [Type Int]: Lit: Int(3)"#]],
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
            Package:
                Item 0 [0-157] (Public):
                    Namespace (Ident 21 [10-15] "input"): Item 1
                Item 1 [22-155] (Public):
                    Parent: 0
                    Callable 0 [22-155] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-155] [Type Unit]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60] [Type Qubit]: Bind: Ident 6 [59-60] "a"
                                Expr _id_ [59-60] [Type Qubit]: Call:
                                    Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [59-60] [Type Unit]: Unit
                            Stmt _id_ [80-130]: Expr: Expr _id_ [80-130] [Type Unit]: Expr Block: Block 12 [96-130] [Type Unit]:
                                Stmt _id_ [84-85]: Local (Immutable):
                                    Pat _id_ [84-85] [Type Qubit]: Bind: Ident 10 [84-85] "b"
                                    Expr _id_ [84-85] [Type Qubit]: Call:
                                        Expr _id_ [84-85] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                        Expr _id_ [84-85] [Type Unit]: Unit
                                Stmt 13 [110-120]: Local (Immutable):
                                    Pat 14 [114-115] [Type Int]: Bind: Ident 15 [114-115] "x"
                                    Expr 16 [118-119] [Type Int]: Lit: Int(3)
                                Stmt _id_ [84-85]: Semi: Expr _id_ [84-85] [Type Unit]: Call:
                                    Expr _id_ [84-85] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [84-85] [Type Qubit]: Var: Local 10
                            Stmt 17 [139-149]: Local (Immutable):
                                Pat 18 [143-144] [Type Int]: Bind: Ident 19 [143-144] "y"
                                Expr 20 [147-148] [Type Int]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [59-60] [Type Qubit]: Var: Local 6"#]],
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
            Package:
                Item 0 [0-353] (Public):
                    Namespace (Ident 54 [10-15] "input"): Item 1
                Item 1 [22-351] (Public):
                    Parent: 0
                    Callable 0 [22-351] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-351] [Type Unit]:
                            Stmt 4 [55-66]: Local (Immutable):
                                Pat 5 [59-61] [Type Int]: Bind: Ident 6 [59-61] "x1"
                                Expr 7 [64-65] [Type Int]: Lit: Int(3)
                            Stmt _id_ [79-80]: Local (Immutable):
                                Pat _id_ [79-80] [Type Qubit]: Bind: Ident 10 [79-80] "a"
                                Expr _id_ [79-80] [Type Qubit]: Call:
                                    Expr _id_ [79-80] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [79-80] [Type Unit]: Unit
                            Stmt 12 [100-111]: Local (Immutable):
                                Pat 13 [104-106] [Type Int]: Bind: Ident 14 [104-106] "x2"
                                Expr 15 [109-110] [Type Int]: Lit: Int(3)
                            Stmt 16 [120-208]: Expr: Expr 17 [120-208] [Type Unit]: Expr Block: Block 18 [120-208] [Type Unit]:
                                Stmt 19 [134-145]: Local (Immutable):
                                    Pat 20 [138-140] [Type Int]: Bind: Ident 21 [138-140] "y1"
                                    Expr 22 [143-144] [Type Int]: Lit: Int(3)
                                Stmt _id_ [162-163]: Local (Immutable):
                                    Pat _id_ [162-163] [Type Qubit]: Bind: Ident 25 [162-163] "b"
                                    Expr _id_ [162-163] [Type Qubit]: Call:
                                        Expr _id_ [162-163] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                        Expr _id_ [162-163] [Type Unit]: Unit
                                Stmt 27 [187-198]: Local (Immutable):
                                    Pat 28 [191-193] [Type Int]: Bind: Ident 29 [191-193] "y2"
                                    Expr 30 [196-197] [Type Int]: Lit: Int(3)
                                Stmt _id_ [162-163]: Semi: Expr _id_ [162-163] [Type Unit]: Call:
                                    Expr _id_ [162-163] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [162-163] [Type Qubit]: Var: Local 25
                            Stmt 31 [217-228]: Local (Immutable):
                                Pat 32 [221-223] [Type Int]: Bind: Ident 33 [221-223] "x3"
                                Expr 34 [226-227] [Type Int]: Lit: Int(3)
                            Stmt 35 [237-325]: Expr: Expr 36 [237-325] [Type Unit]: Expr Block: Block 37 [237-325] [Type Unit]:
                                Stmt 38 [251-262]: Local (Immutable):
                                    Pat 39 [255-257] [Type Int]: Bind: Ident 40 [255-257] "z1"
                                    Expr 41 [260-261] [Type Int]: Lit: Int(3)
                                Stmt _id_ [279-280]: Local (Immutable):
                                    Pat _id_ [279-280] [Type Qubit]: Bind: Ident 44 [279-280] "c"
                                    Expr _id_ [279-280] [Type Qubit]: Call:
                                        Expr _id_ [279-280] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                        Expr _id_ [279-280] [Type Unit]: Unit
                                Stmt 46 [304-315]: Local (Immutable):
                                    Pat 47 [308-310] [Type Int]: Bind: Ident 48 [308-310] "z2"
                                    Expr 49 [313-314] [Type Int]: Lit: Int(3)
                                Stmt _id_ [279-280]: Semi: Expr _id_ [279-280] [Type Unit]: Call:
                                    Expr _id_ [279-280] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [279-280] [Type Qubit]: Var: Local 44
                            Stmt 50 [334-345]: Local (Immutable):
                                Pat 51 [338-340] [Type Int]: Bind: Ident 52 [338-340] "x4"
                                Expr 53 [343-344] [Type Int]: Lit: Int(3)
                            Stmt _id_ [79-80]: Semi: Expr _id_ [79-80] [Type Unit]: Call:
                                Expr _id_ [79-80] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [79-80] [Type Qubit]: Var: Local 10"#]],
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
            Package:
                Item 0 [0-241] (Public):
                    Namespace (Ident 30 [10-15] "input"): Item 1
                Item 1 [22-239] (Public):
                    Parent: 0
                    Callable 0 [22-239] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-239] [Type Unit]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60] [Type Qubit]: Bind: Ident 6 [59-60] "a"
                                Expr _id_ [59-60] [Type Qubit]: Call:
                                    Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [59-60] [Type Unit]: Unit
                            Stmt 8 [80-151]: Expr: Expr 9 [80-151] [Type Unit]: If:
                                Expr 10 [83-87] [Type Bool]: Lit: Bool(true)
                                Block 11 [88-151] [Type Unit]:
                                    Stmt _id_ [106-107]: Local (Immutable):
                                        Pat _id_ [106-107] [Type Qubit]: Bind: Ident 14 [106-107] "b"
                                        Expr _id_ [106-107] [Type Qubit]: Call:
                                            Expr _id_ [106-107] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                            Expr _id_ [106-107] [Type Unit]: Unit
                                    Stmt 16 [131-141]: Semi: Expr _id_ [131-140] [Type ?2]: Expr Block: Block _id_ [131-140] [Type ?2]:
                                        Stmt _id_ [138-140]: Local (Immutable):
                                            Pat _id_ [138-140] [Type Unit]: Bind: Ident 31 [138-140] "generated_ident_31"
                                            Expr 18 [138-140] [Type Unit]: Unit
                                        Stmt _id_ [106-107]: Semi: Expr _id_ [106-107] [Type Unit]: Call:
                                            Expr _id_ [106-107] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                            Expr _id_ [106-107] [Type Qubit]: Var: Local 14
                                        Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                            Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                            Expr _id_ [59-60] [Type Qubit]: Var: Local 6
                                        Stmt _id_ [131-140]: Semi: Expr _id_ [131-140] [Type ?2]: Return: Expr _id_ [138-140] [Type Unit]: Var: Local 31
                                    Stmt _id_ [106-107]: Semi: Expr _id_ [106-107] [Type Unit]: Call:
                                        Expr _id_ [106-107] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                        Expr _id_ [106-107] [Type Qubit]: Var: Local 14
                            Stmt _id_ [161-233]: Local (Immutable):
                                Pat _id_ [161-233] [Type Unit]: Bind: Ident 33 [161-233] "generated_ident_33"
                                Expr 20 [161-233] [Type Unit]: If:
                                    Expr 21 [164-169] [Type Bool]: Lit: Bool(false)
                                    Block 22 [170-233] [Type Unit]:
                                        Stmt _id_ [188-189]: Local (Immutable):
                                            Pat _id_ [188-189] [Type Qubit]: Bind: Ident 25 [188-189] "c"
                                            Expr _id_ [188-189] [Type Qubit]: Call:
                                                Expr _id_ [188-189] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                                Expr _id_ [188-189] [Type Unit]: Unit
                                        Stmt 27 [213-223]: Semi: Expr _id_ [213-222] [Type ?5]: Expr Block: Block _id_ [213-222] [Type ?5]:
                                            Stmt _id_ [220-222]: Local (Immutable):
                                                Pat _id_ [220-222] [Type Unit]: Bind: Ident 32 [220-222] "generated_ident_32"
                                                Expr 29 [220-222] [Type Unit]: Unit
                                            Stmt _id_ [188-189]: Semi: Expr _id_ [188-189] [Type Unit]: Call:
                                                Expr _id_ [188-189] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                                Expr _id_ [188-189] [Type Qubit]: Var: Local 25
                                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                                Expr _id_ [59-60] [Type Qubit]: Var: Local 6
                                            Stmt _id_ [213-222]: Semi: Expr _id_ [213-222] [Type ?5]: Return: Expr _id_ [220-222] [Type Unit]: Var: Local 32
                                        Stmt _id_ [188-189]: Semi: Expr _id_ [188-189] [Type Unit]: Call:
                                            Expr _id_ [188-189] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                            Expr _id_ [188-189] [Type Qubit]: Var: Local 25
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [59-60] [Type Qubit]: Var: Local 6
                            Stmt _id_ [161-233]: Expr: Expr _id_ [161-233] [Type Unit]: Var: Local 33"#]],
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
            Package:
                Item 0 [0-172] (Public):
                    Namespace (Ident 26 [10-15] "input"): Item 1
                Item 1 [22-170] (Public):
                    Parent: 0
                    Callable 0 [22-170] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-170] [Type Unit]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60] [Type Qubit]: Bind: Ident 6 [59-60] "a"
                                Expr _id_ [59-60] [Type Qubit]: Call:
                                    Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [59-60] [Type Unit]: Unit
                            Stmt 8 [80-92]: Local (Immutable):
                                Pat 9 [84-85] [Type Int]: Bind: Ident 10 [84-85] "x"
                                Expr 11 [88-91] [Type Int]: Expr Block: Block 12 [88-91] [Type Int]:
                                    Stmt 13 [89-90]: Expr: Expr 14 [89-90] [Type Int]: Lit: Int(3)
                            Stmt 15 [101-164]: Local (Immutable):
                                Pat 16 [105-106] [Type Int]: Bind: Ident 17 [105-106] "y"
                                Expr 18 [109-163] [Type Int]: Expr Block: Block 19 [109-163] [Type Int]:
                                    Stmt _id_ [127-128]: Local (Immutable):
                                        Pat _id_ [127-128] [Type Qubit]: Bind: Ident 22 [127-128] "b"
                                        Expr _id_ [127-128] [Type Qubit]: Call:
                                            Expr _id_ [127-128] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                            Expr _id_ [127-128] [Type Unit]: Unit
                                    Stmt _id_ [152-153]: Local (Immutable):
                                        Pat _id_ [152-153] [Type Int]: Bind: Ident 27 [152-153] "generated_ident_27"
                                        Expr 25 [152-153] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [127-128]: Semi: Expr _id_ [127-128] [Type Unit]: Call:
                                        Expr _id_ [127-128] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                        Expr _id_ [127-128] [Type Qubit]: Var: Local 22
                                    Stmt _id_ [152-153]: Expr: Expr _id_ [152-153] [Type Int]: Var: Local 27
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [59-60] [Type Qubit]: Var: Local 6"#]],
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
            Package:
                Item 0 [0-152] (Public):
                    Namespace (Ident 20 [10-15] "input"): Item 1
                Item 1 [22-150] (Public):
                    Parent: 0
                    Callable 0 [22-150] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 3 [45-150] [Type Unit]:
                            Stmt _id_ [59-60]: Local (Immutable):
                                Pat _id_ [59-60] [Type (Qubit)[]]: Bind: Ident 6 [59-60] "a"
                                Expr _id_ [59-60] [Type Qubit]: Call:
                                    Expr _id_ [59-60] [Type (Int => (Qubit)[])]: Var: Item 3 (Package 0)
                                    Expr 8 [69-123] [Type Int]: Expr Block: Block 9 [69-123] [Type Int]:
                                        Stmt _id_ [87-88]: Local (Immutable):
                                            Pat _id_ [87-88] [Type Qubit]: Bind: Ident 12 [87-88] "b"
                                            Expr _id_ [87-88] [Type Qubit]: Call:
                                                Expr _id_ [87-88] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                                Expr _id_ [87-88] [Type Unit]: Unit
                                        Stmt _id_ [112-113]: Local (Immutable):
                                            Pat _id_ [112-113] [Type Int]: Bind: Ident 21 [112-113] "generated_ident_21"
                                            Expr 15 [112-113] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [87-88]: Semi: Expr _id_ [87-88] [Type Unit]: Call:
                                            Expr _id_ [87-88] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                            Expr _id_ [87-88] [Type Qubit]: Var: Local 12
                                        Stmt _id_ [112-113]: Expr: Expr _id_ [112-113] [Type Int]: Var: Local 21
                            Stmt 16 [134-144]: Local (Immutable):
                                Pat 17 [138-139] [Type Int]: Bind: Ident 18 [138-139] "x"
                                Expr 19 [142-143] [Type Int]: Lit: Int(3)
                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                Expr _id_ [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 4 (Package 0)
                                Expr _id_ [59-60] [Type (Qubit)[]]: Var: Local 6"#]],
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
            Package:
                Item 0 [0-149] (Public):
                    Namespace (Ident 18 [10-15] "input"): Item 1
                Item 1 [22-147] (Public):
                    Parent: 0
                    Callable 0 [22-147] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 3 [44-147] [Type Int]:
                            Stmt _id_ [58-59]: Local (Immutable):
                                Pat _id_ [58-59] [Type Qubit]: Bind: Ident 6 [58-59] "a"
                                Expr _id_ [58-59] [Type Qubit]: Call:
                                    Expr _id_ [58-59] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                    Expr _id_ [58-59] [Type Unit]: Unit
                            Stmt 8 [79-141]: Semi: Expr _id_ [79-140] [Type ?2]: Expr Block: Block _id_ [79-140] [Type ?2]:
                                Stmt _id_ [86-140]: Local (Immutable):
                                    Pat _id_ [86-140] [Type Int]: Bind: Ident 19 [86-140] "generated_ident_19"
                                    Expr 10 [86-140] [Type Int]: Expr Block: Block 11 [86-140] [Type Int]:
                                        Stmt _id_ [104-105]: Local (Immutable):
                                            Pat _id_ [104-105] [Type Qubit]: Bind: Ident 14 [104-105] "b"
                                            Expr _id_ [104-105] [Type Qubit]: Call:
                                                Expr _id_ [104-105] [Type (Unit => Qubit)]: Var: Item 1 (Package 0)
                                                Expr _id_ [104-105] [Type Unit]: Unit
                                        Stmt _id_ [129-130]: Local (Immutable):
                                            Pat _id_ [129-130] [Type Int]: Bind: Ident 20 [129-130] "generated_ident_20"
                                            Expr 17 [129-130] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [104-105]: Semi: Expr _id_ [104-105] [Type Unit]: Call:
                                            Expr _id_ [104-105] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                            Expr _id_ [104-105] [Type Qubit]: Var: Local 14
                                        Stmt _id_ [129-130]: Expr: Expr _id_ [129-130] [Type Int]: Var: Local 20
                                Stmt _id_ [58-59]: Semi: Expr _id_ [58-59] [Type Unit]: Call:
                                    Expr _id_ [58-59] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                    Expr _id_ [58-59] [Type Qubit]: Var: Local 6
                                Stmt _id_ [79-140]: Semi: Expr _id_ [79-140] [Type ?2]: Return: Expr _id_ [86-140] [Type Int]: Var: Local 19
                            Stmt _id_ [58-59]: Semi: Expr _id_ [58-59] [Type Unit]: Call:
                                Expr _id_ [58-59] [Type (Qubit => Unit)]: Var: Item 2 (Package 0)
                                Expr _id_ [58-59] [Type Qubit]: Var: Local 6"#]],
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
            Package:
                Item 0 [0-161] (Public):
                    Namespace (Ident 20 [10-15] "input"): Item 1
                Item 1 [22-159] (Public):
                    Parent: 0
                    Callable 0 [22-159] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 3 [44-159] [Type Int]:
                            Stmt 4 [54-95]: Expr: Expr 5 [54-95] [Type Unit]: If:
                                Expr 6 [57-61] [Type Bool]: Lit: Bool(true)
                                Block 7 [62-95] [Type Unit]:
                                    Stmt 8 [76-85]: Semi: Expr 9 [76-84] [Type ?0]: Return: Expr 10 [83-84] [Type Int]: Lit: Int(3)
                            Stmt 11 [105-153]: Expr: Expr 12 [105-153] [Type Int]: Expr Block: Block 13 [105-153] [Type Int]:
                                Stmt 14 [119-129]: Local (Immutable):
                                    Pat 15 [123-124] [Type Int]: Bind: Ident 16 [123-124] "x"
                                    Expr 17 [127-128] [Type Int]: Lit: Int(4)
                                Stmt 18 [142-143]: Expr: Expr 19 [142-143] [Type Int]: Var: Local 16"#]],
    );
}
