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
                    Namespace (Ident 29 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: Block: Block 4 [56-58]: <empty>
                Item 2 [63-216] (Public):
                    Parent: 0
                    Callable 5 [63-216] (Operation):
                        name: Ident 6 [73-74] "A"
                        input: Pat 7 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 8 [84-216] [Type Unit]:
                            Stmt 9 [94-210]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 11 [101-148] [Type Unit]:
                                    Stmt 12 [115-120]: Semi: Expr 13 [115-119] [Type Unit]: Call:
                                        Expr 14 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [117-118] [Type Int]: Lit: Int(1)
                                    Stmt 16 [133-138]: Semi: Expr 17 [133-137] [Type Unit]: Call:
                                        Expr 18 [133-134] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 19 [135-136] [Type Int]: Lit: Int(2)
                                Stmt 31 [0-0]: Local (Immutable):
                                    Pat 32 [0-0] [Type Unit]: Bind: Ident 30 [0-0] "apply_res"
                                    Expr _id_ [0-0] [Type Unit]: Expr Block: Block 20 [163-210] [Type Unit]:
                                        Stmt 21 [177-182]: Semi: Expr 22 [177-181] [Type Unit]: Call:
                                            Expr 23 [177-178] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 24 [179-180] [Type Int]: Lit: Int(3)
                                        Stmt 25 [195-200]: Semi: Expr 26 [195-199] [Type Unit]: Call:
                                            Expr 27 [195-196] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 28 [197-198] [Type Int]: Lit: Int(4)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 11 [101-148] [Type Unit]:
                                    Stmt 16 [133-138]: Semi: Expr 17 [133-137] [Type Unit]: Call:
                                        Expr _id_ [133-134] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 18 [133-134] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 19 [135-136] [Type Int]: Lit: Int(2)
                                    Stmt 12 [115-120]: Semi: Expr 13 [115-119] [Type Unit]: Call:
                                        Expr _id_ [115-116] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 14 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [117-118] [Type Int]: Lit: Int(1)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 30"#]],
    );
}

