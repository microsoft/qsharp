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
                    Namespace (Ident 24 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 4 [68-79] (Body): Impl:
                            Pat 5 [73-76] [Type Qubit]: Elided
                            Block 6 [77-79]: <empty>
                        adj: <none>
                        ctl: SpecDecl 7 [88-113] (Ctl): Impl:
                            Pat 8 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat 9 [100-104] [Type (Qubit)[]]: Bind: Ident 10 [100-104] "ctls"
                                Pat 11 [106-109] [Type Qubit]: Elided
                            Block 12 [111-113]: <empty>
                        ctl-adj: <none>
                Item 2 [124-182] (Public):
                    Parent: 0
                    Callable 13 [124-182] (Operation):
                        name: Ident 14 [134-135] "B"
                        input: Pat 15 [136-145] [Type Qubit]: Bind: Ident 16 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 17 [124-182] (Body): Impl:
                            Pat 18 [124-182] [Type Qubit]: Elided
                            Block 19 [161-182] [Type Unit]:
                                Stmt 20 [171-176]: Semi: Expr 21 [171-175] [Type Unit]: Call:
                                    Expr 22 [171-172] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 23 [173-174] [Type Qubit]: Var: Local 16
                        adj: <none>
                        ctl: SpecDecl _id_ [124-182] (Ctl): Impl:
                            Pat _id_ [124-182] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [124-182] [Type (Qubit)[]]: Bind: Ident 25 [124-182] "ctls"
                                Pat _id_ [124-182] [Type Qubit]: Elided
                            Block 19 [161-182] [Type Unit]:
                                Stmt 20 [171-176]: Semi: Expr 21 [171-175] [Type Unit]: Call:
                                    Expr 22 [171-172] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                        Expr 22 [171-172] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 23 [173-174] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr _id_ [173-174] [Type (Qubit)[]]: Var: Local 25
                                        Expr 23 [173-174] [Type Qubit]: Var: Local 16
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
                    Namespace (Ident 25 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 4 [68-79] (Body): Impl:
                            Pat 5 [73-76] [Type Qubit]: Elided
                            Block 6 [77-79]: <empty>
                        adj: <none>
                        ctl: SpecDecl 7 [88-113] (Ctl): Impl:
                            Pat 8 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat 9 [100-104] [Type (Qubit)[]]: Bind: Ident 10 [100-104] "ctls"
                                Pat 11 [106-109] [Type Qubit]: Elided
                            Block 12 [111-113]: <empty>
                        ctl-adj: <none>
                Item 2 [124-240] (Public):
                    Parent: 0
                    Callable 13 [124-240] (Operation):
                        name: Ident 14 [134-135] "B"
                        input: Pat 15 [136-145] [Type Qubit]: Bind: Ident 16 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 17 [171-209] (Body): Impl:
                            Pat 18 [176-179] [Type Qubit]: Elided
                            Block 19 [180-209] [Type Unit]:
                                Stmt 20 [194-199]: Semi: Expr 21 [194-198] [Type Unit]: Call:
                                    Expr 22 [194-195] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 23 [196-197] [Type Qubit]: Var: Local 16
                        adj: <none>
                        ctl: SpecDecl 24 [218-234] (Ctl): Impl:
                            Pat _id_ [218-234] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [218-234] [Type (Qubit)[]]: Bind: Ident 26 [218-234] "ctls"
                                Pat _id_ [218-234] [Type Qubit]: Elided
                            Block 19 [180-209] [Type Unit]:
                                Stmt 20 [194-199]: Semi: Expr 21 [194-198] [Type Unit]: Call:
                                    Expr 22 [194-195] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                        Expr 22 [194-195] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                    Expr 23 [196-197] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr _id_ [196-197] [Type (Qubit)[]]: Var: Local 26
                                        Expr 23 [196-197] [Type Qubit]: Var: Local 16
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
                    Namespace (Ident 35 [10-14] "test"): Item 1, Item 2
                Item 1 [21-148] (Public):
                    Parent: 0
                    Callable 0 [21-148] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [74-85] (Body): Impl:
                            Pat 5 [79-82] [Type Qubit]: Elided
                            Block 6 [83-85]: <empty>
                        adj: SpecDecl 7 [94-108] (Adj): Impl:
                            Pat 8 [102-105] [Type Qubit]: Elided
                            Block 9 [106-108]: <empty>
                        ctl: SpecDecl 10 [117-142] (Ctl): Impl:
                            Pat 11 [128-139] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat 12 [129-133] [Type (Qubit)[]]: Bind: Ident 13 [129-133] "ctls"
                                Pat 14 [135-138] [Type Qubit]: Elided
                            Block 15 [140-142]: <empty>
                        ctl-adj: SpecDecl _id_ [21-148] (CtlAdj): Impl:
                            Pat _id_ [21-148] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [21-148] [Type (Qubit)[]]: Bind: Ident 36 [21-148] "ctls"
                                Pat _id_ [21-148] [Type Qubit]: Elided
                            Block 9 [106-108]: <empty>
                Item 2 [153-308] (Public):
                    Parent: 0
                    Callable 16 [153-308] (Operation):
                        name: Ident 17 [163-164] "B"
                        input: Pat 18 [165-174] [Type Qubit]: Bind: Ident 19 [165-166] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 20 [206-244] (Body): Impl:
                            Pat 21 [211-214] [Type Qubit]: Elided
                            Block 22 [215-244] [Type Unit]:
                                Stmt 23 [229-234]: Semi: Expr 24 [229-233] [Type Unit]: Call:
                                    Expr 25 [229-230] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 26 [231-232] [Type Qubit]: Var: Local 19
                        adj: SpecDecl 27 [253-302] (Adj): Impl:
                            Pat 28 [261-264] [Type Qubit]: Elided
                            Block 29 [265-302] [Type Unit]:
                                Stmt 30 [279-292]: Semi: Expr 31 [279-291] [Type Unit]: Call:
                                    Expr 32 [279-288] [Type (Qubit => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 33 [287-288] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 34 [289-290] [Type Qubit]: Var: Local 19
                        ctl: SpecDecl _id_ [153-308] (Ctl): Impl:
                            Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 37 [153-308] "ctls"
                                Pat _id_ [153-308] [Type Qubit]: Elided
                            Block 22 [215-244] [Type Unit]:
                                Stmt 23 [229-234]: Semi: Expr 24 [229-233] [Type Unit]: Call:
                                    Expr 25 [229-230] [Type (((Qubit)[], Qubit) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 25 [229-230] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 26 [231-232] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr _id_ [231-232] [Type (Qubit)[]]: Var: Local 37
                                        Expr 26 [231-232] [Type Qubit]: Var: Local 19
                        ctl-adj: SpecDecl _id_ [153-308] (CtlAdj): Impl:
                            Pat _id_ [153-308] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [153-308] [Type (Qubit)[]]: Bind: Ident 38 [153-308] "ctls"
                                Pat _id_ [153-308] [Type Qubit]: Elided
                            Block 29 [265-302] [Type Unit]:
                                Stmt 30 [279-292]: Semi: Expr 31 [279-291] [Type Unit]: Call:
                                    Expr 32 [279-288] [Type (((Qubit)[], Qubit) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 32 [279-288] [Type (Qubit => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 33 [287-288] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 34 [289-290] [Type ((Qubit)[], Qubit)]: Tuple:
                                        Expr _id_ [289-290] [Type (Qubit)[]]: Var: Local 38
                                        Expr 34 [289-290] [Type Qubit]: Var: Local 19"#]],
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
                    Namespace (Ident 32 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 4 [68-79] (Body): Impl:
                            Pat 5 [73-76] [Type Qubit]: Elided
                            Block 6 [77-79]: <empty>
                        adj: <none>
                        ctl: SpecDecl 7 [88-113] (Ctl): Impl:
                            Pat 8 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat 9 [100-104] [Type (Qubit)[]]: Bind: Ident 10 [100-104] "ctls"
                                Pat 11 [106-109] [Type Qubit]: Elided
                            Block 12 [111-113]: <empty>
                        ctl-adj: <none>
                Item 2 [124-257] (Public):
                    Parent: 0
                    Callable 13 [124-257] (Operation):
                        name: Ident 14 [134-135] "B"
                        input: Pat 15 [136-145] [Type Qubit]: Bind: Ident 16 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 17 [124-257] (Body): Impl:
                            Pat 18 [124-257] [Type Qubit]: Elided
                            Block 19 [161-257] [Type Unit]:
                                Stmt 20 [171-251]: Expr: Expr 21 [171-251] [Type Unit]: Conjugate:
                                    Block 22 [178-207] [Type Unit]:
                                        Stmt 23 [192-197]: Semi: Expr 24 [192-196] [Type Unit]: Call:
                                            Expr 25 [192-193] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 26 [194-195] [Type Qubit]: Var: Local 16
                                    Block 27 [222-251] [Type Unit]:
                                        Stmt 28 [236-241]: Semi: Expr 29 [236-240] [Type Unit]: Call:
                                            Expr 30 [236-237] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 31 [238-239] [Type Qubit]: Var: Local 16
                        adj: <none>
                        ctl: SpecDecl _id_ [124-257] (Ctl): Impl:
                            Pat _id_ [124-257] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [124-257] [Type (Qubit)[]]: Bind: Ident 33 [124-257] "ctls"
                                Pat _id_ [124-257] [Type Qubit]: Elided
                            Block 19 [161-257] [Type Unit]:
                                Stmt 20 [171-251]: Expr: Expr 21 [171-251] [Type Unit]: Conjugate:
                                    Block 22 [178-207] [Type Unit]:
                                        Stmt 23 [192-197]: Semi: Expr 24 [192-196] [Type Unit]: Call:
                                            Expr 25 [192-193] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 26 [194-195] [Type Qubit]: Var: Local 16
                                    Block 27 [222-251] [Type Unit]:
                                        Stmt 28 [236-241]: Semi: Expr 29 [236-240] [Type Unit]: Call:
                                            Expr 30 [236-237] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                                Expr 30 [236-237] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                            Expr 31 [238-239] [Type ((Qubit)[], Qubit)]: Tuple:
                                                Expr _id_ [238-239] [Type (Qubit)[]]: Var: Local 33
                                                Expr 31 [238-239] [Type Qubit]: Var: Local 16
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
                    Namespace (Ident 26 [10-14] "test"): Item 1, Item 2, Item 3
                Item 1 [21-45] (Public):
                    Parent: 0
                    Callable 0 [21-45] (Function):
                        name: Ident 1 [30-33] "Foo"
                        input: Pat 2 [33-35] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [21-45] (Body): Impl:
                            Pat 4 [21-45] [Type Unit]: Elided
                            Block 5 [43-45]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [50-80] (Public):
                    Parent: 0
                    Callable 6 [50-80] (Operation):
                        name: Ident 7 [60-61] "A"
                        input: Pat 8 [61-63] [Type Unit]: Unit
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 9 [50-80] (Body): Impl:
                            Pat 10 [50-80] [Type Unit]: Elided
                            Block 11 [78-80]: <empty>
                        adj: <none>
                        ctl: SpecDecl _id_ [50-80] (Ctl): Impl:
                            Pat _id_ [50-80] [Type ((Qubit)[], Unit)]: Tuple:
                                Pat _id_ [50-80] [Type (Qubit)[]]: Bind: Ident 27 [50-80] "ctls"
                                Pat _id_ [50-80] [Type Unit]: Elided
                            Block 11 [78-80]: <empty>
                        ctl-adj: <none>
                Item 3 [85-148] (Public):
                    Parent: 0
                    Callable 12 [85-148] (Operation):
                        name: Ident 13 [95-96] "B"
                        input: Pat 14 [96-98] [Type Unit]: Unit
                        output: Unit
                        functors: Ctl
                        body: SpecDecl 15 [85-148] (Body): Impl:
                            Pat 16 [85-148] [Type Unit]: Elided
                            Block 17 [113-148] [Type Unit]:
                                Stmt 18 [123-129]: Semi: Expr 19 [123-128] [Type Unit]: Call:
                                    Expr 20 [123-126] [Type (Unit -> Unit)]: Var: Item 1
                                    Expr 21 [126-128] [Type Unit]: Unit
                                Stmt 22 [138-142]: Semi: Expr 23 [138-141] [Type Unit]: Call:
                                    Expr 24 [138-139] [Type (Unit => Unit is Ctl)]: Var: Item 2
                                    Expr 25 [139-141] [Type Unit]: Unit
                        adj: <none>
                        ctl: SpecDecl _id_ [85-148] (Ctl): Impl:
                            Pat _id_ [85-148] [Type ((Qubit)[], Unit)]: Tuple:
                                Pat _id_ [85-148] [Type (Qubit)[]]: Bind: Ident 28 [85-148] "ctls"
                                Pat _id_ [85-148] [Type Unit]: Elided
                            Block 17 [113-148] [Type Unit]:
                                Stmt 18 [123-129]: Semi: Expr 19 [123-128] [Type Unit]: Call:
                                    Expr 20 [123-126] [Type (Unit -> Unit)]: Var: Item 1
                                    Expr 21 [126-128] [Type Unit]: Unit
                                Stmt 22 [138-142]: Semi: Expr 23 [138-141] [Type Unit]: Call:
                                    Expr 24 [138-139] [Type (((Qubit)[], Unit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                        Expr 24 [138-139] [Type (Unit => Unit is Ctl)]: Var: Item 2
                                    Expr 25 [139-141] [Type ((Qubit)[], Unit)]: Tuple:
                                        Expr _id_ [139-141] [Type (Qubit)[]]: Var: Local 28
                                        Expr 25 [139-141] [Type Unit]: Unit
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
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-166] (Public):
                    Parent: 0
                    Callable 7 [67-166] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [114-138] (Body): Impl:
                            Pat 12 [119-122] [Type Qubit]: Elided
                            Block 13 [123-138] [Type Unit]:
                                Stmt 14 [125-130]: Semi: Expr 15 [125-129] [Type Unit]: Call:
                                    Expr 16 [125-126] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [127-128] [Type Int]: Lit: Int(1)
                                Stmt 18 [131-136]: Semi: Expr 19 [131-135] [Type Unit]: Call:
                                    Expr 20 [131-132] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 21 [133-134] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 22 [147-160] (Adj): Impl:
                            Pat 12 [119-122] [Type Qubit]: Elided
                            Block 13 [123-138] [Type Unit]:
                                Stmt 14 [125-130]: Semi: Expr 15 [125-129] [Type Unit]: Call:
                                    Expr 16 [125-126] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [127-128] [Type Int]: Lit: Int(1)
                                Stmt 18 [131-136]: Semi: Expr 19 [131-135] [Type Unit]: Call:
                                    Expr 20 [131-132] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 21 [133-134] [Type Int]: Lit: Int(2)
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
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68] (Body): Impl:
                            Pat 5 [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        adj: SpecDecl _id_ [21-68] (Adj): Impl:
                            Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl: SpecDecl _id_ [21-68] (Ctl): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 24 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl-adj: SpecDecl _id_ [21-68] (CtlAdj): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 25 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                Item 2 [73-178] (Public):
                    Parent: 0
                    Callable 7 [73-178] (Operation):
                        name: Ident 8 [83-84] "A"
                        input: Pat 9 [85-94] [Type Qubit]: Bind: Ident 10 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 11 [126-150] (Body): Impl:
                            Pat 12 [131-134] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 22 [159-172] (Adj): Impl:
                            Pat 12 [131-134] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        ctl: SpecDecl _id_ [73-178] (Ctl): Impl:
                            Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 26 [73-178] "ctls"
                                Pat _id_ [73-178] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 26
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 26
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl _id_ [73-178] (CtlAdj): Impl:
                            Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 26 [73-178] "ctls"
                                Pat _id_ [73-178] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 26
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 26
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)"#]],
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
                    Namespace (Ident 22 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-139] (Public):
                    Parent: 0
                    Callable 7 [67-139] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [67-139] (Body): Impl:
                            Pat 12 [67-139] [Type Qubit]: Elided
                            Block 13 [104-139] [Type Unit]:
                                Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type Unit]: Call:
                                    Expr 16 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [116-117] [Type Int]: Lit: Int(1)
                                Stmt 18 [128-133]: Semi: Expr 19 [128-132] [Type Unit]: Call:
                                    Expr 20 [128-129] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 21 [130-131] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [67-139] (Adj): Impl:
                            Pat _id_ [67-139] [Type Qubit]: Elided
                            Block 13 [104-139] [Type Unit]:
                                Stmt 18 [128-133]: Semi: Expr 19 [128-132] [Type Unit]: Call:
                                    Expr _id_ [128-129] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 20 [128-129] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 21 [130-131] [Type Int]: Lit: Int(2)
                                Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type Unit]: Call:
                                    Expr _id_ [114-115] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 16 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [116-117] [Type Int]: Lit: Int(1)
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
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-198] (Public):
                    Parent: 0
                    Callable 7 [67-198] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [114-170] (Body): Impl:
                            Pat 12 [119-122] [Type Qubit]: Elided
                            Block 13 [123-170] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [155-160]: Semi: Expr 19 [155-159] [Type Unit]: Call:
                                    Expr 20 [155-156] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 21 [157-158] [Type Int]: Lit: Int(2)
                        adj: SpecDecl 22 [179-192] (Adj): Impl:
                            Pat _id_ [179-192] [Type Qubit]: Elided
                            Block 13 [123-170] [Type Unit]:
                                Stmt 18 [155-160]: Semi: Expr 19 [155-159] [Type Unit]: Call:
                                    Expr _id_ [155-156] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 20 [155-156] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 21 [157-158] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
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
                    Namespace (Ident 34 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-236] (Public):
                    Parent: 0
                    Callable 7 [67-236] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [67-236] (Body): Impl:
                            Pat 12 [67-236] [Type Qubit]: Elided
                            Block 13 [104-236] [Type Unit]:
                                Stmt 14 [114-230]: Expr: Expr 15 [114-230] [Type Unit]: Conjugate:
                                    Block 16 [121-168] [Type Unit]:
                                        Stmt 17 [135-140]: Semi: Expr 18 [135-139] [Type Unit]: Call:
                                            Expr 19 [135-136] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 20 [137-138] [Type Int]: Lit: Int(1)
                                        Stmt 21 [153-158]: Semi: Expr 22 [153-157] [Type Unit]: Call:
                                            Expr 23 [153-154] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 24 [155-156] [Type Int]: Lit: Int(2)
                                    Block 25 [183-230] [Type Unit]:
                                        Stmt 26 [197-202]: Semi: Expr 27 [197-201] [Type Unit]: Call:
                                            Expr 28 [197-198] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 29 [199-200] [Type Int]: Lit: Int(3)
                                        Stmt 30 [215-220]: Semi: Expr 31 [215-219] [Type Unit]: Call:
                                            Expr 32 [215-216] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 33 [217-218] [Type Int]: Lit: Int(4)
                        adj: SpecDecl _id_ [67-236] (Adj): Impl:
                            Pat _id_ [67-236] [Type Qubit]: Elided
                            Block 13 [104-236] [Type Unit]:
                                Stmt 14 [114-230]: Expr: Expr 15 [114-230] [Type Unit]: Conjugate:
                                    Block 16 [121-168] [Type Unit]:
                                        Stmt 17 [135-140]: Semi: Expr 18 [135-139] [Type Unit]: Call:
                                            Expr 19 [135-136] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 20 [137-138] [Type Int]: Lit: Int(1)
                                        Stmt 21 [153-158]: Semi: Expr 22 [153-157] [Type Unit]: Call:
                                            Expr 23 [153-154] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 24 [155-156] [Type Int]: Lit: Int(2)
                                    Block 25 [183-230] [Type Unit]:
                                        Stmt 30 [215-220]: Semi: Expr 31 [215-219] [Type Unit]: Call:
                                            Expr _id_ [215-216] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 32 [215-216] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 33 [217-218] [Type Int]: Lit: Int(4)
                                        Stmt 26 [197-202]: Semi: Expr 27 [197-201] [Type Unit]: Call:
                                            Expr _id_ [197-198] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 28 [197-198] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 29 [199-200] [Type Int]: Lit: Int(3)
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
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-266] (Public):
                    Parent: 0
                    Callable 7 [67-266] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [67-266] (Body): Impl:
                            Pat 12 [67-266] [Type Qubit]: Elided
                            Block 13 [104-266] [Type Unit]:
                                Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type Unit]: Call:
                                    Expr 16 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [116-117] [Type Int]: Lit: Int(1)
                                Stmt 18 [128-166]: Local (Immutable):
                                    Pat 19 [132-135] [Type Bool]: Bind: Ident 20 [132-135] "val"
                                    Expr 21 [138-165] [Type Bool]: If:
                                        Expr 22 [141-145] [Type Bool]: Lit: Bool(true)
                                        Block 23 [146-153] [Type Bool]:
                                            Stmt 24 [147-152]: Expr: Expr 25 [147-152] [Type Bool]: Lit: Bool(false)
                                        Expr 26 [154-165] [Type Bool]: Expr Block: Block 27 [159-165] [Type Bool]:
                                            Stmt 28 [160-164]: Expr: Expr 29 [160-164] [Type Bool]: Lit: Bool(true)
                                Stmt 30 [175-180]: Semi: Expr 31 [175-179] [Type Unit]: Call:
                                    Expr 32 [175-176] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 33 [177-178] [Type Int]: Lit: Int(2)
                                Stmt 34 [189-246]: Expr: Expr 35 [189-246] [Type Unit]: If:
                                    Expr 36 [192-197] [Type Bool]: Lit: Bool(false)
                                    Block 37 [198-211] [Type Unit]:
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
                        adj: SpecDecl _id_ [67-266] (Adj): Impl:
                            Pat _id_ [67-266] [Type Qubit]: Elided
                            Block 13 [104-266] [Type Unit]:
                                Stmt 18 [128-166]: Local (Immutable):
                                    Pat 19 [132-135] [Type Bool]: Bind: Ident 20 [132-135] "val"
                                    Expr 21 [138-165] [Type Bool]: If:
                                        Expr 22 [141-145] [Type Bool]: Lit: Bool(true)
                                        Block 23 [146-153] [Type Bool]:
                                            Stmt 24 [147-152]: Expr: Expr 25 [147-152] [Type Bool]: Lit: Bool(false)
                                        Expr 26 [154-165] [Type Bool]: Expr Block: Block 27 [159-165] [Type Bool]:
                                            Stmt 28 [160-164]: Expr: Expr 29 [160-164] [Type Bool]: Lit: Bool(true)
                                Stmt 60 [255-260]: Semi: Expr 61 [255-259] [Type Unit]: Call:
                                    Expr _id_ [255-256] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 62 [255-256] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 63 [257-258] [Type Int]: Lit: Int(7)
                                Stmt 34 [189-246]: Expr: Expr 35 [189-246] [Type Unit]: If:
                                    Expr 36 [192-197] [Type Bool]: Lit: Bool(false)
                                    Block 37 [198-211] [Type Unit]:
                                        Stmt 42 [205-210]: Semi: Expr 43 [205-209] [Type Unit]: Call:
                                            Expr _id_ [205-206] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 44 [205-206] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 45 [207-208] [Type Int]: Lit: Int(4)
                                        Stmt 38 [199-204]: Semi: Expr 39 [199-203] [Type Unit]: Call:
                                            Expr _id_ [199-200] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 40 [199-200] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 41 [201-202] [Type Int]: Lit: Int(3)
                                    Expr 46 [212-246] [Type Unit]: Expr Block: Block 47 [217-246] [Type Unit]:
                                        Stmt 52 [224-239]: Local (Immutable):
                                            Pat 53 [228-231] [Type Bool]: Bind: Ident 54 [228-231] "val"
                                            Expr 55 [234-238] [Type Bool]: Lit: Bool(true)
                                        Stmt 56 [240-245]: Semi: Expr 57 [240-244] [Type Unit]: Call:
                                            Expr _id_ [240-241] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 58 [240-241] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 59 [242-243] [Type Int]: Lit: Int(6)
                                        Stmt 48 [218-223]: Semi: Expr 49 [218-222] [Type Unit]: Call:
                                            Expr _id_ [218-219] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                Expr 50 [218-219] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 51 [220-221] [Type Int]: Lit: Int(5)
                                Stmt 30 [175-180]: Semi: Expr 31 [175-179] [Type Unit]: Call:
                                    Expr _id_ [175-176] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 32 [175-176] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 33 [177-178] [Type Int]: Lit: Int(2)
                                Stmt 14 [114-119]: Semi: Expr 15 [114-118] [Type Unit]: Call:
                                    Expr _id_ [114-115] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 16 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                    Expr 17 [116-117] [Type Int]: Lit: Int(1)
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
                    Namespace (Ident 30 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-181] (Public):
                    Parent: 0
                    Callable 7 [67-181] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [67-181] (Body): Impl:
                            Pat 12 [67-181] [Type Qubit]: Elided
                            Block 13 [104-181] [Type Unit]:
                                Stmt 14 [114-175]: Expr: Expr 15 [114-175] [Type Unit]: For:
                                    Pat 16 [118-119] [Type Int]: Bind: Ident 17 [118-119] "i"
                                    Expr 18 [123-127] [Type Range]: Range:
                                        Expr 19 [123-124] [Type Int]: Lit: Int(0)
                                        <no step>
                                        Expr 20 [126-127] [Type Int]: Lit: Int(5)
                                    Block 21 [128-175] [Type Unit]:
                                        Stmt 22 [142-147]: Semi: Expr 23 [142-146] [Type Unit]: Call:
                                            Expr 24 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 25 [144-145] [Type Int]: Lit: Int(1)
                                        Stmt 26 [160-165]: Semi: Expr 27 [160-164] [Type Unit]: Call:
                                            Expr 28 [160-161] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 29 [162-163] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [67-181] (Adj): Impl:
                            Pat _id_ [67-181] [Type Qubit]: Elided
                            Block 13 [104-181] [Type Unit]:
                                Stmt 14 [114-175]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Local (Immutable):
                                        Pat _id_ [0-0] [Type Range]: Bind: Ident 31 [0-0] "generated_range"
                                        Expr 18 [123-127] [Type Range]: Range:
                                            Expr 19 [123-124] [Type Int]: Lit: Int(0)
                                            <no step>
                                            Expr 20 [126-127] [Type Int]: Lit: Int(5)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                        Pat 16 [118-119] [Type Int]: Bind: Ident 17 [118-119] "i"
                                        Expr _id_ [0-0] [Type Range]: Range:
                                            Expr _id_ [0-0] [Type Int]: BinOp (Add):
                                                Expr _id_ [0-0] [Type Int]: Field:
                                                    Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                    Prim(Start)
                                                Expr _id_ [0-0] [Type Int]: BinOp (Mul):
                                                    Expr _id_ [0-0] [Type Int]: BinOp (Div):
                                                        Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                            Expr _id_ [0-0] [Type Int]: Field:
                                                                Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                                Prim(End)
                                                            Expr _id_ [0-0] [Type Int]: Field:
                                                                Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                                Prim(Start)
                                                        Expr _id_ [0-0] [Type Int]: Field:
                                                            Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                            Prim(Step)
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                        Prim(Step)
                                            Expr _id_ [0-0] [Type Int]: UnOp (Neg):
                                                Expr _id_ [0-0] [Type Int]: Field:
                                                    Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                    Prim(Step)
                                            Expr _id_ [0-0] [Type Int]: Field:
                                                Expr _id_ [0-0] [Type Range]: Var: Local 31
                                                Prim(Start)
                                        Block 21 [128-175] [Type Unit]:
                                            Stmt 26 [160-165]: Semi: Expr 27 [160-164] [Type Unit]: Call:
                                                Expr _id_ [160-161] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 28 [160-161] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 29 [162-163] [Type Int]: Lit: Int(2)
                                            Stmt 22 [142-147]: Semi: Expr 23 [142-146] [Type Unit]: Call:
                                                Expr _id_ [142-143] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 24 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 25 [144-145] [Type Int]: Lit: Int(1)
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
                    Namespace (Ident 31 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-188] (Public):
                    Parent: 0
                    Callable 7 [67-188] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [67-188] (Body): Impl:
                            Pat 12 [67-188] [Type Qubit]: Elided
                            Block 13 [104-188] [Type Unit]:
                                Stmt 14 [114-182]: Expr: Expr 15 [114-182] [Type Unit]: For:
                                    Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                    Expr 18 [125-134] [Type (Int)[]]: Array:
                                        Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                        Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                        Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                    Block 22 [135-182] [Type Unit]:
                                        Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type Unit]: Call:
                                            Expr 25 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 26 [151-152] [Type Int]: Lit: Int(1)
                                        Stmt 27 [167-172]: Semi: Expr 28 [167-171] [Type Unit]: Call:
                                            Expr 29 [167-168] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 30 [169-170] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [67-188] (Adj): Impl:
                            Pat _id_ [67-188] [Type Qubit]: Elided
                            Block 13 [104-188] [Type Unit]:
                                Stmt 14 [114-182]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Local (Immutable):
                                        Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 32 [0-0] "generated_array"
                                        Expr 18 [125-134] [Type (Int)[]]: Array:
                                            Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                        Pat _id_ [0-0] [Type Int]: Bind: Ident 33 [0-0] "generated_index"
                                        Expr _id_ [0-0] [Type Range]: Range:
                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                Expr _id_ [0-0] [Type Int]: Call:
                                                    Expr _id_ [0-0] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                                    Expr _id_ [0-0] [Type (Int)[]]: Var: Local 32
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                            Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                            Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                        Block 22 [135-182] [Type Unit]:
                                            Stmt _id_ [0-0]: Local (Immutable):
                                                Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                                Expr _id_ [0-0] [Type Int]: Index:
                                                    Expr _id_ [0-0] [Type (Int)[]]: Var: Local 32
                                                    Expr _id_ [0-0] [Type Int]: Var: Local 33
                                            Stmt 27 [167-172]: Semi: Expr 28 [167-171] [Type Unit]: Call:
                                                Expr _id_ [167-168] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 29 [167-168] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 30 [169-170] [Type Int]: Lit: Int(2)
                                            Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type Unit]: Call:
                                                Expr _id_ [149-150] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 25 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [151-152] [Type Int]: Lit: Int(1)
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
                    Namespace (Ident 52 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [21-62] (Body): Impl:
                            Pat 5 [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        adj: SpecDecl _id_ [21-62] (Adj): Impl:
                            Pat _id_ [21-62] [Type Int]: Elided
                            Block 6 [60-62]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [67-318] (Public):
                    Parent: 0
                    Callable 7 [67-318] (Operation):
                        name: Ident 8 [77-78] "A"
                        input: Pat 9 [79-88] [Type Qubit]: Bind: Ident 10 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [67-318] (Body): Impl:
                            Pat 12 [67-318] [Type Qubit]: Elided
                            Block 13 [104-318] [Type Unit]:
                                Stmt 14 [114-312]: Expr: Expr 15 [114-312] [Type Unit]: For:
                                    Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                    Expr 18 [125-134] [Type (Int)[]]: Array:
                                        Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                        Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                        Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                    Block 22 [135-312] [Type Unit]:
                                        Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type Unit]: Call:
                                            Expr 25 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 26 [151-152] [Type Int]: Lit: Int(1)
                                        Stmt 27 [167-197]: Local (Immutable):
                                            Pat 28 [171-174] [Type (Bool)[]]: Bind: Ident 29 [171-174] "arr"
                                            Expr 30 [177-196] [Type (Bool)[]]: Array:
                                                Expr 31 [178-182] [Type Bool]: Lit: Bool(true)
                                                Expr 32 [184-189] [Type Bool]: Lit: Bool(false)
                                                Expr 33 [191-195] [Type Bool]: Lit: Bool(true)
                                        Stmt 34 [210-284]: Expr: Expr 35 [210-284] [Type Unit]: For:
                                            Pat 36 [214-217] [Type Bool]: Bind: Ident 37 [214-217] "val"
                                            Expr 38 [221-224] [Type (Bool)[]]: Var: Local 29
                                            Block 39 [225-284] [Type Unit]:
                                                Stmt 40 [243-248]: Semi: Expr 41 [243-247] [Type Unit]: Call:
                                                    Expr 42 [243-244] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 43 [245-246] [Type Int]: Lit: Int(2)
                                                Stmt 44 [265-270]: Semi: Expr 45 [265-269] [Type Unit]: Call:
                                                    Expr 46 [265-266] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 47 [267-268] [Type Int]: Lit: Int(3)
                                        Stmt 48 [297-302]: Semi: Expr 49 [297-301] [Type Unit]: Call:
                                            Expr 50 [297-298] [Type (Int => Unit is Adj)]: Var: Item 1
                                            Expr 51 [299-300] [Type Int]: Lit: Int(4)
                        adj: SpecDecl _id_ [67-318] (Adj): Impl:
                            Pat _id_ [67-318] [Type Qubit]: Elided
                            Block 13 [104-318] [Type Unit]:
                                Stmt 14 [114-312]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                    Stmt _id_ [0-0]: Local (Immutable):
                                        Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 55 [0-0] "generated_array"
                                        Expr 18 [125-134] [Type (Int)[]]: Array:
                                            Expr 19 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 20 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 21 [132-133] [Type Int]: Lit: Int(2)
                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                        Pat _id_ [0-0] [Type Int]: Bind: Ident 56 [0-0] "generated_index"
                                        Expr _id_ [0-0] [Type Range]: Range:
                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                Expr _id_ [0-0] [Type Int]: Call:
                                                    Expr _id_ [0-0] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                                    Expr _id_ [0-0] [Type (Int)[]]: Var: Local 55
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                            Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                            Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                        Block 22 [135-312] [Type Unit]:
                                            Stmt _id_ [0-0]: Local (Immutable):
                                                Pat 16 [118-121] [Type Int]: Bind: Ident 17 [118-121] "val"
                                                Expr _id_ [0-0] [Type Int]: Index:
                                                    Expr _id_ [0-0] [Type (Int)[]]: Var: Local 55
                                                    Expr _id_ [0-0] [Type Int]: Var: Local 56
                                            Stmt 27 [167-197]: Local (Immutable):
                                                Pat 28 [171-174] [Type (Bool)[]]: Bind: Ident 29 [171-174] "arr"
                                                Expr 30 [177-196] [Type (Bool)[]]: Array:
                                                    Expr 31 [178-182] [Type Bool]: Lit: Bool(true)
                                                    Expr 32 [184-189] [Type Bool]: Lit: Bool(false)
                                                    Expr 33 [191-195] [Type Bool]: Lit: Bool(true)
                                            Stmt 48 [297-302]: Semi: Expr 49 [297-301] [Type Unit]: Call:
                                                Expr _id_ [297-298] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 50 [297-298] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 51 [299-300] [Type Int]: Lit: Int(4)
                                            Stmt 34 [210-284]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat _id_ [0-0] [Type (Bool)[]]: Bind: Ident 53 [0-0] "generated_array"
                                                    Expr 38 [221-224] [Type (Bool)[]]: Var: Local 29
                                                Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                                    Pat _id_ [0-0] [Type Int]: Bind: Ident 54 [0-0] "generated_index"
                                                    Expr _id_ [0-0] [Type Range]: Range:
                                                        Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                            Expr _id_ [0-0] [Type Int]: Call:
                                                                Expr _id_ [0-0] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                                                Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 53
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                        Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                        Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                                    Block 39 [225-284] [Type Unit]:
                                                        Stmt _id_ [0-0]: Local (Immutable):
                                                            Pat 36 [214-217] [Type Bool]: Bind: Ident 37 [214-217] "val"
                                                            Expr _id_ [0-0] [Type Bool]: Index:
                                                                Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 53
                                                                Expr _id_ [0-0] [Type Int]: Var: Local 54
                                                        Stmt 44 [265-270]: Semi: Expr 45 [265-269] [Type Unit]: Call:
                                                            Expr _id_ [265-266] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                                Expr 46 [265-266] [Type (Int => Unit is Adj)]: Var: Item 1
                                                            Expr 47 [267-268] [Type Int]: Lit: Int(3)
                                                        Stmt 40 [243-248]: Semi: Expr 41 [243-247] [Type Unit]: Call:
                                                            Expr _id_ [243-244] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                                Expr 42 [243-244] [Type (Int => Unit is Adj)]: Var: Item 1
                                                            Expr 43 [245-246] [Type Int]: Lit: Int(2)
                                            Stmt 23 [149-154]: Semi: Expr 24 [149-153] [Type Unit]: Call:
                                                Expr _id_ [149-150] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 25 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [151-152] [Type Int]: Lit: Int(1)
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
                    Namespace (Ident 22 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68] (Body): Impl:
                            Pat 5 [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        adj: SpecDecl _id_ [21-68] (Adj): Impl:
                            Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl: SpecDecl _id_ [21-68] (Ctl): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 23 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl-adj: SpecDecl _id_ [21-68] (CtlAdj): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 24 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                Item 2 [73-156] (Public):
                    Parent: 0
                    Callable 7 [73-156] (Operation):
                        name: Ident 8 [83-84] "A"
                        input: Pat 9 [85-94] [Type Qubit]: Bind: Ident 10 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 11 [126-150] (Body): Impl:
                            Pat 12 [131-134] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [73-156] (Adj): Impl:
                            Pat _id_ [73-156] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl _id_ [73-156] (Ctl): Impl:
                            Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 25 [73-156] "ctls"
                                Pat _id_ [73-156] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 25
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 25
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl _id_ [73-156] (CtlAdj): Impl:
                            Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 26 [73-156] "ctls"
                                Pat _id_ [73-156] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 26
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 26
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68] (Body): Impl:
                            Pat 5 [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        adj: SpecDecl _id_ [21-68] (Adj): Impl:
                            Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl: SpecDecl _id_ [21-68] (Ctl): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 24 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl-adj: SpecDecl _id_ [21-68] (CtlAdj): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 25 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                Item 2 [73-189] (Public):
                    Parent: 0
                    Callable 7 [73-189] (Operation):
                        name: Ident 8 [83-84] "A"
                        input: Pat 9 [85-94] [Type Qubit]: Bind: Ident 10 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 11 [126-150] (Body): Impl:
                            Pat 12 [131-134] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [73-189] (Adj): Impl:
                            Pat _id_ [73-189] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl _id_ [73-189] (Ctl): Impl:
                            Pat _id_ [73-189] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-189] [Type (Qubit)[]]: Bind: Ident 26 [73-189] "ctls"
                                Pat _id_ [73-189] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 26
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 26
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl _id_ [73-189] (CtlAdj): Impl:
                            Pat _id_ [73-189] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-189] [Type (Qubit)[]]: Bind: Ident 27 [73-189] "ctls"
                                Pat _id_ [73-189] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 27
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 27
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 43 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68] (Body): Impl:
                            Pat 5 [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        adj: SpecDecl _id_ [21-68] (Adj): Impl:
                            Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl: SpecDecl _id_ [21-68] (Ctl): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 44 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl-adj: SpecDecl _id_ [21-68] (CtlAdj): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 45 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                Item 2 [73-270] (Public):
                    Parent: 0
                    Callable 7 [73-270] (Operation):
                        name: Ident 8 [83-84] "A"
                        input: Pat 9 [85-94] [Type Qubit]: Bind: Ident 10 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 11 [126-150] (Body): Impl:
                            Pat 12 [131-134] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [73-270] (Adj): Impl:
                            Pat _id_ [73-270] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl 22 [159-231] (Ctl): Impl:
                            Pat 23 [170-181] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat 24 [171-175] [Type (Qubit)[]]: Bind: Ident 25 [171-175] "ctls"
                                Pat 26 [177-180] [Type Qubit]: Elided
                            Block 27 [182-231] [Type Unit]:
                                Stmt 28 [184-206]: Semi: Expr 29 [184-205] [Type Unit]: Call:
                                    Expr 30 [184-196] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 31 [195-196] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 32 [196-205] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 33 [197-201] [Type (Qubit)[]]: Var: Local 25
                                        Expr 34 [203-204] [Type Int]: Lit: Int(1)
                                Stmt 35 [207-229]: Semi: Expr 36 [207-228] [Type Unit]: Call:
                                    Expr 37 [207-219] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 38 [218-219] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 39 [219-228] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 40 [220-224] [Type (Qubit)[]]: Var: Local 25
                                        Expr 41 [226-227] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl _id_ [73-270] (CtlAdj): Impl:
                            Pat 23 [170-181] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat 24 [171-175] [Type (Qubit)[]]: Bind: Ident 25 [171-175] "ctls"
                                Pat 26 [177-180] [Type Qubit]: Elided
                            Block 27 [182-231] [Type Unit]:
                                Stmt 35 [207-229]: Semi: Expr 36 [207-228] [Type Unit]: Call:
                                    Expr _id_ [207-219] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 37 [207-219] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 38 [218-219] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 39 [219-228] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 40 [220-224] [Type (Qubit)[]]: Var: Local 25
                                        Expr 41 [226-227] [Type Int]: Lit: Int(2)
                                Stmt 28 [184-206]: Semi: Expr 29 [184-205] [Type Unit]: Call:
                                    Expr _id_ [184-196] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 30 [184-196] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 31 [195-196] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 32 [196-205] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr 33 [197-201] [Type (Qubit)[]]: Var: Local 25
                                        Expr 34 [203-204] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 23 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68] (Public):
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 4 [21-68] (Body): Impl:
                            Pat 5 [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        adj: SpecDecl _id_ [21-68] (Adj): Impl:
                            Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl: SpecDecl _id_ [21-68] (Ctl): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 24 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                        ctl-adj: SpecDecl _id_ [21-68] (CtlAdj): Impl:
                            Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 25 [21-68] "ctls"
                                Pat _id_ [21-68] [Type Int]: Elided
                            Block 6 [66-68]: <empty>
                Item 2 [73-191] (Public):
                    Parent: 0
                    Callable 7 [73-191] (Operation):
                        name: Ident 8 [83-84] "A"
                        input: Pat 9 [85-94] [Type Qubit]: Bind: Ident 10 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 11 [126-150] (Body): Impl:
                            Pat 12 [131-134] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        adj: SpecDecl _id_ [73-191] (Adj): Impl:
                            Pat _id_ [73-191] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type Int]: Lit: Int(1)
                        ctl: SpecDecl _id_ [73-191] (Ctl): Impl:
                            Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 26 [73-191] "ctls"
                                Pat _id_ [73-191] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr 16 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 26
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr 20 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                        Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 26
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                        ctl-adj: SpecDecl 22 [159-185] (CtlAdj): Impl:
                            Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 26 [73-191] "ctls"
                                Pat _id_ [73-191] [Type Qubit]: Elided
                            Block 13 [135-150] [Type Unit]:
                                Stmt 18 [143-148]: Semi: Expr 19 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 20 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 21 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 26
                                        Expr 21 [145-146] [Type Int]: Lit: Int(2)
                                Stmt 14 [137-142]: Semi: Expr 15 [137-141] [Type Unit]: Call:
                                    Expr _id_ [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                        Expr 16 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 16 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                    Expr 17 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                        Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 26
                                        Expr 17 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 36 [10-11] "A"): Item 1, Item 2, Item 3
                Item 1 [18-55] (Public):
                    Parent: 0
                    Callable 0 [18-55] (Operation):
                        name: Ident 1 [28-29] "X"
                        input: Pat 2 [30-39] [Type Qubit]: Bind: Ident 3 [30-31] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 4 [18-55] (Body): Impl:
                            Pat 5 [18-55] [Type Qubit]: Elided
                            Block 6 [53-55]: <empty>
                        adj: SpecDecl _id_ [18-55] (Adj): Impl:
                            Pat _id_ [18-55] [Type Qubit]: Elided
                            Block 6 [53-55]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [60-106] (Public):
                    Parent: 0
                    Callable 7 [60-106] (Operation):
                        name: Ident 8 [70-73] "Foo"
                        input: Pat 9 [74-97] [Type (Qubit => Unit is Adj)]: Bind: Ident 10 [74-76] "op"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 11 [60-106] (Body): Impl:
                            Pat 12 [60-106] [Type (Qubit => Unit is Adj)]: Elided
                            Block 13 [104-106]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [111-151] (Public):
                    Parent: 0
                    Callable 14 [111-151] (Operation):
                        name: Ident 15 [121-124] "Bar"
                        input: Pat 16 [124-126] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 17 [111-151] (Body): Impl:
                            Pat 18 [111-151] [Type Unit]: Elided
                            Block 19 [132-151] [Type Unit]:
                                Stmt 20 [134-149]: Semi: Expr 21 [134-148] [Type Unit]: Call:
                                    Expr 22 [134-137] [Type ((Qubit => Unit is Adj) => Unit)]: Var: Item 2
                                    Expr 23 [138-147] [Type (Qubit => Unit is Adj)]: Closure([], 4)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 4 [138-147] (Internal):
                    Parent: 3
                    Callable 30 [138-147] (Operation):
                        name: Ident 31 [138-147] "lambda"
                        input: Pat 29 [138-147] [Type (Qubit,)]: Tuple:
                            Pat 24 [138-139] [Type Qubit]: Bind: Ident 25 [138-139] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 32 [143-147] (Body): Impl:
                            Pat 33 [138-147] [Type (Qubit,)]: Elided
                            Block 34 [143-147] [Type Unit]:
                                Stmt 35 [143-147]: Expr: Expr 26 [143-147] [Type Unit]: Call:
                                    Expr 27 [143-144] [Type (Qubit => Unit is Adj)]: Var: Item 1
                                    Expr 28 [145-146] [Type Qubit]: Var: Local 25
                        adj: SpecDecl _id_ [138-147] (Adj): Impl:
                            Pat _id_ [138-147] [Type (Qubit,)]: Elided
                            Block 34 [143-147] [Type Unit]:
                                Stmt 35 [143-147]: Expr: Expr 26 [143-147] [Type Unit]: Call:
                                    Expr _id_ [143-144] [Type (Qubit => Unit is Adj)]: UnOp (Functor Adj):
                                        Expr 27 [143-144] [Type (Qubit => Unit is Adj)]: Var: Item 1
                                    Expr 28 [145-146] [Type Qubit]: Var: Local 25
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
                    Namespace (Ident 58 [23-24] "A"): Item 1, Item 2, Item 3, Item 4
                Item 1 [43-75] (Public):
                    Parent: 0
                    Callable 0 [43-75] (Operation):
                        name: Ident 1 [53-56] "Foo"
                        input: Pat 2 [57-66] [Type Qubit]: Bind: Ident 3 [57-58] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 4 [43-75] (Body): Impl:
                            Pat 5 [43-75] [Type Qubit]: Elided
                            Block 6 [73-75]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [92-131] (Public):
                    Parent: 0
                    Callable 7 [92-131] (Operation):
                        name: Ident 8 [102-105] "Bar"
                        input: Pat 9 [106-115] [Type Qubit]: Bind: Ident 10 [106-107] "q"
                        output: Unit
                        functors: Adj
                        body: SpecDecl 11 [92-131] (Body): Impl:
                            Pat 12 [92-131] [Type Qubit]: Elided
                            Block 13 [129-131]: <empty>
                        adj: SpecDecl _id_ [92-131] (Adj): Impl:
                            Pat _id_ [92-131] [Type Qubit]: Elided
                            Block 13 [129-131]: <empty>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [148-193] (Public):
                    Parent: 0
                    Callable 14 [148-193] (Operation):
                        name: Ident 15 [158-161] "Baz"
                        input: Pat 16 [162-171] [Type Qubit]: Bind: Ident 17 [162-163] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: SpecDecl 18 [148-193] (Body): Impl:
                            Pat 19 [148-193] [Type Qubit]: Elided
                            Block 20 [191-193]: <empty>
                        adj: SpecDecl _id_ [148-193] (Adj): Impl:
                            Pat _id_ [148-193] [Type Qubit]: Elided
                            Block 20 [191-193]: <empty>
                        ctl: SpecDecl _id_ [148-193] (Ctl): Impl:
                            Pat _id_ [148-193] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [148-193] [Type (Qubit)[]]: Bind: Ident 59 [148-193] "ctls"
                                Pat _id_ [148-193] [Type Qubit]: Elided
                            Block 20 [191-193]: <empty>
                        ctl-adj: SpecDecl _id_ [148-193] (CtlAdj): Impl:
                            Pat _id_ [148-193] [Type ((Qubit)[], Qubit)]: Tuple:
                                Pat _id_ [148-193] [Type (Qubit)[]]: Bind: Ident 60 [148-193] "ctls"
                                Pat _id_ [148-193] [Type Qubit]: Elided
                            Block 20 [191-193]: <empty>
                Item 4 [210-314] (Public):
                    Parent: 0
                    Callable 21 [210-314] (Operation):
                        name: Ident 22 [220-224] "Main"
                        input: Pat 23 [224-226] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 24 [210-314] (Body): Impl:
                            Pat 25 [210-314] [Type Unit]: Elided
                            Block 26 [232-314] [Type Unit]:
                                Stmt 27 [254-296]: Local (Immutable):
                                    Pat 28 [258-261] [Type ((Qubit => Unit))[]]: Bind: Ident 29 [258-261] "ops"
                                    Expr 30 [264-295] [Type ((Qubit => Unit))[]]: Array:
                                        Expr 31 [265-268] [Type (Qubit => Unit)]: Var: Item 1
                                        Expr 32 [270-281] [Type (Qubit => Unit)]: Closure([], 5)
                                        Expr 45 [283-294] [Type (Qubit => Unit)]: Closure([], 6)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 5 [270-281] (Internal):
                    Parent: 4
                    Callable 39 [270-281] (Operation):
                        name: Ident 40 [270-281] "lambda"
                        input: Pat 38 [270-281] [Type (Qubit,)]: Tuple:
                            Pat 33 [270-271] [Type Qubit]: Bind: Ident 34 [270-271] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 41 [275-281] (Body): Impl:
                            Pat 42 [270-281] [Type (Qubit,)]: Elided
                            Block 43 [275-281] [Type Unit]:
                                Stmt 44 [275-281]: Expr: Expr 35 [275-281] [Type Unit]: Call:
                                    Expr 36 [275-278] [Type (Qubit => Unit is Adj)]: Var: Item 2
                                    Expr 37 [279-280] [Type Qubit]: Var: Local 34
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 6 [283-294] (Internal):
                    Parent: 4
                    Callable 52 [283-294] (Operation):
                        name: Ident 53 [283-294] "lambda"
                        input: Pat 51 [283-294] [Type (Qubit,)]: Tuple:
                            Pat 46 [283-284] [Type Qubit]: Bind: Ident 47 [283-284] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 54 [288-294] (Body): Impl:
                            Pat 55 [283-294] [Type (Qubit,)]: Elided
                            Block 56 [288-294] [Type Unit]:
                                Stmt 57 [288-294]: Expr: Expr 48 [288-294] [Type Unit]: Call:
                                    Expr 49 [288-291] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 3
                                    Expr 50 [292-293] [Type Qubit]: Var: Local 47
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
