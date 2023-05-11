// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

use crate::conjugate_invert::invert_conjugate_exprs;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources);
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);

    let errors = invert_conjugate_exprs(&mut unit);
    if errors.is_empty() {
        expect.assert_eq(&unit.package.to_string());
    } else {
        expect.assert_debug_eq(&errors);
    }
}

#[test]
fn conjugate_invert() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit is Adj {}
                operation A() : Unit {
                    within {
                        B(1);
                        B(2);
                    }
                    apply {
                        B(3);
                        B(4);
                    }
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-218] (Public):
                    Namespace (Ident 30 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: ()
                        functors: Functor Expr 4 [52-55]: Adj
                        body: Block: Block 5 [56-58]: <empty>
                Item 2 [63-216] (Public):
                    Parent: 0
                    Callable 6 [63-216] (Operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type ()]: Unit
                        output: ()
                        body: Block: Block 9 [84-216] [Type ()]:
                            Stmt 10 [94-210]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 12 [101-148] [Type ()]:
                                    Stmt 13 [115-120]: Semi: Expr 14 [115-119] [Type ()]: Call:
                                        Expr 15 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 16 [117-118] [Type Int]: Lit: Int(1)
                                    Stmt 17 [133-138]: Semi: Expr 18 [133-137] [Type ()]: Call:
                                        Expr 19 [133-134] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 20 [135-136] [Type Int]: Lit: Int(2)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 21 [163-210] [Type ()]:
                                    Stmt 22 [177-182]: Semi: Expr 23 [177-181] [Type ()]: Call:
                                        Expr 24 [177-178] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 25 [179-180] [Type Int]: Lit: Int(3)
                                    Stmt 26 [195-200]: Semi: Expr 27 [195-199] [Type ()]: Call:
                                        Expr 28 [195-196] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 29 [197-198] [Type Int]: Lit: Int(4)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 12 [101-148] [Type ()]:
                                    Stmt 17 [133-138]: Semi: Expr 18 [133-137] [Type ()]: Call:
                                        Expr _id_ [133-134] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 19 [133-134] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 20 [135-136] [Type Int]: Lit: Int(2)
                                    Stmt 13 [115-120]: Semi: Expr 14 [115-119] [Type ()]: Call:
                                        Expr _id_ [115-116] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 15 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 16 [117-118] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn nested_conjugate_invert() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit is Adj {}
                operation A() : Unit {
                    within {
                        B(0);
                        within {
                            B(1);
                            B(2);
                        }
                        apply {
                            B(3);
                            B(4);
                        }
                    }
                    apply {
                        B(5);
                        B(6);
                    }
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-357] (Public):
                    Namespace (Ident 46 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: ()
                        functors: Functor Expr 4 [52-55]: Adj
                        body: Block: Block 5 [56-58]: <empty>
                Item 2 [63-355] (Public):
                    Parent: 0
                    Callable 6 [63-355] (Operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type ()]: Unit
                        output: ()
                        body: Block: Block 9 [84-355] [Type ()]:
                            Stmt 10 [94-349]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 12 [101-287] [Type ()]:
                                    Stmt 13 [115-120]: Semi: Expr 14 [115-119] [Type ()]: Call:
                                        Expr 15 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 16 [117-118] [Type Int]: Lit: Int(0)
                                    Stmt 17 [133-277]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 19 [140-199] [Type ()]:
                                            Stmt 20 [158-163]: Semi: Expr 21 [158-162] [Type ()]: Call:
                                                Expr 22 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 23 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt 24 [180-185]: Semi: Expr 25 [180-184] [Type ()]: Call:
                                                Expr 26 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [182-183] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 28 [218-277] [Type ()]:
                                            Stmt 29 [236-241]: Semi: Expr 30 [236-240] [Type ()]: Call:
                                                Expr 31 [236-237] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 32 [238-239] [Type Int]: Lit: Int(3)
                                            Stmt 33 [258-263]: Semi: Expr 34 [258-262] [Type ()]: Call:
                                                Expr 35 [258-259] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 36 [260-261] [Type Int]: Lit: Int(4)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 19 [140-199] [Type ()]:
                                            Stmt 24 [180-185]: Semi: Expr 25 [180-184] [Type ()]: Call:
                                                Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 26 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 20 [158-163]: Semi: Expr 21 [158-162] [Type ()]: Call:
                                                Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 22 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 23 [160-161] [Type Int]: Lit: Int(1)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 37 [302-349] [Type ()]:
                                    Stmt 38 [316-321]: Semi: Expr 39 [316-320] [Type ()]: Call:
                                        Expr 40 [316-317] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 41 [318-319] [Type Int]: Lit: Int(5)
                                    Stmt 42 [334-339]: Semi: Expr 43 [334-338] [Type ()]: Call:
                                        Expr 44 [334-335] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 45 [336-337] [Type Int]: Lit: Int(6)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 12 [101-287] [Type ()]:
                                    Stmt 17 [133-277]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 19 [140-199] [Type ()]:
                                            Stmt 20 [158-163]: Semi: Expr 21 [158-162] [Type ()]: Call:
                                                Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 22 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 23 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt 24 [180-185]: Semi: Expr 25 [180-184] [Type ()]: Call:
                                                Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 26 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [182-183] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 28 [218-277] [Type ()]:
                                            Stmt 33 [258-263]: Semi: Expr 34 [258-262] [Type ()]: Call:
                                                Expr _id_ [258-259] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 35 [258-259] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 36 [260-261] [Type Int]: Lit: Int(4)
                                            Stmt 29 [236-241]: Semi: Expr 30 [236-240] [Type ()]: Call:
                                                Expr _id_ [236-237] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 31 [236-237] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 32 [238-239] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 19 [140-199] [Type ()]:
                                            Stmt 24 [180-185]: Semi: Expr 25 [180-184] [Type ()]: Call:
                                                Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 26 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 20 [158-163]: Semi: Expr 21 [158-162] [Type ()]: Call:
                                                Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 22 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 23 [160-161] [Type Int]: Lit: Int(1)
                                    Stmt 13 [115-120]: Semi: Expr 14 [115-119] [Type ()]: Call:
                                        Expr _id_ [115-116] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 15 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 16 [117-118] [Type Int]: Lit: Int(0)"#]],
    );
}

