// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};
use qsc_hir::mut_visit::MutVisitor;

use crate::loop_unification::LoopUni;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    LoopUni {
        core: store.core(),
        assigner: &mut unit.assigner,
    }
    .visit_package(&mut unit.package);
    expect.assert_eq(&unit.package.to_string());
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
                Item 0 [0-133] (Public):
                    Namespace (Ident 15 [10-14] "test"): Item 1
                Item 1 [21-131] (Public):
                    Parent: 0
                    Callable 0 [21-131] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [36-47] [Type (Int)[]]: Bind: Ident 3 [36-39] "arr"
                        output: Unit
                        functors: empty set
                        body: Block: Block 4 [56-131] [Type Unit]:
                            Stmt 5 [66-125]: Expr: Expr _id_ [66-125] [Type Unit]: Expr Block: Block _id_ [66-125] [Type Unit]:
                                Stmt _id_ [75-78]: Local (Immutable):
                                    Pat _id_ [75-78] [Type (Int)[]]: Bind: Ident 16 [75-78] "array_id_16"
                                    Expr 9 [75-78] [Type (Int)[]]: Var: Local 3
                                Stmt _id_ [75-78]: Local (Immutable):
                                    Pat _id_ [75-78] [Type Int]: Bind: Ident 17 [75-78] "len_id_17"
                                    Expr _id_ [75-78] [Type (Int)[]]: Call:
                                        Expr _id_ [75-78] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                        Expr _id_ [75-78] [Type (Int)[]]: Var: Local 16
                                Stmt _id_ [75-78]: Local (Mutable):
                                    Pat _id_ [75-78] [Type Int]: Bind: Ident 18 [75-78] "index_id_18"
                                    Expr _id_ [75-78] [Type Int]: Lit: Int(0)
                                Stmt _id_ [66-125]: Expr: Expr _id_ [66-125] [Type Unit]: While:
                                    Expr _id_ [75-78] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [75-78] [Type Int]: Var: Local 18
                                        Expr _id_ [75-78] [Type Int]: Var: Local 17
                                    Block 10 [79-125] [Type Unit]:
                                        Stmt _id_ [66-125]: Local (Immutable):
                                            Pat 7 [70-71] [Type Int]: Bind: Ident 8 [70-71] "i"
                                            Expr _id_ [75-78] [Type Int]: Index:
                                                Expr _id_ [75-78] [Type (Int)[]]: Var: Local 16
                                                Expr _id_ [75-78] [Type Int]: Var: Local 18
                                        Stmt 11 [93-115]: Local (Immutable):
                                            Pat 12 [97-98] [Type String]: Bind: Ident 13 [97-98] "x"
                                            Expr 14 [101-114] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [75-78]: Semi: Expr _id_ [75-78] [Type Unit]: AssignOp (Add):
                                            Expr _id_ [75-78] [Type Int]: Var: Local 18
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
                Item 0 [0-148] (Public):
                    Namespace (Ident 18 [10-14] "test"): Item 1
                Item 1 [21-146] (Public):
                    Parent: 0
                    Callable 0 [21-146] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [36-57] [Type ((Int, Double))[]]: Bind: Ident 3 [36-39] "arr"
                        output: Unit
                        functors: empty set
                        body: Block: Block 4 [66-146] [Type Unit]:
                            Stmt 5 [76-140]: Expr: Expr _id_ [76-140] [Type Unit]: Expr Block: Block _id_ [76-140] [Type Unit]:
                                Stmt _id_ [90-93]: Local (Immutable):
                                    Pat _id_ [90-93] [Type ((Int, Double))[]]: Bind: Ident 19 [90-93] "array_id_19"
                                    Expr 12 [90-93] [Type ((Int, Double))[]]: Var: Local 3
                                Stmt _id_ [90-93]: Local (Immutable):
                                    Pat _id_ [90-93] [Type Int]: Bind: Ident 20 [90-93] "len_id_20"
                                    Expr _id_ [90-93] [Type ((Int, Double))[]]: Call:
                                        Expr _id_ [90-93] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                        Expr _id_ [90-93] [Type ((Int, Double))[]]: Var: Local 19
                                Stmt _id_ [90-93]: Local (Mutable):
                                    Pat _id_ [90-93] [Type Int]: Bind: Ident 21 [90-93] "index_id_21"
                                    Expr _id_ [90-93] [Type Int]: Lit: Int(0)
                                Stmt _id_ [76-140]: Expr: Expr _id_ [76-140] [Type Unit]: While:
                                    Expr _id_ [90-93] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [90-93] [Type Int]: Var: Local 21
                                        Expr _id_ [90-93] [Type Int]: Var: Local 20
                                    Block 13 [94-140] [Type Unit]:
                                        Stmt _id_ [76-140]: Local (Immutable):
                                            Pat 7 [80-86] [Type (Int, Double)]: Tuple:
                                                Pat 8 [81-82] [Type Int]: Bind: Ident 9 [81-82] "i"
                                                Pat 10 [84-85] [Type Double]: Bind: Ident 11 [84-85] "d"
                                            Expr _id_ [90-93] [Type (Int, Double)]: Index:
                                                Expr _id_ [90-93] [Type ((Int, Double))[]]: Var: Local 19
                                                Expr _id_ [90-93] [Type Int]: Var: Local 21
                                        Stmt 14 [108-130]: Local (Immutable):
                                            Pat 15 [112-113] [Type String]: Bind: Ident 16 [112-113] "x"
                                            Expr 17 [116-129] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [90-93]: Semi: Expr _id_ [90-93] [Type Unit]: AssignOp (Add):
                                            Expr _id_ [90-93] [Type Int]: Var: Local 21
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
                Item 0 [0-143] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1
                Item 1 [21-141] (Public):
                    Parent: 0
                    Callable 0 [21-141] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [36-47] [Type (Int)[]]: Bind: Ident 3 [36-39] "arr"
                        output: Unit
                        functors: empty set
                        body: Block: Block 4 [56-141] [Type Unit]:
                            Stmt 5 [66-135]: Expr: Expr _id_ [66-135] [Type Unit]: Expr Block: Block _id_ [66-135] [Type Unit]:
                                Stmt _id_ [75-88]: Local (Immutable):
                                    Pat _id_ [75-88] [Type (Int)[]]: Bind: Ident 22 [75-88] "array_id_22"
                                    Expr 9 [75-88] [Type (Int)[]]: Index:
                                        Expr 10 [75-78] [Type (Int)[]]: Var: Local 3
                                        Expr 11 [79-87] [Type Range]: Range:
                                            Expr 12 [79-80] [Type Int]: Lit: Int(6)
                                            Expr 13 [82-84] [Type Int]: UnOp (Neg):
                                                Expr 14 [83-84] [Type Int]: Lit: Int(2)
                                            Expr 15 [86-87] [Type Int]: Lit: Int(2)
                                Stmt _id_ [75-88]: Local (Immutable):
                                    Pat _id_ [75-88] [Type Int]: Bind: Ident 23 [75-88] "len_id_23"
                                    Expr _id_ [75-88] [Type (Int)[]]: Call:
                                        Expr _id_ [75-88] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                        Expr _id_ [75-88] [Type (Int)[]]: Var: Local 22
                                Stmt _id_ [75-88]: Local (Mutable):
                                    Pat _id_ [75-88] [Type Int]: Bind: Ident 24 [75-88] "index_id_24"
                                    Expr _id_ [75-88] [Type Int]: Lit: Int(0)
                                Stmt _id_ [66-135]: Expr: Expr _id_ [66-135] [Type Unit]: While:
                                    Expr _id_ [75-88] [Type Bool]: BinOp (Lt):
                                        Expr _id_ [75-88] [Type Int]: Var: Local 24
                                        Expr _id_ [75-88] [Type Int]: Var: Local 23
                                    Block 16 [89-135] [Type Unit]:
                                        Stmt _id_ [66-135]: Local (Immutable):
                                            Pat 7 [70-71] [Type Int]: Bind: Ident 8 [70-71] "i"
                                            Expr _id_ [75-88] [Type Int]: Index:
                                                Expr _id_ [75-88] [Type (Int)[]]: Var: Local 22
                                                Expr _id_ [75-88] [Type Int]: Var: Local 24
                                        Stmt 17 [103-125]: Local (Immutable):
                                            Pat 18 [107-108] [Type String]: Bind: Ident 19 [107-108] "x"
                                            Expr 20 [111-124] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [75-88]: Semi: Expr _id_ [75-88] [Type Unit]: AssignOp (Add):
                                            Expr _id_ [75-88] [Type Int]: Var: Local 24
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
                Item 0 [0-123] (Public):
                    Namespace (Ident 16 [10-14] "test"): Item 1
                Item 1 [21-121] (Public):
                    Parent: 0
                    Callable 0 [21-121] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 3 [45-121] [Type Unit]:
                            Stmt 4 [55-115]: Expr: Expr _id_ [55-115] [Type Unit]: Expr Block: Block _id_ [55-115] [Type Unit]:
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
                                        Prim(Start)
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 19 [64-68] "step_id_19"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Var: Local 17
                                        Prim(Step)
                                Stmt _id_ [64-68]: Local (Immutable):
                                    Pat _id_ [64-68] [Type Int]: Bind: Ident 20 [64-68] "end_id_20"
                                    Expr _id_ [64-68] [Type Int]: Field:
                                        Expr _id_ [64-68] [Type Range]: Var: Local 17
                                        Prim(End)
                                Stmt _id_ [55-115]: Expr: Expr _id_ [55-115] [Type Unit]: While:
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
                                    Block 11 [69-115] [Type Unit]:
                                        Stmt _id_ [55-115]: Local (Immutable):
                                            Pat 6 [59-60] [Type Int]: Bind: Ident 7 [59-60] "i"
                                            Expr _id_ [64-68] [Type Int]: Var: Local 18
                                        Stmt 12 [83-105]: Local (Immutable):
                                            Pat 13 [87-88] [Type String]: Bind: Ident 14 [87-88] "x"
                                            Expr 15 [91-104] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [64-68]: Semi: Expr _id_ [64-68] [Type Unit]: AssignOp (Add):
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
                Item 0 [0-127] (Public):
                    Namespace (Ident 18 [10-14] "test"): Item 1
                Item 1 [21-125] (Public):
                    Parent: 0
                    Callable 0 [21-125] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 3 [45-125] [Type Unit]:
                            Stmt 4 [55-119]: Expr: Expr _id_ [55-119] [Type Unit]: Expr Block: Block _id_ [55-119] [Type Unit]:
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
                                        Prim(Start)
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 21 [64-72] "step_id_21"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Var: Local 19
                                        Prim(Step)
                                Stmt _id_ [64-72]: Local (Immutable):
                                    Pat _id_ [64-72] [Type Int]: Bind: Ident 22 [64-72] "end_id_22"
                                    Expr _id_ [64-72] [Type Int]: Field:
                                        Expr _id_ [64-72] [Type Range]: Var: Local 19
                                        Prim(End)
                                Stmt _id_ [55-119]: Expr: Expr _id_ [55-119] [Type Unit]: While:
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
                                    Block 13 [73-119] [Type Unit]:
                                        Stmt _id_ [55-119]: Local (Immutable):
                                            Pat 6 [59-60] [Type Int]: Bind: Ident 7 [59-60] "i"
                                            Expr _id_ [64-72] [Type Int]: Var: Local 20
                                        Stmt 14 [87-109]: Local (Immutable):
                                            Pat 15 [91-92] [Type String]: Bind: Ident 16 [91-92] "x"
                                            Expr 17 [95-108] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [64-72]: Semi: Expr _id_ [64-72] [Type Unit]: AssignOp (Add):
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
                Item 0 [0-128] (Public):
                    Namespace (Ident 12 [10-14] "test"): Item 1
                Item 1 [21-126] (Public):
                    Parent: 0
                    Callable 0 [21-126] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 3 [45-126] [Type Unit]:
                            Stmt 4 [55-120]: Semi: Expr _id_ [55-119] [Type Unit]: Expr Block: Block _id_ [55-119] [Type Unit]:
                                Stmt _id_ [115-119]: Local (Mutable):
                                    Pat _id_ [115-119] [Type Bool]: Bind: Ident 13 [115-119] "continue_cond_13"
                                    Expr _id_ [115-119] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [55-119]: Expr: Expr _id_ [55-119] [Type Unit]: While:
                                    Expr _id_ [115-119] [Type Bool]: Var: Local 13
                                    Block 6 [62-108] [Type Unit]:
                                        Stmt 7 [76-98]: Local (Immutable):
                                            Pat 8 [80-81] [Type String]: Bind: Ident 9 [80-81] "x"
                                            Expr 10 [84-97] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [115-119]: Semi: Expr _id_ [115-119] [Type Unit]: Assign:
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
                Item 0 [0-182] (Public):
                    Namespace (Ident 17 [10-14] "test"): Item 1
                Item 1 [21-180] (Public):
                    Parent: 0
                    Callable 0 [21-180] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 3 [45-180] [Type Unit]:
                            Stmt 4 [55-174]: Expr: Expr _id_ [55-174] [Type Unit]: Expr Block: Block _id_ [55-174] [Type Unit]:
                                Stmt _id_ [115-119]: Local (Mutable):
                                    Pat _id_ [115-119] [Type Bool]: Bind: Ident 18 [115-119] "continue_cond_18"
                                    Expr _id_ [115-119] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [55-174]: Expr: Expr _id_ [55-174] [Type Unit]: While:
                                    Expr _id_ [115-119] [Type Bool]: Var: Local 18
                                    Block 6 [62-108] [Type Unit]:
                                        Stmt 7 [76-98]: Local (Immutable):
                                            Pat 8 [80-81] [Type String]: Bind: Ident 9 [80-81] "x"
                                            Expr 10 [84-97] [Type String]: String:
                                                Lit: "Hello World"
                                        Stmt _id_ [115-119]: Semi: Expr _id_ [115-119] [Type Unit]: Assign:
                                            Expr _id_ [115-119] [Type Bool]: Var: Local 18
                                            Expr _id_ [115-119] [Type Bool]: UnOp (NotL):
                                                Expr 11 [115-119] [Type Bool]: Lit: Bool(true)
                                        Stmt _id_ [134-174]: Expr: Expr _id_ [134-174] [Type Unit]: If:
                                            Expr _id_ [115-119] [Type Bool]: Var: Local 18
                                            Block 12 [134-174] [Type Unit]:
                                                Stmt 13 [148-164]: Local (Immutable):
                                                    Pat 14 [152-153] [Type String]: Bind: Ident 15 [152-153] "y"
                                                    Expr 16 [156-163] [Type String]: String:
                                                        Lit: "Fixup""#]],
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
                Item 0 [0-403] (Public):
                    Namespace (Ident 42 [10-14] "test"): Item 1
                Item 1 [21-401] (Public):
                    Parent: 0
                    Callable 0 [21-401] (Operation):
                        name: Ident 1 [31-35] "Main"
                        input: Pat 2 [35-37] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 3 [45-401] [Type Unit]:
                            Stmt 4 [55-68]: Local (Immutable):
                                Pat 5 [59-60] [Type Bool]: Bind: Ident 6 [59-60] "a"
                                Expr 7 [63-67] [Type Bool]: Lit: Bool(true)
                            Stmt 8 [77-91]: Local (Immutable):
                                Pat 9 [81-82] [Type Bool]: Bind: Ident 10 [81-82] "b"
                                Expr 11 [85-90] [Type Bool]: Lit: Bool(false)
                            Stmt 12 [100-113]: Local (Immutable):
                                Pat 13 [104-105] [Type Bool]: Bind: Ident 14 [104-105] "c"
                                Expr 15 [108-112] [Type Bool]: Lit: Bool(true)
                            Stmt 16 [122-395]: Expr: Expr _id_ [122-395] [Type Unit]: Expr Block: Block _id_ [122-395] [Type Unit]:
                                Stmt _id_ [291-292]: Local (Mutable):
                                    Pat _id_ [291-292] [Type Bool]: Bind: Ident 45 [291-292] "continue_cond_45"
                                    Expr _id_ [291-292] [Type Bool]: Lit: Bool(true)
                                Stmt _id_ [122-395]: Expr: Expr _id_ [122-395] [Type Unit]: While:
                                    Expr _id_ [291-292] [Type Bool]: Var: Local 45
                                    Block 18 [129-284] [Type Unit]:
                                        Stmt 19 [143-274]: Expr: Expr _id_ [143-274] [Type Unit]: Expr Block: Block _id_ [143-274] [Type Unit]:
                                            Stmt _id_ [205-206]: Local (Mutable):
                                                Pat _id_ [205-206] [Type Bool]: Bind: Ident 43 [205-206] "continue_cond_43"
                                                Expr _id_ [205-206] [Type Bool]: Lit: Bool(true)
                                            Stmt _id_ [143-274]: Expr: Expr _id_ [143-274] [Type Unit]: While:
                                                Expr _id_ [205-206] [Type Bool]: Var: Local 43
                                                Block 21 [150-198] [Type Unit]:
                                                    Stmt 22 [168-184]: Local (Immutable):
                                                        Pat 23 [172-173] [Type String]: Bind: Ident 24 [172-173] "x"
                                                        Expr 25 [176-183] [Type String]: String:
                                                            Lit: "First"
                                                    Stmt _id_ [205-206]: Semi: Expr _id_ [205-206] [Type Unit]: Assign:
                                                        Expr _id_ [205-206] [Type Bool]: Var: Local 43
                                                        Expr _id_ [205-206] [Type Bool]: UnOp (NotL):
                                                            Expr 26 [205-206] [Type Bool]: Var: Local 6
                                                    Stmt _id_ [225-274]: Expr: Expr _id_ [225-274] [Type Unit]: If:
                                                        Expr _id_ [205-206] [Type Bool]: Var: Local 43
                                                        Block 27 [225-274] [Type Unit]:
                                                            Stmt 28 [243-260]: Local (Immutable):
                                                                Pat 29 [247-248] [Type String]: Bind: Ident 30 [247-248] "y"
                                                                Expr 31 [251-259] [Type String]: String:
                                                                    Lit: "Second"
                                        Stmt _id_ [291-292]: Semi: Expr _id_ [291-292] [Type Unit]: Assign:
                                            Expr _id_ [291-292] [Type Bool]: Var: Local 45
                                            Expr _id_ [291-292] [Type Bool]: UnOp (NotL):
                                                Expr 32 [291-292] [Type Bool]: Var: Local 10
                                        Stmt _id_ [307-395]: Expr: Expr _id_ [307-395] [Type Unit]: If:
                                            Expr _id_ [291-292] [Type Bool]: Var: Local 45
                                            Block 33 [307-395] [Type Unit]:
                                                Stmt 34 [321-385]: Semi: Expr _id_ [321-384] [Type Unit]: Expr Block: Block _id_ [321-384] [Type Unit]:
                                                    Stmt _id_ [383-384]: Local (Mutable):
                                                        Pat _id_ [383-384] [Type Bool]: Bind: Ident 44 [383-384] "continue_cond_44"
                                                        Expr _id_ [383-384] [Type Bool]: Lit: Bool(true)
                                                    Stmt _id_ [321-384]: Expr: Expr _id_ [321-384] [Type Unit]: While:
                                                        Expr _id_ [383-384] [Type Bool]: Var: Local 44
                                                        Block 36 [328-376] [Type Unit]:
                                                            Stmt 37 [346-362]: Local (Immutable):
                                                                Pat 38 [350-351] [Type String]: Bind: Ident 39 [350-351] "z"
                                                                Expr 40 [354-361] [Type String]: String:
                                                                    Lit: "Third"
                                                            Stmt _id_ [383-384]: Semi: Expr _id_ [383-384] [Type Unit]: Assign:
                                                                Expr _id_ [383-384] [Type Bool]: Var: Local 44
                                                                Expr _id_ [383-384] [Type Bool]: UnOp (NotL):
                                                                    Expr 41 [383-384] [Type Bool]: Var: Local 14"#]],
    );
}
