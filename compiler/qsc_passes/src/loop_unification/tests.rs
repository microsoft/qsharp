// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore};

use crate::loop_unification::loop_unification;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new();
    let mut unit = compile(&store, [], [file], "");
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
    let errors = loop_unification(&mut unit);
    if errors.is_empty() {
        expect.assert_eq(&unit.package.to_string());
    } else {
        expect.assert_debug_eq(&errors);
    }
}

#[test]
fn convert_for_array() {
    check(
        indoc! {r#"
        namespace test {
            operation Main(arr : Int[]) : Unit {
                for i in arr {
                    let x = "Stmt";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-126]:
                    Namespace (Ident 16 [10-14] "test"): Item 1
                Item 1 [21-124]:
                    Parent: 0
                    Callable 0 [21-124] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-48] [Type (Int)[]]: Paren:
                            Pat 3 [36-47] [Type (Int)[]]: Bind: Ident 4 [36-39] "arr"
                        output: ()
                        body: Block: Block 5 [56-124] [Type ()]:
                            Stmt 6 [66-118]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [75-78]: Local (Immutable):
                                    Pat _id_ [75-78] [Type (Int)[]]: Bind: Ident 17 [75-78] "__array_id_0__"
                                    Expr 10 [75-78] [Type (Int)[]]: Name: Local 4
                                Stmt _id_ [75-78]: Local (Immutable):
                                    Pat _id_ [75-78] [Type Int]: Bind: Ident 18 [75-78] "__len_id_1__"
                                    Expr _id_ [75-78] [Type Int]: Field:
                                        Expr _id_ [75-78] [Type (Int)[]]: Name: Local 17
                                        Ident _id_ [75-78] "Length"
                                Stmt _id_ [75-78]: Local (Mutable):
                                    Pat _id_ [75-78] [Type Int]: Bind: Ident 19 [75-78] "__index_id_2__"
                                    Expr _id_ [75-78] [Type Int]: Lit: Int(0)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [75-78] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [75-78] [Type Int]: Name: Local 19
                                        Expr _id_ [75-78] [Type Int]: Name: Local 18
                                    Block 11 [79-118] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat 8 [70-71] [Type Int]: Bind: Ident 9 [70-71] "i"
                                            Expr _id_ [75-78] [Type Int]: Index:
                                                Expr _id_ [75-78] [Type (Int)[]]: Name: Local 17
                                                Expr _id_ [75-78] [Type Int]: Name: Local 19
                                        Stmt 12 [93-108]: Local (Immutable):
                                            Pat 13 [97-98] [Type String]: Bind: Ident 14 [97-98] "x"
                                            Expr 15 [101-107] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [75-78]: Semi: Expr _id_ [75-78] [Type ()]: AssignOp (Add):
                                            Expr _id_ [75-78] [Type Int]: Name: Local 19
                                            Expr _id_ [75-78] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn convert_for_array_deconstruct() {
    check(
        indoc! {r#"
        namespace test {
            operation Main(arr : (Int, Double)[]) : Unit {
                for (i, d) in arr {
                    let x = "Stmt";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-141]:
                    Namespace (Ident 19 [10-14] "test"): Item 1
                Item 1 [21-139]:
                    Parent: 0
                    Callable 0 [21-139] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-58] [Type ((Int, Double))[]]: Paren:
                            Pat 3 [36-57] [Type ((Int, Double))[]]: Bind: Ident 4 [36-39] "arr"
                        output: ()
                        body: Block: Block 5 [66-139] [Type ()]:
                            Stmt 6 [76-133]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [90-93]: Local (Immutable):
                                    Pat _id_ [90-93] [Type ((Int, Double))[]]: Bind: Ident 20 [90-93] "__array_id_0__"
                                    Expr 13 [90-93] [Type ((Int, Double))[]]: Name: Local 4
                                Stmt _id_ [90-93]: Local (Immutable):
                                    Pat _id_ [90-93] [Type Int]: Bind: Ident 21 [90-93] "__len_id_1__"
                                    Expr _id_ [90-93] [Type Int]: Field:
                                        Expr _id_ [90-93] [Type ((Int, Double))[]]: Name: Local 20
                                        Ident _id_ [90-93] "Length"
                                Stmt _id_ [90-93]: Local (Mutable):
                                    Pat _id_ [90-93] [Type Int]: Bind: Ident 22 [90-93] "__index_id_2__"
                                    Expr _id_ [90-93] [Type Int]: Lit: Int(0)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [90-93] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [90-93] [Type Int]: Name: Local 22
                                        Expr _id_ [90-93] [Type Int]: Name: Local 21
                                    Block 14 [94-133] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat 8 [80-86] [Type (Int, Double)]: Tuple:
                                                Pat 9 [81-82] [Type Int]: Bind: Ident 10 [81-82] "i"
                                                Pat 11 [84-85] [Type Double]: Bind: Ident 12 [84-85] "d"
                                            Expr _id_ [90-93] [Type (Int, Double)]: Index:
                                                Expr _id_ [90-93] [Type ((Int, Double))[]]: Name: Local 20
                                                Expr _id_ [90-93] [Type Int]: Name: Local 22
                                        Stmt 15 [108-123]: Local (Immutable):
                                            Pat 16 [112-113] [Type String]: Bind: Ident 17 [112-113] "x"
                                            Expr 18 [116-122] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [90-93]: Semi: Expr _id_ [90-93] [Type ()]: AssignOp (Add):
                                            Expr _id_ [90-93] [Type Int]: Name: Local 22
                                            Expr _id_ [90-93] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn convert_for_slice() {
    check(
        indoc! {r#"
        namespace test {
            operation Main(arr : Int[]) : Unit {
                for i in arr[6..-2..2] {
                    let x = "Stmt";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-136]:
                    Namespace (Ident 22 [10-14] "test"): Item 1
                Item 1 [21-134]:
                    Parent: 0
                    Callable 0 [21-134] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-48] [Type (Int)[]]: Paren:
                            Pat 3 [36-47] [Type (Int)[]]: Bind: Ident 4 [36-39] "arr"
                        output: ()
                        body: Block: Block 5 [56-134] [Type ()]:
                            Stmt 6 [66-128]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [75-88]: Local (Immutable):
                                    Pat _id_ [75-88] [Type (Int)[]]: Bind: Ident 23 [75-88] "__array_id_0__"
                                    Expr 10 [75-88] [Type (Int)[]]: Index:
                                        Expr 11 [75-78] [Type (Int)[]]: Name: Local 4
                                        Expr 12 [79-87] [Type Range]: Range:
                                            Expr 13 [79-80] [Type Int]: Lit: Int(6)
                                            Expr 14 [82-84] [Type Int]: UnOp (Neg):
                                                Expr 15 [83-84] [Type Int]: Lit: Int(2)
                                            Expr 16 [86-87] [Type Int]: Lit: Int(2)
                                Stmt _id_ [75-88]: Local (Immutable):
                                    Pat _id_ [75-88] [Type Int]: Bind: Ident 24 [75-88] "__len_id_1__"
                                    Expr _id_ [75-88] [Type Int]: Field:
                                        Expr _id_ [75-88] [Type (Int)[]]: Name: Local 23
                                        Ident _id_ [75-88] "Length"
                                Stmt _id_ [75-88]: Local (Mutable):
                                    Pat _id_ [75-88] [Type Int]: Bind: Ident 25 [75-88] "__index_id_2__"
                                    Expr _id_ [75-88] [Type Int]: Lit: Int(0)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [75-88] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [75-88] [Type Int]: Name: Local 25
                                        Expr _id_ [75-88] [Type Int]: Name: Local 24
                                    Block 17 [89-128] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat 8 [70-71] [Type Int]: Bind: Ident 9 [70-71] "i"
                                            Expr _id_ [75-88] [Type Int]: Index:
                                                Expr _id_ [75-88] [Type (Int)[]]: Name: Local 23
                                                Expr _id_ [75-88] [Type Int]: Name: Local 25
                                        Stmt 18 [103-118]: Local (Immutable):
                                            Pat 19 [107-108] [Type String]: Bind: Ident 20 [107-108] "x"
                                            Expr 21 [111-117] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [75-88]: Semi: Expr _id_ [75-88] [Type ()]: AssignOp (Add):
                                            Expr _id_ [75-88] [Type Int]: Name: Local 25
                                            Expr _id_ [75-88] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn convert_for_range() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                for i in 0..4 {
                    let x = "Stmt";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-116]:
                    Namespace (Ident 16 [10-14] "test"): Item 1
                Item 1 [21-114]:
                    Parent: 0
                    Callable 0 [21-114] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-114] [Type ()]:
                            Stmt 4 [55-108]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Range]: Bind: Ident 17 [64-68] "__range_id_0__"
                                    Expr 8 [64-68] [Type Range]: Range:
                                        Expr 9 [64-65] [Type Int]: Lit: Int(0)
                                        <no step>
                                        Expr 10 [67-68] [Type Int]: Lit: Int(4)
                                Stmt _id_ [64-68]: Local (Mutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 18 [64-68] "__index_id_1__"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Name: Local 17
                                        Ident _id_ [64-68] "Start"
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 19 [64-68] "__step_id_2__"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Name: Local 17
                                        Ident _id_ [64-68] "Step"
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 20 [64-68] "__end_id_3__"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Name: Local 17
                                        Ident _id_ [64-68] "End"
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [64-68] [Type Bool]: BinOp (OrL):
                                        Expr _id_ [64-68] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Gt):
                                                Expr _id_ [64-68] [Type Int]: Name: Local 19
                                                Expr _id_ [64-68] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Lte):
                                                Expr _id_ [64-68] [Type Int]: Name: Local 18
                                                Expr _id_ [64-68] [Type Int]: Name: Local 20
                                        Expr _id_ [64-68] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Lt):
                                                Expr _id_ [64-68] [Type Int]: Name: Local 19
                                                Expr _id_ [64-68] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Gte):
                                                Expr _id_ [64-68] [Type Int]: Name: Local 18
                                                Expr _id_ [64-68] [Type Int]: Name: Local 20
                                    Block 11 [69-108] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat 6 [59-60] [Type Int]: Bind: Ident 7 [59-60] "i"
                                            Expr _id_ [64-68] [Type Int]: Name: Local 18
                                        Stmt 12 [83-98]: Local (Immutable):
                                            Pat 13 [87-88] [Type String]: Bind: Ident 14 [87-88] "x"
                                            Expr 15 [91-97] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [64-68]: Semi: Expr _id_ [64-68] [Type ()]: AssignOp (Add):
                                            Expr _id_ [64-68] [Type Int]: Name: Local 18
                                            Expr _id_ [64-68] [Type Int]: Name: Local 19"#]],
    );
}

#[test]
fn convert_for_reverse_range() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                for i in 4..-1..0 {
                    let x = "Stmt";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-120]:
                    Namespace (Ident 18 [10-14] "test"): Item 1
                Item 1 [21-118]:
                    Parent: 0
                    Callable 0 [21-118] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-118] [Type ()]:
                            Stmt 4 [55-112]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Range]: Bind: Ident 19 [64-72] "__range_id_0__"
                                    Expr 8 [64-72] [Type Range]: Range:
                                        Expr 9 [64-65] [Type Int]: Lit: Int(4)
                                        Expr 10 [67-69] [Type Int]: UnOp (Neg):
                                            Expr 11 [68-69] [Type Int]: Lit: Int(1)
                                        Expr 12 [71-72] [Type Int]: Lit: Int(0)
                                Stmt _id_ [64-72]: Local (Mutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 20 [64-72] "__index_id_1__"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Name: Local 19
                                        Ident _id_ [64-72] "Start"
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 21 [64-72] "__step_id_2__"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Name: Local 19
                                        Ident _id_ [64-72] "Step"
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 22 [64-72] "__end_id_3__"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Name: Local 19
                                        Ident _id_ [64-72] "End"
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [64-72] [Type Bool]: BinOp (OrL):
                                        Expr _id_ [64-72] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Gt):
                                                Expr _id_ [64-72] [Type Int]: Name: Local 21
                                                Expr _id_ [64-72] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Lte):
                                                Expr _id_ [64-72] [Type Int]: Name: Local 20
                                                Expr _id_ [64-72] [Type Int]: Name: Local 22
                                        Expr _id_ [64-72] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Lt):
                                                Expr _id_ [64-72] [Type Int]: Name: Local 21
                                                Expr _id_ [64-72] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Gte):
                                                Expr _id_ [64-72] [Type Int]: Name: Local 20
                                                Expr _id_ [64-72] [Type Int]: Name: Local 22
                                    Block 13 [73-112] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat 6 [59-60] [Type Int]: Bind: Ident 7 [59-60] "i"
                                            Expr _id_ [64-72] [Type Int]: Name: Local 20
                                        Stmt 14 [87-102]: Local (Immutable):
                                            Pat 15 [91-92] [Type String]: Bind: Ident 16 [91-92] "x"
                                            Expr 17 [95-101] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [64-72]: Semi: Expr _id_ [64-72] [Type ()]: AssignOp (Add):
                                            Expr _id_ [64-72] [Type Int]: Name: Local 20
                                            Expr _id_ [64-72] [Type Int]: Name: Local 21"#]],
    );
}

