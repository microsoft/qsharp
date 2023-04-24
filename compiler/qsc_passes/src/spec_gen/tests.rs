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
    assert!(
        unit.context.errors().is_empty(),
        "Compilation errors: {:?}",
        unit.context.errors()
    );
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
                                        Expr 24 [171-172] [Type (Qubit => () is Ctl)]: Name: Item 1
                                        Expr 25 [172-175] [Type Qubit]: Paren: Expr 26 [173-174] [Type Qubit]: Name: Local 19
                            SpecDecl _id_ [124-182] (Ctl): Impl:
                                Pat _id_ [124-182] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-182] [Type (Qubit)[]]: Bind: Ident 28 [124-182] "ctls"
                                    Pat _id_ [124-182] [Type Qubit]: Elided
                                Block 21 [161-182] [Type ()]:
                                    Stmt 22 [171-176]: Semi: Expr 23 [171-175] [Type ()]: Call:
                                        Expr 24 [171-172] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                            Expr 24 [171-172] [Type (Qubit => () is Ctl)]: Name: Item 1
                                        Expr 25 [172-175] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [172-175] [Type (Qubit)[]]: Name: Local 28
                                            Expr 25 [172-175] [Type Qubit]: Paren: Expr 26 [173-174] [Type Qubit]: Name: Local 19"#]],
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
                                        Expr 33 [229-230] [Type (Qubit => () is Adj + Ctl)]: Name: Item 1
                                        Expr 34 [230-233] [Type Qubit]: Paren: Expr 35 [231-232] [Type Qubit]: Name: Local 24
                            SpecDecl 36 [253-302] (Adj): Impl:
                                Pat 37 [261-264] [Type Qubit]: Elided
                                Block 38 [265-302] [Type ()]:
                                    Stmt 39 [279-292]: Semi: Expr 40 [279-291] [Type ()]: Call:
                                        Expr 41 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 42 [287-288] [Type (Qubit => () is Adj + Ctl)]: Name: Item 1
                                        Expr 43 [288-291] [Type Qubit]: Paren: Expr 44 [289-290] [Type Qubit]: Name: Local 24
                            SpecDecl _id_ [153-308] (Ctl): Impl:
                                Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 47 [153-308] "ctls"
                                    Pat _id_ [153-308] [Type Qubit]: Elided
                                Block 30 [215-244] [Type ()]:
                                    Stmt 31 [229-234]: Semi: Expr 32 [229-233] [Type ()]: Call:
                                        Expr 33 [229-230] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 33 [229-230] [Type (Qubit => () is Adj + Ctl)]: Name: Item 1
                                        Expr 34 [230-233] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [230-233] [Type (Qubit)[]]: Name: Local 47
                                            Expr 34 [230-233] [Type Qubit]: Paren: Expr 35 [231-232] [Type Qubit]: Name: Local 24
                            SpecDecl _id_ [153-308] (CtlAdj): Impl:
                                Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 48 [153-308] "ctls"
                                    Pat _id_ [153-308] [Type Qubit]: Elided
                                Block 38 [265-302] [Type ()]:
                                    Stmt 39 [279-292]: Semi: Expr 40 [279-291] [Type ()]: Call:
                                        Expr 41 [279-288] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 41 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 42 [287-288] [Type (Qubit => () is Adj + Ctl)]: Name: Item 1
                                        Expr 43 [288-291] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [288-291] [Type (Qubit)[]]: Name: Local 48
                                            Expr 43 [288-291] [Type Qubit]: Paren: Expr 44 [289-290] [Type Qubit]: Name: Local 24"#]],
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
                                                Expr 27 [192-193] [Type (Qubit => () is Ctl)]: Name: Item 1
                                                Expr 28 [193-196] [Type Qubit]: Paren: Expr 29 [194-195] [Type Qubit]: Name: Local 19
                                        Block 30 [222-251] [Type ()]:
                                            Stmt 31 [236-241]: Semi: Expr 32 [236-240] [Type ()]: Call:
                                                Expr 33 [236-237] [Type (Qubit => () is Ctl)]: Name: Item 1
                                                Expr 34 [237-240] [Type Qubit]: Paren: Expr 35 [238-239] [Type Qubit]: Name: Local 19
                            SpecDecl _id_ [124-257] (Ctl): Impl:
                                Pat _id_ [124-257] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-257] [Type (Qubit)[]]: Bind: Ident 37 [124-257] "ctls"
                                    Pat _id_ [124-257] [Type Qubit]: Elided
                                Block 21 [161-257] [Type ()]:
                                    Stmt 22 [171-251]: Expr: Expr 23 [171-251] [Type ()]: Conjugate:
                                        Block 24 [178-207] [Type ()]:
                                            Stmt 25 [192-197]: Semi: Expr 26 [192-196] [Type ()]: Call:
                                                Expr 27 [192-193] [Type (Qubit => () is Ctl)]: Name: Item 1
                                                Expr 28 [193-196] [Type Qubit]: Paren: Expr 29 [194-195] [Type Qubit]: Name: Local 19
                                        Block 30 [222-251] [Type ()]:
                                            Stmt 31 [236-241]: Semi: Expr 32 [236-240] [Type ()]: Call:
                                                Expr 33 [236-237] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                                    Expr 33 [236-237] [Type (Qubit => () is Ctl)]: Name: Item 1
                                                Expr 34 [237-240] [Type ((Qubit)[], Qubit)]: Tuple:
                                                    Expr _id_ [237-240] [Type (Qubit)[]]: Name: Local 37
                                                    Expr 34 [237-240] [Type Qubit]: Paren: Expr 35 [238-239] [Type Qubit]: Name: Local 19"#]],
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
                                        Expr 16 [123-126] [Type (() -> ())]: Name: Item 1
                                        Expr 17 [126-128] [Type ()]: Unit
                                    Stmt 18 [138-142]: Semi: Expr 19 [138-141] [Type ()]: Call:
                                        Expr 20 [138-139] [Type (() => () is Ctl)]: Name: Item 2
                                        Expr 21 [139-141] [Type ()]: Unit
                            SpecDecl _id_ [85-148] (Ctl): Impl:
                                Pat _id_ [85-148] [Type ((Qubit)[], ())]: Tuple:
                                    Pat _id_ [85-148] [Type (Qubit)[]]: Bind: Ident 24 [85-148] "ctls"
                                    Pat _id_ [85-148] [Type ()]: Elided
                                Block 13 [113-148] [Type ()]:
                                    Stmt 14 [123-129]: Semi: Expr 15 [123-128] [Type ()]: Call:
                                        Expr 16 [123-126] [Type (() -> ())]: Name: Item 1
                                        Expr 17 [126-128] [Type ()]: Unit
                                    Stmt 18 [138-142]: Semi: Expr 19 [138-141] [Type ()]: Call:
                                        Expr 20 [138-139] [Type (((Qubit)[], ()) => () is Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [138-139] [Type (() => () is Ctl)]: Name: Item 2
                                        Expr 21 [139-141] [Type ((Qubit)[], ())]: Tuple:
                                            Expr _id_ [139-141] [Type (Qubit)[]]: Name: Local 24
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
                            SpecDecl _id_ [21-62] (Adj): Gen: Invert
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
                                        Expr 18 [125-126] [Type (Int => () is Adj)]: Name: Item 1
                                        Expr 19 [126-129] [Type Int]: Paren: Expr 20 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 21 [131-136]: Semi: Expr 22 [131-135] [Type ()]: Call:
                                        Expr 23 [131-132] [Type (Int => () is Adj)]: Name: Item 1
                                        Expr 24 [132-135] [Type Int]: Paren: Expr 25 [133-134] [Type Int]: Lit: Int(2)
                            SpecDecl 26 [147-160] (Adj): Impl:
                                Pat 14 [119-122] [Type Qubit]: Elided
                                Block 15 [123-138] [Type ()]:
                                    Stmt 16 [125-130]: Semi: Expr 17 [125-129] [Type ()]: Call:
                                        Expr 18 [125-126] [Type (Int => () is Adj)]: Name: Item 1
                                        Expr 19 [126-129] [Type Int]: Paren: Expr 20 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 21 [131-136]: Semi: Expr 22 [131-135] [Type ()]: Call:
                                        Expr 23 [131-132] [Type (Int => () is Adj)]: Name: Item 1
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
                            SpecDecl _id_ [21-68] (Adj): Gen: Invert
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 8 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Gen: Distribute
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
                                        Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 30 [159-172] (Adj): Impl:
                                Pat 18 [131-134] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (Ctl): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 33 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Name: Local 33
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Name: Local 33
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (CtlAdj): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 33 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 19 [135-150] [Type ()]:
                                    Stmt 20 [137-142]: Semi: Expr 21 [137-141] [Type ()]: Call:
                                        Expr 22 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 23 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [138-141] [Type (Qubit)[]]: Name: Local 33
                                            Expr 23 [138-141] [Type Int]: Paren: Expr 24 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 25 [143-148]: Semi: Expr 26 [143-147] [Type ()]: Call:
                                        Expr 27 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 27 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Item 1
                                        Expr 28 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [144-147] [Type (Qubit)[]]: Name: Local 33
                                            Expr 28 [144-147] [Type Int]: Paren: Expr 29 [145-146] [Type Int]: Lit: Int(2)"#]],
    );
}
