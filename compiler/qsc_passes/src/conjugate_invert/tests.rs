// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore, SourceMap};

use crate::conjugate_invert::invert_conjugate_exprs;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new();
    let mut unit = compile(
        &store,
        [],
        SourceMap::new([("test".into(), file.into())], "".into()),
    );
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
                Item 0 [0-218]:
                    Namespace (Ident 35 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58]:
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-41] [Type Int]: Paren:
                            Pat 3 [33-40] [Type Int]: Bind: Ident 4 [33-34] "i"
                        output: ()
                        functors: Functor Expr 5 [52-55]: Adj
                        body: Block: Block 6 [56-58]: <empty>
                Item 2 [63-216]:
                    Parent: 0
                    Callable 7 [63-216] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type ()]: Unit
                        output: ()
                        body: Block: Block 10 [84-216] [Type ()]:
                            Stmt 11 [94-210]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 13 [101-148] [Type ()]:
                                    Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type ()]: Call:
                                        Expr 16 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [116-119] [Type Int]: Paren: Expr 18 [117-118] [Type Int]: Lit: Int(1)
                                    Stmt 19 [133-138]: Semi: Expr 20 [133-137] [Type ()]: Call:
                                        Expr 21 [133-134] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 22 [134-137] [Type Int]: Paren: Expr 23 [135-136] [Type Int]: Lit: Int(2)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 24 [163-210] [Type ()]:
                                    Stmt 25 [177-182]: Semi: Expr 26 [177-181] [Type ()]: Call:
                                        Expr 27 [177-178] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 28 [178-181] [Type Int]: Paren: Expr 29 [179-180] [Type Int]: Lit: Int(3)
                                    Stmt 30 [195-200]: Semi: Expr 31 [195-199] [Type ()]: Call:
                                        Expr 32 [195-196] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 33 [196-199] [Type Int]: Paren: Expr 34 [197-198] [Type Int]: Lit: Int(4)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 13 [101-148] [Type ()]:
                                    Stmt 19 [133-138]: Semi: Expr 20 [133-137] [Type ()]: Call:
                                        Expr _id_ [133-134] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 21 [133-134] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 22 [134-137] [Type Int]: Paren: Expr 23 [135-136] [Type Int]: Lit: Int(2)
                                    Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type ()]: Call:
                                        Expr _id_ [115-116] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 16 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [116-119] [Type Int]: Paren: Expr 18 [117-118] [Type Int]: Lit: Int(1)"#]],
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
                Item 0 [0-357]:
                    Namespace (Ident 54 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58]:
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-41] [Type Int]: Paren:
                            Pat 3 [33-40] [Type Int]: Bind: Ident 4 [33-34] "i"
                        output: ()
                        functors: Functor Expr 5 [52-55]: Adj
                        body: Block: Block 6 [56-58]: <empty>
                Item 2 [63-355]:
                    Parent: 0
                    Callable 7 [63-355] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type ()]: Unit
                        output: ()
                        body: Block: Block 10 [84-355] [Type ()]:
                            Stmt 11 [94-349]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 13 [101-287] [Type ()]:
                                    Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type ()]: Call:
                                        Expr 16 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [116-119] [Type Int]: Paren: Expr 18 [117-118] [Type Int]: Lit: Int(0)
                                    Stmt 19 [133-277]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 21 [140-199] [Type ()]:
                                            Stmt 22 [158-163]: Semi: Expr 23 [158-162] [Type ()]: Call:
                                                Expr 24 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [159-162] [Type Int]: Paren: Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type ()]: Call:
                                                Expr 29 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 30 [181-184] [Type Int]: Paren: Expr 31 [182-183] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 32 [218-277] [Type ()]:
                                            Stmt 33 [236-241]: Semi: Expr 34 [236-240] [Type ()]: Call:
                                                Expr 35 [236-237] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 36 [237-240] [Type Int]: Paren: Expr 37 [238-239] [Type Int]: Lit: Int(3)
                                            Stmt 38 [258-263]: Semi: Expr 39 [258-262] [Type ()]: Call:
                                                Expr 40 [258-259] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 41 [259-262] [Type Int]: Paren: Expr 42 [260-261] [Type Int]: Lit: Int(4)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 21 [140-199] [Type ()]:
                                            Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type ()]: Call:
                                                Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 29 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 30 [181-184] [Type Int]: Paren: Expr 31 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 22 [158-163]: Semi: Expr 23 [158-162] [Type ()]: Call:
                                                Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 24 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [159-162] [Type Int]: Paren: Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 43 [302-349] [Type ()]:
                                    Stmt 44 [316-321]: Semi: Expr 45 [316-320] [Type ()]: Call:
                                        Expr 46 [316-317] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 47 [317-320] [Type Int]: Paren: Expr 48 [318-319] [Type Int]: Lit: Int(5)
                                    Stmt 49 [334-339]: Semi: Expr 50 [334-338] [Type ()]: Call:
                                        Expr 51 [334-335] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 52 [335-338] [Type Int]: Paren: Expr 53 [336-337] [Type Int]: Lit: Int(6)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 13 [101-287] [Type ()]:
                                    Stmt 19 [133-277]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 21 [140-199] [Type ()]:
                                            Stmt 22 [158-163]: Semi: Expr 23 [158-162] [Type ()]: Call:
                                                Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 24 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [159-162] [Type Int]: Paren: Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type ()]: Call:
                                                Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 29 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 30 [181-184] [Type Int]: Paren: Expr 31 [182-183] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 32 [218-277] [Type ()]:
                                            Stmt 38 [258-263]: Semi: Expr 39 [258-262] [Type ()]: Call:
                                                Expr _id_ [258-259] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 40 [258-259] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 41 [259-262] [Type Int]: Paren: Expr 42 [260-261] [Type Int]: Lit: Int(4)
                                            Stmt 33 [236-241]: Semi: Expr 34 [236-240] [Type ()]: Call:
                                                Expr _id_ [236-237] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 35 [236-237] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 36 [237-240] [Type Int]: Paren: Expr 37 [238-239] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 21 [140-199] [Type ()]:
                                            Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type ()]: Call:
                                                Expr _id_ [180-181] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 29 [180-181] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 30 [181-184] [Type Int]: Paren: Expr 31 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 22 [158-163]: Semi: Expr 23 [158-162] [Type ()]: Call:
                                                Expr _id_ [158-159] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 24 [158-159] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [159-162] [Type Int]: Paren: Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                    Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type ()]: Call:
                                        Expr _id_ [115-116] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 16 [115-116] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [116-119] [Type Int]: Paren: Expr 18 [117-118] [Type Int]: Lit: Int(0)"#]],
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
                Item 0 [0-314]:
                    Namespace (Ident 51 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58]:
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-41] [Type Int]: Paren:
                            Pat 3 [33-40] [Type Int]: Bind: Ident 4 [33-34] "i"
                        output: ()
                        functors: Functor Expr 5 [52-55]: Adj
                        body: Block: Block 6 [56-58]: <empty>
                Item 2 [63-312]:
                    Parent: 0
                    Callable 7 [63-312] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type ()]: Unit
                        output: ()
                        body: Block: Block 10 [84-312] [Type ()]:
                            Stmt 11 [94-108]: Local (Mutable):
                                Pat 12 [102-103] [Type Int]: Bind: Ident 13 [102-103] "a"
                                Expr 14 [106-107] [Type Int]: Lit: Int(1)
                            Stmt 15 [117-306]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 17 [124-194] [Type ()]:
                                    Stmt 18 [138-148]: Local (Immutable):
                                        Pat 19 [142-143] [Type Int]: Bind: Ident 20 [142-143] "x"
                                        Expr 21 [146-147] [Type Int]: Var: Local 13
                                    Stmt 22 [161-166]: Semi: Expr 23 [161-165] [Type ()]: Call:
                                        Expr 24 [161-162] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 25 [162-165] [Type Int]: Paren: Expr 26 [163-164] [Type Int]: Lit: Int(1)
                                    Stmt 27 [179-184]: Semi: Expr 28 [179-183] [Type ()]: Call:
                                        Expr 29 [179-180] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 30 [180-183] [Type Int]: Paren: Expr 31 [181-182] [Type Int]: Lit: Int(2)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 32 [209-306] [Type ()]:
                                    Stmt 33 [223-237]: Local (Mutable):
                                        Pat 34 [231-232] [Type Int]: Bind: Ident 35 [231-232] "b"
                                        Expr 36 [235-236] [Type Int]: Var: Local 13
                                    Stmt 37 [250-260]: Semi: Expr 38 [250-259] [Type ()]: Assign:
                                        Expr 39 [254-255] [Type Int]: Var: Local 35
                                        Expr 40 [258-259] [Type Int]: Lit: Int(0)
                                    Stmt 41 [273-278]: Semi: Expr 42 [273-277] [Type ()]: Call:
                                        Expr 43 [273-274] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 44 [274-277] [Type Int]: Paren: Expr 45 [275-276] [Type Int]: Lit: Int(3)
                                    Stmt 46 [291-296]: Semi: Expr 47 [291-295] [Type ()]: Call:
                                        Expr 48 [291-292] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 49 [292-295] [Type Int]: Paren: Expr 50 [293-294] [Type Int]: Lit: Int(4)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block 17 [124-194] [Type ()]:
                                    Stmt 18 [138-148]: Local (Immutable):
                                        Pat 19 [142-143] [Type Int]: Bind: Ident 20 [142-143] "x"
                                        Expr 21 [146-147] [Type Int]: Var: Local 13
                                    Stmt 27 [179-184]: Semi: Expr 28 [179-183] [Type ()]: Call:
                                        Expr _id_ [179-180] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 29 [179-180] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 30 [180-183] [Type Int]: Paren: Expr 31 [181-182] [Type Int]: Lit: Int(2)
                                    Stmt 22 [161-166]: Semi: Expr 23 [161-165] [Type ()]: Call:
                                        Expr _id_ [161-162] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 24 [161-162] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 25 [162-165] [Type Int]: Paren: Expr 26 [163-164] [Type Int]: Lit: Int(1)"#]],
    );
}
