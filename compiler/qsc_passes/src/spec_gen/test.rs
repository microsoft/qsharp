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
                                SpecDecl 40 [161-182] (Body): Impl:
                                    Pat 41 [161-182]: Elided
                                    Block 30 [161-182]:
                                        Stmt 31 [171-176]: Semi: Expr 32 [171-175]: Call:
                                            Expr 33 [171-172]: Path: Path 34 [171-172] (Ident 35 [171-172] "A")
                                            Expr 36 [172-175]: Paren: Expr 37 [173-174]: Path: Path 38 [173-174] (Ident 39 [173-174] "q")
                                SpecDecl 42 [124-182] (Ctl): Impl:
                                    Pat 48 [124-182]: Tuple:
                                        Pat 49 [124-182]: Bind:
                                            Ident 44 [124-182] "ctls"
                                        Pat 50 [124-182]: Elided
                                    Block 30 [161-182]:
                                        Stmt 31 [171-176]: Semi: Expr 32 [171-175]: Call:
                                            Expr 45 [171-172]: UnOp (Functor Ctl):
                                                Expr 33 [171-172]: Path: Path 34 [171-172] (Ident 35 [171-172] "A")
                                            Expr 46 [172-175]: Tuple:
                                                Expr 47 [172-175]: Path: Path 43 [124-182] (Ident 44 [124-182] "ctls")
                                                Expr 36 [172-175]: Paren: Expr 37 [173-174]: Path: Path 38 [173-174] (Ident 39 [173-174] "q")"#]],
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
                                SpecDecl 62 [21-148] (CtlAdj): Impl:
                                    Pat 67 [21-148]: Tuple:
                                        Pat 68 [21-148]: Bind:
                                            Ident 66 [21-148] "ctls"
                                        Pat 69 [21-148]: Elided
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
                                            Expr 42 [229-230]: Path: Path 43 [229-230] (Ident 44 [229-230] "A")
                                            Expr 45 [230-233]: Paren: Expr 46 [231-232]: Path: Path 47 [231-232] (Ident 48 [231-232] "q")
                                SpecDecl 49 [253-302] (Adj): Impl:
                                    Pat 50 [261-264]: Elided
                                    Block 51 [265-302]:
                                        Stmt 52 [279-292]: Semi: Expr 53 [279-291]: Call:
                                            Expr 54 [279-288]: UnOp (Functor Adj):
                                                Expr 55 [287-288]: Path: Path 56 [287-288] (Ident 57 [287-288] "A")
                                            Expr 58 [288-291]: Paren: Expr 59 [289-290]: Path: Path 60 [289-290] (Ident 61 [289-290] "q")
                                SpecDecl 63 [153-308] (Ctl): Impl:
                                    Pat 75 [153-308]: Tuple:
                                        Pat 76 [153-308]: Bind:
                                            Ident 71 [153-308] "ctls"
                                        Pat 77 [153-308]: Elided
                                    Block 39 [215-244]:
                                        Stmt 40 [229-234]: Semi: Expr 41 [229-233]: Call:
                                            Expr 72 [229-230]: UnOp (Functor Ctl):
                                                Expr 42 [229-230]: Path: Path 43 [229-230] (Ident 44 [229-230] "A")
                                            Expr 73 [230-233]: Tuple:
                                                Expr 74 [230-233]: Path: Path 70 [153-308] (Ident 71 [153-308] "ctls")
                                                Expr 45 [230-233]: Paren: Expr 46 [231-232]: Path: Path 47 [231-232] (Ident 48 [231-232] "q")
                                SpecDecl 64 [153-308] (CtlAdj): Impl:
                                    Pat 83 [153-308]: Tuple:
                                        Pat 84 [153-308]: Bind:
                                            Ident 79 [153-308] "ctls"
                                        Pat 85 [153-308]: Elided
                                    Block 51 [265-302]:
                                        Stmt 52 [279-292]: Semi: Expr 53 [279-291]: Call:
                                            Expr 80 [279-288]: UnOp (Functor Ctl):
                                                Expr 54 [279-288]: UnOp (Functor Adj):
                                                    Expr 55 [287-288]: Path: Path 56 [287-288] (Ident 57 [287-288] "A")
                                            Expr 81 [288-291]: Tuple:
                                                Expr 82 [288-291]: Path: Path 78 [153-308] (Ident 79 [153-308] "ctls")
                                                Expr 58 [288-291]: Paren: Expr 59 [289-290]: Path: Path 60 [289-290] (Ident 61 [289-290] "q")"#]],
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
                                SpecDecl 53 [161-257] (Body): Impl:
                                    Pat 54 [161-257]: Elided
                                    Block 30 [161-257]:
                                        Stmt 31 [171-251]: Expr: Expr 32 [171-251]: Conjugate:
                                            Block 33 [178-207]:
                                                Stmt 34 [192-197]: Semi: Expr 35 [192-196]: Call:
                                                    Expr 36 [192-193]: Path: Path 37 [192-193] (Ident 38 [192-193] "A")
                                                    Expr 39 [193-196]: Paren: Expr 40 [194-195]: Path: Path 41 [194-195] (Ident 42 [194-195] "q")
                                            Block 43 [222-251]:
                                                Stmt 44 [236-241]: Semi: Expr 45 [236-240]: Call:
                                                    Expr 46 [236-237]: Path: Path 47 [236-237] (Ident 48 [236-237] "A")
                                                    Expr 49 [237-240]: Paren: Expr 50 [238-239]: Path: Path 51 [238-239] (Ident 52 [238-239] "q")
                                SpecDecl 55 [124-257] (Ctl): Impl:
                                    Pat 61 [124-257]: Tuple:
                                        Pat 62 [124-257]: Bind:
                                            Ident 57 [124-257] "ctls"
                                        Pat 63 [124-257]: Elided
                                    Block 30 [161-257]:
                                        Stmt 31 [171-251]: Expr: Expr 32 [171-251]: Conjugate:
                                            Block 33 [178-207]:
                                                Stmt 34 [192-197]: Semi: Expr 35 [192-196]: Call:
                                                    Expr 36 [192-193]: Path: Path 37 [192-193] (Ident 38 [192-193] "A")
                                                    Expr 39 [193-196]: Paren: Expr 40 [194-195]: Path: Path 41 [194-195] (Ident 42 [194-195] "q")
                                            Block 43 [222-251]:
                                                Stmt 44 [236-241]: Semi: Expr 45 [236-240]: Call:
                                                    Expr 58 [236-237]: UnOp (Functor Ctl):
                                                        Expr 46 [236-237]: Path: Path 47 [236-237] (Ident 48 [236-237] "A")
                                                    Expr 59 [237-240]: Tuple:
                                                        Expr 60 [237-240]: Path: Path 56 [124-257] (Ident 57 [124-257] "ctls")
                                                        Expr 49 [237-240]: Paren: Expr 50 [238-239]: Path: Path 51 [238-239] (Ident 52 [238-239] "q")"#]],
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
                                SpecDecl 35 [78-80] (Body): Impl:
                                    Pat 36 [78-80]: Elided
                                    Block 15 [78-80]: <empty>
                                SpecDecl 37 [50-80] (Ctl): Impl:
                                    Pat 43 [50-80]: Tuple:
                                        Pat 44 [50-80]: Bind:
                                            Ident 42 [50-80] "ctls"
                                        Pat 45 [50-80]: Elided
                                    Block 15 [78-80]: <empty>
                    Item 16 [85-148]:
                        Callable 17 [85-148] (Operation):
                            name: Ident 18 [95-96] "B"
                            input: Pat 19 [96-98]: Unit
                            output: Type 20 [101-105]: Unit
                            functors: Functor Expr 21 [109-112]: Ctl
                            body: Specializations:
                                SpecDecl 38 [113-148] (Body): Impl:
                                    Pat 39 [113-148]: Elided
                                    Block 22 [113-148]:
                                        Stmt 23 [123-129]: Semi: Expr 24 [123-128]: Call:
                                            Expr 25 [123-126]: Path: Path 26 [123-126] (Ident 27 [123-126] "Foo")
                                            Expr 28 [126-128]: Unit
                                        Stmt 29 [138-142]: Semi: Expr 30 [138-141]: Call:
                                            Expr 31 [138-139]: Path: Path 32 [138-139] (Ident 33 [138-139] "A")
                                            Expr 34 [139-141]: Unit
                                SpecDecl 40 [85-148] (Ctl): Impl:
                                    Pat 51 [85-148]: Tuple:
                                        Pat 52 [85-148]: Bind:
                                            Ident 47 [85-148] "ctls"
                                        Pat 53 [85-148]: Elided
                                    Block 22 [113-148]:
                                        Stmt 23 [123-129]: Semi: Expr 24 [123-128]: Call:
                                            Expr 25 [123-126]: Path: Path 26 [123-126] (Ident 27 [123-126] "Foo")
                                            Expr 28 [126-128]: Unit
                                        Stmt 29 [138-142]: Semi: Expr 30 [138-141]: Call:
                                            Expr 48 [138-139]: UnOp (Functor Ctl):
                                                Expr 31 [138-139]: Path: Path 32 [138-139] (Ident 33 [138-139] "A")
                                            Expr 49 [139-141]: Tuple:
                                                Expr 50 [139-141]: Path: Path 46 [85-148] (Ident 47 [85-148] "ctls")
                                                Expr 34 [139-141]: Unit"#]],
    );
}