#[test]
fn conjugate_invalid_op_fails() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit {}
                operation A() : Unit {
                    within {
                        B(1);
                        B(2);
                    }
                    apply {
                        B(3);
                        B(4);
                    }
                }
            }
        "},
        &expect![[r#"
            [
                AdjGen(
                    MissingAdjFunctor(
                        Span {
                            lo: 126,
                            hi: 127,
                        },
                    ),
                ),
                AdjGen(
                    MissingAdjFunctor(
                        Span {
                            lo: 108,
                            hi: 109,
                        },
                    ),
                ),
            ]
        "#]],
    );
}

#[test]
fn conjugate_not_separable_fail() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit is Adj {}
                operation A() : Unit {
                    within {
                        let x = B(1);
                        B(2);
                    }
                    apply {
                        B(3);
                        B(4);
                    }
                }
            }
        "},
        &expect![[r#"
            [
                AdjGen(
                    LogicSep(
                        OpCallForbidden(
                            Span {
                                lo: 123,
                                hi: 127,
                            },
                        ),
                    ),
                ),
            ]
        "#]],
    );
}

#[test]
fn conjugate_mutable_update_in_apply_fail() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit is Adj {}
                operation A() : Unit {
                    mutable a = 1;
                    within {
                        let x = a;
                        B(2);
                    }
                    apply {
                        set a = 3;
                        B(4);
                    }
                }
            }
        "},
        &expect![[r#"
            [
                ApplyAssign(
                    Span {
                        lo: 209,
                        hi: 210,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn conjugate_mutable_correct_use_succeeds() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit is Adj {}
                operation A() : Unit {
                    mutable a = 1;
                    within {
                        let x = a;
                        B(1);
                        B(2);
                    }
                    apply {
                        mutable b = a;
                        set b = 0;
                        B(3);
                        B(4);
                    }
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-314] (Public):
                    Namespace (Ident 46 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: ()
                        functors: Functor Expr 4 [52-55]: Adj
                        body: Block: Block 5 [56-58]: <empty>
                Item 2 [63-312] (Public):
                    Parent: 0
                    Callable 6 [63-312] (Operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type ()]: Unit
                        output: ()
                        body: Block: Block 9 [84-312] [Type ()]:
                            Stmt 10 [94-108]: Local (Mutable):
                                Pat 11 [102-103] [Type Int]: Bind: Ident 12 [102-103] "a"
                                Expr 13 [106-107] [Type Int]: Lit: Int(1)
                            Stmt 14 [117-306]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 16 [124-194] [Type ()]:
                                    Stmt 17 [138-148]: Local (Immutable):
                                        Pat 18 [142-143] [Type Int]: Bind: Ident 19 [142-143] "x"
                                        Expr 20 [146-147] [Type Int]: Var: Local 12
                                    Stmt 21 [161-166]: Semi: Expr 22 [161-165] [Type ()]: Call:
                                        Expr 23 [161-162] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 24 [163-164] [Type Int]: Lit: Int(1)
                                    Stmt 25 [179-184]: Semi: Expr 26 [179-183] [Type ()]: Call:
                                        Expr 27 [179-180] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 28 [181-182] [Type Int]: Lit: Int(2)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 29 [209-306] [Type ()]:
                                    Stmt 30 [223-237]: Local (Mutable):
                                        Pat 31 [231-232] [Type Int]: Bind: Ident 32 [231-232] "b"
                                        Expr 33 [235-236] [Type Int]: Var: Local 12
                                    Stmt 34 [250-260]: Semi: Expr 35 [250-259] [Type ()]: Assign:
                                        Expr 36 [254-255] [Type Int]: Var: Local 32
                                        Expr 37 [258-259] [Type Int]: Lit: Int(0)
                                    Stmt 38 [273-278]: Semi: Expr 39 [273-277] [Type ()]: Call:
                                        Expr 40 [273-274] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 41 [275-276] [Type Int]: Lit: Int(3)
                                    Stmt 42 [291-296]: Semi: Expr 43 [291-295] [Type ()]: Call:
                                        Expr 44 [291-292] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 45 [293-294] [Type Int]: Lit: Int(4)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 16 [124-194] [Type ()]:
                                    Stmt 17 [138-148]: Local (Immutable):
                                        Pat 18 [142-143] [Type Int]: Bind: Ident 19 [142-143] "x"
                                        Expr 20 [146-147] [Type Int]: Var: Local 12
                                    Stmt 25 [179-184]: Semi: Expr 26 [179-183] [Type ()]: Call:
                                        Expr _id_ [179-180] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 27 [179-180] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 28 [181-182] [Type Int]: Lit: Int(2)
                                    Stmt 21 [161-166]: Semi: Expr 22 [161-165] [Type ()]: Call:
                                        Expr _id_ [161-162] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 23 [161-162] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 24 [163-164] [Type Int]: Lit: Int(1)"#]],
    );
}