#[test]
fn convert_repeat() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                repeat {
                    let x = "Stmt";
                } until true;
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-121]:
                    Namespace (Ident 12 [10-14] "test"): Item 1
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-119] [Type ()]:
                            Stmt 4 [55-113]: Semi: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [108-112]: Local (Mutable):
                                    Pat _id_ [108-112] [Type Bool]: Bind: Ident 13 [108-112] "__continue_cond_0__"
                                    Expr _id_ [108-112] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [108-112] [Type Bool]: Name: Local 13
                                    Block 6 [62-101] [Type ()]:
                                        Stmt 7 [76-91]: Local (Immutable):
                                            Pat 8 [80-81] [Type String]: Bind: Ident 9 [80-81] "x"
                                            Expr 10 [84-90] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [108-112]: Semi: Expr _id_ [108-112] [Type ()]: Assign:
                                            Expr _id_ [108-112] [Type Bool]: Name: Local 13
                                            Expr _id_ [108-112] [Type Bool]: UnOp (NotL):
                                                Expr 11 [108-112] [Type Bool]: Lit: Bool(true)"#]],
    );
}

#[test]
fn convert_repeat_fixup() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                repeat {
                    let x = "Stmt";
                } until true
                fixup {
                    let y = "Fixup";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-175]:
                    Namespace (Ident 17 [10-14] "test"): Item 1
                Item 1 [21-173]:
                    Parent: 0
                    Callable 0 [21-173] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-173] [Type ()]:
                            Stmt 4 [55-167]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [108-112]: Local (Mutable):
                                    Pat _id_ [108-112] [Type Bool]: Bind: Ident 18 [108-112] "__continue_cond_0__"
                                    Expr _id_ [108-112] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [108-112] [Type Bool]: Name: Local 18
                                    Block 6 [62-101] [Type ()]:
                                        Stmt 7 [76-91]: Local (Immutable):
                                            Pat 8 [80-81] [Type String]: Bind: Ident 9 [80-81] "x"
                                            Expr 10 [84-90] [Type String]: Lit: String("Stmt")
                                        Stmt _id_ [108-112]: Semi: Expr _id_ [108-112] [Type ()]: Assign:
                                            Expr _id_ [108-112] [Type Bool]: Name: Local 18
                                            Expr _id_ [108-112] [Type Bool]: UnOp (NotL):
                                                Expr 11 [108-112] [Type Bool]: Lit: Bool(true)
                                        Stmt _id_ [127-167]: Expr: Expr _id_ [127-167] [Type ()]: If:
                                            Expr _id_ [108-112] [Type Bool]: Name: Local 18
                                            Block 12 [127-167] [Type ()]:
                                                Stmt 13 [141-157]: Local (Immutable):
                                                    Pat 14 [145-146] [Type String]: Bind: Ident 15 [145-146] "y"
                                                    Expr 16 [149-156] [Type String]: Lit: String("Fixup")"#]],
    );
}

