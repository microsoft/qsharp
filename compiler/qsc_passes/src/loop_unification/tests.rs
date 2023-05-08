// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

use crate::loop_unification::loop_unification;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
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
                    let x = "Hello World";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-133]:
                    Namespace (Ident 16 [10-14] "test"): Item 1
                Item 1 [21-131]:
                    Parent: 0
                    Callable 0 [21-131] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-48] [Type (Int)[]]: Paren:
                            Pat 3 [36-47] [Type (Int)[]]: Bind: Ident 4 [36-39] "arr"
                        output: ()
                        body: Block: Block 5 [56-131] [Type ()]:
                            Stmt 6 [66-125]: Expr: Expr _id_ [66-125] [Type ()]: Expr Block: Block _id_ [66-125] [Type ()]:
                                Stmt _id_ [75-78]: Local (Immutable):
                                    Pat _id_ [75-78] [Type (Int)[]]: Bind: Ident 17 [75-78] "array_id_17"
                                    Expr 10 [75-78] [Type (Int)[]]: Var: Local 4
                                Stmt _id_ [75-78]: Local (Immutable):
                                    Pat _id_ [75-78] [Type Int]: Bind: Ident 18 [75-78] "len_id_18"
                                    Expr _id_ [75-78] [Type Int]: Field:
                                        Expr _id_ [75-78] [Type (Int)[]]: Var: Local 17
                                        Length
                                Stmt _id_ [75-78]: Local (Mutable):
                                    Pat _id_ [75-78] [Type Int]: Bind: Ident 19 [75-78] "index_id_19"
                                    Expr _id_ [75-78] [Type Int]: Lit: Int(0)
                                Stmt _id_ [66-125]: Expr: Expr _id_ [66-125] [Type ()]: While:
                                    Expr _id_ [75-78] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [75-78] [Type Int]: Var: Local 19
                                        Expr _id_ [75-78] [Type Int]: Var: Local 18
                                    Block 11 [79-125] [Type ()]:
                                        Stmt _id_ [66-125]: Local (Immutable):
                                            Pat 8 [70-71] [Type Int]: Bind: Ident 9 [70-71] "i"
                                            Expr _id_ [75-78] [Type Int]: Index:
                                                Expr _id_ [75-78] [Type (Int)[]]: Var: Local 17
                                                Expr _id_ [75-78] [Type Int]: Var: Local 19
                                        Stmt 12 [93-115]: Local (Immutable):
                                            Pat 13 [97-98] [Type String]: Bind: Ident 14 [97-98] "x"
                                            Expr 15 [101-114] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [75-78]: Semi: Expr _id_ [75-78] [Type ()]: AssignOp (Add):
                                            Expr _id_ [75-78] [Type Int]: Var: Local 19
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
                    let x = "Hello World";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-148]:
                    Namespace (Ident 19 [10-14] "test"): Item 1
                Item 1 [21-146]:
                    Parent: 0
                    Callable 0 [21-146] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-58] [Type ((Int, Double))[]]: Paren:
                            Pat 3 [36-57] [Type ((Int, Double))[]]: Bind: Ident 4 [36-39] "arr"
                        output: ()
                        body: Block: Block 5 [66-146] [Type ()]:
                            Stmt 6 [76-140]: Expr: Expr _id_ [76-140] [Type ()]: Expr Block: Block _id_ [76-140] [Type ()]:
                                Stmt _id_ [90-93]: Local (Immutable):
                                    Pat _id_ [90-93] [Type ((Int, Double))[]]: Bind: Ident 20 [90-93] "array_id_20"
                                    Expr 13 [90-93] [Type ((Int, Double))[]]: Var: Local 4
                                Stmt _id_ [90-93]: Local (Immutable):
                                    Pat _id_ [90-93] [Type Int]: Bind: Ident 21 [90-93] "len_id_21"
                                    Expr _id_ [90-93] [Type Int]: Field:
                                        Expr _id_ [90-93] [Type ((Int, Double))[]]: Var: Local 20
                                        Length
                                Stmt _id_ [90-93]: Local (Mutable):
                                    Pat _id_ [90-93] [Type Int]: Bind: Ident 22 [90-93] "index_id_22"
                                    Expr _id_ [90-93] [Type Int]: Lit: Int(0)
                                Stmt _id_ [76-140]: Expr: Expr _id_ [76-140] [Type ()]: While:
                                    Expr _id_ [90-93] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [90-93] [Type Int]: Var: Local 22
                                        Expr _id_ [90-93] [Type Int]: Var: Local 21
                                    Block 14 [94-140] [Type ()]:
                                        Stmt _id_ [76-140]: Local (Immutable):
                                            Pat 8 [80-86] [Type (Int, Double)]: Tuple:
                                                Pat 9 [81-82] [Type Int]: Bind: Ident 10 [81-82] "i"
                                                Pat 11 [84-85] [Type Double]: Bind: Ident 12 [84-85] "d"
                                            Expr _id_ [90-93] [Type (Int, Double)]: Index:
                                                Expr _id_ [90-93] [Type ((Int, Double))[]]: Var: Local 20
                                                Expr _id_ [90-93] [Type Int]: Var: Local 22
                                        Stmt 15 [108-130]: Local (Immutable):
                                            Pat 16 [112-113] [Type String]: Bind: Ident 17 [112-113] "x"
                                            Expr 18 [116-129] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [90-93]: Semi: Expr _id_ [90-93] [Type ()]: AssignOp (Add):
                                            Expr _id_ [90-93] [Type Int]: Var: Local 22
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
                    let x = "Hello World";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-143]:
                    Namespace (Ident 22 [10-14] "test"): Item 1
                Item 1 [21-141]:
                    Parent: 0
                    Callable 0 [21-141] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-48] [Type (Int)[]]: Paren:
                            Pat 3 [36-47] [Type (Int)[]]: Bind: Ident 4 [36-39] "arr"
                        output: ()
                        body: Block: Block 5 [56-141] [Type ()]:
                            Stmt 6 [66-135]: Expr: Expr _id_ [66-135] [Type ()]: Expr Block: Block _id_ [66-135] [Type ()]:
                                Stmt _id_ [75-88]: Local (Immutable):
                                    Pat _id_ [75-88] [Type (Int)[]]: Bind: Ident 23 [75-88] "array_id_23"
                                    Expr 10 [75-88] [Type (Int)[]]: Index:
                                        Expr 11 [75-78] [Type (Int)[]]: Var: Local 4
                                        Expr 12 [79-87] [Type Range]: Range:
                                            Expr 13 [79-80] [Type Int]: Lit: Int(6)
                                            Expr 14 [82-84] [Type Int]: UnOp (Neg):
                                                Expr 15 [83-84] [Type Int]: Lit: Int(2)
                                            Expr 16 [86-87] [Type Int]: Lit: Int(2)
                                Stmt _id_ [75-88]: Local (Immutable):
                                    Pat _id_ [75-88] [Type Int]: Bind: Ident 24 [75-88] "len_id_24"
                                    Expr _id_ [75-88] [Type Int]: Field:
                                        Expr _id_ [75-88] [Type (Int)[]]: Var: Local 23
                                        Length
                                Stmt _id_ [75-88]: Local (Mutable):
                                    Pat _id_ [75-88] [Type Int]: Bind: Ident 25 [75-88] "index_id_25"
                                    Expr _id_ [75-88] [Type Int]: Lit: Int(0)
                                Stmt _id_ [66-135]: Expr: Expr _id_ [66-135] [Type ()]: While:
                                    Expr _id_ [75-88] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [75-88] [Type Int]: Var: Local 25
                                        Expr _id_ [75-88] [Type Int]: Var: Local 24
                                    Block 17 [89-135] [Type ()]:
                                        Stmt _id_ [66-135]: Local (Immutable):
                                            Pat 8 [70-71] [Type Int]: Bind: Ident 9 [70-71] "i"
                                            Expr _id_ [75-88] [Type Int]: Index:
                                                Expr _id_ [75-88] [Type (Int)[]]: Var: Local 23
                                                Expr _id_ [75-88] [Type Int]: Var: Local 25
                                        Stmt 18 [103-125]: Local (Immutable):
                                            Pat 19 [107-108] [Type String]: Bind: Ident 20 [107-108] "x"
                                            Expr 21 [111-124] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [75-88]: Semi: Expr _id_ [75-88] [Type ()]: AssignOp (Add):
                                            Expr _id_ [75-88] [Type Int]: Var: Local 25
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
                    let x = "Hello World";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-123]:
                    Namespace (Ident 16 [10-14] "test"): Item 1
                Item 1 [21-121]:
                    Parent: 0
                    Callable 0 [21-121] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-121] [Type ()]:
                            Stmt 4 [55-115]: Expr: Expr _id_ [55-115] [Type ()]: Expr Block: Block _id_ [55-115] [Type ()]:
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Range]: Bind: Ident 17 [64-68] "range_id_17"
                                    Expr 8 [64-68] [Type Range]: Range:
                                        Expr 9 [64-65] [Type Int]: Lit: Int(0)
                                        <no step>
                                        Expr 10 [67-68] [Type Int]: Lit: Int(4)
                                Stmt _id_ [64-68]: Local (Mutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 18 [64-68] "index_id_18"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Var: Local 17
                                        Start
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 19 [64-68] "step_id_19"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Var: Local 17
                                        Step
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 20 [64-68] "end_id_20"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Var: Local 17
                                        End
                                Stmt _id_ [55-115]: Expr: Expr _id_ [55-115] [Type ()]: While:
                                    Expr _id_ [64-68] [Type Bool]: BinOp (OrL):
                                        Expr _id_ [64-68] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Gt):
                                                Expr _id_ [64-68] [Type Int]: Var: Local 19
                                                Expr _id_ [64-68] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Lte):
                                                Expr _id_ [64-68] [Type Int]: Var: Local 18
                                                Expr _id_ [64-68] [Type Int]: Var: Local 20
                                        Expr _id_ [64-68] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Lt):
                                                Expr _id_ [64-68] [Type Int]: Var: Local 19
                                                Expr _id_ [64-68] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-68] [Type Bool]: BinOp (Gte):
                                                Expr _id_ [64-68] [Type Int]: Var: Local 18
                                                Expr _id_ [64-68] [Type Int]: Var: Local 20
                                    Block 11 [69-115] [Type ()]:
                                        Stmt _id_ [55-115]: Local (Immutable):
                                            Pat 6 [59-60] [Type Int]: Bind: Ident 7 [59-60] "i"
                                            Expr _id_ [64-68] [Type Int]: Var: Local 18
                                        Stmt 12 [83-105]: Local (Immutable):
                                            Pat 13 [87-88] [Type String]: Bind: Ident 14 [87-88] "x"
                                            Expr 15 [91-104] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [64-68]: Semi: Expr _id_ [64-68] [Type ()]: AssignOp (Add):
                                            Expr _id_ [64-68] [Type Int]: Var: Local 18
                                            Expr _id_ [64-68] [Type Int]: Var: Local 19"#]],
    );
}

