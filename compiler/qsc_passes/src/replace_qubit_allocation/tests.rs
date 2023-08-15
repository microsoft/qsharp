// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::replace_qubit_allocation::ReplaceQubitAllocation;
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap, TargetProfile};
use qsc_hir::{mut_visit::MutVisitor, validate::Validator, visit::Visitor};

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources, TargetProfile::Full);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    ReplaceQubitAllocation::new(store.core(), &mut unit.assigner).visit_package(&mut unit.package);
    Validator::default().visit_package(&unit.package);
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
                                Stmt 17 [59-60]: Local (Immutable):
                                    Pat 18 [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "q"
                                    Expr 15 [59-60] [Type Qubit]: Call:
                                        Expr 14 [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 16 [59-60] [Type Unit]: Unit
                                Stmt 9 [80-90]: Local (Immutable):
                                    Pat 10 [84-85] [Type Int]: Bind: Ident 11 [84-85] "x"
                                    Expr 12 [88-89] [Type Int]: Lit: Int(3)
                                Stmt 20 [59-60]: Semi: Expr 21 [59-60] [Type Unit]: Call:
                                    Expr 19 [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 22 [59-60] [Type Qubit]: Var: Local 7
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
                                Stmt 18 [59-60]: Local (Immutable):
                                    Pat 19 [59-60] [Type (Qubit)[]]: Bind: Ident 7 [59-60] "q"
                                    Expr 16 [59-60] [Type Qubit]: Call:
                                        Expr 15 [59-60] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 9 [69-70] [Type Int]: Lit: Int(3)
                                Stmt 10 [81-91]: Local (Immutable):
                                    Pat 11 [85-86] [Type Int]: Bind: Ident 12 [85-86] "x"
                                    Expr 13 [89-90] [Type Int]: Lit: Int(3)
                                Stmt 21 [59-60]: Semi: Expr 22 [59-60] [Type Unit]: Call:
                                    Expr 20 [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr 23 [59-60] [Type (Qubit)[]]: Var: Local 7
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
                                Stmt 24 [64-71]: Local (Immutable):
                                    Pat 25 [64-71] [Type Qubit]: Bind: Ident 16 [64-71] "@generated_ident_16"
                                    Expr 22 [64-71] [Type Qubit]: Call:
                                        Expr 21 [64-71] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 23 [64-71] [Type Unit]: Unit
                                Stmt 29 [73-80]: Local (Immutable):
                                    Pat 30 [73-80] [Type Qubit]: Bind: Ident 18 [73-80] "@generated_ident_18"
                                    Expr 27 [73-80] [Type Qubit]: Call:
                                        Expr 26 [73-80] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 28 [73-80] [Type Unit]: Unit
                                Stmt 31 [55-82]: Local (Immutable):
                                    Pat 6 [59-60] [Type (Qubit, Qubit)]: Bind: Ident 7 [59-60] "q"
                                    Expr 20 [63-81] [Type (Qubit, Qubit)]: Tuple:
                                        Expr 17 [64-71] [Type Qubit]: Var: Local 16
                                        Expr 19 [73-80] [Type Qubit]: Var: Local 18
                                Stmt 11 [91-101]: Local (Immutable):
                                    Pat 12 [95-96] [Type Int]: Bind: Ident 13 [95-96] "x"
                                    Expr 14 [99-100] [Type Int]: Lit: Int(3)
                                Stmt 33 [73-80]: Semi: Expr 34 [73-80] [Type Unit]: Call:
                                    Expr 32 [73-80] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 35 [73-80] [Type Qubit]: Var: Local 18
                                Stmt 37 [64-71]: Semi: Expr 38 [64-71] [Type Unit]: Call:
                                    Expr 36 [64-71] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 39 [64-71] [Type Qubit]: Var: Local 16
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
                                Stmt 28 [69-76]: Local (Immutable):
                                    Pat 29 [69-76] [Type Qubit]: Bind: Ident 20 [69-76] "@generated_ident_20"
                                    Expr 26 [69-76] [Type Qubit]: Call:
                                        Expr 25 [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 27 [69-76] [Type Unit]: Unit
                                Stmt 33 [78-86]: Local (Immutable):
                                    Pat 34 [78-86] [Type (Qubit)[]]: Bind: Ident 22 [78-86] "@generated_ident_22"
                                    Expr 31 [78-86] [Type Qubit]: Call:
                                        Expr 30 [78-86] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 14 [84-85] [Type Int]: Lit: Int(3)
                                Stmt 35 [55-88]: Local (Immutable):
                                    Pat 6 [59-65] [Type (Qubit, (Qubit)[])]: Tuple:
                                        Pat 7 [60-61] [Type Qubit]: Bind: Ident 8 [60-61] "a"
                                        Pat 9 [63-64] [Type (Qubit)[]]: Bind: Ident 10 [63-64] "b"
                                    Expr 24 [68-87] [Type (Qubit, (Qubit)[])]: Tuple:
                                        Expr 21 [69-76] [Type Qubit]: Var: Local 20
                                        Expr 23 [78-86] [Type (Qubit)[]]: Var: Local 22
                                Stmt 15 [97-107]: Local (Immutable):
                                    Pat 16 [101-102] [Type Int]: Bind: Ident 17 [101-102] "x"
                                    Expr 18 [105-106] [Type Int]: Lit: Int(3)
                                Stmt 37 [78-86]: Semi: Expr 38 [78-86] [Type Unit]: Call:
                                    Expr 36 [78-86] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr 39 [78-86] [Type (Qubit)[]]: Var: Local 22
                                Stmt 41 [69-76]: Semi: Expr 42 [69-76] [Type Unit]: Call:
                                    Expr 40 [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 43 [69-76] [Type Qubit]: Var: Local 20
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
                Item 0 [0-210] (Public):
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
                                Stmt 45 [69-76]: Local (Immutable):
                                    Pat 46 [69-76] [Type Qubit]: Bind: Ident 37 [69-76] "@generated_ident_37"
                                    Expr 43 [69-76] [Type Qubit]: Call:
                                        Expr 42 [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 44 [69-76] [Type Unit]: Unit
                                Stmt 50 [78-85]: Local (Immutable):
                                    Pat 51 [78-85] [Type Qubit]: Bind: Ident 39 [78-85] "@generated_ident_39"
                                    Expr 48 [78-85] [Type Qubit]: Call:
                                        Expr 47 [78-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 49 [78-85] [Type Unit]: Unit
                                Stmt 52 [55-87]: Local (Immutable):
                                    Pat 6 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 7 [60-61] [Type Qubit]: Bind: Ident 8 [60-61] "a"
                                        Pat 9 [63-64] [Type Qubit]: Bind: Ident 10 [63-64] "b"
                                    Expr 41 [68-86] [Type (Qubit, Qubit)]: Tuple:
                                        Expr 38 [69-76] [Type Qubit]: Var: Local 37
                                        Expr 40 [78-85] [Type Qubit]: Var: Local 39
                                Stmt 14 [96-106]: Local (Immutable):
                                    Pat 15 [100-101] [Type Int]: Bind: Ident 16 [100-101] "x"
                                    Expr 17 [104-105] [Type Int]: Lit: Int(3)
                                Stmt 54 [78-85]: Semi: Expr 55 [78-85] [Type Unit]: Call:
                                    Expr 53 [78-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 56 [78-85] [Type Qubit]: Var: Local 39
                                Stmt 58 [69-76]: Semi: Expr 59 [69-76] [Type Unit]: Call:
                                    Expr 57 [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 60 [69-76] [Type Qubit]: Var: Local 37
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [118-208] (Public):
                    Parent: 0
                    Callable 18 [118-208] (operation):
                        name: Ident 19 [128-131] "Bar"
                        input: Pat 20 [131-133] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 21 [118-208]: Impl:
                            Block 22 [141-208] [Type Unit]:
                                Stmt 69 [165-172]: Local (Immutable):
                                    Pat 70 [165-172] [Type Qubit]: Bind: Ident 61 [165-172] "@generated_ident_61"
                                    Expr 67 [165-172] [Type Qubit]: Call:
                                        Expr 66 [165-172] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 68 [165-172] [Type Unit]: Unit
                                Stmt 74 [174-181]: Local (Immutable):
                                    Pat 75 [174-181] [Type Qubit]: Bind: Ident 63 [174-181] "@generated_ident_63"
                                    Expr 72 [174-181] [Type Qubit]: Call:
                                        Expr 71 [174-181] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 73 [174-181] [Type Unit]: Unit
                                Stmt 76 [151-183]: Local (Immutable):
                                    Pat 24 [155-161] [Type (Qubit, Qubit)]: Tuple:
                                        Pat 25 [156-157] [Type Qubit]: Bind: Ident 26 [156-157] "c"
                                        Pat 27 [159-160] [Type Qubit]: Bind: Ident 28 [159-160] "d"
                                    Expr 65 [164-182] [Type (Qubit, Qubit)]: Tuple:
                                        Expr 62 [165-172] [Type Qubit]: Var: Local 61
                                        Expr 64 [174-181] [Type Qubit]: Var: Local 63
                                Stmt 32 [192-202]: Local (Immutable):
                                    Pat 33 [196-197] [Type Int]: Bind: Ident 34 [196-197] "x"
                                    Expr 35 [200-201] [Type Int]: Lit: Int(3)
                                Stmt 78 [174-181]: Semi: Expr 79 [174-181] [Type Unit]: Call:
                                    Expr 77 [174-181] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 80 [174-181] [Type Qubit]: Var: Local 63
                                Stmt 82 [165-172]: Semi: Expr 83 [165-172] [Type Unit]: Call:
                                    Expr 81 [165-172] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 84 [165-172] [Type Qubit]: Var: Local 61
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
                                Stmt 65 [55-173]: Expr: Expr 66 [55-173] [Type Unit]: Expr Block: Block 14 [87-173] [Type Unit]:
                                    Stmt 40 [69-76]: Local (Immutable):
                                        Pat 41 [69-76] [Type Qubit]: Bind: Ident 32 [69-76] "@generated_ident_32"
                                        Expr 38 [69-76] [Type Qubit]: Call:
                                            Expr 37 [69-76] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr 39 [69-76] [Type Unit]: Unit
                                    Stmt 45 [78-85]: Local (Immutable):
                                        Pat 46 [78-85] [Type Qubit]: Bind: Ident 34 [78-85] "@generated_ident_34"
                                        Expr 43 [78-85] [Type Qubit]: Call:
                                            Expr 42 [78-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr 44 [78-85] [Type Unit]: Unit
                                    Stmt 47 [55-173]: Local (Immutable):
                                        Pat 6 [59-65] [Type (Qubit, Qubit)]: Tuple:
                                            Pat 7 [60-61] [Type Qubit]: Bind: Ident 8 [60-61] "a"
                                            Pat 9 [63-64] [Type Qubit]: Bind: Ident 10 [63-64] "b"
                                        Expr 36 [68-86] [Type (Qubit, Qubit)]: Tuple:
                                            Expr 33 [69-76] [Type Qubit]: Var: Local 32
                                            Expr 35 [78-85] [Type Qubit]: Var: Local 34
                                    Stmt 15 [101-111]: Local (Immutable):
                                        Pat 16 [105-106] [Type Int]: Bind: Ident 17 [105-106] "x"
                                        Expr 18 [109-110] [Type Int]: Lit: Int(3)
                                    Stmt 51 [128-129]: Local (Immutable):
                                        Pat 52 [128-129] [Type Qubit]: Bind: Ident 21 [128-129] "c"
                                        Expr 49 [128-129] [Type Qubit]: Call:
                                            Expr 48 [128-129] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr 50 [128-129] [Type Unit]: Unit
                                    Stmt 23 [153-163]: Local (Immutable):
                                        Pat 24 [157-158] [Type Int]: Bind: Ident 25 [157-158] "y"
                                        Expr 26 [161-162] [Type Int]: Lit: Int(3)
                                    Stmt 54 [128-129]: Semi: Expr 55 [128-129] [Type Unit]: Call:
                                        Expr 53 [128-129] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 56 [128-129] [Type Qubit]: Var: Local 21
                                    Stmt 58 [78-85]: Semi: Expr 59 [78-85] [Type Unit]: Call:
                                        Expr 57 [78-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 60 [78-85] [Type Qubit]: Var: Local 34
                                    Stmt 62 [69-76]: Semi: Expr 63 [69-76] [Type Unit]: Call:
                                        Expr 61 [69-76] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 64 [69-76] [Type Qubit]: Var: Local 32
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
                                Stmt 26 [59-60]: Local (Immutable):
                                    Pat 27 [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "a"
                                    Expr 24 [59-60] [Type Qubit]: Call:
                                        Expr 23 [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 25 [59-60] [Type Unit]: Unit
                                Stmt 37 [80-130]: Expr: Expr 38 [80-130] [Type Unit]: Expr Block: Block 13 [96-130] [Type Unit]:
                                    Stmt 31 [84-85]: Local (Immutable):
                                        Pat 32 [84-85] [Type Qubit]: Bind: Ident 11 [84-85] "b"
                                        Expr 29 [84-85] [Type Qubit]: Call:
                                            Expr 28 [84-85] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr 30 [84-85] [Type Unit]: Unit
                                    Stmt 14 [110-120]: Local (Immutable):
                                        Pat 15 [114-115] [Type Int]: Bind: Ident 16 [114-115] "x"
                                        Expr 17 [118-119] [Type Int]: Lit: Int(3)
                                    Stmt 34 [84-85]: Semi: Expr 35 [84-85] [Type Unit]: Call:
                                        Expr 33 [84-85] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 36 [84-85] [Type Qubit]: Var: Local 11
                                Stmt 18 [139-149]: Local (Immutable):
                                    Pat 19 [143-144] [Type Int]: Bind: Ident 20 [143-144] "y"
                                    Expr 21 [147-148] [Type Int]: Lit: Int(3)
                                Stmt 40 [59-60]: Semi: Expr 41 [59-60] [Type Unit]: Call:
                                    Expr 39 [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 42 [59-60] [Type Qubit]: Var: Local 7
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
                                Stmt 59 [79-80]: Local (Immutable):
                                    Pat 60 [79-80] [Type Qubit]: Bind: Ident 11 [79-80] "a"
                                    Expr 57 [79-80] [Type Qubit]: Call:
                                        Expr 56 [79-80] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 58 [79-80] [Type Unit]: Unit
                                Stmt 13 [100-111]: Local (Immutable):
                                    Pat 14 [104-106] [Type Int]: Bind: Ident 15 [104-106] "x2"
                                    Expr 16 [109-110] [Type Int]: Lit: Int(3)
                                Stmt 17 [120-208]: Expr: Expr 18 [120-208] [Type Unit]: Expr Block: Block 19 [120-208] [Type Unit]:
                                    Stmt 20 [134-145]: Local (Immutable):
                                        Pat 21 [138-140] [Type Int]: Bind: Ident 22 [138-140] "y1"
                                        Expr 23 [143-144] [Type Int]: Lit: Int(3)
                                    Stmt 64 [162-163]: Local (Immutable):
                                        Pat 65 [162-163] [Type Qubit]: Bind: Ident 26 [162-163] "b"
                                        Expr 62 [162-163] [Type Qubit]: Call:
                                            Expr 61 [162-163] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr 63 [162-163] [Type Unit]: Unit
                                    Stmt 28 [187-198]: Local (Immutable):
                                        Pat 29 [191-193] [Type Int]: Bind: Ident 30 [191-193] "y2"
                                        Expr 31 [196-197] [Type Int]: Lit: Int(3)
                                    Stmt 67 [162-163]: Semi: Expr 68 [162-163] [Type Unit]: Call:
                                        Expr 66 [162-163] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 69 [162-163] [Type Qubit]: Var: Local 26
                                Stmt 32 [217-228]: Local (Immutable):
                                    Pat 33 [221-223] [Type Int]: Bind: Ident 34 [221-223] "x3"
                                    Expr 35 [226-227] [Type Int]: Lit: Int(3)
                                Stmt 36 [237-325]: Expr: Expr 37 [237-325] [Type Unit]: Expr Block: Block 38 [237-325] [Type Unit]:
                                    Stmt 39 [251-262]: Local (Immutable):
                                        Pat 40 [255-257] [Type Int]: Bind: Ident 41 [255-257] "z1"
                                        Expr 42 [260-261] [Type Int]: Lit: Int(3)
                                    Stmt 73 [279-280]: Local (Immutable):
                                        Pat 74 [279-280] [Type Qubit]: Bind: Ident 45 [279-280] "c"
                                        Expr 71 [279-280] [Type Qubit]: Call:
                                            Expr 70 [279-280] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                            Expr 72 [279-280] [Type Unit]: Unit
                                    Stmt 47 [304-315]: Local (Immutable):
                                        Pat 48 [308-310] [Type Int]: Bind: Ident 49 [308-310] "z2"
                                        Expr 50 [313-314] [Type Int]: Lit: Int(3)
                                    Stmt 76 [279-280]: Semi: Expr 77 [279-280] [Type Unit]: Call:
                                        Expr 75 [279-280] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 78 [279-280] [Type Qubit]: Var: Local 45
                                Stmt 51 [334-345]: Local (Immutable):
                                    Pat 52 [338-340] [Type Int]: Bind: Ident 53 [338-340] "x4"
                                    Expr 54 [343-344] [Type Int]: Lit: Int(3)
                                Stmt 80 [79-80]: Semi: Expr 81 [79-80] [Type Unit]: Call:
                                    Expr 79 [79-80] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 82 [79-80] [Type Qubit]: Var: Local 11
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
                                Stmt 37 [59-60]: Local (Immutable):
                                    Pat 38 [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "a"
                                    Expr 35 [59-60] [Type Qubit]: Call:
                                        Expr 34 [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 36 [59-60] [Type Unit]: Unit
                                Stmt 9 [80-151]: Expr: Expr 10 [80-151] [Type Unit]: If:
                                    Expr 11 [83-87] [Type Bool]: Lit: Bool(true)
                                    Expr 12 [88-151] [Type Unit]: Expr Block: Block 13 [88-151] [Type Unit]:
                                        Stmt 42 [106-107]: Local (Immutable):
                                            Pat 43 [106-107] [Type Qubit]: Bind: Ident 16 [106-107] "b"
                                            Expr 40 [106-107] [Type Qubit]: Call:
                                                Expr 39 [106-107] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                Expr 41 [106-107] [Type Unit]: Unit
                                        Stmt 18 [131-141]: Semi: Expr 58 [131-140] [Type Unit]: Expr Block: Block 59 [131-140] [Type Unit]:
                                            Stmt 45 [138-140]: Local (Immutable):
                                                Pat 46 [138-140] [Type Unit]: Bind: Ident 44 [138-140] "@generated_ident_44"
                                                Expr 20 [138-140] [Type Unit]: Unit
                                            Stmt 48 [106-107]: Semi: Expr 49 [106-107] [Type Unit]: Call:
                                                Expr 47 [106-107] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr 50 [106-107] [Type Qubit]: Var: Local 16
                                            Stmt 52 [59-60]: Semi: Expr 53 [59-60] [Type Unit]: Call:
                                                Expr 51 [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr 54 [59-60] [Type Qubit]: Var: Local 7
                                            Stmt 55 [131-140]: Semi: Expr 56 [131-140] [Type Unit]: Return: Expr 57 [138-140] [Type Unit]: Var: Local 44
                                        Stmt 61 [106-107]: Semi: Expr 62 [106-107] [Type Unit]: Call:
                                            Expr 60 [106-107] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                            Expr 63 [106-107] [Type Qubit]: Var: Local 16
                                Stmt 90 [161-233]: Local (Immutable):
                                    Pat 91 [161-233] [Type Unit]: Bind: Ident 89 [161-233] "@generated_ident_89"
                                    Expr 22 [161-233] [Type Unit]: If:
                                        Expr 23 [164-169] [Type Bool]: Lit: Bool(false)
                                        Expr 24 [170-233] [Type Unit]: Expr Block: Block 25 [170-233] [Type Unit]:
                                            Stmt 67 [188-189]: Local (Immutable):
                                                Pat 68 [188-189] [Type Qubit]: Bind: Ident 28 [188-189] "c"
                                                Expr 65 [188-189] [Type Qubit]: Call:
                                                    Expr 64 [188-189] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr 66 [188-189] [Type Unit]: Unit
                                            Stmt 30 [213-223]: Semi: Expr 83 [213-222] [Type Unit]: Expr Block: Block 84 [213-222] [Type Unit]:
                                                Stmt 70 [220-222]: Local (Immutable):
                                                    Pat 71 [220-222] [Type Unit]: Bind: Ident 69 [220-222] "@generated_ident_69"
                                                    Expr 32 [220-222] [Type Unit]: Unit
                                                Stmt 73 [188-189]: Semi: Expr 74 [188-189] [Type Unit]: Call:
                                                    Expr 72 [188-189] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                    Expr 75 [188-189] [Type Qubit]: Var: Local 28
                                                Stmt 77 [59-60]: Semi: Expr 78 [59-60] [Type Unit]: Call:
                                                    Expr 76 [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                    Expr 79 [59-60] [Type Qubit]: Var: Local 7
                                                Stmt 80 [213-222]: Semi: Expr 81 [213-222] [Type Unit]: Return: Expr 82 [220-222] [Type Unit]: Var: Local 69
                                            Stmt 86 [188-189]: Semi: Expr 87 [188-189] [Type Unit]: Call:
                                                Expr 85 [188-189] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr 88 [188-189] [Type Qubit]: Var: Local 28
                                Stmt 95 [59-60]: Semi: Expr 96 [59-60] [Type Unit]: Call:
                                    Expr 94 [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 97 [59-60] [Type Qubit]: Var: Local 7
                                Stmt 92 [161-233]: Expr: Expr 93 [161-233] [Type Unit]: Var: Local 89
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
                                Stmt 31 [59-60]: Local (Immutable):
                                    Pat 32 [59-60] [Type Qubit]: Bind: Ident 7 [59-60] "a"
                                    Expr 29 [59-60] [Type Qubit]: Call:
                                        Expr 28 [59-60] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 30 [59-60] [Type Unit]: Unit
                                Stmt 9 [80-92]: Local (Immutable):
                                    Pat 10 [84-85] [Type Int]: Bind: Ident 11 [84-85] "x"
                                    Expr 12 [88-91] [Type Int]: Expr Block: Block 13 [88-91] [Type Int]:
                                        Stmt 14 [89-90]: Expr: Expr 15 [89-90] [Type Int]: Lit: Int(3)
                                Stmt 16 [101-164]: Local (Immutable):
                                    Pat 17 [105-106] [Type Int]: Bind: Ident 18 [105-106] "y"
                                    Expr 19 [109-163] [Type Int]: Expr Block: Block 20 [109-163] [Type Int]:
                                        Stmt 36 [127-128]: Local (Immutable):
                                            Pat 37 [127-128] [Type Qubit]: Bind: Ident 23 [127-128] "b"
                                            Expr 34 [127-128] [Type Qubit]: Call:
                                                Expr 33 [127-128] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                Expr 35 [127-128] [Type Unit]: Unit
                                        Stmt 39 [152-153]: Local (Immutable):
                                            Pat 40 [152-153] [Type Int]: Bind: Ident 38 [152-153] "@generated_ident_38"
                                            Expr 26 [152-153] [Type Int]: Lit: Int(3)
                                        Stmt 44 [127-128]: Semi: Expr 45 [127-128] [Type Unit]: Call:
                                            Expr 43 [127-128] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                            Expr 46 [127-128] [Type Qubit]: Var: Local 23
                                        Stmt 41 [152-153]: Expr: Expr 42 [152-153] [Type Int]: Var: Local 38
                                Stmt 48 [59-60]: Semi: Expr 49 [59-60] [Type Unit]: Call:
                                    Expr 47 [59-60] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 50 [59-60] [Type Qubit]: Var: Local 7
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
                                Stmt 39 [59-60]: Local (Immutable):
                                    Pat 40 [59-60] [Type (Qubit)[]]: Bind: Ident 7 [59-60] "a"
                                    Expr 37 [59-60] [Type Qubit]: Call:
                                        Expr 36 [59-60] [Type (Int => (Qubit)[])]: Var: Item 6 (Package 0)
                                        Expr 9 [69-123] [Type Int]: Expr Block: Block 10 [69-123] [Type Int]:
                                            Stmt 25 [87-88]: Local (Immutable):
                                                Pat 26 [87-88] [Type Qubit]: Bind: Ident 13 [87-88] "b"
                                                Expr 23 [87-88] [Type Qubit]: Call:
                                                    Expr 22 [87-88] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr 24 [87-88] [Type Unit]: Unit
                                            Stmt 28 [112-113]: Local (Immutable):
                                                Pat 29 [112-113] [Type Int]: Bind: Ident 27 [112-113] "@generated_ident_27"
                                                Expr 16 [112-113] [Type Int]: Lit: Int(3)
                                            Stmt 33 [87-88]: Semi: Expr 34 [87-88] [Type Unit]: Call:
                                                Expr 32 [87-88] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr 35 [87-88] [Type Qubit]: Var: Local 13
                                            Stmt 30 [112-113]: Expr: Expr 31 [112-113] [Type Int]: Var: Local 27
                                Stmt 17 [134-144]: Local (Immutable):
                                    Pat 18 [138-139] [Type Int]: Bind: Ident 19 [138-139] "x"
                                    Expr 20 [142-143] [Type Int]: Lit: Int(3)
                                Stmt 42 [59-60]: Semi: Expr 43 [59-60] [Type Unit]: Call:
                                    Expr 41 [59-60] [Type ((Qubit)[] => Unit)]: Var: Item 7 (Package 0)
                                    Expr 44 [59-60] [Type (Qubit)[]]: Var: Local 7
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
                                Stmt 23 [58-59]: Local (Immutable):
                                    Pat 24 [58-59] [Type Qubit]: Bind: Ident 7 [58-59] "a"
                                    Expr 21 [58-59] [Type Qubit]: Call:
                                        Expr 20 [58-59] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                        Expr 22 [58-59] [Type Unit]: Unit
                                Stmt 9 [79-141]: Semi: Expr 49 [79-140] [Type Unit]: Expr Block: Block 50 [79-140] [Type Unit]:
                                    Stmt 40 [86-140]: Local (Immutable):
                                        Pat 41 [86-140] [Type Int]: Bind: Ident 25 [86-140] "@generated_ident_25"
                                        Expr 11 [86-140] [Type Int]: Expr Block: Block 12 [86-140] [Type Int]:
                                            Stmt 29 [104-105]: Local (Immutable):
                                                Pat 30 [104-105] [Type Qubit]: Bind: Ident 15 [104-105] "b"
                                                Expr 27 [104-105] [Type Qubit]: Call:
                                                    Expr 26 [104-105] [Type (Unit => Qubit)]: Var: Item 4 (Package 0)
                                                    Expr 28 [104-105] [Type Unit]: Unit
                                            Stmt 32 [129-130]: Local (Immutable):
                                                Pat 33 [129-130] [Type Int]: Bind: Ident 31 [129-130] "@generated_ident_31"
                                                Expr 18 [129-130] [Type Int]: Lit: Int(3)
                                            Stmt 37 [104-105]: Semi: Expr 38 [104-105] [Type Unit]: Call:
                                                Expr 36 [104-105] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                                Expr 39 [104-105] [Type Qubit]: Var: Local 15
                                            Stmt 34 [129-130]: Expr: Expr 35 [129-130] [Type Int]: Var: Local 31
                                    Stmt 43 [58-59]: Semi: Expr 44 [58-59] [Type Unit]: Call:
                                        Expr 42 [58-59] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                        Expr 45 [58-59] [Type Qubit]: Var: Local 7
                                    Stmt 46 [79-140]: Semi: Expr 47 [79-140] [Type Unit]: Return: Expr 48 [86-140] [Type Int]: Var: Local 25
                                Stmt 52 [58-59]: Semi: Expr 53 [58-59] [Type Unit]: Call:
                                    Expr 51 [58-59] [Type (Qubit => Unit)]: Var: Item 5 (Package 0)
                                    Expr 54 [58-59] [Type Qubit]: Var: Local 7
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
                                        Stmt 10 [76-85]: Semi: Expr 11 [76-84] [Type Unit]: Return: Expr 12 [83-84] [Type Int]: Lit: Int(3)
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
