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
                    Namespace (Ident 31 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [32-43]: Paren:
                            Pat 3 [33-42]: Bind:
                                Ident 4 [33-34] "q"
                                Type 5 [37-42]: Prim (Qubit)
                        output: Type 6 [46-50]: Unit
                        functors: Functor Expr 7 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 8 [68-79] (Body): Impl:
                                Pat 9 [73-76]: Elided
                                Block 10 [77-79]: <empty>
                            SpecDecl 11 [88-113] (Ctl): Impl:
                                Pat 12 [99-110]: Tuple:
                                    Pat 13 [100-104]: Bind:
                                        Ident 14 [100-104] "ctls"
                                    Pat 15 [106-109]: Elided
                                Block 16 [111-113]: <empty>
                Item 2 [124-182]:
                    Parent: 0
                    Callable 17 [124-182] (Operation):
                        name: Ident 18 [134-135] "B"
                        input: Pat 19 [135-146]: Paren:
                            Pat 20 [136-145]: Bind:
                                Ident 21 [136-137] "q"
                                Type 22 [140-145]: Prim (Qubit)
                        output: Type 23 [149-153]: Unit
                        functors: Functor Expr 24 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl 32 [161-182] (Body): Impl:
                                Pat 33 [161-182]: Elided
                                Block 25 [161-182]:
                                    Stmt 26 [171-176]: Semi: Expr 27 [171-175]: Call:
                                        Expr 28 [171-172]: Name: Item 1
                                        Expr 29 [172-175]: Paren: Expr 30 [173-174]: Name: Local 21
                            SpecDecl 34 [124-182] (Ctl): Impl:
                                Pat 40 [124-182]: Tuple:
                                    Pat 36 [124-182]: Bind:
                                        Ident 35 [124-182] "ctls"
                                    Pat 41 [124-182]: Elided
                                Block 25 [161-182]:
                                    Stmt 26 [171-176]: Semi: Expr 27 [171-175]: Call:
                                        Expr 37 [171-172]: UnOp (Functor Ctl):
                                            Expr 28 [171-172]: Name: Item 1
                                        Expr 38 [172-175]: Tuple:
                                            Expr 39 [172-175]: Name: Local 35
                                            Expr 29 [172-175]: Paren: Expr 30 [173-174]: Name: Local 21"#]],
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
                    Namespace (Ident 49 [10-14] "test"): Item 1, Item 2
                Item 1 [21-148]:
                    Parent: 0
                    Callable 0 [21-148] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [32-43]: Paren:
                            Pat 3 [33-42]: Bind:
                                Ident 4 [33-34] "q"
                                Type 5 [37-42]: Prim (Qubit)
                        output: Type 6 [46-50]: Unit
                        functors: Functor Expr 7 [54-63]: BinOp Union: (Functor Expr 8 [54-57]: Ctl) (Functor Expr 9 [60-63]: Adj)
                        body: Specializations:
                            SpecDecl 10 [74-85] (Body): Impl:
                                Pat 11 [79-82]: Elided
                                Block 12 [83-85]: <empty>
                            SpecDecl 13 [94-108] (Adj): Impl:
                                Pat 14 [102-105]: Elided
                                Block 15 [106-108]: <empty>
                            SpecDecl 16 [117-142] (Ctl): Impl:
                                Pat 17 [128-139]: Tuple:
                                    Pat 18 [129-133]: Bind:
                                        Ident 19 [129-133] "ctls"
                                    Pat 20 [135-138]: Elided
                                Block 21 [140-142]: <empty>
                            SpecDecl 50 [21-148] (CtlAdj): Impl:
                                Pat 55 [21-148]: Tuple:
                                    Pat 54 [21-148]: Bind:
                                        Ident 53 [21-148] "ctls"
                                    Pat 56 [21-148]: Elided
                                Block 15 [106-108]: <empty>
                Item 2 [153-308]:
                    Parent: 0
                    Callable 22 [153-308] (Operation):
                        name: Ident 23 [163-164] "B"
                        input: Pat 24 [164-175]: Paren:
                            Pat 25 [165-174]: Bind:
                                Ident 26 [165-166] "q"
                                Type 27 [169-174]: Prim (Qubit)
                        output: Type 28 [178-182]: Unit
                        functors: Functor Expr 29 [186-195]: BinOp Union: (Functor Expr 30 [186-189]: Ctl) (Functor Expr 31 [192-195]: Adj)
                        body: Specializations:
                            SpecDecl 32 [206-244] (Body): Impl:
                                Pat 33 [211-214]: Elided
                                Block 34 [215-244]:
                                    Stmt 35 [229-234]: Semi: Expr 36 [229-233]: Call:
                                        Expr 37 [229-230]: Name: Item 1
                                        Expr 38 [230-233]: Paren: Expr 39 [231-232]: Name: Local 26
                            SpecDecl 40 [253-302] (Adj): Impl:
                                Pat 41 [261-264]: Elided
                                Block 42 [265-302]:
                                    Stmt 43 [279-292]: Semi: Expr 44 [279-291]: Call:
                                        Expr 45 [279-288]: UnOp (Functor Adj):
                                            Expr 46 [287-288]: Name: Item 1
                                        Expr 47 [288-291]: Paren: Expr 48 [289-290]: Name: Local 26
                            SpecDecl 51 [153-308] (Ctl): Impl:
                                Pat 62 [153-308]: Tuple:
                                    Pat 58 [153-308]: Bind:
                                        Ident 57 [153-308] "ctls"
                                    Pat 63 [153-308]: Elided
                                Block 34 [215-244]:
                                    Stmt 35 [229-234]: Semi: Expr 36 [229-233]: Call:
                                        Expr 59 [229-230]: UnOp (Functor Ctl):
                                            Expr 37 [229-230]: Name: Item 1
                                        Expr 60 [230-233]: Tuple:
                                            Expr 61 [230-233]: Name: Local 57
                                            Expr 38 [230-233]: Paren: Expr 39 [231-232]: Name: Local 26
                            SpecDecl 52 [153-308] (CtlAdj): Impl:
                                Pat 69 [153-308]: Tuple:
                                    Pat 65 [153-308]: Bind:
                                        Ident 64 [153-308] "ctls"
                                    Pat 70 [153-308]: Elided
                                Block 42 [265-302]:
                                    Stmt 43 [279-292]: Semi: Expr 44 [279-291]: Call:
                                        Expr 66 [279-288]: UnOp (Functor Ctl):
                                            Expr 45 [279-288]: UnOp (Functor Adj):
                                                Expr 46 [287-288]: Name: Item 1
                                        Expr 67 [288-291]: Tuple:
                                            Expr 68 [288-291]: Name: Local 64
                                            Expr 47 [288-291]: Paren: Expr 48 [289-290]: Name: Local 26"#]],
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
                    Namespace (Ident 40 [10-14] "test"): Item 1, Item 2
                Item 1 [21-119]:
                    Parent: 0
                    Callable 0 [21-119] (Operation):
                        name: Ident 1 [31-32] "A"
                        input: Pat 2 [32-43]: Paren:
                            Pat 3 [33-42]: Bind:
                                Ident 4 [33-34] "q"
                                Type 5 [37-42]: Prim (Qubit)
                        output: Type 6 [46-50]: Unit
                        functors: Functor Expr 7 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 8 [68-79] (Body): Impl:
                                Pat 9 [73-76]: Elided
                                Block 10 [77-79]: <empty>
                            SpecDecl 11 [88-113] (Ctl): Impl:
                                Pat 12 [99-110]: Tuple:
                                    Pat 13 [100-104]: Bind:
                                        Ident 14 [100-104] "ctls"
                                    Pat 15 [106-109]: Elided
                                Block 16 [111-113]: <empty>
                Item 2 [124-257]:
                    Parent: 0
                    Callable 17 [124-257] (Operation):
                        name: Ident 18 [134-135] "B"
                        input: Pat 19 [135-146]: Paren:
                            Pat 20 [136-145]: Bind:
                                Ident 21 [136-137] "q"
                                Type 22 [140-145]: Prim (Qubit)
                        output: Type 23 [149-153]: Unit
                        functors: Functor Expr 24 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl 41 [161-257] (Body): Impl:
                                Pat 42 [161-257]: Elided
                                Block 25 [161-257]:
                                    Stmt 26 [171-251]: Expr: Expr 27 [171-251]: Conjugate:
                                        Block 28 [178-207]:
                                            Stmt 29 [192-197]: Semi: Expr 30 [192-196]: Call:
                                                Expr 31 [192-193]: Name: Item 1
                                                Expr 32 [193-196]: Paren: Expr 33 [194-195]: Name: Local 21
                                        Block 34 [222-251]:
                                            Stmt 35 [236-241]: Semi: Expr 36 [236-240]: Call:
                                                Expr 37 [236-237]: Name: Item 1
                                                Expr 38 [237-240]: Paren: Expr 39 [238-239]: Name: Local 21
                            SpecDecl 43 [124-257] (Ctl): Impl:
                                Pat 49 [124-257]: Tuple:
                                    Pat 45 [124-257]: Bind:
                                        Ident 44 [124-257] "ctls"
                                    Pat 50 [124-257]: Elided
                                Block 25 [161-257]:
                                    Stmt 26 [171-251]: Expr: Expr 27 [171-251]: Conjugate:
                                        Block 28 [178-207]:
                                            Stmt 29 [192-197]: Semi: Expr 30 [192-196]: Call:
                                                Expr 31 [192-193]: Name: Item 1
                                                Expr 32 [193-196]: Paren: Expr 33 [194-195]: Name: Local 21
                                        Block 34 [222-251]:
                                            Stmt 35 [236-241]: Semi: Expr 36 [236-240]: Call:
                                                Expr 46 [236-237]: UnOp (Functor Ctl):
                                                    Expr 37 [236-237]: Name: Item 1
                                                Expr 47 [237-240]: Tuple:
                                                    Expr 48 [237-240]: Name: Local 44
                                                    Expr 38 [237-240]: Paren: Expr 39 [238-239]: Name: Local 21"#]],
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
                    Namespace (Ident 25 [10-14] "test"): Item 1, Item 2, Item 3
                Item 1 [21-45]:
                    Parent: 0
                    Callable 0 [21-45] (Function):
                        name: Ident 1 [30-33] "Foo"
                        input: Pat 2 [33-35]: Unit
                        output: Type 3 [38-42]: Unit
                        body: Block: Block 4 [43-45]: <empty>
                Item 2 [50-80]:
                    Parent: 0
                    Callable 5 [50-80] (Operation):
                        name: Ident 6 [60-61] "A"
                        input: Pat 7 [61-63]: Unit
                        output: Type 8 [66-70]: Unit
                        functors: Functor Expr 9 [74-77]: Ctl
                        body: Specializations:
                            SpecDecl 26 [78-80] (Body): Impl:
                                Pat 27 [78-80]: Elided
                                Block 10 [78-80]: <empty>
                            SpecDecl 28 [50-80] (Ctl): Impl:
                                Pat 34 [50-80]: Tuple:
                                    Pat 33 [50-80]: Bind:
                                        Ident 32 [50-80] "ctls"
                                    Pat 35 [50-80]: Elided
                                Block 10 [78-80]: <empty>
                Item 3 [85-148]:
                    Parent: 0
                    Callable 11 [85-148] (Operation):
                        name: Ident 12 [95-96] "B"
                        input: Pat 13 [96-98]: Unit
                        output: Type 14 [101-105]: Unit
                        functors: Functor Expr 15 [109-112]: Ctl
                        body: Specializations:
                            SpecDecl 29 [113-148] (Body): Impl:
                                Pat 30 [113-148]: Elided
                                Block 16 [113-148]:
                                    Stmt 17 [123-129]: Semi: Expr 18 [123-128]: Call:
                                        Expr 19 [123-126]: Name: Item 1
                                        Expr 20 [126-128]: Unit
                                    Stmt 21 [138-142]: Semi: Expr 22 [138-141]: Call:
                                        Expr 23 [138-139]: Name: Item 2
                                        Expr 24 [139-141]: Unit
                            SpecDecl 31 [85-148] (Ctl): Impl:
                                Pat 41 [85-148]: Tuple:
                                    Pat 37 [85-148]: Bind:
                                        Ident 36 [85-148] "ctls"
                                    Pat 42 [85-148]: Elided
                                Block 16 [113-148]:
                                    Stmt 17 [123-129]: Semi: Expr 18 [123-128]: Call:
                                        Expr 19 [123-126]: Name: Item 1
                                        Expr 20 [126-128]: Unit
                                    Stmt 21 [138-142]: Semi: Expr 22 [138-141]: Call:
                                        Expr 38 [138-139]: UnOp (Functor Ctl):
                                            Expr 23 [138-139]: Name: Item 2
                                        Expr 39 [139-141]: Tuple:
                                            Expr 40 [139-141]: Name: Local 36
                                            Expr 24 [139-141]: Unit"#]],
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
                    Namespace (Ident 31 [10-14] "test"): Item 1, Item 2
                Item 1 [21-62]:
                    Parent: 0
                    Callable 0 [21-62] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45]: Paren:
                            Pat 3 [33-44]: Bind:
                                Ident 4 [33-38] "input"
                                Type 5 [41-44]: Prim (Int)
                        output: Type 6 [48-52]: Unit
                        functors: Functor Expr 7 [56-59]: Adj
                        body: Specializations:
                            SpecDecl 32 [60-62] (Body): Impl:
                                Pat 33 [60-62]: Elided
                                Block 8 [60-62]: <empty>
                            SpecDecl 34 [21-62] (Adj): Gen: Invert
                Item 2 [67-166]:
                    Parent: 0
                    Callable 9 [67-166] (Operation):
                        name: Ident 10 [77-78] "A"
                        input: Pat 11 [78-89]: Paren:
                            Pat 12 [79-88]: Bind:
                                Ident 13 [79-80] "q"
                                Type 14 [83-88]: Prim (Qubit)
                        output: Type 15 [92-96]: Unit
                        functors: Functor Expr 16 [100-103]: Adj
                        body: Specializations:
                            SpecDecl 17 [114-138] (Body): Impl:
                                Pat 18 [119-122]: Elided
                                Block 19 [123-138]:
                                    Stmt 20 [125-130]: Semi: Expr 21 [125-129]: Call:
                                        Expr 22 [125-126]: Name: Item 1
                                        Expr 23 [126-129]: Paren: Expr 24 [127-128]: Lit: Int(1)
                                    Stmt 25 [131-136]: Semi: Expr 26 [131-135]: Call:
                                        Expr 27 [131-132]: Name: Item 1
                                        Expr 28 [132-135]: Paren: Expr 29 [133-134]: Lit: Int(2)
                            SpecDecl 30 [147-160] (Adj): Impl:
                                Pat 18 [119-122]: Elided
                                Block 19 [123-138]:
                                    Stmt 20 [125-130]: Semi: Expr 21 [125-129]: Call:
                                        Expr 22 [125-126]: Name: Item 1
                                        Expr 23 [126-129]: Paren: Expr 24 [127-128]: Lit: Int(1)
                                    Stmt 25 [131-136]: Semi: Expr 26 [131-135]: Call:
                                        Expr 27 [131-132]: Name: Item 1
                                        Expr 28 [132-135]: Paren: Expr 29 [133-134]: Lit: Int(2)"#]],
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
                    Namespace (Ident 35 [10-14] "test"): Item 1, Item 2
                Item 1 [21-68]:
                    Parent: 0
                    Callable 0 [21-68] (Operation):
                        name: Ident 1 [31-32] "B"
                        input: Pat 2 [32-45]: Paren:
                            Pat 3 [33-44]: Bind:
                                Ident 4 [33-38] "input"
                                Type 5 [41-44]: Prim (Int)
                        output: Type 6 [48-52]: Unit
                        functors: Functor Expr 7 [56-65]: BinOp Union: (Functor Expr 8 [56-59]: Ctl) (Functor Expr 9 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl 36 [66-68] (Body): Impl:
                                Pat 37 [66-68]: Elided
                                Block 10 [66-68]: <empty>
                            SpecDecl 38 [21-68] (Adj): Gen: Invert
                            SpecDecl 39 [21-68] (Ctl): Impl:
                                Pat 45 [21-68]: Tuple:
                                    Pat 44 [21-68]: Bind:
                                        Ident 43 [21-68] "ctls"
                                    Pat 46 [21-68]: Elided
                                Block 10 [66-68]: <empty>
                            SpecDecl 40 [21-68] (CtlAdj): Gen: Distribute
                Item 2 [73-178]:
                    Parent: 0
                    Callable 11 [73-178] (Operation):
                        name: Ident 12 [83-84] "A"
                        input: Pat 13 [84-95]: Paren:
                            Pat 14 [85-94]: Bind:
                                Ident 15 [85-86] "q"
                                Type 16 [89-94]: Prim (Qubit)
                        output: Type 17 [98-102]: Unit
                        functors: Functor Expr 18 [106-115]: BinOp Union: (Functor Expr 19 [106-109]: Ctl) (Functor Expr 20 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 21 [126-150] (Body): Impl:
                                Pat 22 [131-134]: Elided
                                Block 23 [135-150]:
                                    Stmt 24 [137-142]: Semi: Expr 25 [137-141]: Call:
                                        Expr 26 [137-138]: Name: Item 1
                                        Expr 27 [138-141]: Paren: Expr 28 [139-140]: Lit: Int(1)
                                    Stmt 29 [143-148]: Semi: Expr 30 [143-147]: Call:
                                        Expr 31 [143-144]: Name: Item 1
                                        Expr 32 [144-147]: Paren: Expr 33 [145-146]: Lit: Int(2)
                            SpecDecl 34 [159-172] (Adj): Impl:
                                Pat 22 [131-134]: Elided
                                Block 23 [135-150]:
                                    Stmt 24 [137-142]: Semi: Expr 25 [137-141]: Call:
                                        Expr 26 [137-138]: Name: Item 1
                                        Expr 27 [138-141]: Paren: Expr 28 [139-140]: Lit: Int(1)
                                    Stmt 29 [143-148]: Semi: Expr 30 [143-147]: Call:
                                        Expr 31 [143-144]: Name: Item 1
                                        Expr 32 [144-147]: Paren: Expr 33 [145-146]: Lit: Int(2)
                            SpecDecl 41 [73-178] (Ctl): Impl:
                                Pat 55 [73-178]: Tuple:
                                    Pat 48 [73-178]: Bind:
                                        Ident 47 [73-178] "ctls"
                                    Pat 56 [73-178]: Elided
                                Block 23 [135-150]:
                                    Stmt 24 [137-142]: Semi: Expr 25 [137-141]: Call:
                                        Expr 49 [137-138]: UnOp (Functor Ctl):
                                            Expr 26 [137-138]: Name: Item 1
                                        Expr 50 [138-141]: Tuple:
                                            Expr 51 [138-141]: Name: Local 47
                                            Expr 27 [138-141]: Paren: Expr 28 [139-140]: Lit: Int(1)
                                    Stmt 29 [143-148]: Semi: Expr 30 [143-147]: Call:
                                        Expr 52 [143-144]: UnOp (Functor Ctl):
                                            Expr 31 [143-144]: Name: Item 1
                                        Expr 53 [144-147]: Tuple:
                                            Expr 54 [144-147]: Name: Local 47
                                            Expr 32 [144-147]: Paren: Expr 33 [145-146]: Lit: Int(2)
                            SpecDecl 42 [73-178] (CtlAdj): Impl:
                                Pat 55 [73-178]: Tuple:
                                    Pat 48 [73-178]: Bind:
                                        Ident 47 [73-178] "ctls"
                                    Pat 56 [73-178]: Elided
                                Block 23 [135-150]:
                                    Stmt 24 [137-142]: Semi: Expr 25 [137-141]: Call:
                                        Expr 49 [137-138]: UnOp (Functor Ctl):
                                            Expr 26 [137-138]: Name: Item 1
                                        Expr 50 [138-141]: Tuple:
                                            Expr 51 [138-141]: Name: Local 47
                                            Expr 27 [138-141]: Paren: Expr 28 [139-140]: Lit: Int(1)
                                    Stmt 29 [143-148]: Semi: Expr 30 [143-147]: Call:
                                        Expr 52 [143-144]: UnOp (Functor Ctl):
                                            Expr 31 [143-144]: Name: Item 1
                                        Expr 53 [144-147]: Tuple:
                                            Expr 54 [144-147]: Name: Local 47
                                            Expr 32 [144-147]: Paren: Expr 33 [145-146]: Lit: Int(2)"#]],
    );
}