#[test]
fn convert_for_reverse_range() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                for i in 4..-1..0 {
                    let x = "Hello World";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-127]:
                    Namespace (Ident 18 [10-14] "test"): Item 1
                Item 1 [21-125]:
                    Parent: 0
                    Callable 0 [21-125] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-125] [Type ()]:
                            Stmt 4 [55-119]: Expr: Expr _id_ [55-119] [Type ()]: Expr Block: Block _id_ [55-119] [Type ()]:
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Range]: Bind: Ident 19 [64-72] "range_id_19"
                                    Expr 8 [64-72] [Type Range]: Range:
                                        Expr 9 [64-65] [Type Int]: Lit: Int(4)
                                        Expr 10 [67-69] [Type Int]: UnOp (Neg):
                                            Expr 11 [68-69] [Type Int]: Lit: Int(1)
                                        Expr 12 [71-72] [Type Int]: Lit: Int(0)
                                Stmt _id_ [64-72]: Local (Mutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 20 [64-72] "index_id_20"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Var: Local 19
                                        Start
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 21 [64-72] "step_id_21"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Var: Local 19
                                        Step
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 22 [64-72] "end_id_22"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Var: Local 19
                                        End
                                Stmt _id_ [55-119]: Expr: Expr _id_ [55-119] [Type ()]: While:
                                    Expr _id_ [64-72] [Type Bool]: BinOp (OrL):
                                        Expr _id_ [64-72] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Gt):
                                                Expr _id_ [64-72] [Type Int]: Var: Local 21
                                                Expr _id_ [64-72] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Lte):
                                                Expr _id_ [64-72] [Type Int]: Var: Local 20
                                                Expr _id_ [64-72] [Type Int]: Var: Local 22
                                        Expr _id_ [64-72] [Type Bool]: BinOp (AndL):
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Lt):
                                                Expr _id_ [64-72] [Type Int]: Var: Local 21
                                                Expr _id_ [64-72] [Type Int]: Lit: Int(0)
                                            Expr _id_ [64-72] [Type Bool]: BinOp (Gte):
                                                Expr _id_ [64-72] [Type Int]: Var: Local 20
                                                Expr _id_ [64-72] [Type Int]: Var: Local 22
                                    Block 13 [73-119] [Type ()]:
                                        Stmt _id_ [55-119]: Local (Immutable):
                                            Pat 6 [59-60] [Type Int]: Bind: Ident 7 [59-60] "i"
                                            Expr _id_ [64-72] [Type Int]: Var: Local 20
                                        Stmt 14 [87-109]: Local (Immutable):
                                            Pat 15 [91-92] [Type String]: Bind: Ident 16 [91-92] "x"
                                            Expr 17 [95-108] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [64-72]: Semi: Expr _id_ [64-72] [Type ()]: AssignOp (Add):
                                            Expr _id_ [64-72] [Type Int]: Var: Local 20
                                            Expr _id_ [64-72] [Type Int]: Var: Local 21"#]],
    );
}

