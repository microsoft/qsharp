// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{compile, PackageStore};

use crate::spec_gen::generate_specs;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new();
    let mut unit = compile(&store, [], [file], "");
    assert!(unit.errors.is_empty(), "{:?}", unit.errors);
    let errors = generate_specs(&mut unit);
    if errors.is_empty() {
        expect.assert_eq(&unit.package.to_string());
    } else {
        expect.assert_debug_eq(&errors);
    }
}

#[test]
fn generate_specs_body_intrinsic_should_fail() {
    check(
        indoc! {"
        namespace test {
            operation A(q : Qubit) : Unit is Ctl {
                body intrinsic;
            }
        }
        "},
        &expect![[r#"
            [
                MissingBody(
                    Span {
                        lo: 68,
                        hi: 83,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn generate_specs_body_missing_should_fail() {
    check(
        indoc! {"
        namespace test {
            operation A(q : Qubit) : Unit is Adj {
                adjoint ... {}
            }
        }
        "},
        &expect![[r#"
            [
                MissingBody(
                    Span {
                        lo: 21,
                        hi: 88,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn generate_ctl() {
    check(
        indoc! {"
            namespace test {
                operation A(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (ctls, ...) {}
                }
                operation B(q : Qubit) : Unit is Ctl {
                    A(q);
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-184]:
                    Namespace (Ident 27 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [32-43] [Type Qubit]: Paren:
                            Pat 3 [33-42] [Type Qubit]: Bind: Ident 4 [33-34] "q"
                        output: ()
                        functors: Functor Expr 5 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 6 [68-79] (Body): Impl:
                                Pat 7 [73-76] [Type Qubit]: Elided
                                Block 8 [77-79]: <empty>
                            SpecDecl 9 [88-113] (Ctl): Impl:
                                Pat 10 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 11 [100-104] [Type (Qubit)[]]: Bind: Ident 12 [100-104] "ctls"
                                    Pat 13 [106-109] [Type Qubit]: Elided
                                Block 14 [111-113]: <empty>
                Item 2 [124-182]:
                    Parent: 0
                    Callable 15 [124-182] (Operation):
                        name: Ident 16 [134-135] "B"
                        input: Pat 17 [135-146] [Type Qubit]: Paren:
                            Pat 18 [136-145] [Type Qubit]: Bind: Ident 19 [136-137] "q"
                        output: ()
                        functors: Functor Expr 20 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl _id_ [161-182] (Body): Impl:
                                Pat _id_ [161-182] [Type Qubit]: Elided
                                Block 21 [161-182] [Type ()]:
                                    Stmt 22 [171-176]: Semi: Expr 23 [171-175] [Type ()]: Call:
                                        Expr 24 [171-172] [Type (Qubit => () is Ctl)]: Var: Item 1
                                        Expr 25 [172-175] [Type Qubit]: Paren: Expr 26 [173-174] [Type Qubit]: Var: Local 19
                            SpecDecl _id_ [124-182] (Ctl): Impl:
                                Pat _id_ [124-182] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-182] [Type (Qubit)[]]: Bind: Ident 28 [124-182] "ctls"
                                    Pat _id_ [124-182] [Type Qubit]: Elided
                                Block 21 [161-182] [Type ()]:
                                    Stmt 22 [171-176]: Semi: Expr 23 [171-175] [Type ()]: Call:
                                        Expr 24 [171-172] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                            Expr 24 [171-172] [Type (Qubit => () is Ctl)]: Var: Item 1
                                        Expr 25 [172-175] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [172-175] [Type (Qubit)[]]: Var: Local 28
                                            Expr 25 [172-175] [Type Qubit]: Paren: Expr 26 [173-174] [Type Qubit]: Var: Local 19"#]],
    );
}

#[test]
fn generate_ctladj_distrib() {
    check(
        indoc! {"
            namespace test {
                operation A(q : Qubit) : Unit is Ctl + Adj {
                    body ... {}
                    adjoint ... {}
                    controlled (ctls, ...) {}
                }
                operation B(q : Qubit) : Unit is Ctl + Adj {
                    body ... {
                        A(q);
                    }
                    adjoint ... {
                        Adjoint A(q);
                    }
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-310]:
                    Namespace (Ident 45 [10-14] "test"): Item 1, Item 2
                Item 1 [21-148]:
                    Parent: 0
                    Callable 0 [21-148] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [32-43] [Type Qubit]: Paren:
                            Pat 3 [33-42] [Type Qubit]: Bind: Ident 4 [33-34] "q"
                        output: ()
                        functors: Functor Expr 5 [54-63]: BinOp Union: (Functor Expr 6 [54-57]: Ctl) (Functor Expr 7 [60-63]: Adj)
                        body: Specializations:
                            SpecDecl 8 [74-85] (Body): Impl:
                                Pat 9 [79-82] [Type Qubit]: Elided
                                Block 10 [83-85]: <empty>
                            SpecDecl 11 [94-108] (Adj): Impl:
                                Pat 12 [102-105] [Type Qubit]: Elided
                                Block 13 [106-108]: <empty>
                            SpecDecl 14 [117-142] (Ctl): Impl:
                                Pat 15 [128-139] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 16 [129-133] [Type (Qubit)[]]: Bind: Ident 17 [129-133] "ctls"
                                    Pat 18 [135-138] [Type Qubit]: Elided
                                Block 19 [140-142]: <empty>
                            SpecDecl _id_ [21-148] (CtlAdj): Impl:
                                Pat _id_ [21-148] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [21-148] [Type (Qubit)[]]: Bind: Ident 46 [21-148] "ctls"
                                    Pat _id_ [21-148] [Type Qubit]: Elided
                                Block 13 [106-108]: <empty>
                Item 2 [153-308]:
                    Parent: 0
                    Callable 20 [153-308] (Operation):
                        name: Ident 21 [163-164] "B"
                        input: Pat 22 [164-175] [Type Qubit]: Paren:
                            Pat 23 [165-174] [Type Qubit]: Bind: Ident 24 [165-166] "q"
                        output: ()
                        functors: Functor Expr 25 [186-195]: BinOp Union: (Functor Expr 26 [186-189]: Ctl) (Functor Expr 27 [192-195]: Adj)
                        body: Specializations:
                            SpecDecl 28 [206-244] (Body): Impl:
                                Pat 29 [211-214] [Type Qubit]: Elided
                                Block 30 [215-244] [Type ()]:
                                    Stmt 31 [229-234]: Semi: Expr 32 [229-233] [Type ()]: Call:
                                        Expr 33 [229-230] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 34 [230-233] [Type Qubit]: Paren: Expr 35 [231-232] [Type Qubit]: Var: Local 24
                            SpecDecl 36 [253-302] (Adj): Impl:
                                Pat 37 [261-264] [Type Qubit]: Elided
                                Block 38 [265-302] [Type ()]:
                                    Stmt 39 [279-292]: Semi: Expr 40 [279-291] [Type ()]: Call:
                                        Expr 41 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 42 [287-288] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 43 [288-291] [Type Qubit]: Paren: Expr 44 [289-290] [Type Qubit]: Var: Local 24
                            SpecDecl _id_ [153-308] (Ctl): Impl:
                                Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 47 [153-308] "ctls"
                                    Pat _id_ [153-308] [Type Qubit]: Elided
                                Block 30 [215-244] [Type ()]:
                                    Stmt 31 [229-234]: Semi: Expr 32 [229-233] [Type ()]: Call:
                                        Expr 33 [229-230] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 33 [229-230] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 34 [230-233] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [230-233] [Type (Qubit)[]]: Var: Local 47
                                            Expr 34 [230-233] [Type Qubit]: Paren: Expr 35 [231-232] [Type Qubit]: Var: Local 24
                            SpecDecl _id_ [153-308] (CtlAdj): Impl:
                                Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 48 [153-308] "ctls"
                                    Pat _id_ [153-308] [Type Qubit]: Elided
                                Block 38 [265-302] [Type ()]:
                                    Stmt 39 [279-292]: Semi: Expr 40 [279-291] [Type ()]: Call:
                                        Expr 41 [279-288] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 41 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 42 [287-288] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 43 [288-291] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [288-291] [Type (Qubit)[]]: Var: Local 48
                                            Expr 43 [288-291] [Type Qubit]: Paren: Expr 44 [289-290] [Type Qubit]: Var: Local 24"#]],
    );
}

#[test]
fn generate_ctl_skip_conjugate_apply_block() {
    check(
        indoc! {"
            namespace test {
                operation A(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (ctls, ...) {}
                }
                operation B(q : Qubit) : Unit is Ctl {
                    within {
                        A(q);
                    }
                    apply {
                        A(q);
                    }
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-259]:
                    Namespace (Ident 36 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [32-43] [Type Qubit]: Paren:
                            Pat 3 [33-42] [Type Qubit]: Bind: Ident 4 [33-34] "q"
                        output: ()
                        functors: Functor Expr 5 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 6 [68-79] (Body): Impl:
                                Pat 7 [73-76] [Type Qubit]: Elided
                                Block 8 [77-79]: <empty>
                            SpecDecl 9 [88-113] (Ctl): Impl:
                                Pat 10 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 11 [100-104] [Type (Qubit)[]]: Bind: Ident 12 [100-104] "ctls"
                                    Pat 13 [106-109] [Type Qubit]: Elided
                                Block 14 [111-113]: <empty>
                Item 2 [124-257]:
                    Parent: 0
                    Callable 15 [124-257] (Operation):
                        name: Ident 16 [134-135] "B"
                        input: Pat 17 [135-146] [Type Qubit]: Paren:
                            Pat 18 [136-145] [Type Qubit]: Bind: Ident 19 [136-137] "q"
                        output: ()
                        functors: Functor Expr 20 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl _id_ [161-257] (Body): Impl:
                                Pat _id_ [161-257] [Type Qubit]: Elided
                                Block 21 [161-257] [Type ()]:
                                    Stmt 22 [171-251]: Expr: Expr 23 [171-251] [Type ()]: Conjugate:
                                        Block 24 [178-207] [Type ()]:
                                            Stmt 25 [192-197]: Semi: Expr 26 [192-196] [Type ()]: Call:
                                                Expr 27 [192-193] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 28 [193-196] [Type Qubit]: Paren: Expr 29 [194-195] [Type Qubit]: Var: Local 19
                                        Block 30 [222-251] [Type ()]:
                                            Stmt 31 [236-241]: Semi: Expr 32 [236-240] [Type ()]: Call:
                                                Expr 33 [236-237] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 34 [237-240] [Type Qubit]: Paren: Expr 35 [238-239] [Type Qubit]: Var: Local 19
                            SpecDecl _id_ [124-257] (Ctl): Impl:
                                Pat _id_ [124-257] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-257] [Type (Qubit)[]]: Bind: Ident 37 [124-257] "ctls"
                                    Pat _id_ [124-257] [Type Qubit]: Elided
                                Block 21 [161-257] [Type ()]:
                                    Stmt 22 [171-251]: Expr: Expr 23 [171-251] [Type ()]: Conjugate:
                                        Block 24 [178-207] [Type ()]:
                                            Stmt 25 [192-197]: Semi: Expr 26 [192-196] [Type ()]: Call:
                                                Expr 27 [192-193] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 28 [193-196] [Type Qubit]: Paren: Expr 29 [194-195] [Type Qubit]: Var: Local 19
                                        Block 30 [222-251] [Type ()]:
                                            Stmt 31 [236-241]: Semi: Expr 32 [236-240] [Type ()]: Call:
                                                Expr 33 [236-237] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                                    Expr 33 [236-237] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 34 [237-240] [Type ((Qubit)[], Qubit)]: Tuple:
                                                    Expr _id_ [237-240] [Type (Qubit)[]]: Var: Local 37
                                                    Expr 34 [237-240] [Type Qubit]: Paren: Expr 35 [238-239] [Type Qubit]: Var: Local 19"#]],
    );
}

#[test]
fn generate_ctl_op_missing_functor() {
    check(
        indoc! {"
            namespace test {
                operation A(q : Qubit) : Unit {
                }
                operation B(q : Qubit) : Unit is Ctl {
                    A(q);
                }
            }
        "},
        &expect![[r#"
            [
                CtlGen(
                    MissingCtlFunctor(
                        Span {
                            lo: 110,
                            hi: 111,
                        },
                    ),
                ),
            ]
        "#]],
    );
}

#[test]
fn generate_ctl_with_function_calls() {
    check(
        indoc! {"
            namespace test {
                function Foo() : Unit {}
                operation A() : Unit is Ctl {}
                operation B() : Unit is Ctl {
                    Foo();
                    A();
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-150]:
                    Namespace (Ident 22 [10-14] "test"): Item 1, Item 2, Item 3
                Item 1 [21-45]:
                    Parent: 0
                    Callable 0 [21-45] (Function):
                        name: Ident 1 [30-33] "Foo"
                        input: Pat 2 [33-35] [Type ()]: Unit
                        output: ()
                        body: Block: Block 3 [43-45]: <empty>
                Item 2 [50-80]:
                    Parent: 0
                    Callable 4 [50-80] (Operation):
                        name: Ident 5 [60-61] "A"
                        input: Pat 6 [61-63] [Type ()]: Unit
                        output: ()
                        functors: Functor Expr 7 [74-77]: Ctl
                        body: Specializations:
                            SpecDecl _id_ [78-80] (Body): Impl:
                                Pat _id_ [78-80] [Type ()]: Elided
                                Block 8 [78-80]: <empty>
                            SpecDecl _id_ [50-80] (Ctl): Impl:
                                Pat _id_ [50-80] [Type ((Qubit)[], ())]: Tuple:
                                    Pat _id_ [50-80] [Type (Qubit)[]]: Bind: Ident 23 [50-80] "ctls"
                                    Pat _id_ [50-80] [Type ()]: Elided
                                Block 8 [78-80]: <empty>
                Item 3 [85-148]:
                    Parent: 0
                    Callable 9 [85-148] (Operation):
                        name: Ident 10 [95-96] "B"
                        input: Pat 11 [96-98] [Type ()]: Unit
                        output: ()
                        functors: Functor Expr 12 [109-112]: Ctl
                        body: Specializations:
                            SpecDecl _id_ [113-148] (Body): Impl:
                                Pat _id_ [113-148] [Type ()]: Elided
                                Block 13 [113-148] [Type ()]:
                                    Stmt 14 [123-129]: Semi: Expr 15 [123-128] [Type ()]: Call:
                                        Expr 16 [123-126] [Type (() -> ())]: Var: Item 1
                                        Expr 17 [126-128] [Type ()]: Unit
                                    Stmt 18 [138-142]: Semi: Expr 19 [138-141] [Type ()]: Call:
                                        Expr 20 [138-139] [Type (() => () is Ctl)]: Var: Item 2
                                        Expr 21 [139-141] [Type ()]: Unit
                            SpecDecl _id_ [85-148] (Ctl): Impl:
                                Pat _id_ [85-148] [Type ((Qubit)[], ())]: Tuple:
                                    Pat _id_ [85-148] [Type (Qubit)[]]: Bind: Ident 24 [85-148] "ctls"
                                    Pat _id_ [85-148] [Type ()]: Elided
                                Block 13 [113-148] [Type ()]:
                                    Stmt 14 [123-129]: Semi: Expr 15 [123-128] [Type ()]: Call:
                                        Expr 16 [123-126] [Type (() -> ())]: Var: Item 1
                                        Expr 17 [126-128] [Type ()]: Unit
                                    Stmt 18 [138-142]: Semi: Expr 19 [138-141] [Type ()]: Call:
                                        Expr 20 [138-139] [Type (((Qubit)[], ()) => () is Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [138-139] [Type (() => () is Ctl)]: Var: Item 2
                                        Expr 21 [139-141] [Type ((Qubit)[], ())]: Tuple:
                                            Expr _id_ [139-141] [Type (Qubit)[]]: Var: Local 24
                                            Expr 21 [139-141] [Type ()]: Unit"#]],
    );
}

#[test]
fn generate_adj_self() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    body ... { B(1); B(2); }
                    adjoint self;
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-168]:
                    Namespace (Ident 27 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-166]:
                    Parent: 0
                    Callable 7 [67-166] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl 13 [114-138] (Body): Impl:
                                Pat 14 [119-122] [Type Qubit]: Elided
                                Block 15 [123-138] [Type ()]:
                                    Stmt 16 [125-130]: Semi: Expr 17 [125-129] [Type ()]: Call:
                                        Expr 18 [125-126] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 19 [126-129] [Type Int]: Paren: Expr 20 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 21 [131-136]: Semi: Expr 22 [131-135] [Type ()]: Call:
                                        Expr 23 [131-132] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 24 [132-135] [Type Int]: Paren: Expr 25 [133-134] [Type Int]: Lit: Int(2)
                            SpecDecl 26 [147-160] (Adj): Impl:
                                Pat 14 [119-122] [Type Qubit]: Elided
                                Block 15 [123-138] [Type ()]:
                                    Stmt 16 [125-130]: Semi: Expr 17 [125-129] [Type ()]: Call:
                                        Expr 18 [125-126] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 19 [126-129] [Type Int]: Paren: Expr 20 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 21 [131-136]: Semi: Expr 22 [131-135] [Type ()]: Call:
                                        Expr 23 [131-132] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 24 [132-135] [Type Int]: Paren: Expr 25 [133-134] [Type Int]: Lit: Int(2)"#]],
    );
}

#[test]
fn generate_ctladj_self() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Ctl + Adj {}
                operation A(q : Qubit) : Unit is Ctl + Adj {
                    body ... { B(1); B(2); }
                    adjoint self;
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-180]:
                    Namespace (Ident 31 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-65]: BinOp Union: (Functor Expr 6 [56-59]: Ctl) (Functor Expr 7 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 33 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                Item 2 [73-178]:
                    Parent: 0
                    Callable 9 [73-178] (Operation):
                        name: Ident 10 [83-84] "A"
                        input: Pat 11 [84-95] [Type Qubit]: Paren:
                            Pat 12 [85-94] [Type Qubit]: Bind: Ident 13 [85-86] "q"
                        output: ()
                        functors: Functor Expr 14 [106-115]: BinOp Union: (Functor Expr 15 [106-109]: Ctl) (Functor Expr 16 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 17 [126-150] (Body): Impl:
                                Pat 18 [131-134] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 30 [159-172] (Adj): Impl:
                                Pat 18 [131-134] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (Ctl): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 34 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Var: Local 34
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Var: Local 34
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (CtlAdj): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 34 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Var: Local 34
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Var: Local 34
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)"#]],
    );
}

#[test]
fn generate_adj_invert() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    B(1);
                    B(2);
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-141]:
                    Namespace (Ident 24 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-139]:
                    Parent: 0
                    Callable 7 [67-139] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-139] (Body): Impl:
                                Pat _id_ [104-139] [Type Qubit]: Elided
                                Block 13 [104-139] [Type ()]:
                                    Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type ()]: Call:
                                        Expr 16 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [115-118] [Type Int]: Paren: Expr 18 [116-117] [Type Int]: Lit: Int(1)
                                    Stmt 19 [128-133]: Semi: Expr 20 [128-132] [Type ()]: Call:
                                        Expr 21 [128-129] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 22 [129-132] [Type Int]: Paren: Expr 23 [130-131] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-139] (Adj): Impl:
                                Pat _id_ [67-139] [Type Qubit]: Elided
                                Block 13 [104-139] [Type ()]:
                                    Stmt 19 [128-133]: Semi: Expr 20 [128-132] [Type ()]: Call:
                                        Expr _id_ [128-129] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 21 [128-129] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 22 [129-132] [Type Int]: Paren: Expr 23 [130-131] [Type Int]: Lit: Int(2)
                                    Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type ()]: Call:
                                        Expr _id_ [114-115] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 16 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [115-118] [Type Int]: Paren: Expr 18 [116-117] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_adj_invert_skips_within_block() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
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
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-238]:
                    Namespace (Ident 38 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-236]:
                    Parent: 0
                    Callable 7 [67-236] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-236] (Body): Impl:
                                Pat _id_ [104-236] [Type Qubit]: Elided
                                Block 13 [104-236] [Type ()]:
                                    Stmt 14 [114-230]: Expr: Expr 15 [114-230] [Type ()]: Conjugate:
                                        Block 16 [121-168] [Type ()]:
                                            Stmt 17 [135-140]: Semi: Expr 18 [135-139] [Type ()]: Call:
                                                Expr 19 [135-136] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 20 [136-139] [Type Int]: Paren: Expr 21 [137-138] [Type Int]: Lit: Int(1)
                                            Stmt 22 [153-158]: Semi: Expr 23 [153-157] [Type ()]: Call:
                                                Expr 24 [153-154] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [154-157] [Type Int]: Paren: Expr 26 [155-156] [Type Int]: Lit: Int(2)
                                        Block 27 [183-230] [Type ()]:
                                            Stmt 28 [197-202]: Semi: Expr 29 [197-201] [Type ()]: Call:
                                                Expr 30 [197-198] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 31 [198-201] [Type Int]: Paren: Expr 32 [199-200] [Type Int]: Lit: Int(3)
                                            Stmt 33 [215-220]: Semi: Expr 34 [215-219] [Type ()]: Call:
                                                Expr 35 [215-216] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 36 [216-219] [Type Int]: Paren: Expr 37 [217-218] [Type Int]: Lit: Int(4)
                            SpecDecl _id_ [67-236] (Adj): Impl:
                                Pat _id_ [67-236] [Type Qubit]: Elided
                                Block 13 [104-236] [Type ()]:
                                    Stmt 14 [114-230]: Expr: Expr 15 [114-230] [Type ()]: Conjugate:
                                        Block 16 [121-168] [Type ()]:
                                            Stmt 17 [135-140]: Semi: Expr 18 [135-139] [Type ()]: Call:
                                                Expr 19 [135-136] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 20 [136-139] [Type Int]: Paren: Expr 21 [137-138] [Type Int]: Lit: Int(1)
                                            Stmt 22 [153-158]: Semi: Expr 23 [153-157] [Type ()]: Call:
                                                Expr 24 [153-154] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [154-157] [Type Int]: Paren: Expr 26 [155-156] [Type Int]: Lit: Int(2)
                                        Block 27 [183-230] [Type ()]:
                                            Stmt 33 [215-220]: Semi: Expr 34 [215-219] [Type ()]: Call:
                                                Expr _id_ [215-216] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 35 [215-216] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 36 [216-219] [Type Int]: Paren: Expr 37 [217-218] [Type Int]: Lit: Int(4)
                                            Stmt 28 [197-202]: Semi: Expr 29 [197-201] [Type ()]: Call:
                                                Expr _id_ [197-198] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 30 [197-198] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 31 [198-201] [Type Int]: Paren: Expr 32 [199-200] [Type Int]: Lit: Int(3)"#]],
    );
}

#[test]
fn generate_adj_invert_with_if_exprs() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    B(1);
                    let val = if true {false} else {true};
                    B(2);
                    if false {B(3); B(4);} else {B(5); let val = true; B(6);}
                    B(7);
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-268]:
                    Namespace (Ident 71 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-266]:
                    Parent: 0
                    Callable 7 [67-266] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-266] (Body): Impl:
                                Pat _id_ [104-266] [Type Qubit]: Elided
                                Block 13 [104-266] [Type ()]:
                                    Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type ()]: Call:
                                        Expr 16 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [115-118] [Type Int]: Paren: Expr 18 [116-117] [Type Int]: Lit: Int(1)
                                    Stmt 19 [128-166]: Local (Immutable):
                                        Pat 20 [132-135] [Type Bool]: Bind: Ident 21 [132-135] "val"
                                        Expr 22 [138-165] [Type Bool]: If:
                                            Expr 23 [141-145] [Type Bool]: Lit: Bool(true)
                                            Block 24 [146-153] [Type Bool]:
                                                Stmt 25 [147-152]: Expr: Expr 26 [147-152] [Type Bool]: Lit: Bool(false)
                                            Expr 27 [154-165] [Type Bool]: Expr Block: Block 28 [159-165] [Type Bool]:
                                                Stmt 29 [160-164]: Expr: Expr 30 [160-164] [Type Bool]: Lit: Bool(true)
                                    Stmt 31 [175-180]: Semi: Expr 32 [175-179] [Type ()]: Call:
                                        Expr 33 [175-176] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 34 [176-179] [Type Int]: Paren: Expr 35 [177-178] [Type Int]: Lit: Int(2)
                                    Stmt 36 [189-246]: Expr: Expr 37 [189-246] [Type ()]: If:
                                        Expr 38 [192-197] [Type Bool]: Lit: Bool(false)
                                        Block 39 [198-211] [Type ()]:
                                            Stmt 40 [199-204]: Semi: Expr 41 [199-203] [Type ()]: Call:
                                                Expr 42 [199-200] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 43 [200-203] [Type Int]: Paren: Expr 44 [201-202] [Type Int]: Lit: Int(3)
                                            Stmt 45 [205-210]: Semi: Expr 46 [205-209] [Type ()]: Call:
                                                Expr 47 [205-206] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 48 [206-209] [Type Int]: Paren: Expr 49 [207-208] [Type Int]: Lit: Int(4)
                                        Expr 50 [212-246] [Type ()]: Expr Block: Block 51 [217-246] [Type ()]:
                                            Stmt 52 [218-223]: Semi: Expr 53 [218-222] [Type ()]: Call:
                                                Expr 54 [218-219] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 55 [219-222] [Type Int]: Paren: Expr 56 [220-221] [Type Int]: Lit: Int(5)
                                            Stmt 57 [224-239]: Local (Immutable):
                                                Pat 58 [228-231] [Type Bool]: Bind: Ident 59 [228-231] "val"
                                                Expr 60 [234-238] [Type Bool]: Lit: Bool(true)
                                            Stmt 61 [240-245]: Semi: Expr 62 [240-244] [Type ()]: Call:
                                                Expr 63 [240-241] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 64 [241-244] [Type Int]: Paren: Expr 65 [242-243] [Type Int]: Lit: Int(6)
                                    Stmt 66 [255-260]: Semi: Expr 67 [255-259] [Type ()]: Call:
                                        Expr 68 [255-256] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 69 [256-259] [Type Int]: Paren: Expr 70 [257-258] [Type Int]: Lit: Int(7)
                            SpecDecl _id_ [67-266] (Adj): Impl:
                                Pat _id_ [67-266] [Type Qubit]: Elided
                                Block 13 [104-266] [Type ()]:
                                    Stmt 19 [128-166]: Local (Immutable):
                                        Pat 20 [132-135] [Type Bool]: Bind: Ident 21 [132-135] "val"
                                        Expr 22 [138-165] [Type Bool]: If:
                                            Expr 23 [141-145] [Type Bool]: Lit: Bool(true)
                                            Block 24 [146-153] [Type Bool]:
                                                Stmt 25 [147-152]: Expr: Expr 26 [147-152] [Type Bool]: Lit: Bool(false)
                                            Expr 27 [154-165] [Type Bool]: Expr Block: Block 28 [159-165] [Type Bool]:
                                                Stmt 29 [160-164]: Expr: Expr 30 [160-164] [Type Bool]: Lit: Bool(true)
                                    Stmt 66 [255-260]: Semi: Expr 67 [255-259] [Type ()]: Call:
                                        Expr _id_ [255-256] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 68 [255-256] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 69 [256-259] [Type Int]: Paren: Expr 70 [257-258] [Type Int]: Lit: Int(7)
                                    Stmt 36 [189-246]: Expr: Expr 37 [189-246] [Type ()]: If:
                                        Expr 38 [192-197] [Type Bool]: Lit: Bool(false)
                                        Block 39 [198-211] [Type ()]:
                                            Stmt 45 [205-210]: Semi: Expr 46 [205-209] [Type ()]: Call:
                                                Expr _id_ [205-206] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 47 [205-206] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 48 [206-209] [Type Int]: Paren: Expr 49 [207-208] [Type Int]: Lit: Int(4)
                                            Stmt 40 [199-204]: Semi: Expr 41 [199-203] [Type ()]: Call:
                                                Expr _id_ [199-200] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 42 [199-200] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 43 [200-203] [Type Int]: Paren: Expr 44 [201-202] [Type Int]: Lit: Int(3)
                                        Expr 50 [212-246] [Type ()]: Expr Block: Block 51 [217-246] [Type ()]:
                                            Stmt 57 [224-239]: Local (Immutable):
                                                Pat 58 [228-231] [Type Bool]: Bind: Ident 59 [228-231] "val"
                                                Expr 60 [234-238] [Type Bool]: Lit: Bool(true)
                                            Stmt 61 [240-245]: Semi: Expr 62 [240-244] [Type ()]: Call:
                                                Expr _id_ [240-241] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 63 [240-241] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 64 [241-244] [Type Int]: Paren: Expr 65 [242-243] [Type Int]: Lit: Int(6)
                                            Stmt 52 [218-223]: Semi: Expr 53 [218-222] [Type ()]: Call:
                                                Expr _id_ [218-219] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 54 [218-219] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 55 [219-222] [Type Int]: Paren: Expr 56 [220-221] [Type Int]: Lit: Int(5)
                                    Stmt 31 [175-180]: Semi: Expr 32 [175-179] [Type ()]: Call:
                                        Expr _id_ [175-176] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 33 [175-176] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 34 [176-179] [Type Int]: Paren: Expr 35 [177-178] [Type Int]: Lit: Int(2)
                                    Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type ()]: Call:
                                        Expr _id_ [114-115] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 16 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [115-118] [Type Int]: Paren: Expr 18 [116-117] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_adj_invert_with_range_loop() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    for i in 0..5 {
                        B(1);
                        B(2);
                    }
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-183]:
                    Namespace (Ident 32 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-181]:
                    Parent: 0
                    Callable 7 [67-181] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-181] (Body): Impl:
                                Pat _id_ [104-181] [Type Qubit]: Elided
                                Block 13 [104-181] [Type ()]:
                                    Stmt 14 [114-175]: Expr: Expr 15 [114-175] [Type ()]: For:
                                        Pat 16 [118-119] [Type Int]: Bind: Ident 17 [118-119] "i"
                                        Expr 18 [123-127] [Type Range]: Range:
                                            Expr 19 [123-124] [Type Int]: Lit: Int(0)
                                            <no step>
                                            Expr 20 [126-127] [Type Int]: Lit: Int(5)
                                        Block 21 [128-175] [Type ()]:
                                            Stmt 22 [142-147]: Semi: Expr 23 [142-146] [Type ()]: Call:
                                                Expr 24 [142-143] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 25 [143-146] [Type Int]: Paren: Expr 26 [144-145] [Type Int]: Lit: Int(1)
                                            Stmt 27 [160-165]: Semi: Expr 28 [160-164] [Type ()]: Call:
                                                Expr 29 [160-161] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 30 [161-164] [Type Int]: Paren: Expr 31 [162-163] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-181] (Adj): Impl:
                                Pat _id_ [67-181] [Type Qubit]: Elided
                                Block 13 [104-181] [Type ()]:
                                    Stmt 14 [114-175]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type Range]: Bind: Ident 33 [0-0] "generated_range"
                                            Expr 18 [123-127] [Type Range]: Range:
                                                Expr 19 [123-124] [Type Int]: Lit: Int(0)
                                                <no step>
                                                Expr 20 [126-127] [Type Int]: Lit: Int(5)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                            Pat 16 [118-119] [Type Int]: Bind: Ident 17 [118-119] "i"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Add):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                        Start
                                                    Expr _id_ [0-0] [Type Int]: BinOp (Mul):
                                                        Expr _id_ [0-0] [Type Int]: BinOp (Div):
                                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                                    End
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                                    Start
                                                            Expr _id_ [0-0] [Type Int]: Field:
                                                                Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                                Step
                                                        Expr _id_ [0-0] [Type Int]: Field:
                                                            Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                            Step
                                                Expr _id_ [0-0] [Type Int]: UnOp (Neg):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                        Step
                                                Expr _id_ [0-0] [Type Int]: Field:
                                                    Expr _id_ [0-0] [Type Range]: Var: Local 33
                                                    Start
                                            Block 21 [128-175] [Type ()]:
                                                Stmt 27 [160-165]: Semi: Expr 28 [160-164] [Type ()]: Call:
                                                    Expr _id_ [160-161] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 29 [160-161] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 30 [161-164] [Type Int]: Paren: Expr 31 [162-163] [Type Int]: Lit: Int(2)
                                                Stmt 22 [142-147]: Semi: Expr 23 [142-146] [Type ()]: Call:
                                                    Expr _id_ [142-143] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 24 [142-143] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 25 [143-146] [Type Int]: Paren: Expr 26 [144-145] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_adj_invert_with_array_loop() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    for val in [0, 1, 2] {
                        B(1);
                        B(2);
                    }
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-190]:
                    Namespace (Ident 33 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-188]:
                    Parent: 0
                    Callable 7 [67-188] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-188] (Body): Impl:
                                Pat _id_ [104-188] [Type Qubit]: Elided
                                Block 13 [104-188] [Type ()]:
                                    Stmt 14 [114-182]: Expr: Expr 15 [114-182] [Type ()]: For:
                                        Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                        Expr 18 [125-134] [Type (Int)[]]: Array:
                                            Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                        Block 22 [135-182] [Type ()]:
                                            Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type ()]: Call:
                                                Expr 25 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 26 [150-153] [Type Int]: Paren: Expr 27 [151-152] [Type Int]: Lit: Int(1)
                                            Stmt 28 [167-172]: Semi: Expr 29 [167-171] [Type ()]: Call:
                                                Expr 30 [167-168] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 31 [168-171] [Type Int]: Paren: Expr 32 [169-170] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-188] (Adj): Impl:
                                Pat _id_ [67-188] [Type Qubit]: Elided
                                Block 13 [104-188] [Type ()]:
                                    Stmt 14 [114-182]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 34 [0-0] "generated_array"
                                            Expr 18 [125-134] [Type (Int)[]]: Array:
                                                Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                                Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                                Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                            Pat _id_ [0-0] [Type Int]: Bind: Ident 35 [0-0] "generated_index"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 34
                                                        Length
                                                    Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                            Block 22 [135-182] [Type ()]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                                    Expr _id_ [0-0] [Type Int]: Index:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 34
                                                        Expr _id_ [0-0] [Type Int]: Var: Local 35
                                                Stmt 28 [167-172]: Semi: Expr 29 [167-171] [Type ()]: Call:
                                                    Expr _id_ [167-168] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 30 [167-168] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 31 [168-171] [Type Int]: Paren: Expr 32 [169-170] [Type Int]: Lit: Int(2)
                                                Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type ()]: Call:
                                                    Expr _id_ [149-150] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 25 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 26 [150-153] [Type Int]: Paren: Expr 27 [151-152] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_adj_invert_with_nested_loops() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    for val in [0, 1, 2] {
                        B(1);
                        let arr = [true, false, true];
                        for val in arr {
                            B(2);
                            B(3);
                        }
                        B(4);
                    }
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-320]:
                    Namespace (Ident 56 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 6 [60-62]: <empty>
                Item 2 [67-318]:
                    Parent: 0
                    Callable 7 [67-318] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [78-89] [Type Qubit]: Paren:
                            Pat 10 [79-88] [Type Qubit]: Bind: Ident 11 [79-80] "q"
                        output: ()
                        functors: Functor Expr 12 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-318] (Body): Impl:
                                Pat _id_ [104-318] [Type Qubit]: Elided
                                Block 13 [104-318] [Type ()]:
                                    Stmt 14 [114-312]: Expr: Expr 15 [114-312] [Type ()]: For:
                                        Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                        Expr 18 [125-134] [Type (Int)[]]: Array:
                                            Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                        Block 22 [135-312] [Type ()]:
                                            Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type ()]: Call:
                                                Expr 25 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 26 [150-153] [Type Int]: Paren: Expr 27 [151-152] [Type Int]: Lit: Int(1)
                                            Stmt 28 [167-197]: Local (Immutable):
                                                Pat 29 [171-174] [Type (Bool)[]]: Bind: Ident 30 [171-174] "arr"
                                                Expr 31 [177-196] [Type (Bool)[]]: Array:
                                                    Expr 32 [178-182] [Type Bool]: Lit: Bool(true)
                                                    Expr 33 [184-189] [Type Bool]: Lit: Bool(false)
                                                    Expr 34 [191-195] [Type Bool]: Lit: Bool(true)
                                            Stmt 35 [210-284]: Expr: Expr 36 [210-284] [Type ()]: For:
                                                Pat 37 [214-217] [Type Bool]: Bind: Ident 38 [214-217] "val"
                                                Expr 39 [221-224] [Type (Bool)[]]: Var: Local 30
                                                Block 40 [225-284] [Type ()]:
                                                    Stmt 41 [243-248]: Semi: Expr 42 [243-247] [Type ()]: Call:
                                                        Expr 43 [243-244] [Type (Int => () is Adj)]: Var: Item 1
                                                        Expr 44 [244-247] [Type Int]: Paren: Expr 45 [245-246] [Type Int]: Lit: Int(2)
                                                    Stmt 46 [265-270]: Semi: Expr 47 [265-269] [Type ()]: Call:
                                                        Expr 48 [265-266] [Type (Int => () is Adj)]: Var: Item 1
                                                        Expr 49 [266-269] [Type Int]: Paren: Expr 50 [267-268] [Type Int]: Lit: Int(3)
                                            Stmt 51 [297-302]: Semi: Expr 52 [297-301] [Type ()]: Call:
                                                Expr 53 [297-298] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 54 [298-301] [Type Int]: Paren: Expr 55 [299-300] [Type Int]: Lit: Int(4)
                            SpecDecl _id_ [67-318] (Adj): Impl:
                                Pat _id_ [67-318] [Type Qubit]: Elided
                                Block 13 [104-318] [Type ()]:
                                    Stmt 14 [114-312]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 59 [0-0] "generated_array"
                                            Expr 18 [125-134] [Type (Int)[]]: Array:
                                                Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                                Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                                Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                            Pat _id_ [0-0] [Type Int]: Bind: Ident 60 [0-0] "generated_index"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 59
                                                        Length
                                                    Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                            Block 22 [135-312] [Type ()]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                                    Expr _id_ [0-0] [Type Int]: Index:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 59
                                                        Expr _id_ [0-0] [Type Int]: Var: Local 60
                                                Stmt 28 [167-197]: Local (Immutable):
                                                    Pat 29 [171-174] [Type (Bool)[]]: Bind: Ident 30 [171-174] "arr"
                                                    Expr 31 [177-196] [Type (Bool)[]]: Array:
                                                        Expr 32 [178-182] [Type Bool]: Lit: Bool(true)
                                                        Expr 33 [184-189] [Type Bool]: Lit: Bool(false)
                                                        Expr 34 [191-195] [Type Bool]: Lit: Bool(true)
                                                Stmt 51 [297-302]: Semi: Expr 52 [297-301] [Type ()]: Call:
                                                    Expr _id_ [297-298] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 53 [297-298] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 54 [298-301] [Type Int]: Paren: Expr 55 [299-300] [Type Int]: Lit: Int(4)
                                                Stmt 35 [210-284]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                                    Stmt _id_ [0-0]: Local (Immutable):
                                                        Pat _id_ [0-0] [Type (Bool)[]]: Bind: Ident 57 [0-0] "generated_array"
                                                        Expr 39 [221-224] [Type (Bool)[]]: Var: Local 30
                                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                                        Pat _id_ [0-0] [Type Int]: Bind: Ident 58 [0-0] "generated_index"
                                                        Expr _id_ [0-0] [Type Range]: Range:
                                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 57
                                                                    Length
                                                                Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                                        Block 40 [225-284] [Type ()]:
                                                            Stmt _id_ [0-0]: Local (Immutable):
                                                                Pat 37 [214-217] [Type Bool]: Bind: Ident 38 [214-217] "val"
                                                                Expr _id_ [0-0] [Type Bool]: Index:
                                                                    Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 57
                                                                    Expr _id_ [0-0] [Type Int]: Var: Local 58
                                                            Stmt 46 [265-270]: Semi: Expr 47 [265-269] [Type ()]: Call:
                                                                Expr _id_ [265-266] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                                    Expr 48 [265-266] [Type (Int => () is Adj)]: Var: Item 1
                                                                Expr 49 [266-269] [Type Int]: Paren: Expr 50 [267-268] [Type Int]: Lit: Int(3)
                                                            Stmt 41 [243-248]: Semi: Expr 42 [243-247] [Type ()]: Call:
                                                                Expr _id_ [243-244] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                                    Expr 43 [243-244] [Type (Int => () is Adj)]: Var: Item 1
                                                                Expr 44 [244-247] [Type Int]: Paren: Expr 45 [245-246] [Type Int]: Lit: Int(2)
                                                Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type ()]: Call:
                                                    Expr _id_ [149-150] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 25 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 26 [150-153] [Type Int]: Paren: Expr 27 [151-152] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_ctladj_distribute() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Ctl + Adj {}
                operation A(q : Qubit) : Unit is Ctl + Adj {
                    body ... { B(1); B(2); }
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-158]:
                    Namespace (Ident 30 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-65]: BinOp Union: (Functor Expr 6 [56-59]: Ctl) (Functor Expr 7 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 31 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                Item 2 [73-156]:
                    Parent: 0
                    Callable 9 [73-156] (Operation):
                        name: Ident 10 [83-84] "A"
                        input: Pat 11 [84-95] [Type Qubit]: Paren:
                            Pat 12 [85-94] [Type Qubit]: Bind: Ident 13 [85-86] "q"
                        output: ()
                        functors: Functor Expr 14 [106-115]: BinOp Union: (Functor Expr 15 [106-109]: Ctl) (Functor Expr 16 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 17 [126-150] (Body): Impl:
                                Pat 18 [131-134] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-156] (Adj): Impl:
                                Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                            SpecDecl _id_ [73-156] (Ctl): Impl:
                                Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 33 [73-156] "ctls"
                                    Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Var: Local 33
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Var: Local 33
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-156] (CtlAdj): Impl:
                                Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 34 [73-156] "ctls"
                                    Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr _id_ [143-144] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Var: Local 34
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr _id_ [137-138] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Var: Local 34
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_ctladj_invert() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Ctl + Adj {}
                operation A(q : Qubit) : Unit is Ctl + Adj {
                    body ... { B(1); B(2); }
                    controlled adjoint invert;
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-193]:
                    Namespace (Ident 31 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45] [Type Int]: Paren:
                            Pat 3 [33-44] [Type Int]: Bind: Ident 4 [33-38] "input"
                        output: ()
                        functors: Functor Expr 5 [56-65]: BinOp Union: (Functor Expr 6 [56-59]: Ctl) (Functor Expr 7 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 33 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                Item 2 [73-191]:
                    Parent: 0
                    Callable 9 [73-191] (Operation):
                        name: Ident 10 [83-84] "A"
                        input: Pat 11 [84-95] [Type Qubit]: Paren:
                            Pat 12 [85-94] [Type Qubit]: Bind: Ident 13 [85-86] "q"
                        output: ()
                        functors: Functor Expr 14 [106-115]: BinOp Union: (Functor Expr 15 [106-109]: Ctl) (Functor Expr 16 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 17 [126-150] (Body): Impl:
                                Pat 18 [131-134] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-191] (Adj): Impl:
                                Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                            SpecDecl _id_ [73-191] (Ctl): Impl:
                                Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 34 [73-191] "ctls"
                                    Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Var: Local 34
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Var: Local 34
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 30 [159-185] (CtlAdj): Impl:
                                Pat _id_ [159-185] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Var: Local 34
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Var: Local 34
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)"#]],
    );
}