#[test]
fn generate_adj_self() {
    check(
        indoc! {r#"
            namespace test {
                operation A(q : Qubit) : Unit is Adj {
                    body ... { fail "body impl"; }
                    adjoint self;
                }
            }
        "#},
        &expect![[r#"
            Package 0:
                Namespace 1 [0-128] (Ident 2 [10-14] "test"):
                    Item 3 [21-126]:
                        Callable 4 [21-126] (Operation):
                            name: Ident 5 [31-32] "A"
                            input: Pat 6 [32-43]: Paren:
                                Pat 7 [33-42]: Bind:
                                    Ident 8 [33-34] "q"
                                    Type 9 [37-42]: Prim (Qubit)
                            output: Type 10 [46-50]: Unit
                            functors: Functor Expr 11 [54-57]: Adj
                            body: Specializations:
                                SpecDecl 12 [68-98] (Body): Impl:
                                    Pat 13 [73-76]: Elided
                                    Block 14 [77-98]:
                                        Stmt 15 [79-96]: Semi: Expr 16 [79-95]: Fail: Expr 17 [84-95]: Lit: String("body impl")
                                SpecDecl 18 [107-120] (Adj): Impl:
                                    Pat 13 [73-76]: Elided
                                    Block 14 [77-98]:
                                        Stmt 15 [79-96]: Semi: Expr 16 [79-95]: Fail: Expr 17 [84-95]: Lit: String("body impl")"#]],
    );
}