#[test]
fn convert_repeat() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                repeat {
                    let x = "Hello World";
                } until true;
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-128]:
                    Namespace (Ident 12 [10-14] "test"): Item 1
                Item 1 [21-126]:
                    Parent: 0
                    Callable 0 [21-126] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-126] [Type ()]:
                            Stmt 4 [55-120]: Semi: Expr _id_ [55-119] [Type ()]: Expr Block: Block _id_ [55-119] [Type ()]:
                                Stmt _id_ [115-119]: Local (Mutable):
                                    Pat _id_ [115-119] [Type Bool]: Bind: Ident 13 [115-119] "continue_cond_13"
                                    Expr _id_ [115-119] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [55-119]: Expr: Expr _id_ [55-119] [Type ()]: While:
                                    Expr _id_ [115-119] [Type Bool]: Var: Local 13
                                    Block 6 [62-108] [Type ()]:
                                        Stmt 7 [76-98]: Local (Immutable):
                                            Pat 8 [80-81] [Type String]: Bind: Ident 9 [80-81] "x"
                                            Expr 10 [84-97] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [115-119]: Semi: Expr _id_ [115-119] [Type ()]: Assign:
                                            Expr _id_ [115-119] [Type Bool]: Var: Local 13
                                            Expr _id_ [115-119] [Type Bool]: UnOp (NotL):
                                                Expr 11 [115-119] [Type Bool]: Lit: Bool(true)"#]],
    );
}