#[test]
fn convert_repeat_nested() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                let a = true;
                let b = false;
                let c = true;
                repeat {
                    repeat {
                        let x = "First";
                    } until a
                    fixup {
                        let y = "Second";
                    }
                } until b
                fixup {
                    repeat {
                        let z = "Third";
                    } until c;
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-403]:
                    Namespace (Ident 42 [10-14] "test"): Item 1
                Item 1 [21-401]:
                    Parent: 0
                    Callable 0 [21-401] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-401] [Type ()]:
                            Stmt 4 [55-68]: Local (Immutable):
                                Pat 5 [59-60] [Type Bool]: Bind: Ident 6 [59-60] "a"
                                Expr 7 [63-67] [Type Bool]: Lit: Bool(true)
                            Stmt 8 [77-91]: Local (Immutable):
                                Pat 9 [81-82] [Type Bool]: Bind: Ident 10 [81-82] "b"
                                Expr 11 [85-90] [Type Bool]: Lit: Bool(false)
                            Stmt 12 [100-113]: Local (Immutable):
                                Pat 13 [104-105] [Type Bool]: Bind: Ident 14 [104-105] "c"
                                Expr 15 [108-112] [Type Bool]: Lit: Bool(true)
                            Stmt 16 [122-395]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [291-292]: Local (Mutable):
                                    Pat _id_ [291-292] [Type Bool]: Bind: Ident 43 [291-292] "__continue_cond_0__"
                                    Expr _id_ [291-292] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                    Expr _id_ [291-292] [Type Bool]: Name: Local 43
                                    Block 18 [129-284] [Type ()]:
                                        Stmt 19 [143-274]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                            Stmt _id_ [205-206]: Local (Mutable):
                                                Pat _id_ [205-206] [Type Bool]: Bind: Ident 44 [205-206] "__continue_cond_1__"
                                                Expr _id_ [205-206] [Type Bool]: Lit: Bool(true)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                                Expr _id_ [205-206] [Type Bool]: Name: Local 44
                                                Block 21 [150-198] [Type ()]:
                                                    Stmt 22 [168-184]: Local (Immutable):
                                                        Pat 23 [172-173] [Type String]: Bind: Ident 24 [172-173] "x"
                                                        Expr 25 [176-183] [Type String]: Lit: String("First")
                                                    Stmt _id_ [205-206]: Semi: Expr _id_ [205-206] [Type ()]: Assign:
                                                        Expr _id_ [205-206] [Type Bool]: Name: Local 44
                                                        Expr _id_ [205-206] [Type Bool]: UnOp (NotL):
                                                            Expr 26 [205-206] [Type Bool]: Name: Local 6
                                                    Stmt _id_ [225-274]: Expr: Expr _id_ [225-274] [Type ()]: If:
                                                        Expr _id_ [205-206] [Type Bool]: Name: Local 44
                                                        Block 27 [225-274] [Type ()]:
                                                            Stmt 28 [243-260]: Local (Immutable):
                                                                Pat 29 [247-248] [Type String]: Bind: Ident 30 [247-248] "y"
                                                                Expr 31 [251-259] [Type String]: Lit: String("Second")
                                        Stmt _id_ [291-292]: Semi: Expr _id_ [291-292] [Type ()]: Assign:
                                            Expr _id_ [291-292] [Type Bool]: Name: Local 43
                                            Expr _id_ [291-292] [Type Bool]: UnOp (NotL):
                                                Expr 32 [291-292] [Type Bool]: Name: Local 10
                                        Stmt _id_ [307-395]: Expr: Expr _id_ [307-395] [Type ()]: If:
                                            Expr _id_ [291-292] [Type Bool]: Name: Local 43
                                            Block 33 [307-395] [Type ()]:
                                                Stmt 34 [321-385]: Semi: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                                    Stmt _id_ [383-384]: Local (Mutable):
                                                        Pat _id_ [383-384] [Type Bool]: Bind: Ident 45 [383-384] "__continue_cond_2__"
                                                        Expr _id_ [383-384] [Type Bool]: Lit: Bool(true)
                                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: While:
                                                        Expr _id_ [383-384] [Type Bool]: Name: Local 45
                                                        Block 36 [328-376] [Type ()]:
                                                            Stmt 37 [346-362]: Local (Immutable):
                                                                Pat 38 [350-351] [Type String]: Bind: Ident 39 [350-351] "z"
                                                                Expr 40 [354-361] [Type String]: Lit: String("Third")
                                                            Stmt _id_ [383-384]: Semi: Expr _id_ [383-384] [Type ()]: Assign:
                                                                Expr _id_ [383-384] [Type Bool]: Name: Local 45
                                                                Expr _id_ [383-384] [Type Bool]: UnOp (NotL):
                                                                    Expr 41 [383-384] [Type Bool]: Name: Local 14"#]],
    );
}
