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

    let errors = invert_conjugate_exprs(store.core(), &mut unit);
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
                    Namespace (Ident 31 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58]: Impl:
                            Block 5 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-216] (Public):
                    Parent: 0
                    Callable 6 [63-216] (operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 9 [63-216]: Impl:
                            Block 10 [84-216] [Type Unit]:
                                Stmt 11 [94-210]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 13 [101-148] [Type Unit]:
                                        Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type Unit]: Call:
                                            Expr 16 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 17 [117-118] [Type Int]: Lit: Int(1)
                                        Stmt 18 [133-138]: Semi: Expr 19 [133-137] [Type Unit]: Call:
                                            Expr 20 [133-134] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 21 [135-136] [Type Int]: Lit: Int(2)
                                    Stmt 33 [0-0]: Local (Immutable):
                                        Pat 34 [0-0] [Type Unit]: Bind: Ident 32 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Unit]: Expr Block: Block 22 [163-210] [Type Unit]:
                                            Stmt 23 [177-182]: Semi: Expr 24 [177-181] [Type Unit]: Call:
                                                Expr 25 [177-178] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [179-180] [Type Int]: Lit: Int(3)
                                            Stmt 27 [195-200]: Semi: Expr 28 [195-199] [Type Unit]: Call:
                                                Expr 29 [195-196] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 30 [197-198] [Type Int]: Lit: Int(4)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 13 [101-148] [Type Unit]:
                                        Stmt 18 [133-138]: Semi: Expr 19 [133-137] [Type Unit]: Call:
                                            Expr _id_ [133-134] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 20 [133-134] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 21 [135-136] [Type Int]: Lit: Int(2)
                                        Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type Unit]: Call:
                                            Expr _id_ [115-116] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 16 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 17 [117-118] [Type Int]: Lit: Int(1)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 32
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 37 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58]: Impl:
                            Block 5 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-252] (Public):
                    Parent: 0
                    Callable 6 [63-252] (operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 9 [63-252]: Impl:
                            Block 10 [83-252] [Type Int]:
                                Stmt 11 [93-234]: Local (Immutable):
                                    Pat 12 [97-100] [Type Int]: Bind: Ident 13 [97-100] "val"
                                    Expr _id_ [0-0] [Type Int]: Expr Block: Block _id_ [0-0] [Type Int]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [110-157] [Type Unit]:
                                            Stmt 16 [124-129]: Semi: Expr 17 [124-128] [Type Unit]: Call:
                                                Expr 18 [124-125] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 19 [126-127] [Type Int]: Lit: Int(1)
                                            Stmt 20 [142-147]: Semi: Expr 21 [142-146] [Type Unit]: Call:
                                                Expr 22 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 23 [144-145] [Type Int]: Lit: Int(2)
                                        Stmt 39 [0-0]: Local (Immutable):
                                            Pat 40 [0-0] [Type Int]: Bind: Ident 38 [0-0] "apply_res"
                                            Expr _id_ [0-0] [Type Int]: Expr Block: Block 24 [172-233] [Type Int]:
                                                Stmt 25 [186-191]: Semi: Expr 26 [186-190] [Type Unit]: Call:
                                                    Expr 27 [186-187] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 28 [188-189] [Type Int]: Lit: Int(3)
                                                Stmt 29 [204-209]: Semi: Expr 30 [204-208] [Type Unit]: Call:
                                                    Expr 31 [204-205] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 32 [206-207] [Type Int]: Lit: Int(4)
                                                Stmt 33 [222-223]: Expr: Expr 34 [222-223] [Type Int]: Lit: Int(7)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [110-157] [Type Unit]:
                                            Stmt 20 [142-147]: Semi: Expr 21 [142-146] [Type Unit]: Call:
                                                Expr _id_ [142-143] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 22 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 23 [144-145] [Type Int]: Lit: Int(2)
                                            Stmt 16 [124-129]: Semi: Expr 17 [124-128] [Type Unit]: Call:
                                                Expr _id_ [124-125] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 18 [124-125] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 19 [126-127] [Type Int]: Lit: Int(1)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Int]: Var: Local 38
                                Stmt 35 [243-246]: Expr: Expr 36 [243-246] [Type Int]: Var: Local 13
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 47 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58]: Impl:
                            Block 5 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-355] (Public):
                    Parent: 0
                    Callable 6 [63-355] (operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 9 [63-355]: Impl:
                            Block 10 [84-355] [Type Unit]:
                                Stmt 11 [94-349]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 13 [101-287] [Type Unit]:
                                        Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type Unit]: Call:
                                            Expr 16 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 17 [117-118] [Type Int]: Lit: Int(0)
                                        Stmt 18 [133-277]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 20 [140-199] [Type Unit]:
                                                Stmt 21 [158-163]: Semi: Expr 22 [158-162] [Type Unit]: Call:
                                                    Expr 23 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 24 [160-161] [Type Int]: Lit: Int(1)
                                                Stmt 25 [180-185]: Semi: Expr 26 [180-184] [Type Unit]: Call:
                                                    Expr 27 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 28 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 52 [0-0]: Local (Immutable):
                                                Pat 53 [0-0] [Type Unit]: Bind: Ident 51 [0-0] "apply_res"
                                                Expr _id_ [0-0] [Type Unit]: Expr Block: Block 29 [218-277] [Type Unit]:
                                                    Stmt 30 [236-241]: Semi: Expr 31 [236-240] [Type Unit]: Call:
                                                        Expr 32 [236-237] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 33 [238-239] [Type Int]: Lit: Int(3)
                                                    Stmt 34 [258-263]: Semi: Expr 35 [258-262] [Type Unit]: Call:
                                                        Expr 36 [258-259] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 37 [260-261] [Type Int]: Lit: Int(4)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 20 [140-199] [Type Unit]:
                                                Stmt 25 [180-185]: Semi: Expr 26 [180-184] [Type Unit]: Call:
                                                    Expr _id_ [180-181] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 27 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 28 [182-183] [Type Int]: Lit: Int(2)
                                                Stmt 21 [158-163]: Semi: Expr 22 [158-162] [Type Unit]: Call:
                                                    Expr _id_ [158-159] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 23 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 24 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 51
                                    Stmt 49 [0-0]: Local (Immutable):
                                        Pat 50 [0-0] [Type Unit]: Bind: Ident 48 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Unit]: Expr Block: Block 38 [302-349] [Type Unit]:
                                            Stmt 39 [316-321]: Semi: Expr 40 [316-320] [Type Unit]: Call:
                                                Expr 41 [316-317] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 42 [318-319] [Type Int]: Lit: Int(5)
                                            Stmt 43 [334-339]: Semi: Expr 44 [334-338] [Type Unit]: Call:
                                                Expr 45 [334-335] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 46 [336-337] [Type Int]: Lit: Int(6)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 13 [101-287] [Type Unit]:
                                        Stmt 18 [133-277]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 20 [140-199] [Type Unit]:
                                                Stmt 21 [158-163]: Semi: Expr 22 [158-162] [Type Unit]: Call:
                                                    Expr 23 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 24 [160-161] [Type Int]: Lit: Int(1)
                                                Stmt 25 [180-185]: Semi: Expr 26 [180-184] [Type Unit]: Call:
                                                    Expr 27 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 28 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 55 [0-0]: Local (Immutable):
                                                Pat 56 [0-0] [Type Unit]: Bind: Ident 54 [0-0] "apply_res"
                                                Expr _id_ [0-0] [Type Unit]: Expr Block: Block 29 [218-277] [Type Unit]:
                                                    Stmt 34 [258-263]: Semi: Expr 35 [258-262] [Type Unit]: Call:
                                                        Expr _id_ [258-259] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                            Expr 36 [258-259] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 37 [260-261] [Type Int]: Lit: Int(4)
                                                    Stmt 30 [236-241]: Semi: Expr 31 [236-240] [Type Unit]: Call:
                                                        Expr _id_ [236-237] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                            Expr 32 [236-237] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 33 [238-239] [Type Int]: Lit: Int(3)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 20 [140-199] [Type Unit]:
                                                Stmt 25 [180-185]: Semi: Expr 26 [180-184] [Type Unit]: Call:
                                                    Expr _id_ [180-181] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 27 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 28 [182-183] [Type Int]: Lit: Int(2)
                                                Stmt 21 [158-163]: Semi: Expr 22 [158-162] [Type Unit]: Call:
                                                    Expr _id_ [158-159] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 23 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 24 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 54
                                        Stmt 14 [115-120]: Semi: Expr 15 [115-119] [Type Unit]: Call:
                                            Expr _id_ [115-116] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 16 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 17 [117-118] [Type Int]: Lit: Int(0)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 48
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                    Namespace (Ident 47 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58]: Impl:
                            Block 5 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-312] (Public):
                    Parent: 0
                    Callable 6 [63-312] (operation):
                        name: Ident 7 [73-74] "A"
                        input: Pat 8 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 9 [63-312]: Impl:
                            Block 10 [84-312] [Type Unit]:
                                Stmt 11 [94-108]: Local (Mutable):
                                    Pat 12 [102-103] [Type Int]: Bind: Ident 13 [102-103] "a"
                                    Expr 14 [106-107] [Type Int]: Lit: Int(1)
                                Stmt 15 [117-306]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 17 [124-194] [Type Unit]:
                                        Stmt 18 [138-148]: Local (Immutable):
                                            Pat 19 [142-143] [Type Int]: Bind: Ident 20 [142-143] "x"
                                            Expr 21 [146-147] [Type Int]: Var: Local 13
                                        Stmt 22 [161-166]: Semi: Expr 23 [161-165] [Type Unit]: Call:
                                            Expr 24 [161-162] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 25 [163-164] [Type Int]: Lit: Int(1)
                                        Stmt 26 [179-184]: Semi: Expr 27 [179-183] [Type Unit]: Call:
                                            Expr 28 [179-180] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 29 [181-182] [Type Int]: Lit: Int(2)
                                    Stmt 49 [0-0]: Local (Immutable):
                                        Pat 50 [0-0] [Type Unit]: Bind: Ident 48 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Unit]: Expr Block: Block 30 [209-306] [Type Unit]:
                                            Stmt 31 [223-237]: Local (Mutable):
                                                Pat 32 [231-232] [Type Int]: Bind: Ident 33 [231-232] "b"
                                                Expr 34 [235-236] [Type Int]: Var: Local 13
                                            Stmt 35 [250-260]: Semi: Expr 36 [250-259] [Type Unit]: Assign:
                                                Expr 37 [254-255] [Type Int]: Var: Local 33
                                                Expr 38 [258-259] [Type Int]: Lit: Int(0)
                                            Stmt 39 [273-278]: Semi: Expr 40 [273-277] [Type Unit]: Call:
                                                Expr 41 [273-274] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 42 [275-276] [Type Int]: Lit: Int(3)
                                            Stmt 43 [291-296]: Semi: Expr 44 [291-295] [Type Unit]: Call:
                                                Expr 45 [291-292] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 46 [293-294] [Type Int]: Lit: Int(4)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 17 [124-194] [Type Unit]:
                                        Stmt 18 [138-148]: Local (Immutable):
                                            Pat 19 [142-143] [Type Int]: Bind: Ident 20 [142-143] "x"
                                            Expr 21 [146-147] [Type Int]: Var: Local 13
                                        Stmt 26 [179-184]: Semi: Expr 27 [179-183] [Type Unit]: Call:
                                            Expr _id_ [179-180] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 28 [179-180] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 29 [181-182] [Type Int]: Lit: Int(2)
                                        Stmt 22 [161-166]: Semi: Expr 23 [161-165] [Type Unit]: Call:
                                            Expr _id_ [161-162] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 24 [161-162] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 25 [163-164] [Type Int]: Lit: Int(1)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 48
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}
