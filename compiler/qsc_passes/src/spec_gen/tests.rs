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
            Package 0:
                Namespace 1 [0-184] (Ident 2 [10-14] "test"):
                    Item 3 [21-119]:
                        Callable 4 [21-119] (Operation):
                            name: Ident 5 [31-32] "A"
                            input: Pat 6 [32-43] [Type Qubit]: Paren:
                                Pat 7 [33-42] [Type Qubit]: Bind: Ident 8 [33-34] "q"
                            output: ()
                            functors: Functor Expr 9 [54-57]: Ctl
                            body: Specializations:
                                SpecDecl 10 [68-79] (Body): Impl:
                                    Pat 11 [73-76] [Type Qubit]: Elided
                                    Block 12 [77-79]: <empty>
                                SpecDecl 13 [88-113] (Ctl): Impl:
                                    Pat 14 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat 15 [100-104] [Type (Qubit)[]]: Bind: Ident 16 [100-104] "ctls"
                                        Pat 17 [106-109] [Type Qubit]: Elided
                                    Block 18 [111-113]: <empty>
                    Item 19 [124-182]:
                        Callable 20 [124-182] (Operation):
                            name: Ident 21 [134-135] "B"
                            input: Pat 22 [135-146] [Type Qubit]: Paren:
                                Pat 23 [136-145] [Type Qubit]: Bind: Ident 24 [136-137] "q"
                            output: ()
                            functors: Functor Expr 25 [157-160]: Ctl
                            body: Specializations:
                                SpecDecl _id_ [161-182] (Body): Impl:
                                    Pat _id_ [161-182] [Type Qubit]: Elided
                                    Block 26 [161-182] [Type ()]:
                                        Stmt 27 [171-176]: Semi: Expr 28 [171-175] [Type ()]: Call:
                                            Expr 29 [171-172] [Type (Qubit => () is Ctl)]: Name: Internal(NodeId(5))
                                            Expr 30 [172-175] [Type Qubit]: Paren: Expr 31 [173-174] [Type Qubit]: Name: Internal(NodeId(24))
                                SpecDecl _id_ [124-182] (Ctl): Impl:
                                    Pat _id_ [124-182] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [124-182] [Type (Qubit)[]]: Bind: Ident 32 [124-182] "ctls"
                                        Pat _id_ [124-182] [Type Qubit]: Elided
                                    Block 26 [161-182] [Type ()]:
                                        Stmt 27 [171-176]: Semi: Expr 28 [171-175] [Type ()]: Call:
                                            Expr 29 [171-172] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                                Expr 29 [171-172] [Type (Qubit => () is Ctl)]: Name: Internal(NodeId(5))
                                            Expr 30 [172-175] [Type ((Qubit)[], Qubit)]: Tuple:
                                                Expr _id_ [172-175] [Type (Qubit)[]]: Name: Internal(NodeId(32))
                                                Expr 30 [172-175] [Type Qubit]: Paren: Expr 31 [173-174] [Type Qubit]: Name: Internal(NodeId(24))"#]],
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
            Package 0:
                Namespace 1 [0-310] (Ident 2 [10-14] "test"):
                    Item 3 [21-148]:
                        Callable 4 [21-148] (Operation):
                            name: Ident 5 [31-32] "A"
                            input: Pat 6 [32-43] [Type Qubit]: Paren:
                                Pat 7 [33-42] [Type Qubit]: Bind: Ident 8 [33-34] "q"
                            output: ()
                            functors: Functor Expr 9 [54-63]: BinOp Union: (Functor Expr 10 [54-57]: Ctl) (Functor Expr 11 [60-63]: Adj)
                            body: Specializations:
                                SpecDecl 12 [74-85] (Body): Impl:
                                    Pat 13 [79-82] [Type Qubit]: Elided
                                    Block 14 [83-85]: <empty>
                                SpecDecl 15 [94-108] (Adj): Impl:
                                    Pat 16 [102-105] [Type Qubit]: Elided
                                    Block 17 [106-108]: <empty>
                                SpecDecl 18 [117-142] (Ctl): Impl:
                                    Pat 19 [128-139] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat 20 [129-133] [Type (Qubit)[]]: Bind: Ident 21 [129-133] "ctls"
                                        Pat 22 [135-138] [Type Qubit]: Elided
                                    Block 23 [140-142]: <empty>
                                SpecDecl _id_ [21-148] (CtlAdj): Impl:
                                    Pat _id_ [21-148] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [21-148] [Type (Qubit)[]]: Bind: Ident 50 [21-148] "ctls"
                                        Pat _id_ [21-148] [Type Qubit]: Elided
                                    Block 17 [106-108]: <empty>
                    Item 24 [153-308]:
                        Callable 25 [153-308] (Operation):
                            name: Ident 26 [163-164] "B"
                            input: Pat 27 [164-175] [Type Qubit]: Paren:
                                Pat 28 [165-174] [Type Qubit]: Bind: Ident 29 [165-166] "q"
                            output: ()
                            functors: Functor Expr 30 [186-195]: BinOp Union: (Functor Expr 31 [186-189]: Ctl) (Functor Expr 32 [192-195]: Adj)
                            body: Specializations:
                                SpecDecl 33 [206-244] (Body): Impl:
                                    Pat 34 [211-214] [Type Qubit]: Elided
                                    Block 35 [215-244] [Type ()]:
                                        Stmt 36 [229-234]: Semi: Expr 37 [229-233] [Type ()]: Call:
                                            Expr 38 [229-230] [Type (Qubit => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 39 [230-233] [Type Qubit]: Paren: Expr 40 [231-232] [Type Qubit]: Name: Internal(NodeId(29))
                                SpecDecl 41 [253-302] (Adj): Impl:
                                    Pat 42 [261-264] [Type Qubit]: Elided
                                    Block 43 [265-302] [Type ()]:
                                        Stmt 44 [279-292]: Semi: Expr 45 [279-291] [Type ()]: Call:
                                            Expr 46 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 47 [287-288] [Type (Qubit => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 48 [288-291] [Type Qubit]: Paren: Expr 49 [289-290] [Type Qubit]: Name: Internal(NodeId(29))
                                SpecDecl _id_ [153-308] (Ctl): Impl:
                                    Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 51 [153-308] "ctls"
                                        Pat _id_ [153-308] [Type Qubit]: Elided
                                    Block 35 [215-244] [Type ()]:
                                        Stmt 36 [229-234]: Semi: Expr 37 [229-233] [Type ()]: Call:
                                            Expr 38 [229-230] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 38 [229-230] [Type (Qubit => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 39 [230-233] [Type ((Qubit)[], Qubit)]: Tuple:
                                                Expr _id_ [230-233] [Type (Qubit)[]]: Name: Internal(NodeId(51))
                                                Expr 39 [230-233] [Type Qubit]: Paren: Expr 40 [231-232] [Type Qubit]: Name: Internal(NodeId(29))
                                SpecDecl _id_ [153-308] (CtlAdj): Impl:
                                    Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 52 [153-308] "ctls"
                                        Pat _id_ [153-308] [Type Qubit]: Elided
                                    Block 43 [265-302] [Type ()]:
                                        Stmt 44 [279-292]: Semi: Expr 45 [279-291] [Type ()]: Call:
                                            Expr 46 [279-288] [Type (((Qubit)[], Qubit) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 46 [279-288] [Type (Qubit => () is Adj + Ctl)]: UnOp (Functor Adj):
                                                    Expr 47 [287-288] [Type (Qubit => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 48 [288-291] [Type ((Qubit)[], Qubit)]: Tuple:
                                                Expr _id_ [288-291] [Type (Qubit)[]]: Name: Internal(NodeId(52))
                                                Expr 48 [288-291] [Type Qubit]: Paren: Expr 49 [289-290] [Type Qubit]: Name: Internal(NodeId(29))"#]],
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
            Package 0:
                Namespace 1 [0-259] (Ident 2 [10-14] "test"):
                    Item 3 [21-119]:
                        Callable 4 [21-119] (Operation):
                            name: Ident 5 [31-32] "A"
                            input: Pat 6 [32-43] [Type Qubit]: Paren:
                                Pat 7 [33-42] [Type Qubit]: Bind: Ident 8 [33-34] "q"
                            output: ()
                            functors: Functor Expr 9 [54-57]: Ctl
                            body: Specializations:
                                SpecDecl 10 [68-79] (Body): Impl:
                                    Pat 11 [73-76] [Type Qubit]: Elided
                                    Block 12 [77-79]: <empty>
                                SpecDecl 13 [88-113] (Ctl): Impl:
                                    Pat 14 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat 15 [100-104] [Type (Qubit)[]]: Bind: Ident 16 [100-104] "ctls"
                                        Pat 17 [106-109] [Type Qubit]: Elided
                                    Block 18 [111-113]: <empty>
                    Item 19 [124-257]:
                        Callable 20 [124-257] (Operation):
                            name: Ident 21 [134-135] "B"
                            input: Pat 22 [135-146] [Type Qubit]: Paren:
                                Pat 23 [136-145] [Type Qubit]: Bind: Ident 24 [136-137] "q"
                            output: ()
                            functors: Functor Expr 25 [157-160]: Ctl
                            body: Specializations:
                                SpecDecl _id_ [161-257] (Body): Impl:
                                    Pat _id_ [161-257] [Type Qubit]: Elided
                                    Block 26 [161-257] [Type ()]:
                                        Stmt 27 [171-251]: Expr: Expr 28 [171-251] [Type ()]: Conjugate:
                                            Block 29 [178-207] [Type ()]:
                                                Stmt 30 [192-197]: Semi: Expr 31 [192-196] [Type ()]: Call:
                                                    Expr 32 [192-193] [Type (Qubit => () is Ctl)]: Name: Internal(NodeId(5))
                                                    Expr 33 [193-196] [Type Qubit]: Paren: Expr 34 [194-195] [Type Qubit]: Name: Internal(NodeId(24))
                                            Block 35 [222-251] [Type ()]:
                                                Stmt 36 [236-241]: Semi: Expr 37 [236-240] [Type ()]: Call:
                                                    Expr 38 [236-237] [Type (Qubit => () is Ctl)]: Name: Internal(NodeId(5))
                                                    Expr 39 [237-240] [Type Qubit]: Paren: Expr 40 [238-239] [Type Qubit]: Name: Internal(NodeId(24))
                                SpecDecl _id_ [124-257] (Ctl): Impl:
                                    Pat _id_ [124-257] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [124-257] [Type (Qubit)[]]: Bind: Ident 41 [124-257] "ctls"
                                        Pat _id_ [124-257] [Type Qubit]: Elided
                                    Block 26 [161-257] [Type ()]:
                                        Stmt 27 [171-251]: Expr: Expr 28 [171-251] [Type ()]: Conjugate:
                                            Block 29 [178-207] [Type ()]:
                                                Stmt 30 [192-197]: Semi: Expr 31 [192-196] [Type ()]: Call:
                                                    Expr 32 [192-193] [Type (Qubit => () is Ctl)]: Name: Internal(NodeId(5))
                                                    Expr 33 [193-196] [Type Qubit]: Paren: Expr 34 [194-195] [Type Qubit]: Name: Internal(NodeId(24))
                                            Block 35 [222-251] [Type ()]:
                                                Stmt 36 [236-241]: Semi: Expr 37 [236-240] [Type ()]: Call:
                                                    Expr 38 [236-237] [Type (((Qubit)[], Qubit) => () is Ctl)]: UnOp (Functor Ctl):
                                                        Expr 38 [236-237] [Type (Qubit => () is Ctl)]: Name: Internal(NodeId(5))
                                                    Expr 39 [237-240] [Type ((Qubit)[], Qubit)]: Tuple:
                                                        Expr _id_ [237-240] [Type (Qubit)[]]: Name: Internal(NodeId(41))
                                                        Expr 39 [237-240] [Type Qubit]: Paren: Expr 40 [238-239] [Type Qubit]: Name: Internal(NodeId(24))"#]],
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
            Package 0:
                Namespace 1 [0-150] (Ident 2 [10-14] "test"):
                    Item 3 [21-45]:
                        Callable 4 [21-45] (Function):
                            name: Ident 5 [30-33] "Foo"
                            input: Pat 6 [33-35] [Type ()]: Unit
                            output: ()
                            body: Block: Block 7 [43-45]: <empty>
                    Item 8 [50-80]:
                        Callable 9 [50-80] (Operation):
                            name: Ident 10 [60-61] "A"
                            input: Pat 11 [61-63] [Type ()]: Unit
                            output: ()
                            functors: Functor Expr 12 [74-77]: Ctl
                            body: Specializations:
                                SpecDecl _id_ [78-80] (Body): Impl:
                                    Pat _id_ [78-80] [Type ()]: Elided
                                    Block 13 [78-80]: <empty>
                                SpecDecl _id_ [50-80] (Ctl): Impl:
                                    Pat _id_ [50-80] [Type ((Qubit)[], ())]: Tuple:
                                        Pat _id_ [50-80] [Type (Qubit)[]]: Bind: Ident 28 [50-80] "ctls"
                                        Pat _id_ [50-80] [Type ()]: Elided
                                    Block 13 [78-80]: <empty>
                    Item 14 [85-148]:
                        Callable 15 [85-148] (Operation):
                            name: Ident 16 [95-96] "B"
                            input: Pat 17 [96-98] [Type ()]: Unit
                            output: ()
                            functors: Functor Expr 18 [109-112]: Ctl
                            body: Specializations:
                                SpecDecl _id_ [113-148] (Body): Impl:
                                    Pat _id_ [113-148] [Type ()]: Elided
                                    Block 19 [113-148] [Type ()]:
                                        Stmt 20 [123-129]: Semi: Expr 21 [123-128] [Type ()]: Call:
                                            Expr 22 [123-126] [Type (() -> ())]: Name: Internal(NodeId(5))
                                            Expr 23 [126-128] [Type ()]: Unit
                                        Stmt 24 [138-142]: Semi: Expr 25 [138-141] [Type ()]: Call:
                                            Expr 26 [138-139] [Type (() => () is Ctl)]: Name: Internal(NodeId(10))
                                            Expr 27 [139-141] [Type ()]: Unit
                                SpecDecl _id_ [85-148] (Ctl): Impl:
                                    Pat _id_ [85-148] [Type ((Qubit)[], ())]: Tuple:
                                        Pat _id_ [85-148] [Type (Qubit)[]]: Bind: Ident 29 [85-148] "ctls"
                                        Pat _id_ [85-148] [Type ()]: Elided
                                    Block 19 [113-148] [Type ()]:
                                        Stmt 20 [123-129]: Semi: Expr 21 [123-128] [Type ()]: Call:
                                            Expr 22 [123-126] [Type (() -> ())]: Name: Internal(NodeId(5))
                                            Expr 23 [126-128] [Type ()]: Unit
                                        Stmt 24 [138-142]: Semi: Expr 25 [138-141] [Type ()]: Call:
                                            Expr 26 [138-139] [Type (((Qubit)[], ()) => () is Ctl)]: UnOp (Functor Ctl):
                                                Expr 26 [138-139] [Type (() => () is Ctl)]: Name: Internal(NodeId(10))
                                            Expr 27 [139-141] [Type ((Qubit)[], ())]: Tuple:
                                                Expr _id_ [139-141] [Type (Qubit)[]]: Name: Internal(NodeId(29))
                                                Expr 27 [139-141] [Type ()]: Unit"#]],
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
            Package 0:
                Namespace 1 [0-168] (Ident 2 [10-14] "test"):
                    Item 3 [21-62]:
                        Callable 4 [21-62] (Operation):
                            name: Ident 5 [31-32] "B"
                            input: Pat 6 [32-45] [Type Int]: Paren:
                                Pat 7 [33-44] [Type Int]: Bind: Ident 8 [33-38] "input"
                            output: ()
                            functors: Functor Expr 9 [56-59]: Adj
                            body: Specializations:
                                SpecDecl _id_ [60-62] (Body): Impl:
                                    Pat _id_ [60-62] [Type Int]: Elided
                                    Block 10 [60-62]: <empty>
                                SpecDecl _id_ [21-62] (Adj): Gen: Invert
                    Item 11 [67-166]:
                        Callable 12 [67-166] (Operation):
                            name: Ident 13 [77-78] "A"
                            input: Pat 14 [78-89] [Type Qubit]: Paren:
                                Pat 15 [79-88] [Type Qubit]: Bind: Ident 16 [79-80] "q"
                            output: ()
                            functors: Functor Expr 17 [100-103]: Adj
                            body: Specializations:
                                SpecDecl 18 [114-138] (Body): Impl:
                                    Pat 19 [119-122] [Type Qubit]: Elided
                                    Block 20 [123-138] [Type ()]:
                                        Stmt 21 [125-130]: Semi: Expr 22 [125-129] [Type ()]: Call:
                                            Expr 23 [125-126] [Type (Int => () is Adj)]: Name: Internal(NodeId(5))
                                            Expr 24 [126-129] [Type Int]: Paren: Expr 25 [127-128] [Type Int]: Lit: Int(1)
                                        Stmt 26 [131-136]: Semi: Expr 27 [131-135] [Type ()]: Call:
                                            Expr 28 [131-132] [Type (Int => () is Adj)]: Name: Internal(NodeId(5))
                                            Expr 29 [132-135] [Type Int]: Paren: Expr 30 [133-134] [Type Int]: Lit: Int(2)
                                SpecDecl 31 [147-160] (Adj): Impl:
                                    Pat 19 [119-122] [Type Qubit]: Elided
                                    Block 20 [123-138] [Type ()]:
                                        Stmt 21 [125-130]: Semi: Expr 22 [125-129] [Type ()]: Call:
                                            Expr 23 [125-126] [Type (Int => () is Adj)]: Name: Internal(NodeId(5))
                                            Expr 24 [126-129] [Type Int]: Paren: Expr 25 [127-128] [Type Int]: Lit: Int(1)
                                        Stmt 26 [131-136]: Semi: Expr 27 [131-135] [Type ()]: Call:
                                            Expr 28 [131-132] [Type (Int => () is Adj)]: Name: Internal(NodeId(5))
                                            Expr 29 [132-135] [Type Int]: Paren: Expr 30 [133-134] [Type Int]: Lit: Int(2)"#]],
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
            Package 0:
                Namespace 1 [0-180] (Ident 2 [10-14] "test"):
                    Item 3 [21-68]:
                        Callable 4 [21-68] (Operation):
                            name: Ident 5 [31-32] "B"
                            input: Pat 6 [32-45] [Type Int]: Paren:
                                Pat 7 [33-44] [Type Int]: Bind: Ident 8 [33-38] "input"
                            output: ()
                            functors: Functor Expr 9 [56-65]: BinOp Union: (Functor Expr 10 [56-59]: Ctl) (Functor Expr 11 [62-65]: Adj)
                            body: Specializations:
                                SpecDecl _id_ [66-68] (Body): Impl:
                                    Pat _id_ [66-68] [Type Int]: Elided
                                    Block 12 [66-68]: <empty>
                                SpecDecl _id_ [21-68] (Adj): Gen: Invert
                                SpecDecl _id_ [21-68] (Ctl): Impl:
                                    Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                        Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 36 [21-68] "ctls"
                                        Pat _id_ [21-68] [Type Int]: Elided
                                    Block 12 [66-68]: <empty>
                                SpecDecl _id_ [21-68] (CtlAdj): Gen: Distribute
                    Item 13 [73-178]:
                        Callable 14 [73-178] (Operation):
                            name: Ident 15 [83-84] "A"
                            input: Pat 16 [84-95] [Type Qubit]: Paren:
                                Pat 17 [85-94] [Type Qubit]: Bind: Ident 18 [85-86] "q"
                            output: ()
                            functors: Functor Expr 19 [106-115]: BinOp Union: (Functor Expr 20 [106-109]: Ctl) (Functor Expr 21 [112-115]: Adj)
                            body: Specializations:
                                SpecDecl 22 [126-150] (Body): Impl:
                                    Pat 23 [131-134] [Type Qubit]: Elided
                                    Block 24 [135-150] [Type ()]:
                                        Stmt 25 [137-142]: Semi: Expr 26 [137-141] [Type ()]: Call:
                                            Expr 27 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 28 [138-141] [Type Int]: Paren: Expr 29 [139-140] [Type Int]: Lit: Int(1)
                                        Stmt 30 [143-148]: Semi: Expr 31 [143-147] [Type ()]: Call:
                                            Expr 32 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 33 [144-147] [Type Int]: Paren: Expr 34 [145-146] [Type Int]: Lit: Int(2)
                                SpecDecl 35 [159-172] (Adj): Impl:
                                    Pat 23 [131-134] [Type Qubit]: Elided
                                    Block 24 [135-150] [Type ()]:
                                        Stmt 25 [137-142]: Semi: Expr 26 [137-141] [Type ()]: Call:
                                            Expr 27 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 28 [138-141] [Type Int]: Paren: Expr 29 [139-140] [Type Int]: Lit: Int(1)
                                        Stmt 30 [143-148]: Semi: Expr 31 [143-147] [Type ()]: Call:
                                            Expr 32 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 33 [144-147] [Type Int]: Paren: Expr 34 [145-146] [Type Int]: Lit: Int(2)
                                SpecDecl _id_ [73-178] (Ctl): Impl:
                                    Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 37 [73-178] "ctls"
                                        Pat _id_ [73-178] [Type Qubit]: Elided
                                    Block 24 [135-150] [Type ()]:
                                        Stmt 25 [137-142]: Semi: Expr 26 [137-141] [Type ()]: Call:
                                            Expr 27 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 27 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 28 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                                Expr _id_ [138-141] [Type (Qubit)[]]: Name: Internal(NodeId(37))
                                                Expr 28 [138-141] [Type Int]: Paren: Expr 29 [139-140] [Type Int]: Lit: Int(1)
                                        Stmt 30 [143-148]: Semi: Expr 31 [143-147] [Type ()]: Call:
                                            Expr 32 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 32 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 33 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                                Expr _id_ [144-147] [Type (Qubit)[]]: Name: Internal(NodeId(37))
                                                Expr 33 [144-147] [Type Int]: Paren: Expr 34 [145-146] [Type Int]: Lit: Int(2)
                                SpecDecl _id_ [73-178] (CtlAdj): Impl:
                                    Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 37 [73-178] "ctls"
                                        Pat _id_ [73-178] [Type Qubit]: Elided
                                    Block 24 [135-150] [Type ()]:
                                        Stmt 25 [137-142]: Semi: Expr 26 [137-141] [Type ()]: Call:
                                            Expr 27 [137-138] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 27 [137-138] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 28 [138-141] [Type ((Qubit)[], Int)]: Tuple:
                                                Expr _id_ [138-141] [Type (Qubit)[]]: Name: Internal(NodeId(37))
                                                Expr 28 [138-141] [Type Int]: Paren: Expr 29 [139-140] [Type Int]: Lit: Int(1)
                                        Stmt 30 [143-148]: Semi: Expr 31 [143-147] [Type ()]: Call:
                                            Expr 32 [143-144] [Type (((Qubit)[], Int) => () is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 32 [143-144] [Type (Int => () is Adj + Ctl)]: Name: Internal(NodeId(5))
                                            Expr 33 [144-147] [Type ((Qubit)[], Int)]: Tuple:
                                                Expr _id_ [144-147] [Type (Qubit)[]]: Name: Internal(NodeId(37))
                                                Expr 33 [144-147] [Type Int]: Paren: Expr 34 [145-146] [Type Int]: Lit: Int(2)"#]],
    );
}
