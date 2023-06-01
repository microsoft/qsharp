// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::replace_qubit_allocation::ReplaceQubitAllocation;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_hir::mut_visit::MutVisitor;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    ReplaceQubitAllocation::new(store.core(), &mut unit.assigner).visit_package(&mut unit.package);
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
                    Namespace (Ident 14 [10-15] "input"): Item 1
                Item 1 [22-96] (Public):
                    Parent: 0
                    Callable 0 [22-96] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-96] (Body): Impl:
                            Pat 4 [22-96] [Type Unit]: Elided
                            Block 5 [45-96] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 8 [59-60] "q"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt 10 [80-90]: Local (Immutable):
                                    Pat 11 [84-85] [Type Int]: Bind: Ident 12 [84-85] "x"
                                    Expr 13 [88-89] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 8
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 15 [10-15] "input"): Item 1
                Item 1 [22-97] (Public):
                    Parent: 0
                    Callable 0 [22-97] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-97] (Body): Impl:
                            Pat 4 [22-97] [Type Unit]: Elided
                            Block 5 [45-97] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type (Qubit)[]]: Bind: Ident 8 [59-60] "q"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 10 [69-70] [Type Int]: Lit: Int(3)
                                Stmt 11 [81-91]: Local (Immutable):
                                    Pat 12 [85-86] [Type Int]: Bind: Ident 13 [85-86] "x"
                                    Expr 14 [89-90] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr _id_ [59-60] [Type (Qubit)[]]: Var: Local 8
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 16 [10-15] "input"): Item 1
                Item 1 [22-107] (Public):
                    Parent: 0
                    Callable 0 [22-107] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-107] (Body): Impl:
                            Pat 4 [22-107] [Type Unit]: Elided
                            Block 5 [45-107] [Type Unit]:
                                Stmt _id_ [64-71]: Local (Immutable):
                                    Pat _id_ [64-71] [Type Qubit]: Bind: Ident 17 [64-71] "generated_ident_17"
                                    Expr _id_ [64-71] [Type Qubit]: Call:
                                        Expr _id_ [64-71] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [64-71] [Type Unit]: Unit
                                Stmt _id_ [73-80]: Local (Immutable):
                                    Pat _id_ [73-80] [Type Qubit]: Bind: Ident 18 [73-80] "generated_ident_18"
                                    Expr _id_ [73-80] [Type Qubit]: Call:
                                        Expr _id_ [73-80] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [73-80] [Type Unit]: Unit
                                Stmt _id_ [55-82]: Local (Immutable):
                                    Pat 7 [59-60] [Type (Qubit, Qubit)]: Bind: Ident 8 [59-60] "q"
                                    Expr _id_ [63-81] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [64-71] [Type Qubit]: Var: Local 17
                                        Expr _id_ [73-80] [Type Qubit]: Var: Local 18
                                Stmt 12 [91-101]: Local (Immutable):
                                    Pat 13 [95-96] [Type Int]: Bind: Ident 14 [95-96] "x"
                                    Expr 15 [99-100] [Type Int]: Lit: Int(3)
                                Stmt _id_ [73-80]: Semi: Expr _id_ [73-80] [Type Unit]: Call:
                                    Expr _id_ [73-80] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [73-80] [Type Qubit]: Var: Local 18
                                Stmt _id_ [64-71]: Semi: Expr _id_ [64-71] [Type Unit]: Call:
                                    Expr _id_ [64-71] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [64-71] [Type Qubit]: Var: Local 17
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 20 [10-15] "input"): Item 1
                Item 1 [22-113] (Public):
                    Parent: 0
                    Callable 0 [22-113] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-113] (Body): Impl:
                            Pat 4 [22-113] [Type Unit]: Elided
                            Block 5 [45-113] [Type Unit]:
                                Stmt _id_ [69-76]: Local (Immutable):
                                    Pat _id_ [69-76] [Type Qubit]: Bind: Ident 21 [69-76] "generated_ident_21"
                                    Expr _id_ [69-76] [Type Qubit]: Call:
                                        Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [69-76] [Type Unit]: Unit
                                Stmt _id_ [78-86]: Local (Immutable):
                                    Pat _id_ [78-86] [Type (Qubit)[]]: Bind: Ident 22 [78-86] "generated_ident_22"
                                    Expr _id_ [78-86] [Type Qubit]: Call:
                                        Expr _id_ [78-86] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 15 [84-85] [Type Int]: Lit: Int(3)
                                Stmt _id_ [55-88]: Local (Immutable):
                                    Pat 7 [59-65] [Type (Qubit, (Qubit)[])]: Tuple:
                                        Pat 8 [60-61] [Type Qubit]: Bind: Ident 9 [60-61] "a"
                                        Pat 10 [63-64] [Type (Qubit)[]]: Bind: Ident 11 [63-64] "b"
                                    Expr _id_ [68-87] [Type (Qubit, (Qubit)[])]: Tuple:
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 21
                                        Expr _id_ [78-86] [Type (Qubit)[]]: Var: Local 22
                                Stmt 16 [97-107]: Local (Immutable):
                                    Pat 17 [101-102] [Type Int]: Bind: Ident 18 [101-102] "x"
                                    Expr 19 [105-106] [Type Int]: Lit: Int(3)
                                Stmt _id_ [78-86]: Semi: Expr _id_ [78-86] [Type Unit]: Call:
                                    Expr _id_ [78-86] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr _id_ [78-86] [Type (Qubit)[]]: Var: Local 22
                                Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                    Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 21
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 38 [10-15] "input"): Item 1, Item 2
                Item 1 [22-112] (Public):
                    Parent: 0
                    Callable 0 [22-112] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-112] (Body): Impl:
                            Pat 4 [22-112] [Type Unit]: Elided
                            Block 5 [45-112] [Type Unit]:
                                Stmt _id_ [69-76]: Local (Immutable):
                                    Pat _id_ [69-76] [Type Qubit]: Bind: Ident 39 [69-76] "generated_ident_39"
                                    Expr _id_ [69-76] [Type Qubit]: Call:
                                        Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [69-76] [Type Unit]: Unit
                                Stmt _id_ [78-85]: Local (Immutable):
                                    Pat _id_ [78-85] [Type Qubit]: Bind: Ident 40 [78-85] "generated_ident_40"
                                    Expr _id_ [78-85] [Type Qubit]: Call:
                                        Expr _id_ [78-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [78-85] [Type Unit]: Unit
                                Stmt _id_ [55-87]: Local (Immutable):
                                    Pat 7 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 8 [60-61] [Type Qubit]: Bind: Ident 9 [60-61] "a"
                                        Pat 10 [63-64] [Type Qubit]: Bind: Ident 11 [63-64] "b"
                                    Expr _id_ [68-86] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 39
                                        Expr _id_ [78-85] [Type Qubit]: Var: Local 40
                                Stmt 15 [96-106]: Local (Immutable):
                                    Pat 16 [100-101] [Type Int]: Bind: Ident 17 [100-101] "x"
                                    Expr 18 [104-105] [Type Int]: Lit: Int(3)
                                Stmt _id_ [78-85]: Semi: Expr _id_ [78-85] [Type Unit]: Call:
                                    Expr _id_ [78-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [78-85] [Type Qubit]: Var: Local 40
                                Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                    Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 39
                        <no adj>
                        <no ctl>
                        <no ctl-adj>
                Item 2 [122-212] (Public):
                    Parent: 0
                    Callable 19 [122-212] (Operation):
                        name: Ident 20 [132-135] "Bar"
                        input: Pat 21 [135-137] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 22 [122-212] (Body): Impl:
                            Pat 23 [122-212] [Type Unit]: Elided
                            Block 24 [145-212] [Type Unit]:
                                Stmt _id_ [169-176]: Local (Immutable):
                                    Pat _id_ [169-176] [Type Qubit]: Bind: Ident 41 [169-176] "generated_ident_41"
                                    Expr _id_ [169-176] [Type Qubit]: Call:
                                        Expr _id_ [169-176] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [169-176] [Type Unit]: Unit
                                Stmt _id_ [178-185]: Local (Immutable):
                                    Pat _id_ [178-185] [Type Qubit]: Bind: Ident 42 [178-185] "generated_ident_42"
                                    Expr _id_ [178-185] [Type Qubit]: Call:
                                        Expr _id_ [178-185] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [178-185] [Type Unit]: Unit
                                Stmt _id_ [155-187]: Local (Immutable):
                                    Pat 26 [159-165] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 27 [160-161] [Type Qubit]: Bind: Ident 28 [160-161] "c"
                                        Pat 29 [163-164] [Type Qubit]: Bind: Ident 30 [163-164] "d"
                                    Expr _id_ [168-186] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [169-176] [Type Qubit]: Var: Local 41
                                        Expr _id_ [178-185] [Type Qubit]: Var: Local 42
                                Stmt 34 [196-206]: Local (Immutable):
                                    Pat 35 [200-201] [Type Int]: Bind: Ident 36 [200-201] "x"
                                    Expr 37 [204-205] [Type Int]: Lit: Int(3)
                                Stmt _id_ [178-185]: Semi: Expr _id_ [178-185] [Type Unit]: Call:
                                    Expr _id_ [178-185] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [178-185] [Type Qubit]: Var: Local 42
                                Stmt _id_ [169-176]: Semi: Expr _id_ [169-176] [Type Unit]: Call:
                                    Expr _id_ [169-176] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [169-176] [Type Qubit]: Var: Local 41
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 32 [10-15] "input"): Item 1
                Item 1 [22-198] (Public):
                    Parent: 0
                    Callable 0 [22-198] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-198] (Body): Impl:
                            Pat 4 [22-198] [Type Unit]: Elided
                            Block 5 [45-198] [Type Unit]:
                                Stmt _id_ [55-173]: Expr: Expr _id_ [55-173] [Type Unit]: Expr Block: Block 15 [87-173] [Type Unit]:
                                    Stmt _id_ [69-76]: Local (Immutable):
                                        Pat _id_ [69-76] [Type Qubit]: Bind: Ident 33 [69-76] "generated_ident_33"
                                        Expr _id_ [69-76] [Type Qubit]: Call:
                                            Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [69-76] [Type Unit]: Unit
                                    Stmt _id_ [78-85]: Local (Immutable):
                                        Pat _id_ [78-85] [Type Qubit]: Bind: Ident 34 [78-85] "generated_ident_34"
                                        Expr _id_ [78-85] [Type Qubit]: Call:
                                            Expr _id_ [78-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [78-85] [Type Unit]: Unit
                                    Stmt _id_ [55-173]: Local (Immutable):
                                        Pat 7 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                            Pat 8 [60-61] [Type Qubit]: Bind: Ident 9 [60-61] "a"
                                            Pat 10 [63-64] [Type Qubit]: Bind: Ident 11 [63-64] "b"
                                        Expr _id_ [68-86] [Type (Qubit, Qubit)]: Tuple:
                                            Expr _id_ [69-76] [Type Qubit]: Var: Local 33
                                            Expr _id_ [78-85] [Type Qubit]: Var: Local 34
                                    Stmt 16 [101-111]: Local (Immutable):
                                        Pat 17 [105-106] [Type Int]: Bind: Ident 18 [105-106] "x"
                                        Expr 19 [109-110] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [128-129]: Local (Immutable):
                                        Pat _id_ [128-129] [Type Qubit]: Bind: Ident 22 [128-129] "c"
                                        Expr _id_ [128-129] [Type Qubit]: Call:
                                            Expr _id_ [128-129] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [128-129] [Type Unit]: Unit
                                    Stmt 24 [153-163]: Local (Immutable):
                                        Pat 25 [157-158] [Type Int]: Bind: Ident 26 [157-158] "y"
                                        Expr 27 [161-162] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [128-129]: Semi: Expr _id_ [128-129] [Type Unit]: Call:
                                        Expr _id_ [128-129] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [128-129] [Type Qubit]: Var: Local 22
                                    Stmt _id_ [78-85]: Semi: Expr _id_ [78-85] [Type Unit]: Call:
                                        Expr _id_ [78-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [78-85] [Type Qubit]: Var: Local 34
                                    Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                        Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 33
                                Stmt 28 [182-192]: Local (Immutable):
                                    Pat 29 [186-187] [Type Int]: Bind: Ident 30 [186-187] "z"
                                    Expr 31 [190-191] [Type Int]: Lit: Int(3)
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 23 [10-15] "input"): Item 1
                Item 1 [22-155] (Public):
                    Parent: 0
                    Callable 0 [22-155] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-155] (Body): Impl:
                            Pat 4 [22-155] [Type Unit]: Elided
                            Block 5 [45-155] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 8 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt _id_ [80-130]: Expr: Expr _id_ [80-130] [Type Unit]: Expr Block: Block 14 [96-130] [Type Unit]:
                                    Stmt _id_ [84-85]: Local (Immutable):
                                        Pat _id_ [84-85] [Type Qubit]: Bind: Ident 12 [84-85] "b"
                                        Expr _id_ [84-85] [Type Qubit]: Call:
                                            Expr _id_ [84-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [84-85] [Type Unit]: Unit
                                    Stmt 15 [110-120]: Local (Immutable):
                                        Pat 16 [114-115] [Type Int]: Bind: Ident 17 [114-115] "x"
                                        Expr 18 [118-119] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [84-85]: Semi: Expr _id_ [84-85] [Type Unit]: Call:
                                        Expr _id_ [84-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [84-85] [Type Qubit]: Var: Local 12
                                Stmt 19 [139-149]: Local (Immutable):
                                    Pat 20 [143-144] [Type Int]: Bind: Ident 21 [143-144] "y"
                                    Expr 22 [147-148] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 8
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 56 [10-15] "input"): Item 1
                Item 1 [22-351] (Public):
                    Parent: 0
                    Callable 0 [22-351] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-351] (Body): Impl:
                            Pat 4 [22-351] [Type Unit]: Elided
                            Block 5 [45-351] [Type Unit]:
                                Stmt 6 [55-66]: Local (Immutable):
                                    Pat 7 [59-61] [Type Int]: Bind: Ident 8 [59-61] "x1"
                                    Expr 9 [64-65] [Type Int]: Lit: Int(3)
                                Stmt _id_ [79-80]: Local (Immutable):
                                    Pat _id_ [79-80] [Type Qubit]: Bind: Ident 12 [79-80] "a"
                                    Expr _id_ [79-80] [Type Qubit]: Call:
                                        Expr _id_ [79-80] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [79-80] [Type Unit]: Unit
                                Stmt 14 [100-111]: Local (Immutable):
                                    Pat 15 [104-106] [Type Int]: Bind: Ident 16 [104-106] "x2"
                                    Expr 17 [109-110] [Type Int]: Lit: Int(3)
                                Stmt 18 [120-208]: Expr: Expr 19 [120-208] [Type Unit]: Expr Block: Block 20 [120-208] [Type Unit]:
                                    Stmt 21 [134-145]: Local (Immutable):
                                        Pat 22 [138-140] [Type Int]: Bind: Ident 23 [138-140] "y1"
                                        Expr 24 [143-144] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [162-163]: Local (Immutable):
                                        Pat _id_ [162-163] [Type Qubit]: Bind: Ident 27 [162-163] "b"
                                        Expr _id_ [162-163] [Type Qubit]: Call:
                                            Expr _id_ [162-163] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [162-163] [Type Unit]: Unit
                                    Stmt 29 [187-198]: Local (Immutable):
                                        Pat 30 [191-193] [Type Int]: Bind: Ident 31 [191-193] "y2"
                                        Expr 32 [196-197] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [162-163]: Semi: Expr _id_ [162-163] [Type Unit]: Call:
                                        Expr _id_ [162-163] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [162-163] [Type Qubit]: Var: Local 27
                                Stmt 33 [217-228]: Local (Immutable):
                                    Pat 34 [221-223] [Type Int]: Bind: Ident 35 [221-223] "x3"
                                    Expr 36 [226-227] [Type Int]: Lit: Int(3)
                                Stmt 37 [237-325]: Expr: Expr 38 [237-325] [Type Unit]: Expr Block: Block 39 [237-325] [Type Unit]:
                                    Stmt 40 [251-262]: Local (Immutable):
                                        Pat 41 [255-257] [Type Int]: Bind: Ident 42 [255-257] "z1"
                                        Expr 43 [260-261] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [279-280]: Local (Immutable):
                                        Pat _id_ [279-280] [Type Qubit]: Bind: Ident 46 [279-280] "c"
                                        Expr _id_ [279-280] [Type Qubit]: Call:
                                            Expr _id_ [279-280] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [279-280] [Type Unit]: Unit
                                    Stmt 48 [304-315]: Local (Immutable):
                                        Pat 49 [308-310] [Type Int]: Bind: Ident 50 [308-310] "z2"
                                        Expr 51 [313-314] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [279-280]: Semi: Expr _id_ [279-280] [Type Unit]: Call:
                                        Expr _id_ [279-280] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [279-280] [Type Qubit]: Var: Local 46
                                Stmt 52 [334-345]: Local (Immutable):
                                    Pat 53 [338-340] [Type Int]: Bind: Ident 54 [338-340] "x4"
                                    Expr 55 [343-344] [Type Int]: Lit: Int(3)
                                Stmt _id_ [79-80]: Semi: Expr _id_ [79-80] [Type Unit]: Call:
                                    Expr _id_ [79-80] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [79-80] [Type Qubit]: Var: Local 12
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 32 [10-15] "input"): Item 1
                Item 1 [22-239] (Public):
                    Parent: 0
                    Callable 0 [22-239] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-239] (Body): Impl:
                            Pat 4 [22-239] [Type Unit]: Elided
                            Block 5 [45-239] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 8 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt 10 [80-151]: Expr: Expr 11 [80-151] [Type Unit]: If:
                                    Expr 12 [83-87] [Type Bool]: Lit: Bool(true)
                                    Block 13 [88-151] [Type Unit]:
                                        Stmt _id_ [106-107]: Local (Immutable):
                                            Pat _id_ [106-107] [Type Qubit]: Bind: Ident 16 [106-107] "b"
                                            Expr _id_ [106-107] [Type Qubit]: Call:
                                                Expr _id_ [106-107] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                Expr _id_ [106-107] [Type Unit]: Unit
                                        Stmt 18 [131-141]: Semi: Expr _id_ [131-140] [Type ?2]: Expr Block: Block _id_ [131-140] [Type ?2]:
                                            Stmt _id_ [138-140]: Local (Immutable):
                                                Pat _id_ [138-140] [Type Unit]: Bind: Ident 33 [138-140] "generated_ident_33"
                                                Expr 20 [138-140] [Type Unit]: Unit
                                            Stmt _id_ [106-107]: Semi: Expr _id_ [106-107] [Type Unit]: Call:
                                                Expr _id_ [106-107] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [106-107] [Type Qubit]: Var: Local 16
                                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [59-60] [Type Qubit]: Var: Local 8
                                            Stmt _id_ [131-140]: Semi: Expr _id_ [131-140] [Type ?2]: Return: Expr _id_ [138-140] [Type Unit]: Var: Local 33
                                        Stmt _id_ [106-107]: Semi: Expr _id_ [106-107] [Type Unit]: Call:
                                            Expr _id_ [106-107] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                            Expr _id_ [106-107] [Type Qubit]: Var: Local 16
                                Stmt _id_ [161-233]: Local (Immutable):
                                    Pat _id_ [161-233] [Type Unit]: Bind: Ident 35 [161-233] "generated_ident_35"
                                    Expr 22 [161-233] [Type Unit]: If:
                                        Expr 23 [164-169] [Type Bool]: Lit: Bool(false)
                                        Block 24 [170-233] [Type Unit]:
                                            Stmt _id_ [188-189]: Local (Immutable):
                                                Pat _id_ [188-189] [Type Qubit]: Bind: Ident 27 [188-189] "c"
                                                Expr _id_ [188-189] [Type Qubit]: Call:
                                                    Expr _id_ [188-189] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr _id_ [188-189] [Type Unit]: Unit
                                            Stmt 29 [213-223]: Semi: Expr _id_ [213-222] [Type ?5]: Expr Block: Block _id_ [213-222] [Type ?5]:
                                                Stmt _id_ [220-222]: Local (Immutable):
                                                    Pat _id_ [220-222] [Type Unit]: Bind: Ident 34 [220-222] "generated_ident_34"
                                                    Expr 31 [220-222] [Type Unit]: Unit
                                                Stmt _id_ [188-189]: Semi: Expr _id_ [188-189] [Type Unit]: Call:
                                                    Expr _id_ [188-189] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                    Expr _id_ [188-189] [Type Qubit]: Var: Local 27
                                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 8
                                                Stmt _id_ [213-222]: Semi: Expr _id_ [213-222] [Type ?5]: Return: Expr _id_ [220-222] [Type Unit]: Var: Local 34
                                            Stmt _id_ [188-189]: Semi: Expr _id_ [188-189] [Type Unit]: Call:
                                                Expr _id_ [188-189] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [188-189] [Type Qubit]: Var: Local 27
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 8
                                Stmt _id_ [161-233]: Expr: Expr _id_ [161-233] [Type Unit]: Var: Local 35
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 28 [10-15] "input"): Item 1
                Item 1 [22-170] (Public):
                    Parent: 0
                    Callable 0 [22-170] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-170] (Body): Impl:
                            Pat 4 [22-170] [Type Unit]: Elided
                            Block 5 [45-170] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 8 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt 10 [80-92]: Local (Immutable):
                                    Pat 11 [84-85] [Type Int]: Bind: Ident 12 [84-85] "x"
                                    Expr 13 [88-91] [Type Int]: Expr Block: Block 14 [88-91] [Type Int]:
                                        Stmt 15 [89-90]: Expr: Expr 16 [89-90] [Type Int]: Lit: Int(3)
                                Stmt 17 [101-164]: Local (Immutable):
                                    Pat 18 [105-106] [Type Int]: Bind: Ident 19 [105-106] "y"
                                    Expr 20 [109-163] [Type Int]: Expr Block: Block 21 [109-163] [Type Int]:
                                        Stmt _id_ [127-128]: Local (Immutable):
                                            Pat _id_ [127-128] [Type Qubit]: Bind: Ident 24 [127-128] "b"
                                            Expr _id_ [127-128] [Type Qubit]: Call:
                                                Expr _id_ [127-128] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                Expr _id_ [127-128] [Type Unit]: Unit
                                        Stmt _id_ [152-153]: Local (Immutable):
                                            Pat _id_ [152-153] [Type Int]: Bind: Ident 29 [152-153] "generated_ident_29"
                                            Expr 27 [152-153] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [127-128]: Semi: Expr _id_ [127-128] [Type Unit]: Call:
                                            Expr _id_ [127-128] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                            Expr _id_ [127-128] [Type Qubit]: Var: Local 24
                                        Stmt _id_ [152-153]: Expr: Expr _id_ [152-153] [Type Int]: Var: Local 29
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 8
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 22 [10-15] "input"): Item 1
                Item 1 [22-150] (Public):
                    Parent: 0
                    Callable 0 [22-150] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-150] (Body): Impl:
                            Pat 4 [22-150] [Type Unit]: Elided
                            Block 5 [45-150] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type (Qubit)[]]: Bind: Ident 8 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 10 [69-123] [Type Int]: Expr Block: Block 11 [69-123] [Type Int]:
                                            Stmt _id_ [87-88]: Local (Immutable):
                                                Pat _id_ [87-88] [Type Qubit]: Bind: Ident 14 [87-88] "b"
                                                Expr _id_ [87-88] [Type Qubit]: Call:
                                                    Expr _id_ [87-88] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr _id_ [87-88] [Type Unit]: Unit
                                            Stmt _id_ [112-113]: Local (Immutable):
                                                Pat _id_ [112-113] [Type Int]: Bind: Ident 23 [112-113] "generated_ident_23"
                                                Expr 17 [112-113] [Type Int]: Lit: Int(3)
                                            Stmt _id_ [87-88]: Semi: Expr _id_ [87-88] [Type Unit]: Call:
                                                Expr _id_ [87-88] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [87-88] [Type Qubit]: Var: Local 14
                                            Stmt _id_ [112-113]: Expr: Expr _id_ [112-113] [Type Int]: Var: Local 23
                                Stmt 18 [134-144]: Local (Immutable):
                                    Pat 19 [138-139] [Type Int]: Bind: Ident 20 [138-139] "x"
                                    Expr 21 [142-143] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr _id_ [59-60] [Type (Qubit)[]]: Var: Local 8
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 20 [10-15] "input"): Item 1
                Item 1 [22-147] (Public):
                    Parent: 0
                    Callable 0 [22-147] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [22-147] (Body): Impl:
                            Pat 4 [22-147] [Type Unit]: Elided
                            Block 5 [44-147] [Type Int]:
                                Stmt _id_ [58-59]: Local (Immutable):
                                    Pat _id_ [58-59] [Type Qubit]: Bind: Ident 8 [58-59] "a"
                                    Expr _id_ [58-59] [Type Qubit]: Call:
                                        Expr _id_ [58-59] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [58-59] [Type Unit]: Unit
                                Stmt 10 [79-141]: Semi: Expr _id_ [79-140] [Type ?2]: Expr Block: Block _id_ [79-140] [Type ?2]:
                                    Stmt _id_ [86-140]: Local (Immutable):
                                        Pat _id_ [86-140] [Type Int]: Bind: Ident 21 [86-140] "generated_ident_21"
                                        Expr 12 [86-140] [Type Int]: Expr Block: Block 13 [86-140] [Type Int]:
                                            Stmt _id_ [104-105]: Local (Immutable):
                                                Pat _id_ [104-105] [Type Qubit]: Bind: Ident 16 [104-105] "b"
                                                Expr _id_ [104-105] [Type Qubit]: Call:
                                                    Expr _id_ [104-105] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr _id_ [104-105] [Type Unit]: Unit
                                            Stmt _id_ [129-130]: Local (Immutable):
                                                Pat _id_ [129-130] [Type Int]: Bind: Ident 22 [129-130] "generated_ident_22"
                                                Expr 19 [129-130] [Type Int]: Lit: Int(3)
                                            Stmt _id_ [104-105]: Semi: Expr _id_ [104-105] [Type Unit]: Call:
                                                Expr _id_ [104-105] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [104-105] [Type Qubit]: Var: Local 16
                                            Stmt _id_ [129-130]: Expr: Expr _id_ [129-130] [Type Int]: Var: Local 22
                                    Stmt _id_ [58-59]: Semi: Expr _id_ [58-59] [Type Unit]: Call:
                                        Expr _id_ [58-59] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [58-59] [Type Qubit]: Var: Local 8
                                    Stmt _id_ [79-140]: Semi: Expr _id_ [79-140] [Type ?2]: Return: Expr _id_ [86-140] [Type Int]: Var: Local 21
                                Stmt _id_ [58-59]: Semi: Expr _id_ [58-59] [Type Unit]: Call:
                                    Expr _id_ [58-59] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [58-59] [Type Qubit]: Var: Local 8
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
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
                    Namespace (Ident 22 [10-15] "input"): Item 1
                Item 1 [22-159] (Public):
                    Parent: 0
                    Callable 0 [22-159] (Operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [22-159] (Body): Impl:
                            Pat 4 [22-159] [Type Unit]: Elided
                            Block 5 [44-159] [Type Int]:
                                Stmt 6 [54-95]: Expr: Expr 7 [54-95] [Type Unit]: If:
                                    Expr 8 [57-61] [Type Bool]: Lit: Bool(true)
                                    Block 9 [62-95] [Type Unit]:
                                        Stmt 10 [76-85]: Semi: Expr 11 [76-84] [Type ?0]: Return: Expr 12 [83-84] [Type Int]: Lit: Int(3)
                                Stmt 13 [105-153]: Expr: Expr 14 [105-153] [Type Int]: Expr Block: Block 15 [105-153] [Type Int]:
                                    Stmt 16 [119-129]: Local (Immutable):
                                        Pat 17 [123-124] [Type Int]: Bind: Ident 18 [123-124] "x"
                                        Expr 19 [127-128] [Type Int]: Lit: Int(4)
                                    Stmt 20 [142-143]: Expr: Expr 21 [142-143] [Type Int]: Var: Local 18
                        <no adj>
                        <no ctl>
                        <no ctl-adj>"#]],
    );
}