#[test]
fn convert_repeat_fixup() {
    check(
        indoc! {r#"
        namespace test {
            operation Main() : Unit {
                repeat {
                    let x = "Hello World";
                } until true
                fixup {
                    let y = "Fixup";
                }
            }
        }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-182]:
                    Namespace (Ident 17 [10-14] "test"): Item 1
                Item 1 [21-180]:
                    Parent: 0
                    Callable 0 [21-180] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [45-180] [Type ()]:
                            Stmt 4 [55-174]: Expr: Expr _id_ [55-174] [Type ()]: Expr Block: Block _id_ [55-174] [Type ()]:
                                Stmt _id_ [115-119]: Local (Mutable):
                                    Pat _id_ [115-119] [Type Bool]: Bind: Ident 18 [115-119] "continue_cond_18"
                                    Expr _id_ [115-119] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [55-174]: Expr: Expr _id_ [55-174] [Type ()]: While:
                                    Expr _id_ [115-119] [Type Bool]: Var: Local 18
                                    Block 6 [62-108] [Type ()]:
                                        Stmt 7 [76-98]: Local (Immutable):
                                            Pat 8 [80-81] [Type String]: Bind: Ident 9 [80-81] "x"
                                            Expr 10 [84-97] [Type String]: Lit: String("Hello World")
                                        Stmt _id_ [115-119]: Semi: Expr _id_ [115-119] [Type ()]: Assign:
                                            Expr _id_ [115-119] [Type Bool]: Var: Local 18
                                            Expr _id_ [115-119] [Type Bool]: UnOp (NotL):
                                                Expr 11 [115-119] [Type Bool]: Lit: Bool(true)
                                        Stmt _id_ [134-174]: Expr: Expr _id_ [134-174] [Type ()]: If:
                                            Expr _id_ [115-119] [Type Bool]: Var: Local 18
                                            Block 12 [134-174] [Type ()]:
                                                Stmt 13 [148-164]: Local (Immutable):
                                                    Pat 14 [152-153] [Type String]: Bind: Ident 15 [152-153] "y"
                                                    Expr 16 [156-163] [Type String]: Lit: String("Fixup")"#]],
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
                            Stmt 16 [122-395]: Expr: Expr _id_ [122-395] [Type ()]: Expr Block: Block _id_ [122-395] [Type ()]:
                                Stmt _id_ [291-292]: Local (Mutable):
                                    Pat _id_ [291-292] [Type Bool]: Bind: Ident 45 [291-292] "continue_cond_45"
                                    Expr _id_ [291-292] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [122-395]: Expr: Expr _id_ [122-395] [Type ()]: While:
                                    Expr _id_ [291-292] [Type Bool]: Var: Local 45
                                    Block 18 [129-284] [Type ()]:
                                        Stmt 19 [143-274]: Expr: Expr _id_ [143-274] [Type ()]: Expr Block: Block _id_ [143-274] [Type ()]:
                                            Stmt _id_ [205-206]: Local (Mutable):
                                                Pat _id_ [205-206] [Type Bool]: Bind: Ident 43 [205-206] "continue_cond_43"
                                                Expr _id_ [205-206] [Type Bool]: Lit: Bool(true)
                                            Stmt _id_ [143-274]: Expr: Expr _id_ [143-274] [Type ()]: While:
                                                Expr _id_ [205-206] [Type Bool]: Var: Local 43
                                                Block 21 [150-198] [Type ()]:
                                                    Stmt 22 [168-184]: Local (Immutable):
                                                        Pat 23 [172-173] [Type String]: Bind: Ident 24 [172-173] "x"
                                                        Expr 25 [176-183] [Type String]: Lit: String("First")
                                                    Stmt _id_ [205-206]: Semi: Expr _id_ [205-206] [Type ()]: Assign:
                                                        Expr _id_ [205-206] [Type Bool]: Var: Local 43
                                                        Expr _id_ [205-206] [Type Bool]: UnOp (NotL):
                                                            Expr 26 [205-206] [Type Bool]: Var: Local 6
                                                    Stmt _id_ [225-274]: Expr: Expr _id_ [225-274] [Type ()]: If:
                                                        Expr _id_ [205-206] [Type Bool]: Var: Local 43
                                                        Block 27 [225-274] [Type ()]:
                                                            Stmt 28 [243-260]: Local (Immutable):
                                                                Pat 29 [247-248] [Type String]: Bind: Ident 30 [247-248] "y"
                                                                Expr 31 [251-259] [Type String]: Lit: String("Second")
                                        Stmt _id_ [291-292]: Semi: Expr _id_ [291-292] [Type ()]: Assign:
                                            Expr _id_ [291-292] [Type Bool]: Var: Local 45
                                            Expr _id_ [291-292] [Type Bool]: UnOp (NotL):
                                                Expr 32 [291-292] [Type Bool]: Var: Local 10
                                        Stmt _id_ [307-395]: Expr: Expr _id_ [307-395] [Type ()]: If:
                                            Expr _id_ [291-292] [Type Bool]: Var: Local 45
                                            Block 33 [307-395] [Type ()]:
                                                Stmt 34 [321-385]: Semi: Expr _id_ [321-384] [Type ()]: Expr Block: Block _id_ [321-384] [Type ()]:
                                                    Stmt _id_ [383-384]: Local (Mutable):
                                                        Pat _id_ [383-384] [Type Bool]: Bind: Ident 44 [383-384] "continue_cond_44"
                                                        Expr _id_ [383-384] [Type Bool]: Lit: Bool(true)
                                                    Stmt _id_ [321-384]: Expr: Expr _id_ [321-384] [Type ()]: While:
                                                        Expr _id_ [383-384] [Type Bool]: Var: Local 44
                                                        Block 36 [328-376] [Type ()]:
                                                            Stmt 37 [346-362]: Local (Immutable):
                                                                Pat 38 [350-351] [Type String]: Bind: Ident 39 [350-351] "z"
                                                                Expr 40 [354-361] [Type String]: Lit: String("Third")
                                                            Stmt _id_ [383-384]: Semi: Expr _id_ [383-384] [Type ()]: Assign:
                                                                Expr _id_ [383-384] [Type Bool]: Var: Local 44
                                                                Expr _id_ [383-384] [Type Bool]: UnOp (NotL):
                                                                    Expr 41 [383-384] [Type Bool]: Var: Local 14"#]],
    );
}
