// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::too_many_lines)]

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_frontend::compile::{self, compile, PackageStore, SourceMap};

use crate::spec_gen::generate_specs;

fn check(file: &str, expect: &Expect) {
    let store = PackageStore::new(compile::core());
    let sources = SourceMap::new([("test".into(), file.into())], None);
    let mut unit = compile(&store, &[], sources);
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
                    Namespace (Ident 24 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: ()
                        functors: Functor Expr 4 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 5 [68-79] (Body): Impl:
                                Pat 6 [73-76] [Type Qubit]: Elided
                                Block 7 [77-79]: <empty>
                            SpecDecl 8 [88-113] (Ctl): Impl:
                                Pat 9 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 10 [100-104] [Type (Qubit)[]]: Bind: Ident 11 [100-104] "ctls"
                                    Pat 12 [106-109] [Type Qubit]: Elided
                                Block 13 [111-113]: <empty>
                Item 2 [124-182]:
                    Parent: 0
                    Callable 14 [124-182] (Operation):
                        name: Ident 15 [134-135] "B"
                        input: Pat 16 [136-145] [Type Qubit]: Bind: Ident 17 [136-137] "q"
                        output: ()
                        functors: Functor Expr 18 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl _id_ [161-182] (Body): Impl:
                                Pat _id_ [161-182] [Type Qubit]: Elided
                                Block 19 [161-182] [Type ()]:
                                    Stmt 20 [171-176]: Semi: Expr 21 [171-175] [Type ()]: Call:
                                        Expr 22 [171-172] [Type (Qubit => () is Ctl)]: Var: Item 1
                                        Expr 23 [173-174] [Type Qubit]: Var: Local 17
                            SpecDecl _id_ [124-182] (Ctl): Impl:
                                Pat _id_ [124-182] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-182] [Type (Qubit)[]]: Bind: Ident 25 [124-182] "ctls"
                                    Pat _id_ [124-182] [Type Qubit]: Elided
                                Block 19 [161-182] [Type ()]:
                                    Stmt 20 [171-176]: Semi: Expr 21 [171-175] [Type ()]: Call:
                                        Expr 22 [171-172] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [171-172] [Type (Qubit => () is Ctl)]: Var: Item 1
                                        Expr 23 [173-174] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [173-174] [Type (Qubit)[]]: Var: Local 25
                                            Expr 23 [173-174] [Type Qubit]: Var: Local 17"#]],
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
                    Namespace (Ident 41 [10-14] "test"): Item 1, Item 2
                Item 1 [21-148]:
                    Parent: 0
                    Callable 0 [21-148] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: ()
                        functors: Functor Expr 4 [54-63]: BinOp Union: (Functor Expr 5 [54-57]: Ctl) (Functor Expr 6 [60-63]: Adj)
                        body: Specializations:
                            SpecDecl 7 [74-85] (Body): Impl:
                                Pat 8 [79-82] [Type Qubit]: Elided
                                Block 9 [83-85]: <empty>
                            SpecDecl 10 [94-108] (Adj): Impl:
                                Pat 11 [102-105] [Type Qubit]: Elided
                                Block 12 [106-108]: <empty>
                            SpecDecl 13 [117-142] (Ctl): Impl:
                                Pat 14 [128-139] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 15 [129-133] [Type (Qubit)[]]: Bind: Ident 16 [129-133] "ctls"
                                    Pat 17 [135-138] [Type Qubit]: Elided
                                Block 18 [140-142]: <empty>
                            SpecDecl _id_ [21-148] (CtlAdj): Impl:
                                Pat _id_ [21-148] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [21-148] [Type (Qubit)[]]: Bind: Ident 42 [21-148] "ctls"
                                    Pat _id_ [21-148] [Type Qubit]: Elided
                                Block 12 [106-108]: <empty>
                Item 2 [153-308]:
                    Parent: 0
                    Callable 19 [153-308] (Operation):
                        name: Ident 20 [163-164] "B"
                        input: Pat 21 [165-174] [Type Qubit]: Bind: Ident 22 [165-166] "q"
                        output: ()
                        functors: Functor Expr 23 [186-195]: BinOp Union: (Functor Expr 24 [186-189]: Ctl) (Functor Expr 25 [192-195]: Adj)
                        body: Specializations:
                            SpecDecl 26 [206-244] (Body): Impl:
                                Pat 27 [211-214] [Type Qubit]: Elided
                                Block 28 [215-244] [Type ()]:
                                    Stmt 29 [229-234]: Semi: Expr 30 [229-233] [Type ()]: Call:
                                        Expr 31 [229-230] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 32 [231-232] [Type Qubit]: Var: Local 22
                            SpecDecl 33 [253-302] (Adj): Impl:
                                Pat 34 [261-264] [Type Qubit]: Elided
                                Block 35 [265-302] [Type ()]:
                                    Stmt 36 [279-292]: Semi: Expr 37 [279-291] [Type ()]: Call:
                                        Expr 38 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 39 [287-288] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 40 [289-290] [Type Qubit]: Var: Local 22
                            SpecDecl _id_ [153-308] (Ctl): Impl:
                                Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 43 [153-308] "ctls"
                                    Pat _id_ [153-308] [Type Qubit]: Elided
                                Block 28 [215-244] [Type ()]:
                                    Stmt 29 [229-234]: Semi: Expr 30 [229-233] [Type ()]: Call:
                                        Expr 31 [229-230] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 31 [229-230] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 32 [231-232] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [231-232] [Type (Qubit)[]]: Var: Local 43
                                            Expr 32 [231-232] [Type Qubit]: Var: Local 22
                            SpecDecl _id_ [153-308] (CtlAdj): Impl:
                                Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 44 [153-308] "ctls"
                                    Pat _id_ [153-308] [Type Qubit]: Elided
                                Block 35 [265-302] [Type ()]:
                                    Stmt 36 [279-292]: Semi: Expr 37 [279-291] [Type ()]: Call:
                                        Expr 38 [279-288] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 38 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 39 [287-288] [Type (Qubit => () is Adj + Ctl)]: Var: Item 1
                                        Expr 40 [289-290] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [289-290] [Type (Qubit)[]]: Var: Local 44
                                            Expr 40 [289-290] [Type Qubit]: Var: Local 22"#]],
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
                    Namespace (Ident 32 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: ()
                        functors: Functor Expr 4 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 5 [68-79] (Body): Impl:
                                Pat 6 [73-76] [Type Qubit]: Elided
                                Block 7 [77-79]: <empty>
                            SpecDecl 8 [88-113] (Ctl): Impl:
                                Pat 9 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 10 [100-104] [Type (Qubit)[]]: Bind: Ident 11 [100-104] "ctls"
                                    Pat 12 [106-109] [Type Qubit]: Elided
                                Block 13 [111-113]: <empty>
                Item 2 [124-257]:
                    Parent: 0
                    Callable 14 [124-257] (Operation):
                        name: Ident 15 [134-135] "B"
                        input: Pat 16 [136-145] [Type Qubit]: Bind: Ident 17 [136-137] "q"
                        output: ()
                        functors: Functor Expr 18 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl _id_ [161-257] (Body): Impl:
                                Pat _id_ [161-257] [Type Qubit]: Elided
                                Block 19 [161-257] [Type ()]:
                                    Stmt 20 [171-251]: Expr: Expr 21 [171-251] [Type ()]: Conjugate:
                                        Block 22 [178-207] [Type ()]:
                                            Stmt 23 [192-197]: Semi: Expr 24 [192-196] [Type ()]: Call:
                                                Expr 25 [192-193] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 26 [194-195] [Type Qubit]: Var: Local 17
                                        Block 27 [222-251] [Type ()]:
                                            Stmt 28 [236-241]: Semi: Expr 29 [236-240] [Type ()]: Call:
                                                Expr 30 [236-237] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 31 [238-239] [Type Qubit]: Var: Local 17
                            SpecDecl _id_ [124-257] (Ctl): Impl:
                                Pat _id_ [124-257] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-257] [Type (Qubit)[]]: Bind: Ident 33 [124-257] "ctls"
                                    Pat _id_ [124-257] [Type Qubit]: Elided
                                Block 19 [161-257] [Type ()]:
                                    Stmt 20 [171-251]: Expr: Expr 21 [171-251] [Type ()]: Conjugate:
                                        Block 22 [178-207] [Type ()]:
                                            Stmt 23 [192-197]: Semi: Expr 24 [192-196] [Type ()]: Call:
                                                Expr 25 [192-193] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 26 [194-195] [Type Qubit]: Var: Local 17
                                        Block 27 [222-251] [Type ()]:
                                            Stmt 28 [236-241]: Semi: Expr 29 [236-240] [Type ()]: Call:
                                                Expr 30 [236-237] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                                    Expr 30 [236-237] [Type (Qubit => () is Ctl)]: Var: Item 1
                                                Expr 31 [238-239] [Type ((Qubit)[], Qubit)]: Tuple:
                                                    Expr _id_ [238-239] [Type (Qubit)[]]: Var: Local 33
                                                    Expr 31 [238-239] [Type Qubit]: Var: Local 17"#]],
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
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-166]:
                    Parent: 0
                    Callable 6 [67-166] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl 11 [114-138] (Body): Impl:
                                Pat 12 [119-122] [Type Qubit]: Elided
                                Block 13 [123-138] [Type ()]:
                                    Stmt 14 [125-130]: Semi: Expr 15 [125-129] [Type ()]: Call:
                                        Expr 16 [125-126] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 18 [131-136]: Semi: Expr 19 [131-135] [Type ()]: Call:
                                        Expr 20 [131-132] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 21 [133-134] [Type Int]: Lit: Int(2)
                            SpecDecl 22 [147-160] (Adj): Impl:
                                Pat 12 [119-122] [Type Qubit]: Elided
                                Block 13 [123-138] [Type ()]:
                                    Stmt 14 [125-130]: Semi: Expr 15 [125-129] [Type ()]: Call:
                                        Expr 16 [125-126] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 17 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 18 [131-136]: Semi: Expr 19 [131-135] [Type ()]: Call:
                                        Expr 20 [131-132] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 21 [133-134] [Type Int]: Lit: Int(2)"#]],
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
                    Namespace (Ident 27 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-65]: BinOp Union: (Functor Expr 5 [56-59]: Ctl) (Functor Expr 6 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 28 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 29 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                Item 2 [73-178]:
                    Parent: 0
                    Callable 8 [73-178] (Operation):
                        name: Ident 9 [83-84] "A"
                        input: Pat 10 [85-94] [Type Qubit]: Bind: Ident 11 [85-86] "q"
                        output: ()
                        functors: Functor Expr 12 [106-115]: BinOp Union: (Functor Expr 13 [106-109]: Ctl) (Functor Expr 14 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 15 [126-150] (Body): Impl:
                                Pat 16 [131-134] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 26 [159-172] (Adj): Impl:
                                Pat 16 [131-134] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (Ctl): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 30 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 30
                                            Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 30
                                            Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (CtlAdj): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 30 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 30
                                            Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 30
                                            Expr 25 [145-146] [Type Int]: Lit: Int(2)"#]],
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
                    Namespace (Ident 20 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-139]:
                    Parent: 0
                    Callable 6 [67-139] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-139] (Body): Impl:
                                Pat _id_ [104-139] [Type Qubit]: Elided
                                Block 11 [104-139] [Type ()]:
                                    Stmt 12 [114-119]: Semi: Expr 13 [114-118] [Type ()]: Call:
                                        Expr 14 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 15 [116-117] [Type Int]: Lit: Int(1)
                                    Stmt 16 [128-133]: Semi: Expr 17 [128-132] [Type ()]: Call:
                                        Expr 18 [128-129] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 19 [130-131] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-139] (Adj): Impl:
                                Pat _id_ [67-139] [Type Qubit]: Elided
                                Block 11 [104-139] [Type ()]:
                                    Stmt 16 [128-133]: Semi: Expr 17 [128-132] [Type ()]: Call:
                                        Expr _id_ [128-129] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 18 [128-129] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 19 [130-131] [Type Int]: Lit: Int(2)
                                    Stmt 12 [114-119]: Semi: Expr 13 [114-118] [Type ()]: Call:
                                        Expr _id_ [114-115] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 14 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 15 [116-117] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 32 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-236]:
                    Parent: 0
                    Callable 6 [67-236] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-236] (Body): Impl:
                                Pat _id_ [104-236] [Type Qubit]: Elided
                                Block 11 [104-236] [Type ()]:
                                    Stmt 12 [114-230]: Expr: Expr 13 [114-230] [Type ()]: Conjugate:
                                        Block 14 [121-168] [Type ()]:
                                            Stmt 15 [135-140]: Semi: Expr 16 [135-139] [Type ()]: Call:
                                                Expr 17 [135-136] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 18 [137-138] [Type Int]: Lit: Int(1)
                                            Stmt 19 [153-158]: Semi: Expr 20 [153-157] [Type ()]: Call:
                                                Expr 21 [153-154] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 22 [155-156] [Type Int]: Lit: Int(2)
                                        Block 23 [183-230] [Type ()]:
                                            Stmt 24 [197-202]: Semi: Expr 25 [197-201] [Type ()]: Call:
                                                Expr 26 [197-198] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [199-200] [Type Int]: Lit: Int(3)
                                            Stmt 28 [215-220]: Semi: Expr 29 [215-219] [Type ()]: Call:
                                                Expr 30 [215-216] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 31 [217-218] [Type Int]: Lit: Int(4)
                            SpecDecl _id_ [67-236] (Adj): Impl:
                                Pat _id_ [67-236] [Type Qubit]: Elided
                                Block 11 [104-236] [Type ()]:
                                    Stmt 12 [114-230]: Expr: Expr 13 [114-230] [Type ()]: Conjugate:
                                        Block 14 [121-168] [Type ()]:
                                            Stmt 15 [135-140]: Semi: Expr 16 [135-139] [Type ()]: Call:
                                                Expr 17 [135-136] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 18 [137-138] [Type Int]: Lit: Int(1)
                                            Stmt 19 [153-158]: Semi: Expr 20 [153-157] [Type ()]: Call:
                                                Expr 21 [153-154] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 22 [155-156] [Type Int]: Lit: Int(2)
                                        Block 23 [183-230] [Type ()]:
                                            Stmt 28 [215-220]: Semi: Expr 29 [215-219] [Type ()]: Call:
                                                Expr _id_ [215-216] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 30 [215-216] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 31 [217-218] [Type Int]: Lit: Int(4)
                                            Stmt 24 [197-202]: Semi: Expr 25 [197-201] [Type ()]: Call:
                                                Expr _id_ [197-198] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 26 [197-198] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [199-200] [Type Int]: Lit: Int(3)"#]],
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
                    Namespace (Ident 62 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-266]:
                    Parent: 0
                    Callable 6 [67-266] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-266] (Body): Impl:
                                Pat _id_ [104-266] [Type Qubit]: Elided
                                Block 11 [104-266] [Type ()]:
                                    Stmt 12 [114-119]: Semi: Expr 13 [114-118] [Type ()]: Call:
                                        Expr 14 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 15 [116-117] [Type Int]: Lit: Int(1)
                                    Stmt 16 [128-166]: Local (Immutable):
                                        Pat 17 [132-135] [Type Bool]: Bind: Ident 18 [132-135] "val"
                                        Expr 19 [138-165] [Type Bool]: If:
                                            Expr 20 [141-145] [Type Bool]: Lit: Bool(true)
                                            Block 21 [146-153] [Type Bool]:
                                                Stmt 22 [147-152]: Expr: Expr 23 [147-152] [Type Bool]: Lit: Bool(false)
                                            Expr 24 [154-165] [Type Bool]: Expr Block: Block 25 [159-165] [Type Bool]:
                                                Stmt 26 [160-164]: Expr: Expr 27 [160-164] [Type Bool]: Lit: Bool(true)
                                    Stmt 28 [175-180]: Semi: Expr 29 [175-179] [Type ()]: Call:
                                        Expr 30 [175-176] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 31 [177-178] [Type Int]: Lit: Int(2)
                                    Stmt 32 [189-246]: Expr: Expr 33 [189-246] [Type ()]: If:
                                        Expr 34 [192-197] [Type Bool]: Lit: Bool(false)
                                        Block 35 [198-211] [Type ()]:
                                            Stmt 36 [199-204]: Semi: Expr 37 [199-203] [Type ()]: Call:
                                                Expr 38 [199-200] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 39 [201-202] [Type Int]: Lit: Int(3)
                                            Stmt 40 [205-210]: Semi: Expr 41 [205-209] [Type ()]: Call:
                                                Expr 42 [205-206] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 43 [207-208] [Type Int]: Lit: Int(4)
                                        Expr 44 [212-246] [Type ()]: Expr Block: Block 45 [217-246] [Type ()]:
                                            Stmt 46 [218-223]: Semi: Expr 47 [218-222] [Type ()]: Call:
                                                Expr 48 [218-219] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 49 [220-221] [Type Int]: Lit: Int(5)
                                            Stmt 50 [224-239]: Local (Immutable):
                                                Pat 51 [228-231] [Type Bool]: Bind: Ident 52 [228-231] "val"
                                                Expr 53 [234-238] [Type Bool]: Lit: Bool(true)
                                            Stmt 54 [240-245]: Semi: Expr 55 [240-244] [Type ()]: Call:
                                                Expr 56 [240-241] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 57 [242-243] [Type Int]: Lit: Int(6)
                                    Stmt 58 [255-260]: Semi: Expr 59 [255-259] [Type ()]: Call:
                                        Expr 60 [255-256] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 61 [257-258] [Type Int]: Lit: Int(7)
                            SpecDecl _id_ [67-266] (Adj): Impl:
                                Pat _id_ [67-266] [Type Qubit]: Elided
                                Block 11 [104-266] [Type ()]:
                                    Stmt 16 [128-166]: Local (Immutable):
                                        Pat 17 [132-135] [Type Bool]: Bind: Ident 18 [132-135] "val"
                                        Expr 19 [138-165] [Type Bool]: If:
                                            Expr 20 [141-145] [Type Bool]: Lit: Bool(true)
                                            Block 21 [146-153] [Type Bool]:
                                                Stmt 22 [147-152]: Expr: Expr 23 [147-152] [Type Bool]: Lit: Bool(false)
                                            Expr 24 [154-165] [Type Bool]: Expr Block: Block 25 [159-165] [Type Bool]:
                                                Stmt 26 [160-164]: Expr: Expr 27 [160-164] [Type Bool]: Lit: Bool(true)
                                    Stmt 58 [255-260]: Semi: Expr 59 [255-259] [Type ()]: Call:
                                        Expr _id_ [255-256] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 60 [255-256] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 61 [257-258] [Type Int]: Lit: Int(7)
                                    Stmt 32 [189-246]: Expr: Expr 33 [189-246] [Type ()]: If:
                                        Expr 34 [192-197] [Type Bool]: Lit: Bool(false)
                                        Block 35 [198-211] [Type ()]:
                                            Stmt 40 [205-210]: Semi: Expr 41 [205-209] [Type ()]: Call:
                                                Expr _id_ [205-206] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 42 [205-206] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 43 [207-208] [Type Int]: Lit: Int(4)
                                            Stmt 36 [199-204]: Semi: Expr 37 [199-203] [Type ()]: Call:
                                                Expr _id_ [199-200] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 38 [199-200] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 39 [201-202] [Type Int]: Lit: Int(3)
                                        Expr 44 [212-246] [Type ()]: Expr Block: Block 45 [217-246] [Type ()]:
                                            Stmt 50 [224-239]: Local (Immutable):
                                                Pat 51 [228-231] [Type Bool]: Bind: Ident 52 [228-231] "val"
                                                Expr 53 [234-238] [Type Bool]: Lit: Bool(true)
                                            Stmt 54 [240-245]: Semi: Expr 55 [240-244] [Type ()]: Call:
                                                Expr _id_ [240-241] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 56 [240-241] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 57 [242-243] [Type Int]: Lit: Int(6)
                                            Stmt 46 [218-223]: Semi: Expr 47 [218-222] [Type ()]: Call:
                                                Expr _id_ [218-219] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                    Expr 48 [218-219] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 49 [220-221] [Type Int]: Lit: Int(5)
                                    Stmt 28 [175-180]: Semi: Expr 29 [175-179] [Type ()]: Call:
                                        Expr _id_ [175-176] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 30 [175-176] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 31 [177-178] [Type Int]: Lit: Int(2)
                                    Stmt 12 [114-119]: Semi: Expr 13 [114-118] [Type ()]: Call:
                                        Expr _id_ [114-115] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                            Expr 14 [114-115] [Type (Int => () is Adj)]: Var: Item 1
                                        Expr 15 [116-117] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 28 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-181]:
                    Parent: 0
                    Callable 6 [67-181] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-181] (Body): Impl:
                                Pat _id_ [104-181] [Type Qubit]: Elided
                                Block 11 [104-181] [Type ()]:
                                    Stmt 12 [114-175]: Expr: Expr 13 [114-175] [Type ()]: For:
                                        Pat 14 [118-119] [Type Int]: Bind: Ident 15 [118-119] "i"
                                        Expr 16 [123-127] [Type Range]: Range:
                                            Expr 17 [123-124] [Type Int]: Lit: Int(0)
                                            <no step>
                                            Expr 18 [126-127] [Type Int]: Lit: Int(5)
                                        Block 19 [128-175] [Type ()]:
                                            Stmt 20 [142-147]: Semi: Expr 21 [142-146] [Type ()]: Call:
                                                Expr 22 [142-143] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 23 [144-145] [Type Int]: Lit: Int(1)
                                            Stmt 24 [160-165]: Semi: Expr 25 [160-164] [Type ()]: Call:
                                                Expr 26 [160-161] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 27 [162-163] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-181] (Adj): Impl:
                                Pat _id_ [67-181] [Type Qubit]: Elided
                                Block 11 [104-181] [Type ()]:
                                    Stmt 12 [114-175]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type Range]: Bind: Ident 29 [0-0] "generated_range"
                                            Expr 16 [123-127] [Type Range]: Range:
                                                Expr 17 [123-124] [Type Int]: Lit: Int(0)
                                                <no step>
                                                Expr 18 [126-127] [Type Int]: Lit: Int(5)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                            Pat 14 [118-119] [Type Int]: Bind: Ident 15 [118-119] "i"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Add):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                        Start
                                                    Expr _id_ [0-0] [Type Int]: BinOp (Mul):
                                                        Expr _id_ [0-0] [Type Int]: BinOp (Div):
                                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                                    End
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                                    Start
                                                            Expr _id_ [0-0] [Type Int]: Field:
                                                                Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                                Step
                                                        Expr _id_ [0-0] [Type Int]: Field:
                                                            Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                            Step
                                                Expr _id_ [0-0] [Type Int]: UnOp (Neg):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                        Step
                                                Expr _id_ [0-0] [Type Int]: Field:
                                                    Expr _id_ [0-0] [Type Range]: Var: Local 29
                                                    Start
                                            Block 19 [128-175] [Type ()]:
                                                Stmt 24 [160-165]: Semi: Expr 25 [160-164] [Type ()]: Call:
                                                    Expr _id_ [160-161] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 26 [160-161] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 27 [162-163] [Type Int]: Lit: Int(2)
                                                Stmt 20 [142-147]: Semi: Expr 21 [142-146] [Type ()]: Call:
                                                    Expr _id_ [142-143] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 22 [142-143] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 23 [144-145] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 29 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-188]:
                    Parent: 0
                    Callable 6 [67-188] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-188] (Body): Impl:
                                Pat _id_ [104-188] [Type Qubit]: Elided
                                Block 11 [104-188] [Type ()]:
                                    Stmt 12 [114-182]: Expr: Expr 13 [114-182] [Type ()]: For:
                                        Pat 14 [118-121] [Type Int]: Bind: Ident 15 [118-121] "val"
                                        Expr 16 [125-134] [Type (Int)[]]: Array:
                                            Expr 17 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 18 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 19 [132-133] [Type Int]: Lit: Int(2)
                                        Block 20 [135-182] [Type ()]:
                                            Stmt 21 [149-154]: Semi: Expr 22 [149-153] [Type ()]: Call:
                                                Expr 23 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 24 [151-152] [Type Int]: Lit: Int(1)
                                            Stmt 25 [167-172]: Semi: Expr 26 [167-171] [Type ()]: Call:
                                                Expr 27 [167-168] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 28 [169-170] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-188] (Adj): Impl:
                                Pat _id_ [67-188] [Type Qubit]: Elided
                                Block 11 [104-188] [Type ()]:
                                    Stmt 12 [114-182]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 30 [0-0] "generated_array"
                                            Expr 16 [125-134] [Type (Int)[]]: Array:
                                                Expr 17 [126-127] [Type Int]: Lit: Int(0)
                                                Expr 18 [129-130] [Type Int]: Lit: Int(1)
                                                Expr 19 [132-133] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                            Pat _id_ [0-0] [Type Int]: Bind: Ident 31 [0-0] "generated_index"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 30
                                                        Length
                                                    Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                            Block 20 [135-182] [Type ()]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat 14 [118-121] [Type Int]: Bind: Ident 15 [118-121] "val"
                                                    Expr _id_ [0-0] [Type Int]: Index:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 30
                                                        Expr _id_ [0-0] [Type Int]: Var: Local 31
                                                Stmt 25 [167-172]: Semi: Expr 26 [167-171] [Type ()]: Call:
                                                    Expr _id_ [167-168] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 27 [167-168] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 28 [169-170] [Type Int]: Lit: Int(2)
                                                Stmt 21 [149-154]: Semi: Expr 22 [149-153] [Type ()]: Call:
                                                    Expr _id_ [149-150] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 23 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 24 [151-152] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 50 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-59]: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 5 [60-62]: <empty>
                Item 2 [67-318]:
                    Parent: 0
                    Callable 6 [67-318] (Operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: ()
                        functors: Functor Expr 10 [100-103]: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-318] (Body): Impl:
                                Pat _id_ [104-318] [Type Qubit]: Elided
                                Block 11 [104-318] [Type ()]:
                                    Stmt 12 [114-312]: Expr: Expr 13 [114-312] [Type ()]: For:
                                        Pat 14 [118-121] [Type Int]: Bind: Ident 15 [118-121] "val"
                                        Expr 16 [125-134] [Type (Int)[]]: Array:
                                            Expr 17 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 18 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 19 [132-133] [Type Int]: Lit: Int(2)
                                        Block 20 [135-312] [Type ()]:
                                            Stmt 21 [149-154]: Semi: Expr 22 [149-153] [Type ()]: Call:
                                                Expr 23 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 24 [151-152] [Type Int]: Lit: Int(1)
                                            Stmt 25 [167-197]: Local (Immutable):
                                                Pat 26 [171-174] [Type (Bool)[]]: Bind: Ident 27 [171-174] "arr"
                                                Expr 28 [177-196] [Type (Bool)[]]: Array:
                                                    Expr 29 [178-182] [Type Bool]: Lit: Bool(true)
                                                    Expr 30 [184-189] [Type Bool]: Lit: Bool(false)
                                                    Expr 31 [191-195] [Type Bool]: Lit: Bool(true)
                                            Stmt 32 [210-284]: Expr: Expr 33 [210-284] [Type ()]: For:
                                                Pat 34 [214-217] [Type Bool]: Bind: Ident 35 [214-217] "val"
                                                Expr 36 [221-224] [Type (Bool)[]]: Var: Local 27
                                                Block 37 [225-284] [Type ()]:
                                                    Stmt 38 [243-248]: Semi: Expr 39 [243-247] [Type ()]: Call:
                                                        Expr 40 [243-244] [Type (Int => () is Adj)]: Var: Item 1
                                                        Expr 41 [245-246] [Type Int]: Lit: Int(2)
                                                    Stmt 42 [265-270]: Semi: Expr 43 [265-269] [Type ()]: Call:
                                                        Expr 44 [265-266] [Type (Int => () is Adj)]: Var: Item 1
                                                        Expr 45 [267-268] [Type Int]: Lit: Int(3)
                                            Stmt 46 [297-302]: Semi: Expr 47 [297-301] [Type ()]: Call:
                                                Expr 48 [297-298] [Type (Int => () is Adj)]: Var: Item 1
                                                Expr 49 [299-300] [Type Int]: Lit: Int(4)
                            SpecDecl _id_ [67-318] (Adj): Impl:
                                Pat _id_ [67-318] [Type Qubit]: Elided
                                Block 11 [104-318] [Type ()]:
                                    Stmt 12 [114-312]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 53 [0-0] "generated_array"
                                            Expr 16 [125-134] [Type (Int)[]]: Array:
                                                Expr 17 [126-127] [Type Int]: Lit: Int(0)
                                                Expr 18 [129-130] [Type Int]: Lit: Int(1)
                                                Expr 19 [132-133] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                            Pat _id_ [0-0] [Type Int]: Bind: Ident 54 [0-0] "generated_index"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 53
                                                        Length
                                                    Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                            Block 20 [135-312] [Type ()]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat 14 [118-121] [Type Int]: Bind: Ident 15 [118-121] "val"
                                                    Expr _id_ [0-0] [Type Int]: Index:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 53
                                                        Expr _id_ [0-0] [Type Int]: Var: Local 54
                                                Stmt 25 [167-197]: Local (Immutable):
                                                    Pat 26 [171-174] [Type (Bool)[]]: Bind: Ident 27 [171-174] "arr"
                                                    Expr 28 [177-196] [Type (Bool)[]]: Array:
                                                        Expr 29 [178-182] [Type Bool]: Lit: Bool(true)
                                                        Expr 30 [184-189] [Type Bool]: Lit: Bool(false)
                                                        Expr 31 [191-195] [Type Bool]: Lit: Bool(true)
                                                Stmt 46 [297-302]: Semi: Expr 47 [297-301] [Type ()]: Call:
                                                    Expr _id_ [297-298] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 48 [297-298] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 49 [299-300] [Type Int]: Lit: Int(4)
                                                Stmt 32 [210-284]: Expr: Expr _id_ [0-0] [Type ()]: Expr Block: Block _id_ [0-0] [Type ()]:
                                                    Stmt _id_ [0-0]: Local (Immutable):
                                                        Pat _id_ [0-0] [Type (Bool)[]]: Bind: Ident 51 [0-0] "generated_array"
                                                        Expr 36 [221-224] [Type (Bool)[]]: Var: Local 27
                                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type ()]: For:
                                                        Pat _id_ [0-0] [Type Int]: Bind: Ident 52 [0-0] "generated_index"
                                                        Expr _id_ [0-0] [Type Range]: Range:
                                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 51
                                                                    Length
                                                                Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                                        Block 37 [225-284] [Type ()]:
                                                            Stmt _id_ [0-0]: Local (Immutable):
                                                                Pat 34 [214-217] [Type Bool]: Bind: Ident 35 [214-217] "val"
                                                                Expr _id_ [0-0] [Type Bool]: Index:
                                                                    Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 51
                                                                    Expr _id_ [0-0] [Type Int]: Var: Local 52
                                                            Stmt 42 [265-270]: Semi: Expr 43 [265-269] [Type ()]: Call:
                                                                Expr _id_ [265-266] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                                    Expr 44 [265-266] [Type (Int => () is Adj)]: Var: Item 1
                                                                Expr 45 [267-268] [Type Int]: Lit: Int(3)
                                                            Stmt 38 [243-248]: Semi: Expr 39 [243-247] [Type ()]: Call:
                                                                Expr _id_ [243-244] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                                    Expr 40 [243-244] [Type (Int => () is Adj)]: Var: Item 1
                                                                Expr 41 [245-246] [Type Int]: Lit: Int(2)
                                                Stmt 21 [149-154]: Semi: Expr 22 [149-153] [Type ()]: Call:
                                                    Expr _id_ [149-150] [Type (Int => () is Adj)]: UnOp (Functor Adj):
                                                        Expr 23 [149-150] [Type (Int => () is Adj)]: Var: Item 1
                                                    Expr 24 [151-152] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 26 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-65]: BinOp Union: (Functor Expr 5 [56-59]: Ctl) (Functor Expr 6 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 27 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 28 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                Item 2 [73-156]:
                    Parent: 0
                    Callable 8 [73-156] (Operation):
                        name: Ident 9 [83-84] "A"
                        input: Pat 10 [85-94] [Type Qubit]: Bind: Ident 11 [85-86] "q"
                        output: ()
                        functors: Functor Expr 12 [106-115]: BinOp Union: (Functor Expr 13 [106-109]: Ctl) (Functor Expr 14 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 15 [126-150] (Body): Impl:
                                Pat 16 [131-134] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-156] (Adj): Impl:
                                Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type Int]: Lit: Int(1)
                            SpecDecl _id_ [73-156] (Ctl): Impl:
                                Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 29 [73-156] "ctls"
                                    Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 29
                                            Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 29
                                            Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-156] (CtlAdj): Impl:
                                Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 30 [73-156] "ctls"
                                    Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr _id_ [143-144] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 30
                                            Expr 25 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr _id_ [137-138] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 30
                                            Expr 21 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 27 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: ()
                        functors: Functor Expr 4 [56-65]: BinOp Union: (Functor Expr 5 [56-59]: Ctl) (Functor Expr 6 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 28 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 29 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 7 [66-68]: <empty>
                Item 2 [73-191]:
                    Parent: 0
                    Callable 8 [73-191] (Operation):
                        name: Ident 9 [83-84] "A"
                        input: Pat 10 [85-94] [Type Qubit]: Bind: Ident 11 [85-86] "q"
                        output: ()
                        functors: Functor Expr 12 [106-115]: BinOp Union: (Functor Expr 13 [106-109]: Ctl) (Functor Expr 14 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 15 [126-150] (Body): Impl:
                                Pat 16 [131-134] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-191] (Adj): Impl:
                                Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (Int => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type Int]: Lit: Int(1)
                            SpecDecl _id_ [73-191] (Ctl): Impl:
                                Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 30 [73-191] "ctls"
                                    Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr 20 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 30
                                            Expr 21 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr 24 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 30
                                            Expr 25 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 26 [159-185] (CtlAdj): Impl:
                                Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 30 [73-191] "ctls"
                                    Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 17 [135-150] [Type ()]:
                                    Stmt 22 [143-148]: Semi: Expr 23 [143-147] [Type ()]: Call:
                                        Expr _id_ [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 24 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 24 [143-144] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 25 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 30
                                            Expr 25 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 18 [137-142]: Semi: Expr 19 [137-141] [Type ()]: Call:
                                        Expr _id_ [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 20 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 20 [137-138] [Type (Int => () is Adj + Ctl)]: Var: Item 1
                                        Expr 21 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 30
                                            Expr 21 [139-140] [Type Int]: Lit: Int(1)"#]],
    );
}