#[test]
fn conjugate_invert_with_output() {
    check(
        indoc! {"
            namespace Test {
                operation B(i : Int) : Unit is Adj {}
                operation A() : Int {
                    let val = within {
                        B(1);
                        B(2);
                    }
                    apply {
                        B(3);
                        B(4);
                        7
                    };
                    val
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-254] (Public):
                    Namespace (Ident 35 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: Block: Block 4 [56-58]: <empty>
                Item 2 [63-252] (Public):
                    Parent: 0
                    Callable 5 [63-252] (Operation):
                        name: Ident 6 [73-74] "A"
                        input: Pat 7 [74-76] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 8 [83-252] [Type Int]:
                            Stmt 9 [93-234]: Local (Immutable):
                                Pat 10 [97-100] [Type Int]: Bind: Ident 11 [97-100] "val"
                                Expr _id_ [0-0] [Type Int]: Expr Block: Block _id_ [0-0] [Type Int]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 13 [110-157] [Type Unit]:
                                        Stmt 14 [124-129]: Semi: Expr 15 [124-128] [Type Unit]: Call:
                                            Expr 16 [124-125] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 17 [126-127] [Type Int]: Lit: Int(1)
                                        Stmt 18 [142-147]: Semi: Expr 19 [142-146] [Type Unit]: Call:
                                            Expr 20 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 21 [144-145] [Type Int]: Lit: Int(2)
                                    Stmt 37 [0-0]: Local (Immutable):
                                        Pat 38 [0-0] [Type Int]: Bind: Ident 36 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Int]: Expr Block: Block 22 [172-233] [Type Int]:
                                            Stmt 23 [186-191]: Semi: Expr 24 [186-190] [Type Unit]: Call:
                                                Expr 25 [186-187] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [188-189] [Type Int]: Lit: Int(3)
                                            Stmt 27 [204-209]: Semi: Expr 28 [204-208] [Type Unit]: Call:
                                                Expr 29 [204-205] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 30 [206-207] [Type Int]: Lit: Int(4)
                                            Stmt 31 [222-223]: Expr: Expr 32 [222-223] [Type Int]: Lit: Int(7)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 13 [110-157] [Type Unit]:
                                        Stmt 18 [142-147]: Semi: Expr 19 [142-146] [Type Unit]: Call:
                                            Expr _id_ [142-143] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 20 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 21 [144-145] [Type Int]: Lit: Int(2)
                                        Stmt 14 [124-129]: Semi: Expr 15 [124-128] [Type Unit]: Call:
                                            Expr _id_ [124-125] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 16 [124-125] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 17 [126-127] [Type Int]: Lit: Int(1)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Int]: Var: Local 36
                            Stmt 33 [243-246]: Expr: Expr 34 [243-246] [Type Int]: Var: Local 11"#]],
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
                    Namespace (Ident 45 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: Block: Block 4 [56-58]: <empty>
                Item 2 [63-355] (Public):
                    Parent: 0
                    Callable 5 [63-355] (Operation):
                        name: Ident 6 [73-74] "A"
                        input: Pat 7 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 8 [84-355] [Type Unit]:
                            Stmt 9 [94-349]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 11 [101-287] [Type Unit]:
                                    Stmt 12 [115-120]: Semi: Expr 13 [115-119] [Type Unit]: Call:
                                        Expr 14 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [117-118] [Type Int]: Lit: Int(0)
                                    Stmt 16 [133-277]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 18 [140-199] [Type Unit]:
                                            Stmt 19 [158-163]: Semi: Expr 20 [158-162] [Type Unit]: Call:
                                                Expr 21 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 22 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt 23 [180-185]: Semi: Expr 24 [180-184] [Type Unit]: Call:
                                                Expr 25 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [182-183] [Type Int]: Lit: Int(2)
                                        Stmt 50 [0-0]: Local (Immutable):
                                            Pat 51 [0-0] [Type Unit]: Bind: Ident 49 [0-0] "apply_res"
                                            Expr _id_ [0-0] [Type Unit]: Expr Block: Block 27 [218-277] [Type Unit]:
                                                Stmt 28 [236-241]: Semi: Expr 29 [236-240] [Type Unit]: Call:
                                                    Expr 30 [236-237] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 31 [238-239] [Type Int]: Lit: Int(3)
                                                Stmt 32 [258-263]: Semi: Expr 33 [258-262] [Type Unit]: Call:
                                                    Expr 34 [258-259] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 35 [260-261] [Type Int]: Lit: Int(4)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 18 [140-199] [Type Unit]:
                                            Stmt 23 [180-185]: Semi: Expr 24 [180-184] [Type Unit]: Call:
                                                Expr _id_ [180-181] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 25 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 19 [158-163]: Semi: Expr 20 [158-162] [Type Unit]: Call:
                                                Expr _id_ [158-159] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 21 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 22 [160-161] [Type Int]: Lit: Int(1)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 49
                                Stmt 47 [0-0]: Local (Immutable):
                                    Pat 48 [0-0] [Type Unit]: Bind: Ident 46 [0-0] "apply_res"
                                    Expr _id_ [0-0] [Type Unit]: Expr Block: Block 36 [302-349] [Type Unit]:
                                        Stmt 37 [316-321]: Semi: Expr 38 [316-320] [Type Unit]: Call:
                                            Expr 39 [316-317] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 40 [318-319] [Type Int]: Lit: Int(5)
                                        Stmt 41 [334-339]: Semi: Expr 42 [334-338] [Type Unit]: Call:
                                            Expr 43 [334-335] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 44 [336-337] [Type Int]: Lit: Int(6)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 11 [101-287] [Type Unit]:
                                    Stmt 16 [133-277]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 18 [140-199] [Type Unit]:
                                            Stmt 19 [158-163]: Semi: Expr 20 [158-162] [Type Unit]: Call:
                                                Expr 21 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 22 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt 23 [180-185]: Semi: Expr 24 [180-184] [Type Unit]: Call:
                                                Expr 25 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [182-183] [Type Int]: Lit: Int(2)
                                        Stmt 53 [0-0]: Local (Immutable):
                                            Pat 54 [0-0] [Type Unit]: Bind: Ident 52 [0-0] "apply_res"
                                            Expr _id_ [0-0] [Type Unit]: Expr Block: Block 27 [218-277] [Type Unit]:
                                                Stmt 32 [258-263]: Semi: Expr 33 [258-262] [Type Unit]: Call:
                                                    Expr _id_ [258-259] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 34 [258-259] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 35 [260-261] [Type Int]: Lit: Int(4)
                                                Stmt 28 [236-241]: Semi: Expr 29 [236-240] [Type Unit]: Call:
                                                    Expr _id_ [236-237] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 30 [236-237] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 31 [238-239] [Type Int]: Lit: Int(3)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 18 [140-199] [Type Unit]:
                                            Stmt 23 [180-185]: Semi: Expr 24 [180-184] [Type Unit]: Call:
                                                Expr _id_ [180-181] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 25 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 19 [158-163]: Semi: Expr 20 [158-162] [Type Unit]: Call:
                                                Expr _id_ [158-159] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 21 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 22 [160-161] [Type Int]: Lit: Int(1)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 52
                                    Stmt 12 [115-120]: Semi: Expr 13 [115-119] [Type Unit]: Call:
                                        Expr _id_ [115-116] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 14 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [117-118] [Type Int]: Lit: Int(0)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 46"#]],
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
fn conjugate_return_in_apply_fail() {
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
                        return ();
                    }
                }
            }
        "},
        &expect![[r#"
            [
                ReturnForbidden(
                    Span {
                        lo: 205,
                        hi: 214,
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
                    Namespace (Ident 45 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: Block: Block 4 [56-58]: <empty>
                Item 2 [63-312] (Public):
                    Parent: 0
                    Callable 5 [63-312] (Operation):
                        name: Ident 6 [73-74] "A"
                        input: Pat 7 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: 
                        body: Block: Block 8 [84-312] [Type Unit]:
                            Stmt 9 [94-108]: Local (Mutable):
                                Pat 10 [102-103] [Type Int]: Bind: Ident 11 [102-103] "a"
                                Expr 12 [106-107] [Type Int]: Lit: Int(1)
                            Stmt 13 [117-306]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [124-194] [Type Unit]:
                                    Stmt 16 [138-148]: Local (Immutable):
                                        Pat 17 [142-143] [Type Int]: Bind: Ident 18 [142-143] "x"
                                        Expr 19 [146-147] [Type Int]: Var: Local 11
                                    Stmt 20 [161-166]: Semi: Expr 21 [161-165] [Type Unit]: Call:
                                        Expr 22 [161-162] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 23 [163-164] [Type Int]: Lit: Int(1)
                                    Stmt 24 [179-184]: Semi: Expr 25 [179-183] [Type Unit]: Call:
                                        Expr 26 [179-180] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 27 [181-182] [Type Int]: Lit: Int(2)
                                Stmt 47 [0-0]: Local (Immutable):
                                    Pat 48 [0-0] [Type Unit]: Bind: Ident 46 [0-0] "apply_res"
                                    Expr _id_ [0-0] [Type Unit]: Expr Block: Block 28 [209-306] [Type Unit]:
                                        Stmt 29 [223-237]: Local (Mutable):
                                            Pat 30 [231-232] [Type Int]: Bind: Ident 31 [231-232] "b"
                                            Expr 32 [235-236] [Type Int]: Var: Local 11
                                        Stmt 33 [250-260]: Semi: Expr 34 [250-259] [Type Unit]: Assign:
                                            Expr 35 [254-255] [Type Int]: Var: Local 31
                                            Expr 36 [258-259] [Type Int]: Lit: Int(0)
                                        Stmt 37 [273-278]: Semi: Expr 38 [273-277] [Type Unit]: Call:
                                            Expr 39 [273-274] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 40 [275-276] [Type Int]: Lit: Int(3)
                                        Stmt 41 [291-296]: Semi: Expr 42 [291-295] [Type Unit]: Call:
                                            Expr 43 [291-292] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 44 [293-294] [Type Int]: Lit: Int(4)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [124-194] [Type Unit]:
                                    Stmt 16 [138-148]: Local (Immutable):
                                        Pat 17 [142-143] [Type Int]: Bind: Ident 18 [142-143] "x"
                                        Expr 19 [146-147] [Type Int]: Var: Local 11
                                    Stmt 24 [179-184]: Semi: Expr 25 [179-183] [Type Unit]: Call:
                                        Expr _id_ [179-180] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 26 [179-180] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 27 [181-182] [Type Int]: Lit: Int(2)
                                    Stmt 20 [161-166]: Semi: Expr 21 [161-165] [Type Unit]: Call:
                                        Expr _id_ [161-162] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 22 [161-162] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 23 [163-164] [Type Int]: Lit: Int(1)
                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 46"#]],
    );
}
