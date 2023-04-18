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
                            output: ()
                            functors: Functor Expr 9 [54-57]: Ctl
                            body: Specializations:
                                SpecDecl 10 [68-79] (Body): Impl:
                                    Pat 11 [73-76]: Elided
                                    Block 12 [77-79]: <empty>
                                SpecDecl 13 [88-113] (Ctl): Impl:
                                    Pat 14 [99-110]: Tuple:
                                        Pat 15 [100-104]: Bind:
                                            Ident 16 [100-104] "ctls"
                                        Pat 17 [106-109]: Elided
                                    Block 18 [111-113]: <empty>
                    Item 19 [124-182]:
                        Callable 20 [124-182] (Operation):
                            name: Ident 21 [134-135] "B"
                            input: Pat 22 [135-146]: Paren:
                                Pat 23 [136-145]: Bind:
                                    Ident 24 [136-137] "q"
                            output: ()
                            functors: Functor Expr 25 [157-160]: Ctl
                            body: Specializations:
                                SpecDecl 32 [161-182] (Body): Impl:
                                    Pat 33 [161-182]: Elided
                                    Block 26 [161-182]:
                                        Stmt 27 [171-176]: Semi: Expr 28 [171-175]: Call:
                                            Expr 29 [171-172]: Name: Internal(NodeId(5))
                                            Expr 30 [172-175]: Paren: Expr 31 [173-174]: Name: Internal(NodeId(24))
                                SpecDecl 34 [124-182] (Ctl): Impl:
                                    Pat 40 [124-182]: Tuple:
                                        Pat 36 [124-182]: Bind:
                                            Ident 35 [124-182] "ctls"
                                        Pat 41 [124-182]: Elided
                                    Block 26 [161-182]:
                                        Stmt 27 [171-176]: Semi: Expr 28 [171-175]: Call:
                                            Expr 37 [171-172]: UnOp (Functor Ctl):
                                                Expr 29 [171-172]: Name: Internal(NodeId(5))
                                            Expr 38 [172-175]: Tuple:
                                                Expr 39 [172-175]: Name: Internal(NodeId(35))
                                                Expr 30 [172-175]: Paren: Expr 31 [173-174]: Name: Internal(NodeId(24))"#]],
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
                            output: ()
                            functors: Functor Expr 9 [54-63]: BinOp Union: (Functor Expr 10 [54-57]: Ctl) (Functor Expr 11 [60-63]: Adj)
                            body: Specializations:
                                SpecDecl 12 [74-85] (Body): Impl:
                                    Pat 13 [79-82]: Elided
                                    Block 14 [83-85]: <empty>
                                SpecDecl 15 [94-108] (Adj): Impl:
                                    Pat 16 [102-105]: Elided
                                    Block 17 [106-108]: <empty>
                                SpecDecl 18 [117-142] (Ctl): Impl:
                                    Pat 19 [128-139]: Tuple:
                                        Pat 20 [129-133]: Bind:
                                            Ident 21 [129-133] "ctls"
                                        Pat 22 [135-138]: Elided
                                    Block 23 [140-142]: <empty>
                                SpecDecl 50 [21-148] (CtlAdj): Impl:
                                    Pat 55 [21-148]: Tuple:
                                        Pat 54 [21-148]: Bind:
                                            Ident 53 [21-148] "ctls"
                                        Pat 56 [21-148]: Elided
                                    Block 17 [106-108]: <empty>
                    Item 24 [153-308]:
                        Callable 25 [153-308] (Operation):
                            name: Ident 26 [163-164] "B"
                            input: Pat 27 [164-175]: Paren:
                                Pat 28 [165-174]: Bind:
                                    Ident 29 [165-166] "q"
                            output: ()
                            functors: Functor Expr 30 [186-195]: BinOp Union: (Functor Expr 31 [186-189]: Ctl) (Functor Expr 32 [192-195]: Adj)
                            body: Specializations:
                                SpecDecl 33 [206-244] (Body): Impl:
                                    Pat 34 [211-214]: Elided
                                    Block 35 [215-244]:
                                        Stmt 36 [229-234]: Semi: Expr 37 [229-233]: Call:
                                            Expr 38 [229-230]: Name: Internal(NodeId(5))
                                            Expr 39 [230-233]: Paren: Expr 40 [231-232]: Name: Internal(NodeId(29))
                                SpecDecl 41 [253-302] (Adj): Impl:
                                    Pat 42 [261-264]: Elided
                                    Block 43 [265-302]:
                                        Stmt 44 [279-292]: Semi: Expr 45 [279-291]: Call:
                                            Expr 46 [279-288]: UnOp (Functor Adj):
                                                Expr 47 [287-288]: Name: Internal(NodeId(5))
                                            Expr 48 [288-291]: Paren: Expr 49 [289-290]: Name: Internal(NodeId(29))
                                SpecDecl 51 [153-308] (Ctl): Impl:
                                    Pat 62 [153-308]: Tuple:
                                        Pat 58 [153-308]: Bind:
                                            Ident 57 [153-308] "ctls"
                                        Pat 63 [153-308]: Elided
                                    Block 35 [215-244]:
                                        Stmt 36 [229-234]: Semi: Expr 37 [229-233]: Call:
                                            Expr 59 [229-230]: UnOp (Functor Ctl):
                                                Expr 38 [229-230]: Name: Internal(NodeId(5))
                                            Expr 60 [230-233]: Tuple:
                                                Expr 61 [230-233]: Name: Internal(NodeId(57))
                                                Expr 39 [230-233]: Paren: Expr 40 [231-232]: Name: Internal(NodeId(29))
                                SpecDecl 52 [153-308] (CtlAdj): Impl:
                                    Pat 69 [153-308]: Tuple:
                                        Pat 65 [153-308]: Bind:
                                            Ident 64 [153-308] "ctls"
                                        Pat 70 [153-308]: Elided
                                    Block 43 [265-302]:
                                        Stmt 44 [279-292]: Semi: Expr 45 [279-291]: Call:
                                            Expr 66 [279-288]: UnOp (Functor Ctl):
                                                Expr 46 [279-288]: UnOp (Functor Adj):
                                                    Expr 47 [287-288]: Name: Internal(NodeId(5))
                                            Expr 67 [288-291]: Tuple:
                                                Expr 68 [288-291]: Name: Internal(NodeId(64))
                                                Expr 48 [288-291]: Paren: Expr 49 [289-290]: Name: Internal(NodeId(29))"#]],
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
                            output: ()
                            functors: Functor Expr 9 [54-57]: Ctl
                            body: Specializations:
                                SpecDecl 10 [68-79] (Body): Impl:
                                    Pat 11 [73-76]: Elided
                                    Block 12 [77-79]: <empty>
                                SpecDecl 13 [88-113] (Ctl): Impl:
                                    Pat 14 [99-110]: Tuple:
                                        Pat 15 [100-104]: Bind:
                                            Ident 16 [100-104] "ctls"
                                        Pat 17 [106-109]: Elided
                                    Block 18 [111-113]: <empty>
                    Item 19 [124-257]:
                        Callable 20 [124-257] (Operation):
                            name: Ident 21 [134-135] "B"
                            input: Pat 22 [135-146]: Paren:
                                Pat 23 [136-145]: Bind:
                                    Ident 24 [136-137] "q"
                            output: ()
                            functors: Functor Expr 25 [157-160]: Ctl
                            body: Specializations:
                                SpecDecl 41 [161-257] (Body): Impl:
                                    Pat 42 [161-257]: Elided
                                    Block 26 [161-257]:
                                        Stmt 27 [171-251]: Expr: Expr 28 [171-251]: Conjugate:
                                            Block 29 [178-207]:
                                                Stmt 30 [192-197]: Semi: Expr 31 [192-196]: Call:
                                                    Expr 32 [192-193]: Name: Internal(NodeId(5))
                                                    Expr 33 [193-196]: Paren: Expr 34 [194-195]: Name: Internal(NodeId(24))
                                            Block 35 [222-251]:
                                                Stmt 36 [236-241]: Semi: Expr 37 [236-240]: Call:
                                                    Expr 38 [236-237]: Name: Internal(NodeId(5))
                                                    Expr 39 [237-240]: Paren: Expr 40 [238-239]: Name: Internal(NodeId(24))
                                SpecDecl 43 [124-257] (Ctl): Impl:
                                    Pat 49 [124-257]: Tuple:
                                        Pat 45 [124-257]: Bind:
                                            Ident 44 [124-257] "ctls"
                                        Pat 50 [124-257]: Elided
                                    Block 26 [161-257]:
                                        Stmt 27 [171-251]: Expr: Expr 28 [171-251]: Conjugate:
                                            Block 29 [178-207]:
                                                Stmt 30 [192-197]: Semi: Expr 31 [192-196]: Call:
                                                    Expr 32 [192-193]: Name: Internal(NodeId(5))
                                                    Expr 33 [193-196]: Paren: Expr 34 [194-195]: Name: Internal(NodeId(24))
                                            Block 35 [222-251]:
                                                Stmt 36 [236-241]: Semi: Expr 37 [236-240]: Call:
                                                    Expr 46 [236-237]: UnOp (Functor Ctl):
                                                        Expr 38 [236-237]: Name: Internal(NodeId(5))
                                                    Expr 47 [237-240]: Tuple:
                                                        Expr 48 [237-240]: Name: Internal(NodeId(44))
                                                        Expr 39 [237-240]: Paren: Expr 40 [238-239]: Name: Internal(NodeId(24))"#]],
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
                            output: ()
                            body: Block: Block 7 [43-45]: <empty>
                    Item 8 [50-80]:
                        Callable 9 [50-80] (Operation):
                            name: Ident 10 [60-61] "A"
                            input: Pat 11 [61-63]: Unit
                            output: ()
                            functors: Functor Expr 12 [74-77]: Ctl
                            body: Specializations:
                                SpecDecl 28 [78-80] (Body): Impl:
                                    Pat 29 [78-80]: Elided
                                    Block 13 [78-80]: <empty>
                                SpecDecl 30 [50-80] (Ctl): Impl:
                                    Pat 36 [50-80]: Tuple:
                                        Pat 35 [50-80]: Bind:
                                            Ident 34 [50-80] "ctls"
                                        Pat 37 [50-80]: Elided
                                    Block 13 [78-80]: <empty>
                    Item 14 [85-148]:
                        Callable 15 [85-148] (Operation):
                            name: Ident 16 [95-96] "B"
                            input: Pat 17 [96-98]: Unit
                            output: ()
                            functors: Functor Expr 18 [109-112]: Ctl
                            body: Specializations:
                                SpecDecl 31 [113-148] (Body): Impl:
                                    Pat 32 [113-148]: Elided
                                    Block 19 [113-148]:
                                        Stmt 20 [123-129]: Semi: Expr 21 [123-128]: Call:
                                            Expr 22 [123-126]: Name: Internal(NodeId(5))
                                            Expr 23 [126-128]: Unit
                                        Stmt 24 [138-142]: Semi: Expr 25 [138-141]: Call:
                                            Expr 26 [138-139]: Name: Internal(NodeId(10))
                                            Expr 27 [139-141]: Unit
                                SpecDecl 33 [85-148] (Ctl): Impl:
                                    Pat 43 [85-148]: Tuple:
                                        Pat 39 [85-148]: Bind:
                                            Ident 38 [85-148] "ctls"
                                        Pat 44 [85-148]: Elided
                                    Block 19 [113-148]:
                                        Stmt 20 [123-129]: Semi: Expr 21 [123-128]: Call:
                                            Expr 22 [123-126]: Name: Internal(NodeId(5))
                                            Expr 23 [126-128]: Unit
                                        Stmt 24 [138-142]: Semi: Expr 25 [138-141]: Call:
                                            Expr 40 [138-139]: UnOp (Functor Ctl):
                                                Expr 26 [138-139]: Name: Internal(NodeId(10))
                                            Expr 41 [139-141]: Tuple:
                                                Expr 42 [139-141]: Name: Internal(NodeId(38))
                                                Expr 27 [139-141]: Unit"#]],
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
                            output: ()
                            functors: Functor Expr 9 [56-59]: Adj
                            body: Specializations:
                                SpecDecl 32 [60-62] (Body): Impl:
                                    Pat 33 [60-62]: Elided
                                    Block 10 [60-62]: <empty>
                                SpecDecl 34 [21-62] (Adj): Gen: Invert
                    Item 11 [67-166]:
                        Callable 12 [67-166] (Operation):
                            name: Ident 13 [77-78] "A"
                            input: Pat 14 [78-89]: Paren:
                                Pat 15 [79-88]: Bind:
                                    Ident 16 [79-80] "q"
                            output: ()
                            functors: Functor Expr 17 [100-103]: Adj
                            body: Specializations:
                                SpecDecl 18 [114-138] (Body): Impl:
                                    Pat 19 [119-122]: Elided
                                    Block 20 [123-138]:
                                        Stmt 21 [125-130]: Semi: Expr 22 [125-129]: Call:
                                            Expr 23 [125-126]: Name: Internal(NodeId(5))
                                            Expr 24 [126-129]: Paren: Expr 25 [127-128]: Lit: Int(1)
                                        Stmt 26 [131-136]: Semi: Expr 27 [131-135]: Call:
                                            Expr 28 [131-132]: Name: Internal(NodeId(5))
                                            Expr 29 [132-135]: Paren: Expr 30 [133-134]: Lit: Int(2)
                                SpecDecl 31 [147-160] (Adj): Impl:
                                    Pat 19 [119-122]: Elided
                                    Block 20 [123-138]:
                                        Stmt 21 [125-130]: Semi: Expr 22 [125-129]: Call:
                                            Expr 23 [125-126]: Name: Internal(NodeId(5))
                                            Expr 24 [126-129]: Paren: Expr 25 [127-128]: Lit: Int(1)
                                        Stmt 26 [131-136]: Semi: Expr 27 [131-135]: Call:
                                            Expr 28 [131-132]: Name: Internal(NodeId(5))
                                            Expr 29 [132-135]: Paren: Expr 30 [133-134]: Lit: Int(2)"#]],
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
                            input: Pat 6 [32-45]: Paren:
                                Pat 7 [33-44]: Bind:
                                    Ident 8 [33-38] "input"
                                    Type 9 [41-44]: Prim (Int)
                            output: Type 10 [48-52]: Unit
                            functors: Functor Expr 11 [56-65]: BinOp Union: (Functor Expr 12 [56-59]: Ctl) (Functor Expr 13 [62-65]: Adj)
                            body: Specializations:
                                SpecDecl 40 [66-68] (Body): Impl:
                                    Pat 41 [66-68]: Elided
                                    Block 14 [66-68]: <empty>
                                SpecDecl 42 [21-68] (Adj): Gen: Invert
                                SpecDecl 43 [21-68] (Ctl): Impl:
                                    Pat 49 [21-68]: Tuple:
                                        Pat 48 [21-68]: Bind:
                                            Ident 47 [21-68] "ctls"
                                        Pat 50 [21-68]: Elided
                                    Block 14 [66-68]: <empty>
                                SpecDecl 44 [21-68] (CtlAdj): Gen: Distribute
                    Item 15 [73-178]:
                        Callable 16 [73-178] (Operation):
                            name: Ident 17 [83-84] "A"
                            input: Pat 18 [84-95]: Paren:
                                Pat 19 [85-94]: Bind:
                                    Ident 20 [85-86] "q"
                                    Type 21 [89-94]: Prim (Qubit)
                            output: Type 22 [98-102]: Unit
                            functors: Functor Expr 23 [106-115]: BinOp Union: (Functor Expr 24 [106-109]: Ctl) (Functor Expr 25 [112-115]: Adj)
                            body: Specializations:
                                SpecDecl 26 [126-150] (Body): Impl:
                                    Pat 27 [131-134]: Elided
                                    Block 28 [135-150]:
                                        Stmt 29 [137-142]: Semi: Expr 30 [137-141]: Call:
                                            Expr 31 [137-138]: Name: Internal(NodeId(5))
                                            Expr 32 [138-141]: Paren: Expr 33 [139-140]: Lit: Int(1)
                                        Stmt 34 [143-148]: Semi: Expr 35 [143-147]: Call:
                                            Expr 36 [143-144]: Name: Internal(NodeId(5))
                                            Expr 37 [144-147]: Paren: Expr 38 [145-146]: Lit: Int(2)
                                SpecDecl 39 [159-172] (Adj): Impl:
                                    Pat 27 [131-134]: Elided
                                    Block 28 [135-150]:
                                        Stmt 29 [137-142]: Semi: Expr 30 [137-141]: Call:
                                            Expr 31 [137-138]: Name: Internal(NodeId(5))
                                            Expr 32 [138-141]: Paren: Expr 33 [139-140]: Lit: Int(1)
                                        Stmt 34 [143-148]: Semi: Expr 35 [143-147]: Call:
                                            Expr 36 [143-144]: Name: Internal(NodeId(5))
                                            Expr 37 [144-147]: Paren: Expr 38 [145-146]: Lit: Int(2)
                                SpecDecl 45 [73-178] (Ctl): Impl:
                                    Pat 59 [73-178]: Tuple:
                                        Pat 52 [73-178]: Bind:
                                            Ident 51 [73-178] "ctls"
                                        Pat 60 [73-178]: Elided
                                    Block 28 [135-150]:
                                        Stmt 29 [137-142]: Semi: Expr 30 [137-141]: Call:
                                            Expr 53 [137-138]: UnOp (Functor Ctl):
                                                Expr 31 [137-138]: Name: Internal(NodeId(5))
                                            Expr 54 [138-141]: Tuple:
                                                Expr 55 [138-141]: Name: Internal(NodeId(51))
                                                Expr 32 [138-141]: Paren: Expr 33 [139-140]: Lit: Int(1)
                                        Stmt 34 [143-148]: Semi: Expr 35 [143-147]: Call:
                                            Expr 56 [143-144]: UnOp (Functor Ctl):
                                                Expr 36 [143-144]: Name: Internal(NodeId(5))
                                            Expr 57 [144-147]: Tuple:
                                                Expr 58 [144-147]: Name: Internal(NodeId(51))
                                                Expr 37 [144-147]: Paren: Expr 38 [145-146]: Lit: Int(2)
                                SpecDecl 46 [73-178] (CtlAdj): Impl:
                                    Pat 59 [73-178]: Tuple:
                                        Pat 52 [73-178]: Bind:
                                            Ident 51 [73-178] "ctls"
                                        Pat 60 [73-178]: Elided
                                    Block 28 [135-150]:
                                        Stmt 29 [137-142]: Semi: Expr 30 [137-141]: Call:
                                            Expr 53 [137-138]: UnOp (Functor Ctl):
                                                Expr 31 [137-138]: Name: Internal(NodeId(5))
                                            Expr 54 [138-141]: Tuple:
                                                Expr 55 [138-141]: Name: Internal(NodeId(51))
                                                Expr 32 [138-141]: Paren: Expr 33 [139-140]: Lit: Int(1)
                                        Stmt 34 [143-148]: Semi: Expr 35 [143-147]: Call:
                                            Expr 56 [143-144]: UnOp (Functor Ctl):
                                                Expr 36 [143-144]: Name: Internal(NodeId(5))
                                            Expr 57 [144-147]: Tuple:
                                                Expr 58 [144-147]: Name: Internal(NodeId(51))
                                                Expr 37 [144-147]: Paren: Expr 38 [145-146]: Lit: Int(2)"#]],
    );
}
