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

    let errors = generate_specs(store.core(), &mut unit);
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
                Item 0 [0-184] (Public):
                    Namespace (Ident 20 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 4 [68-79]: Impl:
                            Block 5 [77-79]: <empty>
                        adj: <none>
                        ctl: SpecDecl 6 [88-113]: Impl:
                            Pat 7 [100-104] [Type (Qubit)[]]: Bind: Ident 8 [100-104] "ctls"
                            Block 9 [111-113]: <empty>
                        ctl-adj: <none>
                Item 2 [124-182] (Public):
                    Parent: 0
                    Callable 10 [124-182] (operation):
                        name: Ident 11 [134-135] "B"
                        input: Pat 12 [136-145] [Type Qubit]: Bind: Ident 13 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 14 [124-182]: Impl:
                            Block 15 [161-182] [Type Unit]:
                                Stmt 16 [171-176]: Semi: Expr 17 [171-175] [Type Unit]: Call:
                                    Expr 18 [171-172] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 19 [173-174] [Type Qubit]: Var: Local 13
                        adj: <none>
                        ctl: SpecDecl 22 [124-182]: Impl:
                            Pat 23 [124-182] [Type (Qubit)[]]: Bind: Ident 24 [124-182] "ctls"
                            Block 25 [161-182] [Type Unit]:
                                Stmt 26 [171-176]: Semi: Expr 27 [171-175] [Type Unit]: Call:
                                    Expr 28 [171-172] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                        Expr 29 [171-172] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 30 [173-174] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr 31 [173-174] [Type (Qubit)[]]: Var: Local 24
                                        Expr 32 [173-174] [Type Qubit]: Var: Local 13
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn generate_ctl_auto() {
    check(
        indoc! {"
            namespace test {
                operation A(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (ctls, ...) {}
                }
                operation B(q : Qubit) : Unit is Ctl {
                    body ... {
                        A(q);
                    }
                    controlled auto;
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-242] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 4 [68-79]: Impl:
                            Block 5 [77-79]: <empty>
                        adj: <none>
                        ctl: SpecDecl 6 [88-113]: Impl:
                            Pat 7 [100-104] [Type (Qubit)[]]: Bind: Ident 8 [100-104] "ctls"
                            Block 9 [111-113]: <empty>
                        ctl-adj: <none>
                Item 2 [124-240] (Public):
                    Parent: 0
                    Callable 10 [124-240] (operation):
                        name: Ident 11 [134-135] "B"
                        input: Pat 12 [136-145] [Type Qubit]: Bind: Ident 13 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 14 [171-209]: Impl:
                            Block 15 [180-209] [Type Unit]:
                                Stmt 16 [194-199]: Semi: Expr 17 [194-198] [Type Unit]: Call:
                                    Expr 18 [194-195] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 19 [196-197] [Type Qubit]: Var: Local 13
                        adj: <none>
                        ctl: SpecDecl 23 [218-234]: Impl:
                            Pat 24 [218-234] [Type (Qubit)[]]: Bind: Ident 25 [218-234] "ctls"
                            Block 26 [180-209] [Type Unit]:
                                Stmt 27 [194-199]: Semi: Expr 28 [194-198] [Type Unit]: Call:
                                    Expr 29 [194-195] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                        Expr 30 [194-195] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 31 [196-197] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr 32 [196-197] [Type (Qubit)[]]: Var: Local 25
                                        Expr 33 [196-197] [Type Qubit]: Var: Local 13
                        ctl-adj: <none>"#]],
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
                Item 0 [0-310] (Public):
                    Namespace (Ident 29 [10-14] "test"): Item 1, Item 2
                Item 1 [21-148] (Public):
                    Parent: 0
                    Callable 0 [21-148] (operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [74-85]: Impl:
                            Block 5 [83-85]: <empty>
                        adj: SpecDecl 6 [94-108]: Impl:
                            Block 7 [106-108]: <empty>
                        ctl: SpecDecl 8 [117-142]: Impl:
                            Pat 9 [129-133] [Type (Qubit)[]]: Bind: Ident 10 [129-133] "ctls"
                            Block 11 [140-142]: <empty>
                        ctl-adj: SpecDecl 31 [21-148]: Impl:
                            Pat 32 [21-148] [Type (Qubit)[]]: Bind: Ident 33 [21-148] "ctls"
                            Block 34 [106-108]: <empty>
                Item 2 [153-308] (Public):
                    Parent: 0
                    Callable 12 [153-308] (operation):
                        name: Ident 13 [163-164] "B"
                        input: Pat 14 [165-174] [Type Qubit]: Bind: Ident 15 [165-166] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 16 [206-244]: Impl:
                            Block 17 [215-244] [Type Unit]:
                                Stmt 18 [229-234]: Semi: Expr 19 [229-233] [Type Unit]: Call:
                                    Expr 20 [229-230] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [231-232] [Type Qubit]: Var: Local 15
                        adj: SpecDecl 22 [253-302]: Impl:
                            Block 23 [265-302] [Type Unit]:
                                Stmt 24 [279-292]: Semi: Expr 25 [279-291] [Type Unit]: Call:
                                    Expr 26 [279-288] [Type (Qubit => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 27 [287-288] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 28 [289-290] [Type Qubit]: Var: Local 15
                        ctl: SpecDecl 36 [153-308]: Impl:
                            Pat 37 [153-308] [Type (Qubit)[]]: Bind: Ident 38 [153-308] "ctls"
                            Block 39 [215-244] [Type Unit]:
                                Stmt 40 [229-234]: Semi: Expr 41 [229-233] [Type Unit]: Call:
                                    Expr 42 [229-230] [Type (((Qubit)[], Qubit) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 43 [229-230] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 44 [231-232] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr 45 [231-232] [Type (Qubit)[]]: Var: Local 38
                                        Expr 46 [231-232] [Type Qubit]: Var: Local 15
                        ctl-adj: SpecDecl 48 [153-308]: Impl:
                            Pat 49 [153-308] [Type (Qubit)[]]: Bind: Ident 50 [153-308] "ctls"
                            Block 51 [265-302] [Type Unit]:
                                Stmt 52 [279-292]: Semi: Expr 53 [279-291] [Type Unit]: Call:
                                    Expr 54 [279-288] [Type (((Qubit)[], Qubit) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 55 [279-288] [Type (Qubit => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 56 [287-288] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 57 [289-290] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr 58 [289-290] [Type (Qubit)[]]: Var: Local 50
                                        Expr 59 [289-290] [Type Qubit]: Var: Local 15"#]],
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
                Item 0 [0-259] (Public):
                    Namespace (Ident 28 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 4 [68-79]: Impl:
                            Block 5 [77-79]: <empty>
                        adj: <none>
                        ctl: SpecDecl 6 [88-113]: Impl:
                            Pat 7 [100-104] [Type (Qubit)[]]: Bind: Ident 8 [100-104] "ctls"
                            Block 9 [111-113]: <empty>
                        ctl-adj: <none>
                Item 2 [124-257] (Public):
                    Parent: 0
                    Callable 10 [124-257] (operation):
                        name: Ident 11 [134-135] "B"
                        input: Pat 12 [136-145] [Type Qubit]: Bind: Ident 13 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 14 [124-257]: Impl:
                            Block 15 [161-257] [Type Unit]:
                                Stmt 16 [171-251]: Expr: Expr 17 [171-251] [Type Unit]: Conjugate:
                                    Block 18 [178-207] [Type Unit]:
                                        Stmt 19 [192-197]: Semi: Expr 20 [192-196] [Type Unit]: Call:
                                            Expr 21 [192-193] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 22 [194-195] [Type Qubit]: Var: Local 13
                                    Block 23 [222-251] [Type Unit]:
                                        Stmt 24 [236-241]: Semi: Expr 25 [236-240] [Type Unit]: Call:
                                            Expr 26 [236-237] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 27 [238-239] [Type Qubit]: Var: Local 13
                        adj: <none>
                        ctl: SpecDecl 30 [124-257]: Impl:
                            Pat 31 [124-257] [Type (Qubit)[]]: Bind: Ident 32 [124-257] "ctls"
                            Block 33 [161-257] [Type Unit]:
                                Stmt 34 [171-251]: Expr: Expr 35 [171-251] [Type Unit]: Conjugate:
                                    Block 36 [178-207] [Type Unit]:
                                        Stmt 37 [192-197]: Semi: Expr 38 [192-196] [Type Unit]: Call:
                                            Expr 39 [192-193] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 40 [194-195] [Type Qubit]: Var: Local 13
                                    Block 41 [222-251] [Type Unit]:
                                        Stmt 42 [236-241]: Semi: Expr 43 [236-240] [Type Unit]: Call:
                                            Expr 44 [236-237] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                                Expr 45 [236-237] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 46 [238-239] [Type ((Qubit)[], Qubit)]: Tuple:
                                                Expr 47 [238-239] [Type (Qubit)[]]: Var: Local 32
                                                Expr 48 [238-239] [Type Qubit]: Var: Local 13
                        ctl-adj: <none>"#]],
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
                Item 0 [0-150] (Public):
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2, Item 3
                Item 1 [21-45] (Public):
                    Parent: 0
                    Callable 0 [21-45] (function):
                        name: Ident 1 [30-33] "Foo"
                        input: Pat 2 [33-35] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [21-45]: Impl:
                            Block 4 [43-45]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [50-80] (Public):
                    Parent: 0
                    Callable 5 [50-80] (operation):
                        name: Ident 6 [60-61] "A"
                        input: Pat 7 [61-63] [Type Unit]: Unit
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 8 [50-80]: Impl:
                            Block 9 [78-80]: <empty>
                        adj: <none>
                        ctl: SpecDecl 25 [50-80]: Impl:
                            Pat 26 [50-80] [Type (Qubit)[]]: Bind: Ident 27 [50-80] "ctls"
                            Block 28 [78-80]: <empty>
                        ctl-adj: <none>
                Item 3 [85-148] (Public):
                    Parent: 0
                    Callable 10 [85-148] (operation):
                        name: Ident 11 [95-96] "B"
                        input: Pat 12 [96-98] [Type Unit]: Unit
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 13 [85-148]: Impl:
                            Block 14 [113-148] [Type Unit]:
                                Stmt 15 [123-129]: Semi: Expr 16 [123-128] [Type Unit]: Call:
                                    Expr 17 [123-126] [Type (Unit -> Unit)]: Var: Item 1
                                    Expr 18 [126-128] [Type Unit]: Unit
                                Stmt 19 [138-142]: Semi: Expr 20 [138-141] [Type Unit]: Call:
                                    Expr 21 [138-139] [Type (Unit => Unit is Ctl)]: Var: Item 2
                                    Expr 22 [139-141] [Type Unit]: Unit
                        adj: <none>
                        ctl: SpecDecl 30 [85-148]: Impl:
                            Pat 31 [85-148] [Type (Qubit)[]]: Bind: Ident 32 [85-148] "ctls"
                            Block 33 [113-148] [Type Unit]:
                                Stmt 34 [123-129]: Semi: Expr 35 [123-128] [Type Unit]: Call:
                                    Expr 36 [123-126] [Type (Unit -> Unit)]: Var: Item 1
                                    Expr 37 [126-128] [Type Unit]: Unit
                                Stmt 38 [138-142]: Semi: Expr 39 [138-141] [Type Unit]: Call:
                                    Expr 40 [138-139] [Type (((Qubit)[], Unit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                        Expr 41 [138-139] [Type (Unit => Unit is Ctl)]: Var: Item 2
                                    Expr 42 [139-141] [Type ((Qubit)[], Unit)]: Tuple:
                                        Expr 43 [139-141] [Type (Qubit)[]]: Var: Local 32
                                        Expr 44 [139-141] [Type Unit]: Unit
                        ctl-adj: <none>"#]],
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
                Item 0 [0-168] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 22 [21-62]: Impl:
                            Block 23 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-166] (Public):
                    Parent: 0
                    Callable 6 [67-166] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [114-138]: Impl:
                            Block 11 [123-138] [Type Unit]:
                                Stmt 12 [125-130]: Semi: Expr 13 [125-129] [Type Unit]: Call:
                                    Expr 14 [125-126] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 15 [127-128] [Type Int]: Lit: Int(1)
                                Stmt 16 [131-136]: Semi: Expr 17 [131-135] [Type Unit]: Call:
                                    Expr 18 [131-132] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 19 [133-134] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 24 [147-160]: Impl:
                            Block 25 [123-138] [Type Unit]:
                                Stmt 26 [125-130]: Semi: Expr 27 [125-129] [Type Unit]: Call:
                                    Expr 28 [125-126] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 29 [127-128] [Type Int]: Lit: Int(1)
                                Stmt 30 [131-136]: Semi: Expr 31 [131-135] [Type Unit]: Call:
                                    Expr 32 [131-132] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 33 [133-134] [Type Int]: Lit: Int(2)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-180] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68]: Impl:
                            Block 5 [66-68]: <empty>
                        adj: SpecDecl 27 [21-68]: Impl:
                            Block 28 [66-68]: <empty>
                        ctl: SpecDecl 23 [21-68]: Impl:
                            Pat 24 [21-68] [Type (Qubit)[]]: Bind: Ident 25 [21-68] "ctls"
                            Block 26 [66-68]: <empty>
                        ctl-adj: SpecDecl 30 [21-68]: Impl:
                            Pat 31 [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                            Block 33 [66-68]: <empty>
                Item 2 [73-178] (Public):
                    Parent: 0
                    Callable 6 [73-178] (operation):
                        name: Ident 7 [83-84] "A"
                        input: Pat 8 [85-94] [Type Qubit]: Bind: Ident 9 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 10 [126-150]: Impl:
                            Block 11 [135-150] [Type Unit]:
                                Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                    Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                    Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 19 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 53 [159-172]: Impl:
                            Block 54 [135-150] [Type Unit]:
                                Stmt 55 [137-142]: Semi: Expr 56 [137-141] [Type Unit]: Call:
                                    Expr 57 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 58 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 59 [143-148]: Semi: Expr 60 [143-147] [Type Unit]: Call:
                                    Expr 61 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 62 [145-146] [Type Int]: Lit: Int(2)
                        ctl: SpecDecl 35 [73-178]: Impl:
                            Pat 36 [73-178] [Type (Qubit)[]]: Bind: Ident 37 [73-178] "ctls"
                            Block 38 [135-150] [Type Unit]:
                                Stmt 39 [137-142]: Semi: Expr 40 [137-141] [Type Unit]: Call:
                                    Expr 41 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 42 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 43 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 44 [139-140] [Type (Qubit)[]]: Var: Local 37
                                        Expr 45 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 46 [143-148]: Semi: Expr 47 [143-147] [Type Unit]: Call:
                                    Expr 48 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 49 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 50 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 51 [145-146] [Type (Qubit)[]]: Var: Local 37
                                        Expr 52 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl 63 [73-178]: Impl:
                            Pat 64 [73-178] [Type (Qubit)[]]: Bind: Ident 65 [73-178] "ctls"
                            Block 66 [135-150] [Type Unit]:
                                Stmt 67 [137-142]: Semi: Expr 68 [137-141] [Type Unit]: Call:
                                    Expr 69 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 70 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 71 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 72 [139-140] [Type (Qubit)[]]: Var: Local 65
                                        Expr 73 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 74 [143-148]: Semi: Expr 75 [143-147] [Type Unit]: Call:
                                    Expr 76 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 77 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 78 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 79 [145-146] [Type (Qubit)[]]: Var: Local 65
                                        Expr 80 [145-146] [Type Int]: Lit: Int(2)"#]],
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
                Item 0 [0-141] (Public):
                    Namespace (Ident 20 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 21 [21-62]: Impl:
                            Block 22 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-139] (Public):
                    Parent: 0
                    Callable 6 [67-139] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [67-139]: Impl:
                            Block 11 [104-139] [Type Unit]:
                                Stmt 12 [114-119]: Semi: Expr 13 [114-118] [Type Unit]: Call:
                                    Expr 14 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 15 [116-117] [Type Int]: Lit: Int(1)
                                Stmt 16 [128-133]: Semi: Expr 17 [128-132] [Type Unit]: Call:
                                    Expr 18 [128-129] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 19 [130-131] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 23 [67-139]: Impl:
                            Block 24 [104-139] [Type Unit]:
                                Stmt 25 [128-133]: Semi: Expr 26 [128-132] [Type Unit]: Call:
                                    Expr 27 [128-129] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 28 [128-129] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 29 [130-131] [Type Int]: Lit: Int(2)
                                Stmt 30 [114-119]: Semi: Expr 31 [114-118] [Type Unit]: Call:
                                    Expr 32 [114-115] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 33 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 34 [116-117] [Type Int]: Lit: Int(1)
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn generate_adj_auto() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Adj {}
                operation A(q : Qubit) : Unit is Adj {
                    body ... {
                        B(1);
                        B(2);
                    }
                    adjoint auto;
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-200] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 22 [21-62]: Impl:
                            Block 23 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-198] (Public):
                    Parent: 0
                    Callable 6 [67-198] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [114-170]: Impl:
                            Block 11 [123-170] [Type Unit]:
                                Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                    Expr 14 [137-138] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 16 [155-160]: Semi: Expr 17 [155-159] [Type Unit]: Call:
                                    Expr 18 [155-156] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 19 [157-158] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 24 [179-192]: Impl:
                            Block 25 [123-170] [Type Unit]:
                                Stmt 26 [155-160]: Semi: Expr 27 [155-159] [Type Unit]: Call:
                                    Expr 28 [155-156] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 29 [155-156] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 30 [157-158] [Type Int]: Lit: Int(2)
                                Stmt 31 [137-142]: Semi: Expr 32 [137-141] [Type Unit]: Call:
                                    Expr 33 [137-138] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 34 [137-138] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 35 [139-140] [Type Int]: Lit: Int(1)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-238] (Public):
                    Namespace (Ident 32 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 33 [21-62]: Impl:
                            Block 34 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-236] (Public):
                    Parent: 0
                    Callable 6 [67-236] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [67-236]: Impl:
                            Block 11 [104-236] [Type Unit]:
                                Stmt 12 [114-230]: Expr: Expr 13 [114-230] [Type Unit]: Conjugate:
                                    Block 14 [121-168] [Type Unit]:
                                        Stmt 15 [135-140]: Semi: Expr 16 [135-139] [Type Unit]: Call:
                                            Expr 17 [135-136] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 18 [137-138] [Type Int]: Lit: Int(1)
                                        Stmt 19 [153-158]: Semi: Expr 20 [153-157] [Type Unit]: Call:
                                            Expr 21 [153-154] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 22 [155-156] [Type Int]: Lit: Int(2)
                                    Block 23 [183-230] [Type Unit]:
                                        Stmt 24 [197-202]: Semi: Expr 25 [197-201] [Type Unit]: Call:
                                            Expr 26 [197-198] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 27 [199-200] [Type Int]: Lit: Int(3)
                                        Stmt 28 [215-220]: Semi: Expr 29 [215-219] [Type Unit]: Call:
                                            Expr 30 [215-216] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 31 [217-218] [Type Int]: Lit: Int(4)
                        adj: SpecDecl 35 [67-236]: Impl:
                            Block 36 [104-236] [Type Unit]:
                                Stmt 37 [114-230]: Expr: Expr 38 [114-230] [Type Unit]: Conjugate:
                                    Block 39 [121-168] [Type Unit]:
                                        Stmt 40 [135-140]: Semi: Expr 41 [135-139] [Type Unit]: Call:
                                            Expr 42 [135-136] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 43 [137-138] [Type Int]: Lit: Int(1)
                                        Stmt 44 [153-158]: Semi: Expr 45 [153-157] [Type Unit]: Call:
                                            Expr 46 [153-154] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 47 [155-156] [Type Int]: Lit: Int(2)
                                    Block 48 [183-230] [Type Unit]:
                                        Stmt 49 [215-220]: Semi: Expr 50 [215-219] [Type Unit]: Call:
                                            Expr 51 [215-216] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 52 [215-216] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 53 [217-218] [Type Int]: Lit: Int(4)
                                        Stmt 54 [197-202]: Semi: Expr 55 [197-201] [Type Unit]: Call:
                                            Expr 56 [197-198] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 57 [197-198] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 58 [199-200] [Type Int]: Lit: Int(3)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-268] (Public):
                    Namespace (Ident 64 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 65 [21-62]: Impl:
                            Block 66 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-266] (Public):
                    Parent: 0
                    Callable 6 [67-266] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [67-266]: Impl:
                            Block 11 [104-266] [Type Unit]:
                                Stmt 12 [114-119]: Semi: Expr 13 [114-118] [Type Unit]: Call:
                                    Expr 14 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 15 [116-117] [Type Int]: Lit: Int(1)
                                Stmt 16 [128-166]: Local (Immutable):
                                    Pat 17 [132-135] [Type Bool]: Bind: Ident 18 [132-135] "val"
                                    Expr 19 [138-165] [Type Bool]: If:
                                        Expr 20 [141-145] [Type Bool]: Lit: Bool(true)
                                        Expr 21 [146-153] [Type Bool]: Expr Block: Block 22 [146-153] [Type Bool]:
                                            Stmt 23 [147-152]: Expr: Expr 24 [147-152] [Type Bool]: Lit: Bool(false)
                                        Expr 25 [154-165] [Type Bool]: Expr Block: Block 26 [159-165] [Type Bool]:
                                            Stmt 27 [160-164]: Expr: Expr 28 [160-164] [Type Bool]: Lit: Bool(true)
                                Stmt 29 [175-180]: Semi: Expr 30 [175-179] [Type Unit]: Call:
                                    Expr 31 [175-176] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 32 [177-178] [Type Int]: Lit: Int(2)
                                Stmt 33 [189-246]: Expr: Expr 34 [189-246] [Type Unit]: If:
                                    Expr 35 [192-197] [Type Bool]: Lit: Bool(false)
                                    Expr 36 [198-211] [Type Unit]: Expr Block: Block 37 [198-211] [Type Unit]:
                                        Stmt 38 [199-204]: Semi: Expr 39 [199-203] [Type Unit]: Call:
                                            Expr 40 [199-200] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 41 [201-202] [Type Int]: Lit: Int(3)
                                        Stmt 42 [205-210]: Semi: Expr 43 [205-209] [Type Unit]: Call:
                                            Expr 44 [205-206] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 45 [207-208] [Type Int]: Lit: Int(4)
                                    Expr 46 [212-246] [Type Unit]: Expr Block: Block 47 [217-246] [Type Unit]:
                                        Stmt 48 [218-223]: Semi: Expr 49 [218-222] [Type Unit]: Call:
                                            Expr 50 [218-219] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 51 [220-221] [Type Int]: Lit: Int(5)
                                        Stmt 52 [224-239]: Local (Immutable):
                                            Pat 53 [228-231] [Type Bool]: Bind: Ident 54 [228-231] "val"
                                            Expr 55 [234-238] [Type Bool]: Lit: Bool(true)
                                        Stmt 56 [240-245]: Semi: Expr 57 [240-244] [Type Unit]: Call:
                                            Expr 58 [240-241] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 59 [242-243] [Type Int]: Lit: Int(6)
                                Stmt 60 [255-260]: Semi: Expr 61 [255-259] [Type Unit]: Call:
                                    Expr 62 [255-256] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 63 [257-258] [Type Int]: Lit: Int(7)
                        adj: SpecDecl 67 [67-266]: Impl:
                            Block 68 [104-266] [Type Unit]:
                                Stmt 69 [128-166]: Local (Immutable):
                                    Pat 70 [132-135] [Type Bool]: Bind: Ident 71 [132-135] "val"
                                    Expr 72 [138-165] [Type Bool]: If:
                                        Expr 73 [141-145] [Type Bool]: Lit: Bool(true)
                                        Expr 74 [146-153] [Type Bool]: Expr Block: Block 75 [146-153] [Type Bool]:
                                            Stmt 76 [147-152]: Expr: Expr 77 [147-152] [Type Bool]: Lit: Bool(false)
                                        Expr 78 [154-165] [Type Bool]: Expr Block: Block 79 [159-165] [Type Bool]:
                                            Stmt 80 [160-164]: Expr: Expr 81 [160-164] [Type Bool]: Lit: Bool(true)
                                Stmt 82 [255-260]: Semi: Expr 83 [255-259] [Type Unit]: Call:
                                    Expr 84 [255-256] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 85 [255-256] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 86 [257-258] [Type Int]: Lit: Int(7)
                                Stmt 87 [189-246]: Expr: Expr 88 [189-246] [Type Unit]: If:
                                    Expr 89 [192-197] [Type Bool]: Lit: Bool(false)
                                    Expr 90 [198-211] [Type Unit]: Expr Block: Block 91 [198-211] [Type Unit]:
                                        Stmt 92 [205-210]: Semi: Expr 93 [205-209] [Type Unit]: Call:
                                            Expr 94 [205-206] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 95 [205-206] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 96 [207-208] [Type Int]: Lit: Int(4)
                                        Stmt 97 [199-204]: Semi: Expr 98 [199-203] [Type Unit]: Call:
                                            Expr 99 [199-200] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 100 [199-200] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 101 [201-202] [Type Int]: Lit: Int(3)
                                    Expr 102 [212-246] [Type Unit]: Expr Block: Block 103 [217-246] [Type Unit]:
                                        Stmt 104 [224-239]: Local (Immutable):
                                            Pat 105 [228-231] [Type Bool]: Bind: Ident 106 [228-231] "val"
                                            Expr 107 [234-238] [Type Bool]: Lit: Bool(true)
                                        Stmt 108 [240-245]: Semi: Expr 109 [240-244] [Type Unit]: Call:
                                            Expr 110 [240-241] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 111 [240-241] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 112 [242-243] [Type Int]: Lit: Int(6)
                                        Stmt 113 [218-223]: Semi: Expr 114 [218-222] [Type Unit]: Call:
                                            Expr 115 [218-219] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 116 [218-219] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 117 [220-221] [Type Int]: Lit: Int(5)
                                Stmt 118 [175-180]: Semi: Expr 119 [175-179] [Type Unit]: Call:
                                    Expr 120 [175-176] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 121 [175-176] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 122 [177-178] [Type Int]: Lit: Int(2)
                                Stmt 123 [114-119]: Semi: Expr 124 [114-118] [Type Unit]: Call:
                                    Expr 125 [114-115] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 126 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 127 [116-117] [Type Int]: Lit: Int(1)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-183] (Public):
                    Namespace (Ident 28 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 29 [21-62]: Impl:
                            Block 30 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-181] (Public):
                    Parent: 0
                    Callable 6 [67-181] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [67-181]: Impl:
                            Block 11 [104-181] [Type Unit]:
                                Stmt 12 [114-175]: Expr: Expr 13 [114-175] [Type Unit]: For:
                                    Pat 14 [118-119] [Type Int]: Bind: Ident 15 [118-119] "i"
                                    Expr 16 [123-127] [Type Range]: Range:
                                        Expr 17 [123-124] [Type Int]: Lit: Int(0)
                                        <no step>
                                        Expr 18 [126-127] [Type Int]: Lit: Int(5)
                                    Block 19 [128-175] [Type Unit]:
                                        Stmt 20 [142-147]: Semi: Expr 21 [142-146] [Type Unit]: Call:
                                            Expr 22 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 23 [144-145] [Type Int]: Lit: Int(1)
                                        Stmt 24 [160-165]: Semi: Expr 25 [160-164] [Type Unit]: Call:
                                            Expr 26 [160-161] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 27 [162-163] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 32 [67-181]: Impl:
                            Block 33 [104-181] [Type Unit]:
                                Stmt 34 [114-175]: Expr: Expr 35 [0-0] [Type Unit]: Expr Block: Block 36 [0-0] [Type Unit]:
                                    Stmt 37 [0-0]: Local (Immutable):
                                        Pat 38 [0-0] [Type Range]: Bind: Ident 39 [0-0] "generated_range"
                                        Expr 40 [123-127] [Type Range]: Range:
                                            Expr 41 [123-124] [Type Int]: Lit: Int(0)
                                            <no step>
                                            Expr 42 [126-127] [Type Int]: Lit: Int(5)
                                    Stmt 43 [0-0]: Expr: Expr 44 [0-0] [Type Unit]: For:
                                        Pat 45 [118-119] [Type Int]: Bind: Ident 46 [118-119] "i"
                                        Expr 47 [0-0] [Type Range]: Range:
                                            Expr 48 [0-0] [Type Int]: BinOp (Add):
                                                Expr 49 [0-0] [Type Int]: Field:
                                                    Expr 50 [0-0] [Type Range]: Var: Local 39
                                                    Prim(Start)
                                                Expr 51 [0-0] [Type Int]: BinOp (Mul):
                                                    Expr 52 [0-0] [Type Int]: BinOp (Div):
                                                        Expr 53 [0-0] [Type Int]: BinOp (Sub):
                                                            Expr 54 [0-0] [Type Int]: Field:
                                                                Expr 55 [0-0] [Type Range]: Var: Local 39
                                                                Prim(End)
                                                            Expr 56 [0-0] [Type Int]: Field:
                                                                Expr 57 [0-0] [Type Range]: Var: Local 39
                                                                Prim(Start)
                                                        Expr 58 [0-0] [Type Int]: Field:
                                                            Expr 59 [0-0] [Type Range]: Var: Local 39
                                                            Prim(Step)
                                                    Expr 60 [0-0] [Type Int]: Field:
                                                        Expr 61 [0-0] [Type Range]: Var: Local 39
                                                        Prim(Step)
                                            Expr 62 [0-0] [Type Int]: UnOp (Neg):
                                                Expr 63 [0-0] [Type Int]: Field:
                                                    Expr 64 [0-0] [Type Range]: Var: Local 39
                                                    Prim(Step)
                                            Expr 65 [0-0] [Type Int]: Field:
                                                Expr 66 [0-0] [Type Range]: Var: Local 39
                                                Prim(Start)
                                        Block 67 [128-175] [Type Unit]:
                                            Stmt 68 [160-165]: Semi: Expr 69 [160-164] [Type Unit]: Call:
                                                Expr 70 [160-161] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 71 [160-161] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 72 [162-163] [Type Int]: Lit: Int(2)
                                            Stmt 73 [142-147]: Semi: Expr 74 [142-146] [Type Unit]: Call:
                                                Expr 75 [142-143] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 76 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 77 [144-145] [Type Int]: Lit: Int(1)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-190] (Public):
                    Namespace (Ident 29 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 30 [21-62]: Impl:
                            Block 31 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-188] (Public):
                    Parent: 0
                    Callable 6 [67-188] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [67-188]: Impl:
                            Block 11 [104-188] [Type Unit]:
                                Stmt 12 [114-182]: Expr: Expr 13 [114-182] [Type Unit]: For:
                                    Pat 14 [118-121] [Type Int]: Bind: Ident 15 [118-121] "val"
                                    Expr 16 [125-134] [Type (Int)[]]: Array:
                                        Expr 17 [126-127] [Type Int]: Lit: Int(0)
                                        Expr 18 [129-130] [Type Int]: Lit: Int(1)
                                        Expr 19 [132-133] [Type Int]: Lit: Int(2)
                                    Block 20 [135-182] [Type Unit]:
                                        Stmt 21 [149-154]: Semi: Expr 22 [149-153] [Type Unit]: Call:
                                            Expr 23 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 24 [151-152] [Type Int]: Lit: Int(1)
                                        Stmt 25 [167-172]: Semi: Expr 26 [167-171] [Type Unit]: Call:
                                            Expr 27 [167-168] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 28 [169-170] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 34 [67-188]: Impl:
                            Block 35 [104-188] [Type Unit]:
                                Stmt 36 [114-182]: Expr: Expr 37 [0-0] [Type Unit]: Expr Block: Block 38 [0-0] [Type Unit]:
                                    Stmt 39 [0-0]: Local (Immutable):
                                        Pat 40 [0-0] [Type (Int)[]]: Bind: Ident 41 [0-0] "generated_array"
                                        Expr 42 [125-134] [Type (Int)[]]: Array:
                                            Expr 43 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 44 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 45 [132-133] [Type Int]: Lit: Int(2)
                                    Stmt 46 [0-0]: Expr: Expr 47 [0-0] [Type Unit]: For:
                                        Pat 48 [0-0] [Type Int]: Bind: Ident 49 [0-0] "generated_index"
                                        Expr 50 [0-0] [Type Range]: Range:
                                            Expr 51 [0-0] [Type Int]: BinOp (Sub):
                                                Expr 52 [0-0] [Type Int]: Call:
                                                    Expr 53 [0-0] [Type ((Int)[] -> Int)]: Var:
                                                        res: Item 1 (Package 0)
                                                        generics:
                                                            Int
                                                    Expr 54 [0-0] [Type (Int)[]]: Var: Local 41
                                                Expr 55 [0-0] [Type Int]: Lit: Int(1)
                                            Expr 56 [0-0] [Type Int]: Lit: Int(-1)
                                            Expr 57 [0-0] [Type Int]: Lit: Int(0)
                                        Block 58 [135-182] [Type Unit]:
                                            Stmt 59 [0-0]: Local (Immutable):
                                                Pat 60 [118-121] [Type Int]: Bind: Ident 61 [118-121] "val"
                                                Expr 62 [0-0] [Type Int]: Index:
                                                    Expr 63 [0-0] [Type (Int)[]]: Var: Local 41
                                                    Expr 64 [0-0] [Type Int]: Var: Local 49
                                            Stmt 65 [167-172]: Semi: Expr 66 [167-171] [Type Unit]: Call:
                                                Expr 67 [167-168] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 68 [167-168] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 69 [169-170] [Type Int]: Lit: Int(2)
                                            Stmt 70 [149-154]: Semi: Expr 71 [149-153] [Type Unit]: Call:
                                                Expr 72 [149-150] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 73 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 74 [151-152] [Type Int]: Lit: Int(1)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-320] (Public):
                    Namespace (Ident 50 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62]: Impl:
                            Block 5 [60-62]: <empty>
                        adj: SpecDecl 51 [21-62]: Impl:
                            Block 52 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-318] (Public):
                    Parent: 0
                    Callable 6 [67-318] (operation):
                        name: Ident 7 [77-78] "A"
                        input: Pat 8 [79-88] [Type Qubit]: Bind: Ident 9 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [67-318]: Impl:
                            Block 11 [104-318] [Type Unit]:
                                Stmt 12 [114-312]: Expr: Expr 13 [114-312] [Type Unit]: For:
                                    Pat 14 [118-121] [Type Int]: Bind: Ident 15 [118-121] "val"
                                    Expr 16 [125-134] [Type (Int)[]]: Array:
                                        Expr 17 [126-127] [Type Int]: Lit: Int(0)
                                        Expr 18 [129-130] [Type Int]: Lit: Int(1)
                                        Expr 19 [132-133] [Type Int]: Lit: Int(2)
                                    Block 20 [135-312] [Type Unit]:
                                        Stmt 21 [149-154]: Semi: Expr 22 [149-153] [Type Unit]: Call:
                                            Expr 23 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 24 [151-152] [Type Int]: Lit: Int(1)
                                        Stmt 25 [167-197]: Local (Immutable):
                                            Pat 26 [171-174] [Type (Bool)[]]: Bind: Ident 27 [171-174] "arr"
                                            Expr 28 [177-196] [Type (Bool)[]]: Array:
                                                Expr 29 [178-182] [Type Bool]: Lit: Bool(true)
                                                Expr 30 [184-189] [Type Bool]: Lit: Bool(false)
                                                Expr 31 [191-195] [Type Bool]: Lit: Bool(true)
                                        Stmt 32 [210-284]: Expr: Expr 33 [210-284] [Type Unit]: For:
                                            Pat 34 [214-217] [Type Bool]: Bind: Ident 35 [214-217] "val"
                                            Expr 36 [221-224] [Type (Bool)[]]: Var: Local 27
                                            Block 37 [225-284] [Type Unit]:
                                                Stmt 38 [243-248]: Semi: Expr 39 [243-247] [Type Unit]: Call:
                                                    Expr 40 [243-244] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 41 [245-246] [Type Int]: Lit: Int(2)
                                                Stmt 42 [265-270]: Semi: Expr 43 [265-269] [Type Unit]: Call:
                                                    Expr 44 [265-266] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 45 [267-268] [Type Int]: Lit: Int(3)
                                        Stmt 46 [297-302]: Semi: Expr 47 [297-301] [Type Unit]: Call:
                                            Expr 48 [297-298] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 49 [299-300] [Type Int]: Lit: Int(4)
                        adj: SpecDecl 57 [67-318]: Impl:
                            Block 58 [104-318] [Type Unit]:
                                Stmt 59 [114-312]: Expr: Expr 60 [0-0] [Type Unit]: Expr Block: Block 61 [0-0] [Type Unit]:
                                    Stmt 62 [0-0]: Local (Immutable):
                                        Pat 63 [0-0] [Type (Int)[]]: Bind: Ident 64 [0-0] "generated_array"
                                        Expr 65 [125-134] [Type (Int)[]]: Array:
                                            Expr 66 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 67 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 68 [132-133] [Type Int]: Lit: Int(2)
                                    Stmt 69 [0-0]: Expr: Expr 70 [0-0] [Type Unit]: For:
                                        Pat 71 [0-0] [Type Int]: Bind: Ident 72 [0-0] "generated_index"
                                        Expr 73 [0-0] [Type Range]: Range:
                                            Expr 74 [0-0] [Type Int]: BinOp (Sub):
                                                Expr 75 [0-0] [Type Int]: Call:
                                                    Expr 76 [0-0] [Type ((Int)[] -> Int)]: Var:
                                                        res: Item 1 (Package 0)
                                                        generics:
                                                            Int
                                                    Expr 77 [0-0] [Type (Int)[]]: Var: Local 64
                                                Expr 78 [0-0] [Type Int]: Lit: Int(1)
                                            Expr 79 [0-0] [Type Int]: Lit: Int(-1)
                                            Expr 80 [0-0] [Type Int]: Lit: Int(0)
                                        Block 81 [135-312] [Type Unit]:
                                            Stmt 82 [0-0]: Local (Immutable):
                                                Pat 83 [118-121] [Type Int]: Bind: Ident 84 [118-121] "val"
                                                Expr 85 [0-0] [Type Int]: Index:
                                                    Expr 86 [0-0] [Type (Int)[]]: Var: Local 64
                                                    Expr 87 [0-0] [Type Int]: Var: Local 72
                                            Stmt 88 [167-197]: Local (Immutable):
                                                Pat 89 [171-174] [Type (Bool)[]]: Bind: Ident 90 [171-174] "arr"
                                                Expr 91 [177-196] [Type (Bool)[]]: Array:
                                                    Expr 92 [178-182] [Type Bool]: Lit: Bool(true)
                                                    Expr 93 [184-189] [Type Bool]: Lit: Bool(false)
                                                    Expr 94 [191-195] [Type Bool]: Lit: Bool(true)
                                            Stmt 95 [297-302]: Semi: Expr 96 [297-301] [Type Unit]: Call:
                                                Expr 97 [297-298] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 98 [297-298] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 99 [299-300] [Type Int]: Lit: Int(4)
                                            Stmt 100 [210-284]: Expr: Expr 101 [0-0] [Type Unit]: Expr Block: Block 102 [0-0] [Type Unit]:
                                                Stmt 103 [0-0]: Local (Immutable):
                                                    Pat 104 [0-0] [Type (Bool)[]]: Bind: Ident 105 [0-0] "generated_array"
                                                    Expr 106 [221-224] [Type (Bool)[]]: Var: Local 90
                                                Stmt 107 [0-0]: Expr: Expr 108 [0-0] [Type Unit]: For:
                                                    Pat 109 [0-0] [Type Int]: Bind: Ident 110 [0-0] "generated_index"
                                                    Expr 111 [0-0] [Type Range]: Range:
                                                        Expr 112 [0-0] [Type Int]: BinOp (Sub):
                                                            Expr 113 [0-0] [Type Int]: Call:
                                                                Expr 114 [0-0] [Type ((Bool)[] -> Int)]: Var:
                                                                    res: Item 1 (Package 0)
                                                                    generics:
                                                                        Bool
                                                                Expr 115 [0-0] [Type (Bool)[]]: Var: Local 105
                                                            Expr 116 [0-0] [Type Int]: Lit: Int(1)
                                                        Expr 117 [0-0] [Type Int]: Lit: Int(-1)
                                                        Expr 118 [0-0] [Type Int]: Lit: Int(0)
                                                    Block 119 [225-284] [Type Unit]:
                                                        Stmt 120 [0-0]: Local (Immutable):
                                                            Pat 121 [214-217] [Type Bool]: Bind: Ident 122 [214-217] "val"
                                                            Expr 123 [0-0] [Type Bool]: Index:
                                                                Expr 124 [0-0] [Type (Bool)[]]: Var: Local 105
                                                                Expr 125 [0-0] [Type Int]: Var: Local 110
                                                        Stmt 126 [265-270]: Semi: Expr 127 [265-269] [Type Unit]: Call:
                                                            Expr 128 [265-266] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                                Expr 129 [265-266] [Type (Int => Unit is Adj)]: Var: Item 1
                                                            Expr 130 [267-268] [Type Int]: Lit: Int(3)
                                                        Stmt 131 [243-248]: Semi: Expr 132 [243-247] [Type Unit]: Call:
                                                            Expr 133 [243-244] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                                Expr 134 [243-244] [Type (Int => Unit is Adj)]: Var: Item 1
                                                            Expr 135 [245-246] [Type Int]: Lit: Int(2)
                                            Stmt 136 [149-154]: Semi: Expr 137 [149-153] [Type Unit]: Call:
                                                Expr 138 [149-150] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 139 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 140 [151-152] [Type Int]: Lit: Int(1)
                        ctl: <none>
                        ctl-adj: <none>"#]],
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
                Item 0 [0-158] (Public):
                    Namespace (Ident 20 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68]: Impl:
                            Block 5 [66-68]: <empty>
                        adj: SpecDecl 26 [21-68]: Impl:
                            Block 27 [66-68]: <empty>
                        ctl: SpecDecl 22 [21-68]: Impl:
                            Pat 23 [21-68] [Type (Qubit)[]]: Bind: Ident 24 [21-68] "ctls"
                            Block 25 [66-68]: <empty>
                        ctl-adj: SpecDecl 29 [21-68]: Impl:
                            Pat 30 [21-68] [Type (Qubit)[]]: Bind: Ident 31 [21-68] "ctls"
                            Block 32 [66-68]: <empty>
                Item 2 [73-156] (Public):
                    Parent: 0
                    Callable 6 [73-156] (operation):
                        name: Ident 7 [83-84] "A"
                        input: Pat 8 [85-94] [Type Qubit]: Bind: Ident 9 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 10 [126-150]: Impl:
                            Block 11 [135-150] [Type Unit]:
                                Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                    Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                    Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 19 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 52 [73-156]: Impl:
                            Block 53 [135-150] [Type Unit]:
                                Stmt 54 [143-148]: Semi: Expr 55 [143-147] [Type Unit]: Call:
                                    Expr 56 [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 57 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 58 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 59 [137-142]: Semi: Expr 60 [137-141] [Type Unit]: Call:
                                    Expr 61 [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 62 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 63 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl 34 [73-156]: Impl:
                            Pat 35 [73-156] [Type (Qubit)[]]: Bind: Ident 36 [73-156] "ctls"
                            Block 37 [135-150] [Type Unit]:
                                Stmt 38 [137-142]: Semi: Expr 39 [137-141] [Type Unit]: Call:
                                    Expr 40 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 41 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 42 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 43 [139-140] [Type (Qubit)[]]: Var: Local 36
                                        Expr 44 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 45 [143-148]: Semi: Expr 46 [143-147] [Type Unit]: Call:
                                    Expr 47 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 48 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 49 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 50 [145-146] [Type (Qubit)[]]: Var: Local 36
                                        Expr 51 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl 65 [73-156]: Impl:
                            Pat 66 [73-156] [Type (Qubit)[]]: Bind: Ident 67 [73-156] "ctls"
                            Block 68 [135-150] [Type Unit]:
                                Stmt 69 [143-148]: Semi: Expr 70 [143-147] [Type Unit]: Call:
                                    Expr 71 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 72 [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 73 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 74 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 75 [145-146] [Type (Qubit)[]]: Var: Local 67
                                        Expr 76 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 77 [137-142]: Semi: Expr 78 [137-141] [Type Unit]: Call:
                                    Expr 79 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 80 [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 81 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 82 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 83 [139-140] [Type (Qubit)[]]: Var: Local 67
                                        Expr 84 [139-140] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_ctladj_auto_to_distribute() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Ctl + Adj {}
                operation A(q : Qubit) : Unit is Ctl + Adj {
                    body ... { B(1); B(2); }
                    controlled adjoint auto;
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-191] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68]: Impl:
                            Block 5 [66-68]: <empty>
                        adj: SpecDecl 27 [21-68]: Impl:
                            Block 28 [66-68]: <empty>
                        ctl: SpecDecl 23 [21-68]: Impl:
                            Pat 24 [21-68] [Type (Qubit)[]]: Bind: Ident 25 [21-68] "ctls"
                            Block 26 [66-68]: <empty>
                        ctl-adj: SpecDecl 30 [21-68]: Impl:
                            Pat 31 [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                            Block 33 [66-68]: <empty>
                Item 2 [73-189] (Public):
                    Parent: 0
                    Callable 6 [73-189] (operation):
                        name: Ident 7 [83-84] "A"
                        input: Pat 8 [85-94] [Type Qubit]: Bind: Ident 9 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 10 [126-150]: Impl:
                            Block 11 [135-150] [Type Unit]:
                                Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                    Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                    Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 19 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 53 [73-189]: Impl:
                            Block 54 [135-150] [Type Unit]:
                                Stmt 55 [143-148]: Semi: Expr 56 [143-147] [Type Unit]: Call:
                                    Expr 57 [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 58 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 59 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 60 [137-142]: Semi: Expr 61 [137-141] [Type Unit]: Call:
                                    Expr 62 [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 63 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 64 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl 35 [73-189]: Impl:
                            Pat 36 [73-189] [Type (Qubit)[]]: Bind: Ident 37 [73-189] "ctls"
                            Block 38 [135-150] [Type Unit]:
                                Stmt 39 [137-142]: Semi: Expr 40 [137-141] [Type Unit]: Call:
                                    Expr 41 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 42 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 43 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 44 [139-140] [Type (Qubit)[]]: Var: Local 37
                                        Expr 45 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 46 [143-148]: Semi: Expr 47 [143-147] [Type Unit]: Call:
                                    Expr 48 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 49 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 50 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 51 [145-146] [Type (Qubit)[]]: Var: Local 37
                                        Expr 52 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl 66 [73-189]: Impl:
                            Pat 67 [73-189] [Type (Qubit)[]]: Bind: Ident 68 [73-189] "ctls"
                            Block 69 [135-150] [Type Unit]:
                                Stmt 70 [143-148]: Semi: Expr 71 [143-147] [Type Unit]: Call:
                                    Expr 72 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 73 [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 74 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 75 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 76 [145-146] [Type (Qubit)[]]: Var: Local 68
                                        Expr 77 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 78 [137-142]: Semi: Expr 79 [137-141] [Type Unit]: Call:
                                    Expr 80 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 81 [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 82 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 83 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 84 [139-140] [Type (Qubit)[]]: Var: Local 68
                                        Expr 85 [139-140] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn generate_ctladj_auto_to_invert() {
    check(
        indoc! {r#"
            namespace test {
                operation B(input : Int) : Unit is Ctl + Adj {}
                operation A(q : Qubit) : Unit is Ctl + Adj {
                    body ... { B(1); B(2); }
                    controlled (ctls, ...) { Controlled B(ctls, 1); Controlled B(ctls, 2); }
                    controlled adjoint auto;
                }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-272] (Public):
                    Namespace (Ident 39 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68]: Impl:
                            Block 5 [66-68]: <empty>
                        adj: SpecDecl 45 [21-68]: Impl:
                            Block 46 [66-68]: <empty>
                        ctl: SpecDecl 41 [21-68]: Impl:
                            Pat 42 [21-68] [Type (Qubit)[]]: Bind: Ident 43 [21-68] "ctls"
                            Block 44 [66-68]: <empty>
                        ctl-adj: SpecDecl 48 [21-68]: Impl:
                            Pat 49 [21-68] [Type (Qubit)[]]: Bind: Ident 50 [21-68] "ctls"
                            Block 51 [66-68]: <empty>
                Item 2 [73-270] (Public):
                    Parent: 0
                    Callable 6 [73-270] (operation):
                        name: Ident 7 [83-84] "A"
                        input: Pat 8 [85-94] [Type Qubit]: Bind: Ident 9 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 10 [126-150]: Impl:
                            Block 11 [135-150] [Type Unit]:
                                Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                    Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                    Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 19 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 52 [73-270]: Impl:
                            Block 53 [135-150] [Type Unit]:
                                Stmt 54 [143-148]: Semi: Expr 55 [143-147] [Type Unit]: Call:
                                    Expr 56 [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 57 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 58 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 59 [137-142]: Semi: Expr 60 [137-141] [Type Unit]: Call:
                                    Expr 61 [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 62 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 63 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl 20 [159-231]: Impl:
                            Pat 21 [171-175] [Type (Qubit)[]]: Bind: Ident 22 [171-175] "ctls"
                            Block 23 [182-231] [Type Unit]:
                                Stmt 24 [184-206]: Semi: Expr 25 [184-205] [Type Unit]: Call:
                                    Expr 26 [184-196] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 27 [195-196] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 28 [196-205] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 29 [197-201] [Type (Qubit)[]]: Var: Local 22
                                        Expr 30 [203-204] [Type Int]: Lit: Int(1)
                                Stmt 31 [207-229]: Semi: Expr 32 [207-228] [Type Unit]: Call:
                                    Expr 33 [207-219] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 34 [218-219] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 35 [219-228] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 36 [220-224] [Type (Qubit)[]]: Var: Local 22
                                        Expr 37 [226-227] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl 64 [73-270]: Impl:
                            Pat 65 [171-175] [Type (Qubit)[]]: Bind: Ident 66 [171-175] "ctls"
                            Block 67 [182-231] [Type Unit]:
                                Stmt 68 [207-229]: Semi: Expr 69 [207-228] [Type Unit]: Call:
                                    Expr 70 [207-219] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 71 [207-219] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 72 [218-219] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 73 [219-228] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 74 [220-224] [Type (Qubit)[]]: Var: Local 66
                                        Expr 75 [226-227] [Type Int]: Lit: Int(2)
                                Stmt 76 [184-206]: Semi: Expr 77 [184-205] [Type Unit]: Call:
                                    Expr 78 [184-196] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 79 [184-196] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 80 [195-196] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 81 [196-205] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 82 [197-201] [Type (Qubit)[]]: Var: Local 66
                                        Expr 83 [203-204] [Type Int]: Lit: Int(1)"#]],
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
                Item 0 [0-193] (Public):
                    Namespace (Ident 21 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68]: Impl:
                            Block 5 [66-68]: <empty>
                        adj: SpecDecl 27 [21-68]: Impl:
                            Block 28 [66-68]: <empty>
                        ctl: SpecDecl 23 [21-68]: Impl:
                            Pat 24 [21-68] [Type (Qubit)[]]: Bind: Ident 25 [21-68] "ctls"
                            Block 26 [66-68]: <empty>
                        ctl-adj: SpecDecl 30 [21-68]: Impl:
                            Pat 31 [21-68] [Type (Qubit)[]]: Bind: Ident 32 [21-68] "ctls"
                            Block 33 [66-68]: <empty>
                Item 2 [73-191] (Public):
                    Parent: 0
                    Callable 6 [73-191] (operation):
                        name: Ident 7 [83-84] "A"
                        input: Pat 8 [85-94] [Type Qubit]: Bind: Ident 9 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 10 [126-150]: Impl:
                            Block 11 [135-150] [Type Unit]:
                                Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                    Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                    Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 19 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 53 [73-191]: Impl:
                            Block 54 [135-150] [Type Unit]:
                                Stmt 55 [143-148]: Semi: Expr 56 [143-147] [Type Unit]: Call:
                                    Expr 57 [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 58 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 59 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 60 [137-142]: Semi: Expr 61 [137-141] [Type Unit]: Call:
                                    Expr 62 [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 63 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 64 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl 35 [73-191]: Impl:
                            Pat 36 [73-191] [Type (Qubit)[]]: Bind: Ident 37 [73-191] "ctls"
                            Block 38 [135-150] [Type Unit]:
                                Stmt 39 [137-142]: Semi: Expr 40 [137-141] [Type Unit]: Call:
                                    Expr 41 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 42 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 43 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 44 [139-140] [Type (Qubit)[]]: Var: Local 37
                                        Expr 45 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 46 [143-148]: Semi: Expr 47 [143-147] [Type Unit]: Call:
                                    Expr 48 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 49 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 50 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 51 [145-146] [Type (Qubit)[]]: Var: Local 37
                                        Expr 52 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl 65 [159-185]: Impl:
                            Pat 66 [73-191] [Type (Qubit)[]]: Bind: Ident 67 [73-191] "ctls"
                            Block 68 [135-150] [Type Unit]:
                                Stmt 69 [143-148]: Semi: Expr 70 [143-147] [Type Unit]: Call:
                                    Expr 71 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 72 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 73 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 74 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 75 [145-146] [Type (Qubit)[]]: Var: Local 67
                                        Expr 76 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 77 [137-142]: Semi: Expr 78 [137-141] [Type Unit]: Call:
                                    Expr 79 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 80 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 81 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 82 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 83 [139-140] [Type (Qubit)[]]: Var: Local 67
                                        Expr 84 [139-140] [Type Int]: Lit: Int(1)"#]],
    );
}

#[test]
fn lambda_adj_calls_adj() {
    check(
        indoc! {r#"
            namespace A {
                operation X(q : Qubit) : () is Adj {}
                operation Foo(op : Qubit => () is Adj) : () {}
                operation Bar() : () { Foo(q => X(q)); }
            }
        "#},
        &expect![[r#"
            Package:
                Item 0 [0-153] (Public):
                    Namespace (Ident 32 [10-11] "A"): Item 1, Item 2, Item 3
                Item 1 [18-55] (Public):
                    Parent: 0
                    Callable 0 [18-55] (operation):
                        name: Ident 1 [28-29] "X"
                        input: Pat 2 [30-39] [Type Qubit]: Bind: Ident 3 [30-31] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [18-55]: Impl:
                            Block 5 [53-55]: <empty>
                        adj: SpecDecl 33 [18-55]: Impl:
                            Block 34 [53-55]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [60-106] (Public):
                    Parent: 0
                    Callable 6 [60-106] (operation):
                        name: Ident 7 [70-73] "Foo"
                        generics:
                            0: functor (Adj)
                        input: Pat 8 [74-97] [Type (Qubit => Unit is 0)]: Bind: Ident 9 [74-76] "op"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 10 [60-106]: Impl:
                            Block 11 [104-106]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [111-151] (Public):
                    Parent: 0
                    Callable 12 [111-151] (operation):
                        name: Ident 13 [121-124] "Bar"
                        input: Pat 14 [124-126] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 15 [111-151]: Impl:
                            Block 16 [132-151] [Type Unit]:
                                Stmt 17 [134-149]: Semi: Expr 18 [134-148] [Type Unit]: Call:
                                    Expr 19 [134-137] [Type ((Qubit => Unit is Adj) => Unit)]: Var:
                                        res: Item 2
                                        generics:
                                            Adj
                                    Expr 20 [138-147] [Type (Qubit => Unit is Adj)]: Closure([], 4)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 4 [138-147] (Internal):
                    Parent: 3
                    Callable 27 [138-147] (operation):
                        name: Ident 28 [138-147] "lambda"
                        input: Pat 26 [138-147] [Type (Qubit,)]: Tuple:
                            Pat 21 [138-139] [Type Qubit]: Bind: Ident 22 [138-139] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 29 [143-147]: Impl:
                            Block 30 [143-147] [Type Unit]:
                                Stmt 31 [143-147]: Expr: Expr 23 [143-147] [Type Unit]: Call:
                                    Expr 24 [143-144] [Type (Qubit => Unit is Adj)]: Var: Item 1
                                    Expr 25 [145-146] [Type Qubit]: Var: Local 22
                        adj: SpecDecl 35 [138-147]: Impl:
                            Block 36 [143-147] [Type Unit]:
                                Stmt 37 [143-147]: Expr: Expr 38 [143-147] [Type Unit]: Call:
                                    Expr 39 [143-144] [Type (Qubit => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 40 [143-144] [Type (Qubit => Unit is Adj)]: Var: Item 1
                                    Expr 41 [145-146] [Type Qubit]: Var: Local 22
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_adj_calls_non_adj() {
    check(
        indoc! {r#"
            namespace A {
                operation M(q : Qubit) : Result { Zero }
                operation Foo(op : Qubit => () is Adj) : () {}
                operation Bar() : () { Foo(q => { M(q); }); }
            }
        "#},
        &expect![[r#"
            [
                AdjGen(
                    MissingAdjFunctor(
                        Span {
                            lo: 148,
                            hi: 149,
                        },
                    ),
                ),
            ]
        "#]],
    );
}

#[test]
fn op_array_forget_functors_with_lambdas() {
    check(
        "
            namespace A {
                operation Foo(q : Qubit) : () {}
                operation Bar(q : Qubit) : () is Adj {}
                operation Baz(q : Qubit) : () is Adj + Ctl {}
                operation Main() : () {
                    let ops = [Foo, q => Bar(q), q => Baz(q)];
                }
            }
        ",
        &expect![[r#"
            Package:
                Item 0 [13-328] (Public):
                    Namespace (Ident 52 [23-24] "A"): Item 1, Item 2, Item 3, Item 4
                Item 1 [43-75] (Public):
                    Parent: 0
                    Callable 0 [43-75] (operation):
                        name: Ident 1 [53-56] "Foo"
                        input: Pat 2 [57-66] [Type Qubit]: Bind: Ident 3 [57-58] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 4 [43-75]: Impl:
                            Block 5 [73-75]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [92-131] (Public):
                    Parent: 0
                    Callable 6 [92-131] (operation):
                        name: Ident 7 [102-105] "Bar"
                        input: Pat 8 [106-115] [Type Qubit]: Bind: Ident 9 [106-107] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 10 [92-131]: Impl:
                            Block 11 [129-131]: <empty>
                        adj: SpecDecl 53 [92-131]: Impl:
                            Block 54 [129-131]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [148-193] (Public):
                    Parent: 0
                    Callable 12 [148-193] (operation):
                        name: Ident 13 [158-161] "Baz"
                        input: Pat 14 [162-171] [Type Qubit]: Bind: Ident 15 [162-163] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 16 [148-193]: Impl:
                            Block 17 [191-193]: <empty>
                        adj: SpecDecl 60 [148-193]: Impl:
                            Block 61 [191-193]: <empty>
                        ctl: SpecDecl 56 [148-193]: Impl:
                            Pat 57 [148-193] [Type (Qubit)[]]: Bind: Ident 58 [148-193] "ctls"
                            Block 59 [191-193]: <empty>
                        ctl-adj: SpecDecl 63 [148-193]: Impl:
                            Pat 64 [148-193] [Type (Qubit)[]]: Bind: Ident 65 [148-193] "ctls"
                            Block 66 [191-193]: <empty>
                Item 4 [210-314] (Public):
                    Parent: 0
                    Callable 18 [210-314] (operation):
                        name: Ident 19 [220-224] "Main"
                        input: Pat 20 [224-226] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 21 [210-314]: Impl:
                            Block 22 [232-314] [Type Unit]:
                                Stmt 23 [254-296]: Local (Immutable):
                                    Pat 24 [258-261] [Type ((Qubit => Unit))[]]: Bind: Ident 25 [258-261] "ops"
                                    Expr 26 [264-295] [Type ((Qubit => Unit))[]]: Array:
                                        Expr 27 [265-268] [Type (Qubit => Unit)]: Var: Item 1
                                        Expr 28 [270-281] [Type (Qubit => Unit)]: Closure([], 5)
                                        Expr 40 [283-294] [Type (Qubit => Unit)]: Closure([], 6)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 5 [270-281] (Internal):
                    Parent: 4
                    Callable 35 [270-281] (operation):
                        name: Ident 36 [270-281] "lambda"
                        input: Pat 34 [270-281] [Type (Qubit,)]: Tuple:
                            Pat 29 [270-271] [Type Qubit]: Bind: Ident 30 [270-271] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 37 [275-281]: Impl:
                            Block 38 [275-281] [Type Unit]:
                                Stmt 39 [275-281]: Expr: Expr 31 [275-281] [Type Unit]: Call:
                                    Expr 32 [275-278] [Type (Qubit => Unit is Adj)]: Var: Item 2
                                    Expr 33 [279-280] [Type Qubit]: Var: Local 30
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 6 [283-294] (Internal):
                    Parent: 4
                    Callable 47 [283-294] (operation):
                        name: Ident 48 [283-294] "lambda"
                        input: Pat 46 [283-294] [Type (Qubit,)]: Tuple:
                            Pat 41 [283-284] [Type Qubit]: Bind: Ident 42 [283-284] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 49 [288-294]: Impl:
                            Block 50 [288-294] [Type Unit]:
                                Stmt 51 [288-294]: Expr: Expr 43 [288-294] [Type Unit]: Call:
                                    Expr 44 [288-291] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 3
                                    Expr 45 [292-293] [Type Qubit]: Var: Local 42
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn op_array_unsupported_functors_with_lambdas() {
    check(
        "
            namespace A {
                operation Foo(q : Qubit) : () {}
                operation Bar(q : Qubit) : () is Adj {}
                operation Baz(q : Qubit) : () is Adj + Ctl {}
                operation Main() : () {
                    let ops = [q => Foo(q), q => Bar(q), Baz];
                }
            }
        ",
        &expect![[r#"
            [
                CtlGen(
                    MissingCtlFunctor(
                        Span {
                            lo: 270,
                            hi: 273,
                        },
                    ),
                ),
                AdjGen(
                    MissingAdjFunctor(
                        Span {
                            lo: 270,
                            hi: 273,
                        },
                    ),
                ),
                CtlGen(
                    MissingCtlFunctor(
                        Span {
                            lo: 270,
                            hi: 273,
                        },
                    ),
                ),
                CtlGen(
                    MissingCtlFunctor(
                        Span {
                            lo: 283,
                            hi: 286,
                        },
                    ),
                ),
                CtlGen(
                    MissingCtlFunctor(
                        Span {
                            lo: 283,
                            hi: 286,
                        },
                    ),
                ),
            ]
        "#]],
    );
}
