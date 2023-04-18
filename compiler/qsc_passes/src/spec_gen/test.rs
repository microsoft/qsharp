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
                            input: Pat 6 [32-43]: Paren:
                                Pat 7 [33-42]: Bind:
                                    Ident 8 [33-34] "q"
                                    Type 9 [37-42]: Prim (Qubit)
                            output: Type 10 [46-50]: Unit
                            functors: Functor Expr 11 [54-57]: Ctl
                            body: Specializations:
                                SpecDecl 12 [68-79] (Body): Impl:
                                    Pat 13 [73-76]: Elided
                                    Block 14 [77-79]: <empty>
                                SpecDecl 15 [88-113] (Ctl): Impl:
                                    Pat 16 [99-110]: Tuple:
                                        Pat 17 [100-104]: Bind:
                                            Ident 18 [100-104] "ctls"
                                        Pat 19 [106-109]: Elided
                                    Block 20 [111-113]: <empty>
                    Item 21 [124-182]:
                        Callable 22 [124-182] (Operation):
                            name: Ident 23 [134-135] "B"
                            input: Pat 24 [135-146]: Paren:
                                Pat 25 [136-145]: Bind:
                                    Ident 26 [136-137] "q"
                                    Type 27 [140-145]: Prim (Qubit)
                            output: Type 28 [149-153]: Unit
                            functors: Functor Expr 29 [157-160]: Ctl
                            body: Specializations:
                                SpecDecl 36 [161-182] (Body): Impl:
                                    Pat 37 [161-182]: Elided
                                    Block 30 [161-182]:
                                        Stmt 31 [171-176]: Semi: Expr 32 [171-175]: Call:
                                            Expr 33 [171-172]: Name: Internal(NodeId(5))
                                            Expr 34 [172-175]: Paren: Expr 35 [173-174]: Name: Internal(NodeId(26))
                                SpecDecl 38 [124-182] (Ctl): Impl:
                                    Pat 44 [124-182]: Tuple:
                                        Pat 40 [124-182]: Bind:
                                            Ident 39 [124-182] "ctls"
                                        Pat 45 [124-182]: Elided
                                    Block 30 [161-182]:
                                        Stmt 31 [171-176]: Semi: Expr 32 [171-175]: Call:
                                            Expr 41 [171-172]: UnOp (Functor Ctl):
                                                Expr 33 [171-172]: Name: Internal(NodeId(5))
                                            Expr 42 [172-175]: Tuple:
                                                Expr 43 [172-175]: Name: Internal(NodeId(39))
                                                Expr 34 [172-175]: Paren: Expr 35 [173-174]: Name: Internal(NodeId(26))"#]],
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
                            input: Pat 6 [32-43]: Paren:
                                Pat 7 [33-42]: Bind:
                                    Ident 8 [33-34] "q"
                                    Type 9 [37-42]: Prim (Qubit)
                            output: Type 10 [46-50]: Unit
                            functors: Functor Expr 11 [54-63]: BinOp Union: (Functor Expr 12 [54-57]: Ctl) (Functor Expr 13 [60-63]: Adj)
                            body: Specializations:
                                SpecDecl 14 [74-85] (Body): Impl:
                                    Pat 15 [79-82]: Elided
                                    Block 16 [83-85]: <empty>
                                SpecDecl 17 [94-108] (Adj): Impl:
                                    Pat 18 [102-105]: Elided
                                    Block 19 [106-108]: <empty>
                                SpecDecl 20 [117-142] (Ctl): Impl:
                                    Pat 21 [128-139]: Tuple:
                                        Pat 22 [129-133]: Bind:
                                            Ident 23 [129-133] "ctls"
                                        Pat 24 [135-138]: Elided
                                    Block 25 [140-142]: <empty>
                                SpecDecl 54 [21-148] (CtlAdj): Impl:
                                    Pat 59 [21-148]: Tuple:
                                        Pat 58 [21-148]: Bind:
                                            Ident 57 [21-148] "ctls"
                                        Pat 60 [21-148]: Elided
                                    Block 19 [106-108]: <empty>
                    Item 26 [153-308]:
                        Callable 27 [153-308] (Operation):
                            name: Ident 28 [163-164] "B"
                            input: Pat 29 [164-175]: Paren:
                                Pat 30 [165-174]: Bind:
                                    Ident 31 [165-166] "q"
                                    Type 32 [169-174]: Prim (Qubit)
                            output: Type 33 [178-182]: Unit
                            functors: Functor Expr 34 [186-195]: BinOp Union: (Functor Expr 35 [186-189]: Ctl) (Functor Expr 36 [192-195]: Adj)
                            body: Specializations:
                                SpecDecl 37 [206-244] (Body): Impl:
                                    Pat 38 [211-214]: Elided
                                    Block 39 [215-244]:
                                        Stmt 40 [229-234]: Semi: Expr 41 [229-233]: Call:
                                            Expr 42 [229-230]: Name: Internal(NodeId(5))
                                            Expr 43 [230-233]: Paren: Expr 44 [231-232]: Name: Internal(NodeId(31))
                                SpecDecl 45 [253-302] (Adj): Impl:
                                    Pat 46 [261-264]: Elided
                                    Block 47 [265-302]:
                                        Stmt 48 [279-292]: Semi: Expr 49 [279-291]: Call:
                                            Expr 50 [279-288]: UnOp (Functor Adj):
                                                Expr 51 [287-288]: Name: Internal(NodeId(5))
                                            Expr 52 [288-291]: Paren: Expr 53 [289-290]: Name: Internal(NodeId(31))
                                SpecDecl 55 [153-308] (Ctl): Impl:
                                    Pat 66 [153-308]: Tuple:
                                        Pat 62 [153-308]: Bind:
                                            Ident 61 [153-308] "ctls"
                                        Pat 67 [153-308]: Elided
                                    Block 39 [215-244]:
                                        Stmt 40 [229-234]: Semi: Expr 41 [229-233]: Call:
                                            Expr 63 [229-230]: UnOp (Functor Ctl):
                                                Expr 42 [229-230]: Name: Internal(NodeId(5))
                                            Expr 64 [230-233]: Tuple:
                                                Expr 65 [230-233]: Name: Internal(NodeId(61))
                                                Expr 43 [230-233]: Paren: Expr 44 [231-232]: Name: Internal(NodeId(31))
                                SpecDecl 56 [153-308] (CtlAdj): Impl:
                                    Pat 73 [153-308]: Tuple:
                                        Pat 69 [153-308]: Bind:
                                            Ident 68 [153-308] "ctls"
                                        Pat 74 [153-308]: Elided
                                    Block 47 [265-302]:
                                        Stmt 48 [279-292]: Semi: Expr 49 [279-291]: Call:
                                            Expr 70 [279-288]: UnOp (Functor Ctl):
                                                Expr 50 [279-288]: UnOp (Functor Adj):
                                                    Expr 51 [287-288]: Name: Internal(NodeId(5))
                                            Expr 71 [288-291]: Tuple:
                                                Expr 72 [288-291]: Name: Internal(NodeId(68))
                                                Expr 52 [288-291]: Paren: Expr 53 [289-290]: Name: Internal(NodeId(31))"#]],
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
                            input: Pat 6 [32-43]: Paren:
                                Pat 7 [33-42]: Bind:
                                    Ident 8 [33-34] "q"
                                    Type 9 [37-42]: Prim (Qubit)
                            output: Type 10 [46-50]: Unit
                            functors: Functor Expr 11 [54-57]: Ctl
                            body: Specializations:
                                SpecDecl 12 [68-79] (Body): Impl:
                                    Pat 13 [73-76]: Elided
                                    Block 14 [77-79]: <empty>
                                SpecDecl 15 [88-113] (Ctl): Impl:
                                    Pat 16 [99-110]: Tuple:
                                        Pat 17 [100-104]: Bind:
                                            Ident 18 [100-104] "ctls"
                                        Pat 19 [106-109]: Elided
                                    Block 20 [111-113]: <empty>
                    Item 21 [124-257]:
                        Callable 22 [124-257] (Operation):
                            name: Ident 23 [134-135] "B"
                            input: Pat 24 [135-146]: Paren:
                                Pat 25 [136-145]: Bind:
                                    Ident 26 [136-137] "q"
                                    Type 27 [140-145]: Prim (Qubit)
                            output: Type 28 [149-153]: Unit
                            functors: Functor Expr 29 [157-160]: Ctl
                            body: Specializations:
                                SpecDecl 45 [161-257] (Body): Impl:
                                    Pat 46 [161-257]: Elided
                                    Block 30 [161-257]:
                                        Stmt 31 [171-251]: Expr: Expr 32 [171-251]: Conjugate:
                                            Block 33 [178-207]:
                                                Stmt 34 [192-197]: Semi: Expr 35 [192-196]: Call:
                                                    Expr 36 [192-193]: Name: Internal(NodeId(5))
                                                    Expr 37 [193-196]: Paren: Expr 38 [194-195]: Name: Internal(NodeId(26))
                                            Block 39 [222-251]:
                                                Stmt 40 [236-241]: Semi: Expr 41 [236-240]: Call:
                                                    Expr 42 [236-237]: Name: Internal(NodeId(5))
                                                    Expr 43 [237-240]: Paren: Expr 44 [238-239]: Name: Internal(NodeId(26))
                                SpecDecl 47 [124-257] (Ctl): Impl:
                                    Pat 53 [124-257]: Tuple:
                                        Pat 49 [124-257]: Bind:
                                            Ident 48 [124-257] "ctls"
                                        Pat 54 [124-257]: Elided
                                    Block 30 [161-257]:
                                        Stmt 31 [171-251]: Expr: Expr 32 [171-251]: Conjugate:
                                            Block 33 [178-207]:
                                                Stmt 34 [192-197]: Semi: Expr 35 [192-196]: Call:
                                                    Expr 36 [192-193]: Name: Internal(NodeId(5))
                                                    Expr 37 [193-196]: Paren: Expr 38 [194-195]: Name: Internal(NodeId(26))
                                            Block 39 [222-251]:
                                                Stmt 40 [236-241]: Semi: Expr 41 [236-240]: Call:
                                                    Expr 50 [236-237]: UnOp (Functor Ctl):
                                                        Expr 42 [236-237]: Name: Internal(NodeId(5))
                                                    Expr 51 [237-240]: Tuple:
                                                        Expr 52 [237-240]: Name: Internal(NodeId(48))
                                                        Expr 43 [237-240]: Paren: Expr 44 [238-239]: Name: Internal(NodeId(26))"#]],
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
                            input: Pat 6 [33-35]: Unit
                            output: Type 7 [38-42]: Unit
                            body: Block: Block 8 [43-45]: <empty>
                    Item 9 [50-80]:
                        Callable 10 [50-80] (Operation):
                            name: Ident 11 [60-61] "A"
                            input: Pat 12 [61-63]: Unit
                            output: Type 13 [66-70]: Unit
                            functors: Functor Expr 14 [74-77]: Ctl
                            body: Specializations:
                                SpecDecl 31 [78-80] (Body): Impl:
                                    Pat 32 [78-80]: Elided
                                    Block 15 [78-80]: <empty>
                                SpecDecl 33 [50-80] (Ctl): Impl:
                                    Pat 39 [50-80]: Tuple:
                                        Pat 38 [50-80]: Bind:
                                            Ident 37 [50-80] "ctls"
                                        Pat 40 [50-80]: Elided
                                    Block 15 [78-80]: <empty>
                    Item 16 [85-148]:
                        Callable 17 [85-148] (Operation):
                            name: Ident 18 [95-96] "B"
                            input: Pat 19 [96-98]: Unit
                            output: Type 20 [101-105]: Unit
                            functors: Functor Expr 21 [109-112]: Ctl
                            body: Specializations:
                                SpecDecl 34 [113-148] (Body): Impl:
                                    Pat 35 [113-148]: Elided
                                    Block 22 [113-148]:
                                        Stmt 23 [123-129]: Semi: Expr 24 [123-128]: Call:
                                            Expr 25 [123-126]: Name: Internal(NodeId(5))
                                            Expr 26 [126-128]: Unit
                                        Stmt 27 [138-142]: Semi: Expr 28 [138-141]: Call:
                                            Expr 29 [138-139]: Name: Internal(NodeId(11))
                                            Expr 30 [139-141]: Unit
                                SpecDecl 36 [85-148] (Ctl): Impl:
                                    Pat 46 [85-148]: Tuple:
                                        Pat 42 [85-148]: Bind:
                                            Ident 41 [85-148] "ctls"
                                        Pat 47 [85-148]: Elided
                                    Block 22 [113-148]:
                                        Stmt 23 [123-129]: Semi: Expr 24 [123-128]: Call:
                                            Expr 25 [123-126]: Name: Internal(NodeId(5))
                                            Expr 26 [126-128]: Unit
                                        Stmt 27 [138-142]: Semi: Expr 28 [138-141]: Call:
                                            Expr 43 [138-139]: UnOp (Functor Ctl):
                                                Expr 29 [138-139]: Name: Internal(NodeId(11))
                                            Expr 44 [139-141]: Tuple:
                                                Expr 45 [139-141]: Name: Internal(NodeId(41))
                                                Expr 30 [139-141]: Unit"#]],
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
                            input: Pat 6 [32-45]: Paren:
                                Pat 7 [33-44]: Bind:
                                    Ident 8 [33-38] "input"
                                    Type 9 [41-44]: Prim (Int)
                            output: Type 10 [48-52]: Unit
                            functors: Functor Expr 11 [56-59]: Adj
                            body: Specializations:
                                SpecDecl 36 [60-62] (Body): Impl:
                                    Pat 37 [60-62]: Elided
                                    Block 12 [60-62]: <empty>
                                SpecDecl 38 [21-62] (Adj): Gen: Invert
                    Item 13 [67-166]:
                        Callable 14 [67-166] (Operation):
                            name: Ident 15 [77-78] "A"
                            input: Pat 16 [78-89]: Paren:
                                Pat 17 [79-88]: Bind:
                                    Ident 18 [79-80] "q"
                                    Type 19 [83-88]: Prim (Qubit)
                            output: Type 20 [92-96]: Unit
                            functors: Functor Expr 21 [100-103]: Adj
                            body: Specializations:
                                SpecDecl 22 [114-138] (Body): Impl:
                                    Pat 23 [119-122]: Elided
                                    Block 24 [123-138]:
                                        Stmt 25 [125-130]: Semi: Expr 26 [125-129]: Call:
                                            Expr 27 [125-126]: Name: Internal(NodeId(5))
                                            Expr 28 [126-129]: Paren: Expr 29 [127-128]: Lit: Int(1)
                                        Stmt 30 [131-136]: Semi: Expr 31 [131-135]: Call:
                                            Expr 32 [131-132]: Name: Internal(NodeId(5))
                                            Expr 33 [132-135]: Paren: Expr 34 [133-134]: Lit: Int(2)
                                SpecDecl 35 [147-160] (Adj): Impl:
                                    Pat 23 [119-122]: Elided
                                    Block 24 [123-138]:
                                        Stmt 25 [125-130]: Semi: Expr 26 [125-129]: Call:
                                            Expr 27 [125-126]: Name: Internal(NodeId(5))
                                            Expr 28 [126-129]: Paren: Expr 29 [127-128]: Lit: Int(1)
                                        Stmt 30 [131-136]: Semi: Expr 31 [131-135]: Call:
                                            Expr 32 [131-132]: Name: Internal(NodeId(5))
                                            Expr 33 [132-135]: Paren: Expr 34 [133-134]: Lit: Int(2)"#]],
    );
}
