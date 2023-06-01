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
                    Namespace (Ident 33 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58] (Body): Impl:
                            Pat 5 [21-58] [Type Int]: Elided
                            Block 6 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-216] (Public):
                    Parent: 0
                    Callable 7 [63-216] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 10 [63-216] (Body): Impl:
                            Pat 11 [63-216] [Type Unit]: Elided
                            Block 12 [84-216] [Type Unit]:
                                Stmt 13 [94-210]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [101-148] [Type Unit]:
                                        Stmt 16 [115-120]: Semi: Expr 17 [115-119] [Type Unit]: Call:
                                            Expr 18 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 19 [117-118] [Type Int]: Lit: Int(1)
                                        Stmt 20 [133-138]: Semi: Expr 21 [133-137] [Type Unit]: Call:
                                            Expr 22 [133-134] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 23 [135-136] [Type Int]: Lit: Int(2)
                                    Stmt 35 [0-0]: Local (Immutable):
                                        Pat 36 [0-0] [Type Unit]: Bind: Ident 34 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Unit]: Expr Block: Block 24 [163-210] [Type Unit]:
                                            Stmt 25 [177-182]: Semi: Expr 26 [177-181] [Type Unit]: Call:
                                                Expr 27 [177-178] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 28 [179-180] [Type Int]: Lit: Int(3)
                                            Stmt 29 [195-200]: Semi: Expr 30 [195-199] [Type Unit]: Call:
                                                Expr 31 [195-196] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 32 [197-198] [Type Int]: Lit: Int(4)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [101-148] [Type Unit]:
                                        Stmt 20 [133-138]: Semi: Expr 21 [133-137] [Type Unit]: Call:
                                            Expr _id_ [133-134] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 22 [133-134] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 23 [135-136] [Type Int]: Lit: Int(2)
                                        Stmt 16 [115-120]: Semi: Expr 17 [115-119] [Type Unit]: Call:
                                            Expr _id_ [115-116] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 18 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 19 [117-118] [Type Int]: Lit: Int(1)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 34
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
                    Namespace (Ident 39 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58] (Body): Impl:
                            Pat 5 [21-58] [Type Int]: Elided
                            Block 6 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-252] (Public):
                    Parent: 0
                    Callable 7 [63-252] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 10 [63-252] (Body): Impl:
                            Pat 11 [63-252] [Type Unit]: Elided
                            Block 12 [83-252] [Type Int]:
                                Stmt 13 [93-234]: Local (Immutable):
                                    Pat 14 [97-100] [Type Int]: Bind: Ident 15 [97-100] "val"
                                    Expr _id_ [0-0] [Type Int]: Expr Block: Block _id_ [0-0] [Type Int]:
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 17 [110-157] [Type Unit]:
                                            Stmt 18 [124-129]: Semi: Expr 19 [124-128] [Type Unit]: Call:
                                                Expr 20 [124-125] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 21 [126-127] [Type Int]: Lit: Int(1)
                                            Stmt 22 [142-147]: Semi: Expr 23 [142-146] [Type Unit]: Call:
                                                Expr 24 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 25 [144-145] [Type Int]: Lit: Int(2)
                                        Stmt 41 [0-0]: Local (Immutable):
                                            Pat 42 [0-0] [Type Int]: Bind: Ident 40 [0-0] "apply_res"
                                            Expr _id_ [0-0] [Type Int]: Expr Block: Block 26 [172-233] [Type Int]:
                                                Stmt 27 [186-191]: Semi: Expr 28 [186-190] [Type Unit]: Call:
                                                    Expr 29 [186-187] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 30 [188-189] [Type Int]: Lit: Int(3)
                                                Stmt 31 [204-209]: Semi: Expr 32 [204-208] [Type Unit]: Call:
                                                    Expr 33 [204-205] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 34 [206-207] [Type Int]: Lit: Int(4)
                                                Stmt 35 [222-223]: Expr: Expr 36 [222-223] [Type Int]: Lit: Int(7)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 17 [110-157] [Type Unit]:
                                            Stmt 22 [142-147]: Semi: Expr 23 [142-146] [Type Unit]: Call:
                                                Expr _id_ [142-143] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 24 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 25 [144-145] [Type Int]: Lit: Int(2)
                                            Stmt 18 [124-129]: Semi: Expr 19 [124-128] [Type Unit]: Call:
                                                Expr _id_ [124-125] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 20 [124-125] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 21 [126-127] [Type Int]: Lit: Int(1)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Int]: Var: Local 40
                                Stmt 37 [243-246]: Expr: Expr 38 [243-246] [Type Int]: Var: Local 15
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
                    Namespace (Ident 49 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58] (Body): Impl:
                            Pat 5 [21-58] [Type Int]: Elided
                            Block 6 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-355] (Public):
                    Parent: 0
                    Callable 7 [63-355] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 10 [63-355] (Body): Impl:
                            Pat 11 [63-355] [Type Unit]: Elided
                            Block 12 [84-355] [Type Unit]:
                                Stmt 13 [94-349]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [101-287] [Type Unit]:
                                        Stmt 16 [115-120]: Semi: Expr 17 [115-119] [Type Unit]: Call:
                                            Expr 18 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 19 [117-118] [Type Int]: Lit: Int(0)
                                        Stmt 20 [133-277]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 22 [140-199] [Type Unit]:
                                                Stmt 23 [158-163]: Semi: Expr 24 [158-162] [Type Unit]: Call:
                                                    Expr 25 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                                Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type Unit]: Call:
                                                    Expr 29 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 30 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 54 [0-0]: Local (Immutable):
                                                Pat 55 [0-0] [Type Unit]: Bind: Ident 53 [0-0] "apply_res"
                                                Expr _id_ [0-0] [Type Unit]: Expr Block: Block 31 [218-277] [Type Unit]:
                                                    Stmt 32 [236-241]: Semi: Expr 33 [236-240] [Type Unit]: Call:
                                                        Expr 34 [236-237] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 35 [238-239] [Type Int]: Lit: Int(3)
                                                    Stmt 36 [258-263]: Semi: Expr 37 [258-262] [Type Unit]: Call:
                                                        Expr 38 [258-259] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 39 [260-261] [Type Int]: Lit: Int(4)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 22 [140-199] [Type Unit]:
                                                Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type Unit]: Call:
                                                    Expr _id_ [180-181] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 29 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 30 [182-183] [Type Int]: Lit: Int(2)
                                                Stmt 23 [158-163]: Semi: Expr 24 [158-162] [Type Unit]: Call:
                                                    Expr _id_ [158-159] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 25 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 53
                                    Stmt 51 [0-0]: Local (Immutable):
                                        Pat 52 [0-0] [Type Unit]: Bind: Ident 50 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Unit]: Expr Block: Block 40 [302-349] [Type Unit]:
                                            Stmt 41 [316-321]: Semi: Expr 42 [316-320] [Type Unit]: Call:
                                                Expr 43 [316-317] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 44 [318-319] [Type Int]: Lit: Int(5)
                                            Stmt 45 [334-339]: Semi: Expr 46 [334-338] [Type Unit]: Call:
                                                Expr 47 [334-335] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 48 [336-337] [Type Int]: Lit: Int(6)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 15 [101-287] [Type Unit]:
                                        Stmt 20 [133-277]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 22 [140-199] [Type Unit]:
                                                Stmt 23 [158-163]: Semi: Expr 24 [158-162] [Type Unit]: Call:
                                                    Expr 25 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                                Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type Unit]: Call:
                                                    Expr 29 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 30 [182-183] [Type Int]: Lit: Int(2)
                                            Stmt 57 [0-0]: Local (Immutable):
                                                Pat 58 [0-0] [Type Unit]: Bind: Ident 56 [0-0] "apply_res"
                                                Expr _id_ [0-0] [Type Unit]: Expr Block: Block 31 [218-277] [Type Unit]:
                                                    Stmt 36 [258-263]: Semi: Expr 37 [258-262] [Type Unit]: Call:
                                                        Expr _id_ [258-259] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                            Expr 38 [258-259] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 39 [260-261] [Type Int]: Lit: Int(4)
                                                    Stmt 32 [236-241]: Semi: Expr 33 [236-240] [Type Unit]: Call:
                                                        Expr _id_ [236-237] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                            Expr 34 [236-237] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 35 [238-239] [Type Int]: Lit: Int(3)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 22 [140-199] [Type Unit]:
                                                Stmt 27 [180-185]: Semi: Expr 28 [180-184] [Type Unit]: Call:
                                                    Expr _id_ [180-181] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 29 [180-181] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 30 [182-183] [Type Int]: Lit: Int(2)
                                                Stmt 23 [158-163]: Semi: Expr 24 [158-162] [Type Unit]: Call:
                                                    Expr _id_ [158-159] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 25 [158-159] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 26 [160-161] [Type Int]: Lit: Int(1)
                                            Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 56
                                        Stmt 16 [115-120]: Semi: Expr 17 [115-119] [Type Unit]: Call:
                                            Expr _id_ [115-116] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 18 [115-116] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 19 [117-118] [Type Int]: Lit: Int(0)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 50
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
                    Namespace (Ident 49 [10-14] "Test"): Item 1, Item 2
                Item 1 [21-58] (Public):
                    Parent: 0
                    Callable 0 [21-58] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-40] [Type Int]: Bind: Ident 3 [33-34] "i"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-58] (Body): Impl:
                            Pat 5 [21-58] [Type Int]: Elided
                            Block 6 [56-58]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [63-312] (Public):
                    Parent: 0
                    Callable 7 [63-312] (Operation):
                        name: Ident 8 [73-74] "A"
                        input: Pat 9 [74-76] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 10 [63-312] (Body): Impl:
                            Pat 11 [63-312] [Type Unit]: Elided
                            Block 12 [84-312] [Type Unit]:
                                Stmt 13 [94-108]: Local (Mutable):
                                    Pat 14 [102-103] [Type Int]: Bind: Ident 15 [102-103] "a"
                                    Expr 16 [106-107] [Type Int]: Lit: Int(1)
                                Stmt 17 [117-306]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 19 [124-194] [Type Unit]:
                                        Stmt 20 [138-148]: Local (Immutable):
                                            Pat 21 [142-143] [Type Int]: Bind: Ident 22 [142-143] "x"
                                            Expr 23 [146-147] [Type Int]: Var: Local 15
                                        Stmt 24 [161-166]: Semi: Expr 25 [161-165] [Type Unit]: Call:
                                            Expr 26 [161-162] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 27 [163-164] [Type Int]: Lit: Int(1)
                                        Stmt 28 [179-184]: Semi: Expr 29 [179-183] [Type Unit]: Call:
                                            Expr 30 [179-180] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 31 [181-182] [Type Int]: Lit: Int(2)
                                    Stmt 51 [0-0]: Local (Immutable):
                                        Pat 52 [0-0] [Type Unit]: Bind: Ident 50 [0-0] "apply_res"
                                        Expr _id_ [0-0] [Type Unit]: Expr Block: Block 32 [209-306] [Type Unit]:
                                            Stmt 33 [223-237]: Local (Mutable):
                                                Pat 34 [231-232] [Type Int]: Bind: Ident 35 [231-232] "b"
                                                Expr 36 [235-236] [Type Int]: Var: Local 15
                                            Stmt 37 [250-260]: Semi: Expr 38 [250-259] [Type Unit]: Assign:
                                                Expr 39 [254-255] [Type Int]: Var: Local 35
                                                Expr 40 [258-259] [Type Int]: Lit: Int(0)
                                            Stmt 41 [273-278]: Semi: Expr 42 [273-277] [Type Unit]: Call:
                                                Expr 43 [273-274] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 44 [275-276] [Type Int]: Lit: Int(3)
                                            Stmt 45 [291-296]: Semi: Expr 46 [291-295] [Type Unit]: Call:
                                                Expr 47 [291-292] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 48 [293-294] [Type Int]: Lit: Int(4)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block 19 [124-194] [Type Unit]:
                                        Stmt 20 [138-148]: Local (Immutable):
                                            Pat 21 [142-143] [Type Int]: Bind: Ident 22 [142-143] "x"
                                            Expr 23 [146-147] [Type Int]: Var: Local 15
                                        Stmt 28 [179-184]: Semi: Expr 29 [179-183] [Type Unit]: Call:
                                            Expr _id_ [179-180] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 30 [179-180] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 31 [181-182] [Type Int]: Lit: Int(2)
                                        Stmt 24 [161-166]: Semi: Expr 25 [161-165] [Type Unit]: Call:
                                            Expr _id_ [161-162] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 26 [161-162] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 27 [163-164] [Type Int]: Lit: Int(1)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: Var: Local 50
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}
