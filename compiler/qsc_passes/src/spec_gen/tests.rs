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
                Item 0 [0-184] (Public):
                    Namespace (Ident 22 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl 4 [68-79] (Body): Impl:
                                Pat 5 [73-76] [Type Qubit]: Elided
                                Block 6 [77-79]: <empty>
                            SpecDecl 7 [88-113] (Ctl): Impl:
                                Pat 8 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 9 [100-104] [Type (Qubit)[]]: Bind: Ident 10 [100-104] "ctls"
                                    Pat 11 [106-109] [Type Qubit]: Elided
                                Block 12 [111-113]: <empty>
                Item 2 [124-182] (Public):
                    Parent: 0
                    Callable 13 [124-182] (Operation):
                        name: Ident 14 [134-135] "B"
                        input: Pat 15 [136-145] [Type Qubit]: Bind: Ident 16 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl _id_ [161-182] (Body): Impl:
                                Pat _id_ [161-182] [Type Qubit]: Elided
                                Block 17 [161-182] [Type Unit]:
                                    Stmt 18 [171-176]: Semi: Expr 19 [171-175] [Type Unit]: Call:
                                        Expr 20 [171-172] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                        Expr 21 [173-174] [Type Qubit]: Var: Local 16
                            SpecDecl _id_ [124-182] (Ctl): Impl:
                                Pat _id_ [124-182] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-182] [Type (Qubit)[]]: Bind: Ident 23 [124-182] "ctls"
                                    Pat _id_ [124-182] [Type Qubit]: Elided
                                Block 17 [161-182] [Type Unit]:
                                    Stmt 18 [171-176]: Semi: Expr 19 [171-175] [Type Unit]: Call:
                                        Expr 20 [171-172] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                            Expr 20 [171-172] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                        Expr 21 [173-174] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [173-174] [Type (Qubit)[]]: Var: Local 23
                                            Expr 21 [173-174] [Type Qubit]: Var: Local 16"#]],
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
                        body: Specializations:
                            SpecDecl 4 [68-79] (Body): Impl:
                                Pat 5 [73-76] [Type Qubit]: Elided
                                Block 6 [77-79]: <empty>
                            SpecDecl 7 [88-113] (Ctl): Impl:
                                Pat 8 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 9 [100-104] [Type (Qubit)[]]: Bind: Ident 10 [100-104] "ctls"
                                    Pat 11 [106-109] [Type Qubit]: Elided
                                Block 12 [111-113]: <empty>
                Item 2 [124-240] (Public):
                    Parent: 0
                    Callable 13 [124-240] (Operation):
                        name: Ident 14 [134-135] "B"
                        input: Pat 15 [136-145] [Type Qubit]: Bind: Ident 16 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl 17 [171-209] (Body): Impl:
                                Pat 18 [176-179] [Type Qubit]: Elided
                                Block 19 [180-209] [Type Unit]:
                                    Stmt 20 [194-199]: Semi: Expr 21 [194-198] [Type Unit]: Call:
                                        Expr 22 [194-195] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                        Expr 23 [196-197] [Type Qubit]: Var: Local 16
                            SpecDecl 24 [218-234] (Ctl): Impl:
                                Pat _id_ [218-234] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [218-234] [Type (Qubit)[]]: Bind: Ident 26 [218-234] "ctls"
                                    Pat _id_ [218-234] [Type Qubit]: Elided
                                Block 19 [180-209] [Type Unit]:
                                    Stmt 20 [194-199]: Semi: Expr 21 [194-198] [Type Unit]: Call:
                                        Expr 22 [194-195] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                            Expr 22 [194-195] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                        Expr 23 [196-197] [Type ((Qubit)[], Qubit)]: Tuple:
                                            Expr _id_ [196-197] [Type (Qubit)[]]: Var: Local 26
                                            Expr 23 [196-197] [Type Qubit]: Var: Local 16"#]],
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
                        body: Specializations:
                            SpecDecl 4 [74-85] (Body): Impl:
                                Pat 5 [79-82] [Type Qubit]: Elided
                                Block 6 [83-85]: <empty>
                            SpecDecl 7 [94-108] (Adj): Impl:
                                Pat 8 [102-105] [Type Qubit]: Elided
                                Block 9 [106-108]: <empty>
                            SpecDecl 10 [117-142] (Ctl): Impl:
                                Pat 11 [128-139] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 12 [129-133] [Type (Qubit)[]]: Bind: Ident 13 [129-133] "ctls"
                                    Pat 14 [135-138] [Type Qubit]: Elided
                                Block 15 [140-142]: <empty>
                            SpecDecl _id_ [21-148] (CtlAdj): Impl:
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
                        body: Specializations:
                            SpecDecl 20 [206-244] (Body): Impl:
                                Pat 21 [211-214] [Type Qubit]: Elided
                                Block 22 [215-244] [Type Unit]:
                                    Stmt 23 [229-234]: Semi: Expr 24 [229-233] [Type Unit]: Call:
                                        Expr 25 [229-230] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 26 [231-232] [Type Qubit]: Var: Local 19
                            SpecDecl 27 [253-302] (Adj): Impl:
                                Pat 28 [261-264] [Type Qubit]: Elided
                                Block 29 [265-302] [Type Unit]:
                                    Stmt 30 [279-292]: Semi: Expr 31 [279-291] [Type Unit]: Call:
                                        Expr 32 [279-288] [Type (Qubit => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 33 [287-288] [Type (Qubit => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 34 [289-290] [Type Qubit]: Var: Local 19
                            SpecDecl _id_ [153-308] (Ctl): Impl:
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
                            SpecDecl _id_ [153-308] (CtlAdj): Impl:
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
                    Namespace (Ident 30 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119] (Public):
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [33-42] [Type Qubit]: Bind: Ident 3 [33-34] "q"
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl 4 [68-79] (Body): Impl:
                                Pat 5 [73-76] [Type Qubit]: Elided
                                Block 6 [77-79]: <empty>
                            SpecDecl 7 [88-113] (Ctl): Impl:
                                Pat 8 [99-110] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat 9 [100-104] [Type (Qubit)[]]: Bind: Ident 10 [100-104] "ctls"
                                    Pat 11 [106-109] [Type Qubit]: Elided
                                Block 12 [111-113]: <empty>
                Item 2 [124-257] (Public):
                    Parent: 0
                    Callable 13 [124-257] (Operation):
                        name: Ident 14 [134-135] "B"
                        input: Pat 15 [136-145] [Type Qubit]: Bind: Ident 16 [136-137] "q"
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl _id_ [161-257] (Body): Impl:
                                Pat _id_ [161-257] [Type Qubit]: Elided
                                Block 17 [161-257] [Type Unit]:
                                    Stmt 18 [171-251]: Expr: Expr 19 [171-251] [Type Unit]: Conjugate:
                                        Block 20 [178-207] [Type Unit]:
                                            Stmt 21 [192-197]: Semi: Expr 22 [192-196] [Type Unit]: Call:
                                                Expr 23 [192-193] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                                Expr 24 [194-195] [Type Qubit]: Var: Local 16
                                        Block 25 [222-251] [Type Unit]:
                                            Stmt 26 [236-241]: Semi: Expr 27 [236-240] [Type Unit]: Call:
                                                Expr 28 [236-237] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                                Expr 29 [238-239] [Type Qubit]: Var: Local 16
                            SpecDecl _id_ [124-257] (Ctl): Impl:
                                Pat _id_ [124-257] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [124-257] [Type (Qubit)[]]: Bind: Ident 31 [124-257] "ctls"
                                    Pat _id_ [124-257] [Type Qubit]: Elided
                                Block 17 [161-257] [Type Unit]:
                                    Stmt 18 [171-251]: Expr: Expr 19 [171-251] [Type Unit]: Conjugate:
                                        Block 20 [178-207] [Type Unit]:
                                            Stmt 21 [192-197]: Semi: Expr 22 [192-196] [Type Unit]: Call:
                                                Expr 23 [192-193] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                                Expr 24 [194-195] [Type Qubit]: Var: Local 16
                                        Block 25 [222-251] [Type Unit]:
                                            Stmt 26 [236-241]: Semi: Expr 27 [236-240] [Type Unit]: Call:
                                                Expr 28 [236-237] [Type (((Qubit)[], Qubit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                                    Expr 28 [236-237] [Type (Qubit => Unit is Ctl)]: Var: Item 1
                                                Expr 29 [238-239] [Type ((Qubit)[], Qubit)]: Tuple:
                                                    Expr _id_ [238-239] [Type (Qubit)[]]: Var: Local 31
                                                    Expr 29 [238-239] [Type Qubit]: Var: Local 16"#]],
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
                    Namespace (Ident 20 [10-14] "test"): Item 1, Item 2, Item 3
                Item 1 [21-45] (Public):
                    Parent: 0
                    Callable 0 [21-45] (Function):
                        name: Ident 1 [30-33] "Foo"
                        input: Pat 2 [33-35] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 3 [43-45]: <empty>
                Item 2 [50-80] (Public):
                    Parent: 0
                    Callable 4 [50-80] (Operation):
                        name: Ident 5 [60-61] "A"
                        input: Pat 6 [61-63] [Type Unit]: Unit
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl _id_ [78-80] (Body): Impl:
                                Pat _id_ [78-80] [Type Unit]: Elided
                                Block 7 [78-80]: <empty>
                            SpecDecl _id_ [50-80] (Ctl): Impl:
                                Pat _id_ [50-80] [Type ((Qubit)[], Unit)]: Tuple:
                                    Pat _id_ [50-80] [Type (Qubit)[]]: Bind: Ident 21 [50-80] "ctls"
                                    Pat _id_ [50-80] [Type Unit]: Elided
                                Block 7 [78-80]: <empty>
                Item 3 [85-148] (Public):
                    Parent: 0
                    Callable 8 [85-148] (Operation):
                        name: Ident 9 [95-96] "B"
                        input: Pat 10 [96-98] [Type Unit]: Unit
                        output: Unit
                        functors: Ctl
                        body: Specializations:
                            SpecDecl _id_ [113-148] (Body): Impl:
                                Pat _id_ [113-148] [Type Unit]: Elided
                                Block 11 [113-148] [Type Unit]:
                                    Stmt 12 [123-129]: Semi: Expr 13 [123-128] [Type Unit]: Call:
                                        Expr 14 [123-126] [Type (Unit -> Unit)]: Var: Item 1
                                        Expr 15 [126-128] [Type Unit]: Unit
                                    Stmt 16 [138-142]: Semi: Expr 17 [138-141] [Type Unit]: Call:
                                        Expr 18 [138-139] [Type (Unit => Unit is Ctl)]: Var: Item 2
                                        Expr 19 [139-141] [Type Unit]: Unit
                            SpecDecl _id_ [85-148] (Ctl): Impl:
                                Pat _id_ [85-148] [Type ((Qubit)[], Unit)]: Tuple:
                                    Pat _id_ [85-148] [Type (Qubit)[]]: Bind: Ident 22 [85-148] "ctls"
                                    Pat _id_ [85-148] [Type Unit]: Elided
                                Block 11 [113-148] [Type Unit]:
                                    Stmt 12 [123-129]: Semi: Expr 13 [123-128] [Type Unit]: Call:
                                        Expr 14 [123-126] [Type (Unit -> Unit)]: Var: Item 1
                                        Expr 15 [126-128] [Type Unit]: Unit
                                    Stmt 16 [138-142]: Semi: Expr 17 [138-141] [Type Unit]: Call:
                                        Expr 18 [138-139] [Type (((Qubit)[], Unit) => Unit is Ctl)]: UnOp (Functor Ctl):
                                            Expr 18 [138-139] [Type (Unit => Unit is Ctl)]: Var: Item 2
                                        Expr 19 [139-141] [Type ((Qubit)[], Unit)]: Tuple:
                                            Expr _id_ [139-141] [Type (Qubit)[]]: Var: Local 22
                                            Expr 19 [139-141] [Type Unit]: Unit"#]],
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
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-166] (Public):
                    Parent: 0
                    Callable 5 [67-166] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl 9 [114-138] (Body): Impl:
                                Pat 10 [119-122] [Type Qubit]: Elided
                                Block 11 [123-138] [Type Unit]:
                                    Stmt 12 [125-130]: Semi: Expr 13 [125-129] [Type Unit]: Call:
                                        Expr 14 [125-126] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 16 [131-136]: Semi: Expr 17 [131-135] [Type Unit]: Call:
                                        Expr 18 [131-132] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 19 [133-134] [Type Int]: Lit: Int(2)
                            SpecDecl 20 [147-160] (Adj): Impl:
                                Pat 10 [119-122] [Type Qubit]: Elided
                                Block 11 [123-138] [Type Unit]:
                                    Stmt 12 [125-130]: Semi: Expr 13 [125-129] [Type Unit]: Call:
                                        Expr 14 [125-126] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [127-128] [Type Int]: Lit: Int(1)
                                    Stmt 16 [131-136]: Semi: Expr 17 [131-135] [Type Unit]: Call:
                                        Expr 18 [131-132] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 19 [133-134] [Type Int]: Lit: Int(2)"#]],
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
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 22 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 23 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                Item 2 [73-178] (Public):
                    Parent: 0
                    Callable 5 [73-178] (Operation):
                        name: Ident 6 [83-84] "A"
                        input: Pat 7 [85-94] [Type Qubit]: Bind: Ident 8 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: Specializations:
                            SpecDecl 9 [126-150] (Body): Impl:
                                Pat 10 [131-134] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 20 [159-172] (Adj): Impl:
                                Pat 10 [131-134] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (Ctl): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 24 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 24
                                            Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 24
                                            Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-178] (CtlAdj): Impl:
                                Pat _id_ [73-178] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-178] [Type (Qubit)[]]: Bind: Ident 24 [73-178] "ctls"
                                    Pat _id_ [73-178] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 24
                                            Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 24
                                            Expr 19 [145-146] [Type Int]: Lit: Int(2)"#]],
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
                    Namespace (Ident 18 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-139] (Public):
                    Parent: 0
                    Callable 5 [67-139] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-139] (Body): Impl:
                                Pat _id_ [104-139] [Type Qubit]: Elided
                                Block 9 [104-139] [Type Unit]:
                                    Stmt 10 [114-119]: Semi: Expr 11 [114-118] [Type Unit]: Call:
                                        Expr 12 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 13 [116-117] [Type Int]: Lit: Int(1)
                                    Stmt 14 [128-133]: Semi: Expr 15 [128-132] [Type Unit]: Call:
                                        Expr 16 [128-129] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 17 [130-131] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-139] (Adj): Impl:
                                Pat _id_ [67-139] [Type Qubit]: Elided
                                Block 9 [104-139] [Type Unit]:
                                    Stmt 14 [128-133]: Semi: Expr 15 [128-132] [Type Unit]: Call:
                                        Expr _id_ [128-129] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 16 [128-129] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 17 [130-131] [Type Int]: Lit: Int(2)
                                    Stmt 10 [114-119]: Semi: Expr 11 [114-118] [Type Unit]: Call:
                                        Expr _id_ [114-115] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 12 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 13 [116-117] [Type Int]: Lit: Int(1)"#]],
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
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-198] (Public):
                    Parent: 0
                    Callable 5 [67-198] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl 9 [114-170] (Body): Impl:
                                Pat 10 [119-122] [Type Qubit]: Elided
                                Block 11 [123-170] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [155-160]: Semi: Expr 17 [155-159] [Type Unit]: Call:
                                        Expr 18 [155-156] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 19 [157-158] [Type Int]: Lit: Int(2)
                            SpecDecl 20 [179-192] (Adj): Impl:
                                Pat _id_ [179-192] [Type Qubit]: Elided
                                Block 11 [123-170] [Type Unit]:
                                    Stmt 16 [155-160]: Semi: Expr 17 [155-159] [Type Unit]: Call:
                                        Expr _id_ [155-156] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 18 [155-156] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 19 [157-158] [Type Int]: Lit: Int(2)
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr _id_ [137-138] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 30 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-236] (Public):
                    Parent: 0
                    Callable 5 [67-236] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-236] (Body): Impl:
                                Pat _id_ [104-236] [Type Qubit]: Elided
                                Block 9 [104-236] [Type Unit]:
                                    Stmt 10 [114-230]: Expr: Expr 11 [114-230] [Type Unit]: Conjugate:
                                        Block 12 [121-168] [Type Unit]:
                                            Stmt 13 [135-140]: Semi: Expr 14 [135-139] [Type Unit]: Call:
                                                Expr 15 [135-136] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 16 [137-138] [Type Int]: Lit: Int(1)
                                            Stmt 17 [153-158]: Semi: Expr 18 [153-157] [Type Unit]: Call:
                                                Expr 19 [153-154] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 20 [155-156] [Type Int]: Lit: Int(2)
                                        Block 21 [183-230] [Type Unit]:
                                            Stmt 22 [197-202]: Semi: Expr 23 [197-201] [Type Unit]: Call:
                                                Expr 24 [197-198] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 25 [199-200] [Type Int]: Lit: Int(3)
                                            Stmt 26 [215-220]: Semi: Expr 27 [215-219] [Type Unit]: Call:
                                                Expr 28 [215-216] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 29 [217-218] [Type Int]: Lit: Int(4)
                            SpecDecl _id_ [67-236] (Adj): Impl:
                                Pat _id_ [67-236] [Type Qubit]: Elided
                                Block 9 [104-236] [Type Unit]:
                                    Stmt 10 [114-230]: Expr: Expr 11 [114-230] [Type Unit]: Conjugate:
                                        Block 12 [121-168] [Type Unit]:
                                            Stmt 13 [135-140]: Semi: Expr 14 [135-139] [Type Unit]: Call:
                                                Expr 15 [135-136] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 16 [137-138] [Type Int]: Lit: Int(1)
                                            Stmt 17 [153-158]: Semi: Expr 18 [153-157] [Type Unit]: Call:
                                                Expr 19 [153-154] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 20 [155-156] [Type Int]: Lit: Int(2)
                                        Block 21 [183-230] [Type Unit]:
                                            Stmt 26 [215-220]: Semi: Expr 27 [215-219] [Type Unit]: Call:
                                                Expr _id_ [215-216] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 28 [215-216] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 29 [217-218] [Type Int]: Lit: Int(4)
                                            Stmt 22 [197-202]: Semi: Expr 23 [197-201] [Type Unit]: Call:
                                                Expr _id_ [197-198] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 24 [197-198] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 25 [199-200] [Type Int]: Lit: Int(3)"#]],
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
                    Namespace (Ident 60 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-266] (Public):
                    Parent: 0
                    Callable 5 [67-266] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-266] (Body): Impl:
                                Pat _id_ [104-266] [Type Qubit]: Elided
                                Block 9 [104-266] [Type Unit]:
                                    Stmt 10 [114-119]: Semi: Expr 11 [114-118] [Type Unit]: Call:
                                        Expr 12 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 13 [116-117] [Type Int]: Lit: Int(1)
                                    Stmt 14 [128-166]: Local (Immutable):
                                        Pat 15 [132-135] [Type Bool]: Bind: Ident 16 [132-135] "val"
                                        Expr 17 [138-165] [Type Bool]: If:
                                            Expr 18 [141-145] [Type Bool]: Lit: Bool(true)
                                            Block 19 [146-153] [Type Bool]:
                                                Stmt 20 [147-152]: Expr: Expr 21 [147-152] [Type Bool]: Lit: Bool(false)
                                            Expr 22 [154-165] [Type Bool]: Expr Block: Block 23 [159-165] [Type Bool]:
                                                Stmt 24 [160-164]: Expr: Expr 25 [160-164] [Type Bool]: Lit: Bool(true)
                                    Stmt 26 [175-180]: Semi: Expr 27 [175-179] [Type Unit]: Call:
                                        Expr 28 [175-176] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 29 [177-178] [Type Int]: Lit: Int(2)
                                    Stmt 30 [189-246]: Expr: Expr 31 [189-246] [Type Unit]: If:
                                        Expr 32 [192-197] [Type Bool]: Lit: Bool(false)
                                        Block 33 [198-211] [Type Unit]:
                                            Stmt 34 [199-204]: Semi: Expr 35 [199-203] [Type Unit]: Call:
                                                Expr 36 [199-200] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 37 [201-202] [Type Int]: Lit: Int(3)
                                            Stmt 38 [205-210]: Semi: Expr 39 [205-209] [Type Unit]: Call:
                                                Expr 40 [205-206] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 41 [207-208] [Type Int]: Lit: Int(4)
                                        Expr 42 [212-246] [Type Unit]: Expr Block: Block 43 [217-246] [Type Unit]:
                                            Stmt 44 [218-223]: Semi: Expr 45 [218-222] [Type Unit]: Call:
                                                Expr 46 [218-219] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 47 [220-221] [Type Int]: Lit: Int(5)
                                            Stmt 48 [224-239]: Local (Immutable):
                                                Pat 49 [228-231] [Type Bool]: Bind: Ident 50 [228-231] "val"
                                                Expr 51 [234-238] [Type Bool]: Lit: Bool(true)
                                            Stmt 52 [240-245]: Semi: Expr 53 [240-244] [Type Unit]: Call:
                                                Expr 54 [240-241] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 55 [242-243] [Type Int]: Lit: Int(6)
                                    Stmt 56 [255-260]: Semi: Expr 57 [255-259] [Type Unit]: Call:
                                        Expr 58 [255-256] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 59 [257-258] [Type Int]: Lit: Int(7)
                            SpecDecl _id_ [67-266] (Adj): Impl:
                                Pat _id_ [67-266] [Type Qubit]: Elided
                                Block 9 [104-266] [Type Unit]:
                                    Stmt 14 [128-166]: Local (Immutable):
                                        Pat 15 [132-135] [Type Bool]: Bind: Ident 16 [132-135] "val"
                                        Expr 17 [138-165] [Type Bool]: If:
                                            Expr 18 [141-145] [Type Bool]: Lit: Bool(true)
                                            Block 19 [146-153] [Type Bool]:
                                                Stmt 20 [147-152]: Expr: Expr 21 [147-152] [Type Bool]: Lit: Bool(false)
                                            Expr 22 [154-165] [Type Bool]: Expr Block: Block 23 [159-165] [Type Bool]:
                                                Stmt 24 [160-164]: Expr: Expr 25 [160-164] [Type Bool]: Lit: Bool(true)
                                    Stmt 56 [255-260]: Semi: Expr 57 [255-259] [Type Unit]: Call:
                                        Expr _id_ [255-256] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 58 [255-256] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 59 [257-258] [Type Int]: Lit: Int(7)
                                    Stmt 30 [189-246]: Expr: Expr 31 [189-246] [Type Unit]: If:
                                        Expr 32 [192-197] [Type Bool]: Lit: Bool(false)
                                        Block 33 [198-211] [Type Unit]:
                                            Stmt 38 [205-210]: Semi: Expr 39 [205-209] [Type Unit]: Call:
                                                Expr _id_ [205-206] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 40 [205-206] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 41 [207-208] [Type Int]: Lit: Int(4)
                                            Stmt 34 [199-204]: Semi: Expr 35 [199-203] [Type Unit]: Call:
                                                Expr _id_ [199-200] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 36 [199-200] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 37 [201-202] [Type Int]: Lit: Int(3)
                                        Expr 42 [212-246] [Type Unit]: Expr Block: Block 43 [217-246] [Type Unit]:
                                            Stmt 48 [224-239]: Local (Immutable):
                                                Pat 49 [228-231] [Type Bool]: Bind: Ident 50 [228-231] "val"
                                                Expr 51 [234-238] [Type Bool]: Lit: Bool(true)
                                            Stmt 52 [240-245]: Semi: Expr 53 [240-244] [Type Unit]: Call:
                                                Expr _id_ [240-241] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 54 [240-241] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 55 [242-243] [Type Int]: Lit: Int(6)
                                            Stmt 44 [218-223]: Semi: Expr 45 [218-222] [Type Unit]: Call:
                                                Expr _id_ [218-219] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                    Expr 46 [218-219] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 47 [220-221] [Type Int]: Lit: Int(5)
                                    Stmt 26 [175-180]: Semi: Expr 27 [175-179] [Type Unit]: Call:
                                        Expr _id_ [175-176] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 28 [175-176] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 29 [177-178] [Type Int]: Lit: Int(2)
                                    Stmt 10 [114-119]: Semi: Expr 11 [114-118] [Type Unit]: Call:
                                        Expr _id_ [114-115] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 12 [114-115] [Type (Int => Unit is Adj)]: Var: Item 1
                                        Expr 13 [116-117] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 26 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-181] (Public):
                    Parent: 0
                    Callable 5 [67-181] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-181] (Body): Impl:
                                Pat _id_ [104-181] [Type Qubit]: Elided
                                Block 9 [104-181] [Type Unit]:
                                    Stmt 10 [114-175]: Expr: Expr 11 [114-175] [Type Unit]: For:
                                        Pat 12 [118-119] [Type Int]: Bind: Ident 13 [118-119] "i"
                                        Expr 14 [123-127] [Type Range]: Range:
                                            Expr 15 [123-124] [Type Int]: Lit: Int(0)
                                            <no step>
                                            Expr 16 [126-127] [Type Int]: Lit: Int(5)
                                        Block 17 [128-175] [Type Unit]:
                                            Stmt 18 [142-147]: Semi: Expr 19 [142-146] [Type Unit]: Call:
                                                Expr 20 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 21 [144-145] [Type Int]: Lit: Int(1)
                                            Stmt 22 [160-165]: Semi: Expr 23 [160-164] [Type Unit]: Call:
                                                Expr 24 [160-161] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 25 [162-163] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-181] (Adj): Impl:
                                Pat _id_ [67-181] [Type Qubit]: Elided
                                Block 9 [104-181] [Type Unit]:
                                    Stmt 10 [114-175]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type Range]: Bind: Ident 27 [0-0] "generated_range"
                                            Expr 14 [123-127] [Type Range]: Range:
                                                Expr 15 [123-124] [Type Int]: Lit: Int(0)
                                                <no step>
                                                Expr 16 [126-127] [Type Int]: Lit: Int(5)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                            Pat 12 [118-119] [Type Int]: Bind: Ident 13 [118-119] "i"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Add):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                        Prim(Start)
                                                    Expr _id_ [0-0] [Type Int]: BinOp (Mul):
                                                        Expr _id_ [0-0] [Type Int]: BinOp (Div):
                                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                                    Prim(End)
                                                                Expr _id_ [0-0] [Type Int]: Field:
                                                                    Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                                    Prim(Start)
                                                            Expr _id_ [0-0] [Type Int]: Field:
                                                                Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                                Prim(Step)
                                                        Expr _id_ [0-0] [Type Int]: Field:
                                                            Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                            Prim(Step)
                                                Expr _id_ [0-0] [Type Int]: UnOp (Neg):
                                                    Expr _id_ [0-0] [Type Int]: Field:
                                                        Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                        Prim(Step)
                                                Expr _id_ [0-0] [Type Int]: Field:
                                                    Expr _id_ [0-0] [Type Range]: Var: Local 27
                                                    Prim(Start)
                                            Block 17 [128-175] [Type Unit]:
                                                Stmt 22 [160-165]: Semi: Expr 23 [160-164] [Type Unit]: Call:
                                                    Expr _id_ [160-161] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 24 [160-161] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 25 [162-163] [Type Int]: Lit: Int(2)
                                                Stmt 18 [142-147]: Semi: Expr 19 [142-146] [Type Unit]: Call:
                                                    Expr _id_ [142-143] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 20 [142-143] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 21 [144-145] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 27 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-188] (Public):
                    Parent: 0
                    Callable 5 [67-188] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-188] (Body): Impl:
                                Pat _id_ [104-188] [Type Qubit]: Elided
                                Block 9 [104-188] [Type Unit]:
                                    Stmt 10 [114-182]: Expr: Expr 11 [114-182] [Type Unit]: For:
                                        Pat 12 [118-121] [Type Int]: Bind: Ident 13 [118-121] "val"
                                        Expr 14 [125-134] [Type (Int)[]]: Array:
                                            Expr 15 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 16 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 17 [132-133] [Type Int]: Lit: Int(2)
                                        Block 18 [135-182] [Type Unit]:
                                            Stmt 19 [149-154]: Semi: Expr 20 [149-153] [Type Unit]: Call:
                                                Expr 21 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 22 [151-152] [Type Int]: Lit: Int(1)
                                            Stmt 23 [167-172]: Semi: Expr 24 [167-171] [Type Unit]: Call:
                                                Expr 25 [167-168] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 26 [169-170] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [67-188] (Adj): Impl:
                                Pat _id_ [67-188] [Type Qubit]: Elided
                                Block 9 [104-188] [Type Unit]:
                                    Stmt 10 [114-182]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 28 [0-0] "generated_array"
                                            Expr 14 [125-134] [Type (Int)[]]: Array:
                                                Expr 15 [126-127] [Type Int]: Lit: Int(0)
                                                Expr 16 [129-130] [Type Int]: Lit: Int(1)
                                                Expr 17 [132-133] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                            Pat _id_ [0-0] [Type Int]: Bind: Ident 29 [0-0] "generated_index"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                    Expr _id_ [0-0] [Type Int]: Call:
                                                        Expr _id_ [0-0] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 28
                                                    Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                            Block 18 [135-182] [Type Unit]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat 12 [118-121] [Type Int]: Bind: Ident 13 [118-121] "val"
                                                    Expr _id_ [0-0] [Type Int]: Index:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 28
                                                        Expr _id_ [0-0] [Type Int]: Var: Local 29
                                                Stmt 23 [167-172]: Semi: Expr 24 [167-171] [Type Unit]: Call:
                                                    Expr _id_ [167-168] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 25 [167-168] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 26 [169-170] [Type Int]: Lit: Int(2)
                                                Stmt 19 [149-154]: Semi: Expr 20 [149-153] [Type Unit]: Call:
                                                    Expr _id_ [149-150] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 21 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 22 [151-152] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 48 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62] (Public):
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [60-62] (Body): Impl:
                                Pat _id_ [60-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                            SpecDecl _id_ [21-62] (Adj): Impl:
                                Pat _id_ [21-62] [Type Int]: Elided
                                Block 4 [60-62]: <empty>
                Item 2 [67-318] (Public):
                    Parent: 0
                    Callable 5 [67-318] (Operation):
                        name: Ident 6 [77-78] "A"
                        input: Pat 7 [79-88] [Type Qubit]: Bind: Ident 8 [79-80] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [104-318] (Body): Impl:
                                Pat _id_ [104-318] [Type Qubit]: Elided
                                Block 9 [104-318] [Type Unit]:
                                    Stmt 10 [114-312]: Expr: Expr 11 [114-312] [Type Unit]: For:
                                        Pat 12 [118-121] [Type Int]: Bind: Ident 13 [118-121] "val"
                                        Expr 14 [125-134] [Type (Int)[]]: Array:
                                            Expr 15 [126-127] [Type Int]: Lit: Int(0)
                                            Expr 16 [129-130] [Type Int]: Lit: Int(1)
                                            Expr 17 [132-133] [Type Int]: Lit: Int(2)
                                        Block 18 [135-312] [Type Unit]:
                                            Stmt 19 [149-154]: Semi: Expr 20 [149-153] [Type Unit]: Call:
                                                Expr 21 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 22 [151-152] [Type Int]: Lit: Int(1)
                                            Stmt 23 [167-197]: Local (Immutable):
                                                Pat 24 [171-174] [Type (Bool)[]]: Bind: Ident 25 [171-174] "arr"
                                                Expr 26 [177-196] [Type (Bool)[]]: Array:
                                                    Expr 27 [178-182] [Type Bool]: Lit: Bool(true)
                                                    Expr 28 [184-189] [Type Bool]: Lit: Bool(false)
                                                    Expr 29 [191-195] [Type Bool]: Lit: Bool(true)
                                            Stmt 30 [210-284]: Expr: Expr 31 [210-284] [Type Unit]: For:
                                                Pat 32 [214-217] [Type Bool]: Bind: Ident 33 [214-217] "val"
                                                Expr 34 [221-224] [Type (Bool)[]]: Var: Local 25
                                                Block 35 [225-284] [Type Unit]:
                                                    Stmt 36 [243-248]: Semi: Expr 37 [243-247] [Type Unit]: Call:
                                                        Expr 38 [243-244] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 39 [245-246] [Type Int]: Lit: Int(2)
                                                    Stmt 40 [265-270]: Semi: Expr 41 [265-269] [Type Unit]: Call:
                                                        Expr 42 [265-266] [Type (Int => Unit is Adj)]: Var: Item 1
                                                        Expr 43 [267-268] [Type Int]: Lit: Int(3)
                                            Stmt 44 [297-302]: Semi: Expr 45 [297-301] [Type Unit]: Call:
                                                Expr 46 [297-298] [Type (Int => Unit is Adj)]: Var: Item 1
                                                Expr 47 [299-300] [Type Int]: Lit: Int(4)
                            SpecDecl _id_ [67-318] (Adj): Impl:
                                Pat _id_ [67-318] [Type Qubit]: Elided
                                Block 9 [104-318] [Type Unit]:
                                    Stmt 10 [114-312]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                        Stmt _id_ [0-0]: Local (Immutable):
                                            Pat _id_ [0-0] [Type (Int)[]]: Bind: Ident 51 [0-0] "generated_array"
                                            Expr 14 [125-134] [Type (Int)[]]: Array:
                                                Expr 15 [126-127] [Type Int]: Lit: Int(0)
                                                Expr 16 [129-130] [Type Int]: Lit: Int(1)
                                                Expr 17 [132-133] [Type Int]: Lit: Int(2)
                                        Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                            Pat _id_ [0-0] [Type Int]: Bind: Ident 52 [0-0] "generated_index"
                                            Expr _id_ [0-0] [Type Range]: Range:
                                                Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                    Expr _id_ [0-0] [Type Int]: Call:
                                                        Expr _id_ [0-0] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 51
                                                    Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                            Block 18 [135-312] [Type Unit]:
                                                Stmt _id_ [0-0]: Local (Immutable):
                                                    Pat 12 [118-121] [Type Int]: Bind: Ident 13 [118-121] "val"
                                                    Expr _id_ [0-0] [Type Int]: Index:
                                                        Expr _id_ [0-0] [Type (Int)[]]: Var: Local 51
                                                        Expr _id_ [0-0] [Type Int]: Var: Local 52
                                                Stmt 23 [167-197]: Local (Immutable):
                                                    Pat 24 [171-174] [Type (Bool)[]]: Bind: Ident 25 [171-174] "arr"
                                                    Expr 26 [177-196] [Type (Bool)[]]: Array:
                                                        Expr 27 [178-182] [Type Bool]: Lit: Bool(true)
                                                        Expr 28 [184-189] [Type Bool]: Lit: Bool(false)
                                                        Expr 29 [191-195] [Type Bool]: Lit: Bool(true)
                                                Stmt 44 [297-302]: Semi: Expr 45 [297-301] [Type Unit]: Call:
                                                    Expr _id_ [297-298] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 46 [297-298] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 47 [299-300] [Type Int]: Lit: Int(4)
                                                Stmt 30 [210-284]: Expr: Expr _id_ [0-0] [Type Unit]: Expr Block: Block _id_ [0-0] [Type Unit]:
                                                    Stmt _id_ [0-0]: Local (Immutable):
                                                        Pat _id_ [0-0] [Type (Bool)[]]: Bind: Ident 49 [0-0] "generated_array"
                                                        Expr 34 [221-224] [Type (Bool)[]]: Var: Local 25
                                                    Stmt _id_ [0-0]: Expr: Expr _id_ [0-0] [Type Unit]: For:
                                                        Pat _id_ [0-0] [Type Int]: Bind: Ident 50 [0-0] "generated_index"
                                                        Expr _id_ [0-0] [Type Range]: Range:
                                                            Expr _id_ [0-0] [Type Int]: BinOp (Sub):
                                                                Expr _id_ [0-0] [Type Int]: Call:
                                                                    Expr _id_ [0-0] [Type (('T)[] -> Int)]: Var: Item 1 (Package 0)
                                                                    Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 49
                                                                Expr _id_ [0-0] [Type Int]: Lit: Int(1)
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(-1)
                                                            Expr _id_ [0-0] [Type Int]: Lit: Int(0)
                                                        Block 35 [225-284] [Type Unit]:
                                                            Stmt _id_ [0-0]: Local (Immutable):
                                                                Pat 32 [214-217] [Type Bool]: Bind: Ident 33 [214-217] "val"
                                                                Expr _id_ [0-0] [Type Bool]: Index:
                                                                    Expr _id_ [0-0] [Type (Bool)[]]: Var: Local 49
                                                                    Expr _id_ [0-0] [Type Int]: Var: Local 50
                                                            Stmt 40 [265-270]: Semi: Expr 41 [265-269] [Type Unit]: Call:
                                                                Expr _id_ [265-266] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                                    Expr 42 [265-266] [Type (Int => Unit is Adj)]: Var: Item 1
                                                                Expr 43 [267-268] [Type Int]: Lit: Int(3)
                                                            Stmt 36 [243-248]: Semi: Expr 37 [243-247] [Type Unit]: Call:
                                                                Expr _id_ [243-244] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                                    Expr 38 [243-244] [Type (Int => Unit is Adj)]: Var: Item 1
                                                                Expr 39 [245-246] [Type Int]: Lit: Int(2)
                                                Stmt 19 [149-154]: Semi: Expr 20 [149-153] [Type Unit]: Call:
                                                    Expr _id_ [149-150] [Type (Int => Unit is Adj)]: UnOp (Functor Adj):
                                                        Expr 21 [149-150] [Type (Int => Unit is Adj)]: Var: Item 1
                                                    Expr 22 [151-152] [Type Int]: Lit: Int(1)"#]],
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
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 21 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 22 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                Item 2 [73-156] (Public):
                    Parent: 0
                    Callable 5 [73-156] (Operation):
                        name: Ident 6 [83-84] "A"
                        input: Pat 7 [85-94] [Type Qubit]: Bind: Ident 8 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: Specializations:
                            SpecDecl 9 [126-150] (Body): Impl:
                                Pat 10 [131-134] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-156] (Adj): Impl:
                                Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                            SpecDecl _id_ [73-156] (Ctl): Impl:
                                Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 23 [73-156] "ctls"
                                    Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 23
                                            Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 23
                                            Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-156] (CtlAdj): Impl:
                                Pat _id_ [73-156] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-156] [Type (Qubit)[]]: Bind: Ident 24 [73-156] "ctls"
                                    Pat _id_ [73-156] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr _id_ [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 24
                                            Expr 19 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr _id_ [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                                Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 24
                                            Expr 15 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [33-44] [Type Int]: Bind: Ident 3 [33-38] "input"
                        output: Unit
                        functors: Adj + Ctl
                        body: Specializations:
                            SpecDecl _id_ [66-68] (Body): Impl:
                                Pat _id_ [66-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Adj): Impl:
                                Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (Ctl): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 22 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                            SpecDecl _id_ [21-68] (CtlAdj): Impl:
                                Pat _id_ [21-68] [Type ((Qubit)[], Int)]: Tuple:
                                    Pat _id_ [21-68] [Type (Qubit)[]]: Bind: Ident 23 [21-68] "ctls"
                                    Pat _id_ [21-68] [Type Int]: Elided
                                Block 4 [66-68]: <empty>
                Item 2 [73-191] (Public):
                    Parent: 0
                    Callable 5 [73-191] (Operation):
                        name: Ident 6 [83-84] "A"
                        input: Pat 7 [85-94] [Type Qubit]: Bind: Ident 8 [85-86] "q"
                        output: Unit
                        functors: Adj + Ctl
                        body: Specializations:
                            SpecDecl 9 [126-150] (Body): Impl:
                                Pat 10 [131-134] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl _id_ [73-191] (Adj): Impl:
                                Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr _id_ [143-144] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr _id_ [137-138] [Type (Int => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type Int]: Lit: Int(1)
                            SpecDecl _id_ [73-191] (Ctl): Impl:
                                Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 24 [73-191] "ctls"
                                    Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr 14 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 24
                                            Expr 15 [139-140] [Type Int]: Lit: Int(1)
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr 18 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                            Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 24
                                            Expr 19 [145-146] [Type Int]: Lit: Int(2)
                            SpecDecl 20 [159-185] (CtlAdj): Impl:
                                Pat _id_ [73-191] [Type ((Qubit)[], Qubit)]: Tuple:
                                    Pat _id_ [73-191] [Type (Qubit)[]]: Bind: Ident 24 [73-191] "ctls"
                                    Pat _id_ [73-191] [Type Qubit]: Elided
                                Block 11 [135-150] [Type Unit]:
                                    Stmt 16 [143-148]: Semi: Expr 17 [143-147] [Type Unit]: Call:
                                        Expr _id_ [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 18 [143-144] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 18 [143-144] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 19 [145-146] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [145-146] [Type (Qubit)[]]: Var: Local 24
                                            Expr 19 [145-146] [Type Int]: Lit: Int(2)
                                    Stmt 12 [137-142]: Semi: Expr 13 [137-141] [Type Unit]: Call:
                                        Expr _id_ [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Adj):
                                            Expr 14 [137-138] [Type (((Qubit)[], Int) => Unit is Adj + Ctl)]: UnOp (Functor Ctl):
                                                Expr 14 [137-138] [Type (Int => Unit is Adj + Ctl)]: Var: Item 1
                                        Expr 15 [139-140] [Type ((Qubit)[], Int)]: Tuple:
                                            Expr _id_ [139-140] [Type (Qubit)[]]: Var: Local 24
                                            Expr 15 [139-140] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 28 [10-11] "A"): Item 1, Item 2, Item 3
                Item 1 [18-55] (Public):
                    Parent: 0
                    Callable 0 [18-55] (Operation):
                        name: Ident 1 [28-29] "X"
                        input: Pat 2 [30-39] [Type Qubit]: Bind: Ident 3 [30-31] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [53-55] (Body): Impl:
                                Pat _id_ [53-55] [Type Qubit]: Elided
                                Block 4 [53-55]: <empty>
                            SpecDecl _id_ [18-55] (Adj): Impl:
                                Pat _id_ [18-55] [Type Qubit]: Elided
                                Block 4 [53-55]: <empty>
                Item 2 [60-106] (Public):
                    Parent: 0
                    Callable 5 [60-106] (Operation):
                        name: Ident 6 [70-73] "Foo"
                        input: Pat 7 [74-97] [Type (Qubit => Unit is Adj)]: Bind: Ident 8 [74-76] "op"
                        output: Unit
                        functors: empty set
                        body: Block: Block 9 [104-106]: <empty>
                Item 3 [111-151] (Public):
                    Parent: 0
                    Callable 10 [111-151] (Operation):
                        name: Ident 11 [121-124] "Bar"
                        input: Pat 12 [124-126] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: Block: Block 13 [132-151] [Type Unit]:
                            Stmt 14 [134-149]: Semi: Expr 15 [134-148] [Type Unit]: Call:
                                Expr 16 [134-137] [Type ((Qubit => Unit is Adj) => Unit)]: Var: Item 2
                                Expr 17 [138-147] [Type (Qubit => Unit is Adj)]: Closure([], 4)
                Item 4 [138-147] (Internal):
                    Parent: 3
                    Callable 24 [138-147] (Operation):
                        name: Ident 25 [138-147] "lambda"
                        input: Pat 23 [138-147] [Type (Qubit,)]: Tuple:
                            Pat 18 [138-139] [Type Qubit]: Bind: Ident 19 [138-139] "q"
                        output: Unit
                        functors: Adj
                        body: Specializations:
                            SpecDecl _id_ [143-147] (Body): Impl:
                                Pat _id_ [143-147] [Type (Qubit,)]: Elided
                                Block 26 [143-147] [Type Unit]:
                                    Stmt 27 [143-147]: Expr: Expr 20 [143-147] [Type Unit]: Call:
                                        Expr 21 [143-144] [Type (Qubit => Unit is Adj)]: Var: Item 1
                                        Expr 22 [145-146] [Type Qubit]: Var: Local 19
                            SpecDecl _id_ [138-147] (Adj): Impl:
                                Pat _id_ [138-147] [Type (Qubit,)]: Elided
                                Block 26 [143-147] [Type Unit]:
                                    Stmt 27 [143-147]: Expr: Expr 20 [143-147] [Type Unit]: Call:
                                        Expr _id_ [143-144] [Type (Qubit => Unit is Adj)]: UnOp (Functor Adj):
                                            Expr 21 [143-144] [Type (Qubit => Unit is Adj)]: Var: Item 1
                                        Expr 22 [145-146] [Type Qubit]: Var: Local 19"#]],
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
