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
                    Namespace (Ident 13 [10-15] "input"): Item 1
                Item 1 [22-96] (Public):
                    Parent: 0
                    Callable 0 [22-96] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-96]: Impl:
                            Block 4 [45-96] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "q"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt 9 [80-90]: Local (Immutable):
                                    Pat 10 [84-85] [Type Int]: Bind: Ident 11 [84-85] "x"
                                    Expr 12 [88-89] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 7
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 14 [10-15] "input"): Item 1
                Item 1 [22-97] (Public):
                    Parent: 0
                    Callable 0 [22-97] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-97]: Impl:
                            Block 4 [45-97] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type (Qubit)[]]: Bind: Ident 7 [59-60] "q"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 9 [69-70] [Type Int]: Lit: Int(3)
                                Stmt 10 [81-91]: Local (Immutable):
                                    Pat 11 [85-86] [Type Int]: Bind: Ident 12 [85-86] "x"
                                    Expr 13 [89-90] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr _id_ [59-60] [Type (Qubit)[]]: Var: Local 7
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 15 [10-15] "input"): Item 1
                Item 1 [22-107] (Public):
                    Parent: 0
                    Callable 0 [22-107] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-107]: Impl:
                            Block 4 [45-107] [Type Unit]:
                                Stmt _id_ [64-71]: Local (Immutable):
                                    Pat _id_ [64-71] [Type Qubit]: Bind: Ident 16 [64-71] "generated_ident_16"
                                    Expr _id_ [64-71] [Type Qubit]: Call:
                                        Expr _id_ [64-71] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [64-71] [Type Unit]: Unit
                                Stmt _id_ [73-80]: Local (Immutable):
                                    Pat _id_ [73-80] [Type Qubit]: Bind: Ident 17 [73-80] "generated_ident_17"
                                    Expr _id_ [73-80] [Type Qubit]: Call:
                                        Expr _id_ [73-80] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [73-80] [Type Unit]: Unit
                                Stmt _id_ [55-82]: Local (Immutable):
                                    Pat 6 [59-60] [Type (Qubit, Qubit)]: Bind: Ident 7 [59-60] "q"
                                    Expr _id_ [63-81] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [64-71] [Type Qubit]: Var: Local 16
                                        Expr _id_ [73-80] [Type Qubit]: Var: Local 17
                                Stmt 11 [91-101]: Local (Immutable):
                                    Pat 12 [95-96] [Type Int]: Bind: Ident 13 [95-96] "x"
                                    Expr 14 [99-100] [Type Int]: Lit: Int(3)
                                Stmt _id_ [73-80]: Semi: Expr _id_ [73-80] [Type Unit]: Call:
                                    Expr _id_ [73-80] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [73-80] [Type Qubit]: Var: Local 17
                                Stmt _id_ [64-71]: Semi: Expr _id_ [64-71] [Type Unit]: Call:
                                    Expr _id_ [64-71] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [64-71] [Type Qubit]: Var: Local 16
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 19 [10-15] "input"): Item 1
                Item 1 [22-113] (Public):
                    Parent: 0
                    Callable 0 [22-113] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-113]: Impl:
                            Block 4 [45-113] [Type Unit]:
                                Stmt _id_ [69-76]: Local (Immutable):
                                    Pat _id_ [69-76] [Type Qubit]: Bind: Ident 20 [69-76] "generated_ident_20"
                                    Expr _id_ [69-76] [Type Qubit]: Call:
                                        Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [69-76] [Type Unit]: Unit
                                Stmt _id_ [78-86]: Local (Immutable):
                                    Pat _id_ [78-86] [Type (Qubit)[]]: Bind: Ident 21 [78-86] "generated_ident_21"
                                    Expr _id_ [78-86] [Type Qubit]: Call:
                                        Expr _id_ [78-86] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 14 [84-85] [Type Int]: Lit: Int(3)
                                Stmt _id_ [55-88]: Local (Immutable):
                                    Pat 6 [59-65] [Type (Qubit, (Qubit)[])]: Tuple:
                                        Pat 7 [60-61] [Type Qubit]: Bind: Ident 8 [60-61] "a"
                                        Pat 9 [63-64] [Type (Qubit)[]]: Bind: Ident 10 [63-64] "b"
                                    Expr _id_ [68-87] [Type (Qubit, (Qubit)[])]: Tuple:
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 20
                                        Expr _id_ [78-86] [Type (Qubit)[]]: Var: Local 21
                                Stmt 15 [97-107]: Local (Immutable):
                                    Pat 16 [101-102] [Type Int]: Bind: Ident 17 [101-102] "x"
                                    Expr 18 [105-106] [Type Int]: Lit: Int(3)
                                Stmt _id_ [78-86]: Semi: Expr _id_ [78-86] [Type Unit]: Call:
                                    Expr _id_ [78-86] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr _id_ [78-86] [Type (Qubit)[]]: Var: Local 21
                                Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                    Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 20
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 36 [10-15] "input"): Item 1, Item 2
                Item 1 [22-112] (Public):
                    Parent: 0
                    Callable 0 [22-112] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-112]: Impl:
                            Block 4 [45-112] [Type Unit]:
                                Stmt _id_ [69-76]: Local (Immutable):
                                    Pat _id_ [69-76] [Type Qubit]: Bind: Ident 37 [69-76] "generated_ident_37"
                                    Expr _id_ [69-76] [Type Qubit]: Call:
                                        Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [69-76] [Type Unit]: Unit
                                Stmt _id_ [78-85]: Local (Immutable):
                                    Pat _id_ [78-85] [Type Qubit]: Bind: Ident 38 [78-85] "generated_ident_38"
                                    Expr _id_ [78-85] [Type Qubit]: Call:
                                        Expr _id_ [78-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [78-85] [Type Unit]: Unit
                                Stmt _id_ [55-87]: Local (Immutable):
                                    Pat 6 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 7 [60-61] [Type Qubit]: Bind: Ident 8 [60-61] "a"
                                        Pat 9 [63-64] [Type Qubit]: Bind: Ident 10 [63-64] "b"
                                    Expr _id_ [68-86] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 37
                                        Expr _id_ [78-85] [Type Qubit]: Var: Local 38
                                Stmt 14 [96-106]: Local (Immutable):
                                    Pat 15 [100-101] [Type Int]: Bind: Ident 16 [100-101] "x"
                                    Expr 17 [104-105] [Type Int]: Lit: Int(3)
                                Stmt _id_ [78-85]: Semi: Expr _id_ [78-85] [Type Unit]: Call:
                                    Expr _id_ [78-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [78-85] [Type Qubit]: Var: Local 38
                                Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                    Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [69-76] [Type Qubit]: Var: Local 37
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [122-212] (Public):
                    Parent: 0
                    Callable 18 [122-212] (operation):
                        name: Ident 19 [132-135] "Bar"
                        input: Pat 20 [135-137] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 21 [122-212]: Impl:
                            Block 22 [145-212] [Type Unit]:
                                Stmt _id_ [169-176]: Local (Immutable):
                                    Pat _id_ [169-176] [Type Qubit]: Bind: Ident 39 [169-176] "generated_ident_39"
                                    Expr _id_ [169-176] [Type Qubit]: Call:
                                        Expr _id_ [169-176] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [169-176] [Type Unit]: Unit
                                Stmt _id_ [178-185]: Local (Immutable):
                                    Pat _id_ [178-185] [Type Qubit]: Bind: Ident 40 [178-185] "generated_ident_40"
                                    Expr _id_ [178-185] [Type Qubit]: Call:
                                        Expr _id_ [178-185] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [178-185] [Type Unit]: Unit
                                Stmt _id_ [155-187]: Local (Immutable):
                                    Pat 24 [159-165] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 25 [160-161] [Type Qubit]: Bind: Ident 26 [160-161] "c"
                                        Pat 27 [163-164] [Type Qubit]: Bind: Ident 28 [163-164] "d"
                                    Expr _id_ [168-186] [Type (Qubit, Qubit)]: Tuple:
                                        Expr _id_ [169-176] [Type Qubit]: Var: Local 39
                                        Expr _id_ [178-185] [Type Qubit]: Var: Local 40
                                Stmt 32 [196-206]: Local (Immutable):
                                    Pat 33 [200-201] [Type Int]: Bind: Ident 34 [200-201] "x"
                                    Expr 35 [204-205] [Type Int]: Lit: Int(3)
                                Stmt _id_ [178-185]: Semi: Expr _id_ [178-185] [Type Unit]: Call:
                                    Expr _id_ [178-185] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [178-185] [Type Qubit]: Var: Local 40
                                Stmt _id_ [169-176]: Semi: Expr _id_ [169-176] [Type Unit]: Call:
                                    Expr _id_ [169-176] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [169-176] [Type Qubit]: Var: Local 39
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 31 [10-15] "input"): Item 1
                Item 1 [22-198] (Public):
                    Parent: 0
                    Callable 0 [22-198] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-198]: Impl:
                            Block 4 [45-198] [Type Unit]:
                                Stmt _id_ [55-173]: Expr: Expr _id_ [55-173] [Type Unit]: Expr Block: Block 14 [87-173] [Type Unit]:
                                    Stmt _id_ [69-76]: Local (Immutable):
                                        Pat _id_ [69-76] [Type Qubit]: Bind: Ident 32 [69-76] "generated_ident_32"
                                        Expr _id_ [69-76] [Type Qubit]: Call:
                                            Expr _id_ [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [69-76] [Type Unit]: Unit
                                    Stmt _id_ [78-85]: Local (Immutable):
                                        Pat _id_ [78-85] [Type Qubit]: Bind: Ident 33 [78-85] "generated_ident_33"
                                        Expr _id_ [78-85] [Type Qubit]: Call:
                                            Expr _id_ [78-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [78-85] [Type Unit]: Unit
                                    Stmt _id_ [55-173]: Local (Immutable):
                                        Pat 6 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                            Pat 7 [60-61] [Type Qubit]: Bind: Ident 8 [60-61] "a"
                                            Pat 9 [63-64] [Type Qubit]: Bind: Ident 10 [63-64] "b"
                                        Expr _id_ [68-86] [Type (Qubit, Qubit)]: Tuple:
                                            Expr _id_ [69-76] [Type Qubit]: Var: Local 32
                                            Expr _id_ [78-85] [Type Qubit]: Var: Local 33
                                    Stmt 15 [101-111]: Local (Immutable):
                                        Pat 16 [105-106] [Type Int]: Bind: Ident 17 [105-106] "x"
                                        Expr 18 [109-110] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [128-129]: Local (Immutable):
                                        Pat _id_ [128-129] [Type Qubit]: Bind: Ident 21 [128-129] "c"
                                        Expr _id_ [128-129] [Type Qubit]: Call:
                                            Expr _id_ [128-129] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [128-129] [Type Unit]: Unit
                                    Stmt 23 [153-163]: Local (Immutable):
                                        Pat 24 [157-158] [Type Int]: Bind: Ident 25 [157-158] "y"
                                        Expr 26 [161-162] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [128-129]: Semi: Expr _id_ [128-129] [Type Unit]: Call:
                                        Expr _id_ [128-129] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [128-129] [Type Qubit]: Var: Local 21
                                    Stmt _id_ [78-85]: Semi: Expr _id_ [78-85] [Type Unit]: Call:
                                        Expr _id_ [78-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [78-85] [Type Qubit]: Var: Local 33
                                    Stmt _id_ [69-76]: Semi: Expr _id_ [69-76] [Type Unit]: Call:
                                        Expr _id_ [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [69-76] [Type Qubit]: Var: Local 32
                                Stmt 27 [182-192]: Local (Immutable):
                                    Pat 28 [186-187] [Type Int]: Bind: Ident 29 [186-187] "z"
                                    Expr 30 [190-191] [Type Int]: Lit: Int(3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 22 [10-15] "input"): Item 1
                Item 1 [22-155] (Public):
                    Parent: 0
                    Callable 0 [22-155] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-155]: Impl:
                            Block 4 [45-155] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt _id_ [80-130]: Expr: Expr _id_ [80-130] [Type Unit]: Expr Block: Block 13 [96-130] [Type Unit]:
                                    Stmt _id_ [84-85]: Local (Immutable):
                                        Pat _id_ [84-85] [Type Qubit]: Bind: Ident 11 [84-85] "b"
                                        Expr _id_ [84-85] [Type Qubit]: Call:
                                            Expr _id_ [84-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [84-85] [Type Unit]: Unit
                                    Stmt 14 [110-120]: Local (Immutable):
                                        Pat 15 [114-115] [Type Int]: Bind: Ident 16 [114-115] "x"
                                        Expr 17 [118-119] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [84-85]: Semi: Expr _id_ [84-85] [Type Unit]: Call:
                                        Expr _id_ [84-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [84-85] [Type Qubit]: Var: Local 11
                                Stmt 18 [139-149]: Local (Immutable):
                                    Pat 19 [143-144] [Type Int]: Bind: Ident 20 [143-144] "y"
                                    Expr 21 [147-148] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 7
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 55 [10-15] "input"): Item 1
                Item 1 [22-351] (Public):
                    Parent: 0
                    Callable 0 [22-351] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-351]: Impl:
                            Block 4 [45-351] [Type Unit]:
                                Stmt 5 [55-66]: Local (Immutable):
                                    Pat 6 [59-61] [Type Int]: Bind: Ident 7 [59-61] "x1"
                                    Expr 8 [64-65] [Type Int]: Lit: Int(3)
                                Stmt _id_ [79-80]: Local (Immutable):
                                    Pat _id_ [79-80] [Type Qubit]: Bind: Ident 11 [79-80] "a"
                                    Expr _id_ [79-80] [Type Qubit]: Call:
                                        Expr _id_ [79-80] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [79-80] [Type Unit]: Unit
                                Stmt 13 [100-111]: Local (Immutable):
                                    Pat 14 [104-106] [Type Int]: Bind: Ident 15 [104-106] "x2"
                                    Expr 16 [109-110] [Type Int]: Lit: Int(3)
                                Stmt 17 [120-208]: Expr: Expr 18 [120-208] [Type Unit]: Expr Block: Block 19 [120-208] [Type Unit]:
                                    Stmt 20 [134-145]: Local (Immutable):
                                        Pat 21 [138-140] [Type Int]: Bind: Ident 22 [138-140] "y1"
                                        Expr 23 [143-144] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [162-163]: Local (Immutable):
                                        Pat _id_ [162-163] [Type Qubit]: Bind: Ident 26 [162-163] "b"
                                        Expr _id_ [162-163] [Type Qubit]: Call:
                                            Expr _id_ [162-163] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [162-163] [Type Unit]: Unit
                                    Stmt 28 [187-198]: Local (Immutable):
                                        Pat 29 [191-193] [Type Int]: Bind: Ident 30 [191-193] "y2"
                                        Expr 31 [196-197] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [162-163]: Semi: Expr _id_ [162-163] [Type Unit]: Call:
                                        Expr _id_ [162-163] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [162-163] [Type Qubit]: Var: Local 26
                                Stmt 32 [217-228]: Local (Immutable):
                                    Pat 33 [221-223] [Type Int]: Bind: Ident 34 [221-223] "x3"
                                    Expr 35 [226-227] [Type Int]: Lit: Int(3)
                                Stmt 36 [237-325]: Expr: Expr 37 [237-325] [Type Unit]: Expr Block: Block 38 [237-325] [Type Unit]:
                                    Stmt 39 [251-262]: Local (Immutable):
                                        Pat 40 [255-257] [Type Int]: Bind: Ident 41 [255-257] "z1"
                                        Expr 42 [260-261] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [279-280]: Local (Immutable):
                                        Pat _id_ [279-280] [Type Qubit]: Bind: Ident 45 [279-280] "c"
                                        Expr _id_ [279-280] [Type Qubit]: Call:
                                            Expr _id_ [279-280] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr _id_ [279-280] [Type Unit]: Unit
                                    Stmt 47 [304-315]: Local (Immutable):
                                        Pat 48 [308-310] [Type Int]: Bind: Ident 49 [308-310] "z2"
                                        Expr 50 [313-314] [Type Int]: Lit: Int(3)
                                    Stmt _id_ [279-280]: Semi: Expr _id_ [279-280] [Type Unit]: Call:
                                        Expr _id_ [279-280] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [279-280] [Type Qubit]: Var: Local 45
                                Stmt 51 [334-345]: Local (Immutable):
                                    Pat 52 [338-340] [Type Int]: Bind: Ident 53 [338-340] "x4"
                                    Expr 54 [343-344] [Type Int]: Lit: Int(3)
                                Stmt _id_ [79-80]: Semi: Expr _id_ [79-80] [Type Unit]: Call:
                                    Expr _id_ [79-80] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [79-80] [Type Qubit]: Var: Local 11
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 33 [10-15] "input"): Item 1
                Item 1 [22-239] (Public):
                    Parent: 0
                    Callable 0 [22-239] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-239]: Impl:
                            Block 4 [45-239] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt 9 [80-151]: Expr: Expr 10 [80-151] [Type Unit]: If:
                                    Expr 11 [83-87] [Type Bool]: Lit: Bool(true)
                                    Expr 12 [88-151] [Type Unit]: Expr Block: Block 13 [88-151] [Type Unit]:
                                        Stmt _id_ [106-107]: Local (Immutable):
                                            Pat _id_ [106-107] [Type Qubit]: Bind: Ident 16 [106-107] "b"
                                            Expr _id_ [106-107] [Type Qubit]: Call:
                                                Expr _id_ [106-107] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                Expr _id_ [106-107] [Type Unit]: Unit
                                        Stmt 18 [131-141]: Semi: Expr _id_ [131-140] [Type ?2]: Expr Block: Block _id_ [131-140] [Type ?2]:
                                            Stmt _id_ [138-140]: Local (Immutable):
                                                Pat _id_ [138-140] [Type Unit]: Bind: Ident 34 [138-140] "generated_ident_34"
                                                Expr 20 [138-140] [Type Unit]: Unit
                                            Stmt _id_ [106-107]: Semi: Expr _id_ [106-107] [Type Unit]: Call:
                                                Expr _id_ [106-107] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [106-107] [Type Qubit]: Var: Local 16
                                            Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                                Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [59-60] [Type Qubit]: Var: Local 7
                                            Stmt _id_ [131-140]: Semi: Expr _id_ [131-140] [Type ?2]: Return: Expr _id_ [138-140] [Type Unit]: Var: Local 34
                                        Stmt _id_ [106-107]: Semi: Expr _id_ [106-107] [Type Unit]: Call:
                                            Expr _id_ [106-107] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                            Expr _id_ [106-107] [Type Qubit]: Var: Local 16
                                Stmt _id_ [161-233]: Local (Immutable):
                                    Pat _id_ [161-233] [Type Unit]: Bind: Ident 36 [161-233] "generated_ident_36"
                                    Expr 22 [161-233] [Type Unit]: If:
                                        Expr 23 [164-169] [Type Bool]: Lit: Bool(false)
                                        Expr 24 [170-233] [Type Unit]: Expr Block: Block 25 [170-233] [Type Unit]:
                                            Stmt _id_ [188-189]: Local (Immutable):
                                                Pat _id_ [188-189] [Type Qubit]: Bind: Ident 28 [188-189] "c"
                                                Expr _id_ [188-189] [Type Qubit]: Call:
                                                    Expr _id_ [188-189] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr _id_ [188-189] [Type Unit]: Unit
                                            Stmt 30 [213-223]: Semi: Expr _id_ [213-222] [Type ?5]: Expr Block: Block _id_ [213-222] [Type ?5]:
                                                Stmt _id_ [220-222]: Local (Immutable):
                                                    Pat _id_ [220-222] [Type Unit]: Bind: Ident 35 [220-222] "generated_ident_35"
                                                    Expr 32 [220-222] [Type Unit]: Unit
                                                Stmt _id_ [188-189]: Semi: Expr _id_ [188-189] [Type Unit]: Call:
                                                    Expr _id_ [188-189] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                    Expr _id_ [188-189] [Type Qubit]: Var: Local 28
                                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 7
                                                Stmt _id_ [213-222]: Semi: Expr _id_ [213-222] [Type ?5]: Return: Expr _id_ [220-222] [Type Unit]: Var: Local 35
                                            Stmt _id_ [188-189]: Semi: Expr _id_ [188-189] [Type Unit]: Call:
                                                Expr _id_ [188-189] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [188-189] [Type Qubit]: Var: Local 28
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 7
                                Stmt _id_ [161-233]: Expr: Expr _id_ [161-233] [Type Unit]: Var: Local 36
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 27 [10-15] "input"): Item 1
                Item 1 [22-170] (Public):
                    Parent: 0
                    Callable 0 [22-170] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-170]: Impl:
                            Block 4 [45-170] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [59-60] [Type Unit]: Unit
                                Stmt 9 [80-92]: Local (Immutable):
                                    Pat 10 [84-85] [Type Int]: Bind: Ident 11 [84-85] "x"
                                    Expr 12 [88-91] [Type Int]: Expr Block: Block 13 [88-91] [Type Int]:
                                        Stmt 14 [89-90]: Expr: Expr 15 [89-90] [Type Int]: Lit: Int(3)
                                Stmt 16 [101-164]: Local (Immutable):
                                    Pat 17 [105-106] [Type Int]: Bind: Ident 18 [105-106] "y"
                                    Expr 19 [109-163] [Type Int]: Expr Block: Block 20 [109-163] [Type Int]:
                                        Stmt _id_ [127-128]: Local (Immutable):
                                            Pat _id_ [127-128] [Type Qubit]: Bind: Ident 23 [127-128] "b"
                                            Expr _id_ [127-128] [Type Qubit]: Call:
                                                Expr _id_ [127-128] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                Expr _id_ [127-128] [Type Unit]: Unit
                                        Stmt _id_ [152-153]: Local (Immutable):
                                            Pat _id_ [152-153] [Type Int]: Bind: Ident 28 [152-153] "generated_ident_28"
                                            Expr 26 [152-153] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [127-128]: Semi: Expr _id_ [127-128] [Type Unit]: Call:
                                            Expr _id_ [127-128] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                            Expr _id_ [127-128] [Type Qubit]: Var: Local 23
                                        Stmt _id_ [152-153]: Expr: Expr _id_ [152-153] [Type Int]: Var: Local 28
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [59-60] [Type Qubit]: Var: Local 7
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 21 [10-15] "input"): Item 1
                Item 1 [22-150] (Public):
                    Parent: 0
                    Callable 0 [22-150] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [22-150]: Impl:
                            Block 4 [45-150] [Type Unit]:
                                Stmt _id_ [59-60]: Local (Immutable):
                                    Pat _id_ [59-60] [Type (Qubit)[]]: Bind: Ident 7 [59-60] "a"
                                    Expr _id_ [59-60] [Type Qubit]: Call:
                                        Expr _id_ [59-60] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 9 [69-123] [Type Int]: Expr Block: Block 10 [69-123] [Type Int]:
                                            Stmt _id_ [87-88]: Local (Immutable):
                                                Pat _id_ [87-88] [Type Qubit]: Bind: Ident 13 [87-88] "b"
                                                Expr _id_ [87-88] [Type Qubit]: Call:
                                                    Expr _id_ [87-88] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr _id_ [87-88] [Type Unit]: Unit
                                            Stmt _id_ [112-113]: Local (Immutable):
                                                Pat _id_ [112-113] [Type Int]: Bind: Ident 22 [112-113] "generated_ident_22"
                                                Expr 16 [112-113] [Type Int]: Lit: Int(3)
                                            Stmt _id_ [87-88]: Semi: Expr _id_ [87-88] [Type Unit]: Call:
                                                Expr _id_ [87-88] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [87-88] [Type Qubit]: Var: Local 13
                                            Stmt _id_ [112-113]: Expr: Expr _id_ [112-113] [Type Int]: Var: Local 22
                                Stmt 17 [134-144]: Local (Immutable):
                                    Pat 18 [138-139] [Type Int]: Bind: Ident 19 [138-139] "x"
                                    Expr 20 [142-143] [Type Int]: Lit: Int(3)
                                Stmt _id_ [59-60]: Semi: Expr _id_ [59-60] [Type Unit]: Call:
                                    Expr _id_ [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr _id_ [59-60] [Type (Qubit)[]]: Var: Local 7
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 19 [10-15] "input"): Item 1
                Item 1 [22-147] (Public):
                    Parent: 0
                    Callable 0 [22-147] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [22-147]: Impl:
                            Block 4 [44-147] [Type Int]:
                                Stmt _id_ [58-59]: Local (Immutable):
                                    Pat _id_ [58-59] [Type Qubit]: Bind: Ident 7 [58-59] "a"
                                    Expr _id_ [58-59] [Type Qubit]: Call:
                                        Expr _id_ [58-59] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr _id_ [58-59] [Type Unit]: Unit
                                Stmt 9 [79-141]: Semi: Expr _id_ [79-140] [Type ?2]: Expr Block: Block _id_ [79-140] [Type ?2]:
                                    Stmt _id_ [86-140]: Local (Immutable):
                                        Pat _id_ [86-140] [Type Int]: Bind: Ident 20 [86-140] "generated_ident_20"
                                        Expr 11 [86-140] [Type Int]: Expr Block: Block 12 [86-140] [Type Int]:
                                            Stmt _id_ [104-105]: Local (Immutable):
                                                Pat _id_ [104-105] [Type Qubit]: Bind: Ident 15 [104-105] "b"
                                                Expr _id_ [104-105] [Type Qubit]: Call:
                                                    Expr _id_ [104-105] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr _id_ [104-105] [Type Unit]: Unit
                                            Stmt _id_ [129-130]: Local (Immutable):
                                                Pat _id_ [129-130] [Type Int]: Bind: Ident 21 [129-130] "generated_ident_21"
                                                Expr 18 [129-130] [Type Int]: Lit: Int(3)
                                            Stmt _id_ [104-105]: Semi: Expr _id_ [104-105] [Type Unit]: Call:
                                                Expr _id_ [104-105] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr _id_ [104-105] [Type Qubit]: Var: Local 15
                                            Stmt _id_ [129-130]: Expr: Expr _id_ [129-130] [Type Int]: Var: Local 21
                                    Stmt _id_ [58-59]: Semi: Expr _id_ [58-59] [Type Unit]: Call:
                                        Expr _id_ [58-59] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr _id_ [58-59] [Type Qubit]: Var: Local 7
                                    Stmt _id_ [79-140]: Semi: Expr _id_ [79-140] [Type ?2]: Return: Expr _id_ [86-140] [Type Int]: Var: Local 20
                                Stmt _id_ [58-59]: Semi: Expr _id_ [58-59] [Type Unit]: Call:
                                    Expr _id_ [58-59] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr _id_ [58-59] [Type Qubit]: Var: Local 7
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Callable 0 [22-159] (operation):
                        name: Ident 1 [32-35] "Foo"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [22-159]: Impl:
                            Block 4 [44-159] [Type Int]:
                                Stmt 5 [54-95]: Expr: Expr 6 [54-95] [Type Unit]: If:
                                    Expr 7 [57-61] [Type Bool]: Lit: Bool(true)
                                    Expr 8 [62-95] [Type Unit]: Expr Block: Block 9 [62-95] [Type Unit]:
                                        Stmt 10 [76-85]: Semi: Expr 11 [76-84] [Type ?0]: Return: Expr 12 [83-84] [Type Int]: Lit: Int(3)
                                Stmt 13 [105-153]: Expr: Expr 14 [105-153] [Type Int]: Expr Block: Block 15 [105-153] [Type Int]:
                                    Stmt 16 [119-129]: Local (Immutable):
                                        Pat 17 [123-124] [Type Int]: Bind: Ident 18 [123-124] "x"
                                        Expr 19 [127-128] [Type Int]: Lit: Int(4)
                                    Stmt 20 [142-143]: Expr: Expr 21 [142-143] [Type Int]: Var: Local 18
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}
