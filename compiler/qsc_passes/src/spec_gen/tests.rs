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
            Package 33:
                Item 0 [21-119]:
                    Callable 1 [21-119] (Operation):
                        name: Ident 2 [31-32] "A"
                        input: Pat 3 [32-43]: Paren:
                            Pat 4 [33-42]: Bind:
                                Ident 5 [33-34] "q"
                                Type 6 [37-42]: Prim (Qubit)
                        output: Type 7 [46-50]: Unit
                        functors: Functor Expr 8 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 9 [68-79] (Body): Impl:
                                Pat 10 [73-76]: Elided
                                Block 11 [77-79]: <empty>
                            SpecDecl 12 [88-113] (Ctl): Impl:
                                Pat 13 [99-110]: Tuple:
                                    Pat 14 [100-104]: Bind:
                                        Ident 15 [100-104] "ctls"
                                    Pat 16 [106-109]: Elided
                                Block 17 [111-113]: <empty>
                Item 18 [124-182]:
                    Callable 19 [124-182] (Operation):
                        name: Ident 20 [134-135] "B"
                        input: Pat 21 [135-146]: Paren:
                            Pat 22 [136-145]: Bind:
                                Ident 23 [136-137] "q"
                                Type 24 [140-145]: Prim (Qubit)
                        output: Type 25 [149-153]: Unit
                        functors: Functor Expr 26 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl 34 [161-182] (Body): Impl:
                                Pat 35 [161-182]: Elided
                                Block 27 [161-182]:
                                    Stmt 28 [171-176]: Semi: Expr 29 [171-175]: Call:
                                        Expr 30 [171-172]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 31 [172-175]: Paren: Expr 32 [173-174]: Name: Local(NodeId(23))
                            SpecDecl 36 [124-182] (Ctl): Impl:
                                Pat 42 [124-182]: Tuple:
                                    Pat 38 [124-182]: Bind:
                                        Ident 37 [124-182] "ctls"
                                    Pat 43 [124-182]: Elided
                                Block 27 [161-182]:
                                    Stmt 28 [171-176]: Semi: Expr 29 [171-175]: Call:
                                        Expr 39 [171-172]: UnOp (Functor Ctl):
                                            Expr 30 [171-172]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 40 [172-175]: Tuple:
                                            Expr 41 [172-175]: Name: Local(NodeId(37))
                                            Expr 31 [172-175]: Paren: Expr 32 [173-174]: Name: Local(NodeId(23))"#]],
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
            Package 51:
                Item 0 [21-148]:
                    Callable 1 [21-148] (Operation):
                        name: Ident 2 [31-32] "A"
                        input: Pat 3 [32-43]: Paren:
                            Pat 4 [33-42]: Bind:
                                Ident 5 [33-34] "q"
                                Type 6 [37-42]: Prim (Qubit)
                        output: Type 7 [46-50]: Unit
                        functors: Functor Expr 8 [54-63]: BinOp Union: (Functor Expr 9 [54-57]: Ctl) (Functor Expr 10 [60-63]: Adj)
                        body: Specializations:
                            SpecDecl 11 [74-85] (Body): Impl:
                                Pat 12 [79-82]: Elided
                                Block 13 [83-85]: <empty>
                            SpecDecl 14 [94-108] (Adj): Impl:
                                Pat 15 [102-105]: Elided
                                Block 16 [106-108]: <empty>
                            SpecDecl 17 [117-142] (Ctl): Impl:
                                Pat 18 [128-139]: Tuple:
                                    Pat 19 [129-133]: Bind:
                                        Ident 20 [129-133] "ctls"
                                    Pat 21 [135-138]: Elided
                                Block 22 [140-142]: <empty>
                            SpecDecl 52 [21-148] (CtlAdj): Impl:
                                Pat 57 [21-148]: Tuple:
                                    Pat 56 [21-148]: Bind:
                                        Ident 55 [21-148] "ctls"
                                    Pat 58 [21-148]: Elided
                                Block 16 [106-108]: <empty>
                Item 23 [153-308]:
                    Callable 24 [153-308] (Operation):
                        name: Ident 25 [163-164] "B"
                        input: Pat 26 [164-175]: Paren:
                            Pat 27 [165-174]: Bind:
                                Ident 28 [165-166] "q"
                                Type 29 [169-174]: Prim (Qubit)
                        output: Type 30 [178-182]: Unit
                        functors: Functor Expr 31 [186-195]: BinOp Union: (Functor Expr 32 [186-189]: Ctl) (Functor Expr 33 [192-195]: Adj)
                        body: Specializations:
                            SpecDecl 34 [206-244] (Body): Impl:
                                Pat 35 [211-214]: Elided
                                Block 36 [215-244]:
                                    Stmt 37 [229-234]: Semi: Expr 38 [229-233]: Call:
                                        Expr 39 [229-230]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 40 [230-233]: Paren: Expr 41 [231-232]: Name: Local(NodeId(28))
                            SpecDecl 42 [253-302] (Adj): Impl:
                                Pat 43 [261-264]: Elided
                                Block 44 [265-302]:
                                    Stmt 45 [279-292]: Semi: Expr 46 [279-291]: Call:
                                        Expr 47 [279-288]: UnOp (Functor Adj):
                                            Expr 48 [287-288]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 49 [288-291]: Paren: Expr 50 [289-290]: Name: Local(NodeId(28))
                            SpecDecl 53 [153-308] (Ctl): Impl:
                                Pat 64 [153-308]: Tuple:
                                    Pat 60 [153-308]: Bind:
                                        Ident 59 [153-308] "ctls"
                                    Pat 65 [153-308]: Elided
                                Block 36 [215-244]:
                                    Stmt 37 [229-234]: Semi: Expr 38 [229-233]: Call:
                                        Expr 61 [229-230]: UnOp (Functor Ctl):
                                            Expr 39 [229-230]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 62 [230-233]: Tuple:
                                            Expr 63 [230-233]: Name: Local(NodeId(59))
                                            Expr 40 [230-233]: Paren: Expr 41 [231-232]: Name: Local(NodeId(28))
                            SpecDecl 54 [153-308] (CtlAdj): Impl:
                                Pat 71 [153-308]: Tuple:
                                    Pat 67 [153-308]: Bind:
                                        Ident 66 [153-308] "ctls"
                                    Pat 72 [153-308]: Elided
                                Block 44 [265-302]:
                                    Stmt 45 [279-292]: Semi: Expr 46 [279-291]: Call:
                                        Expr 68 [279-288]: UnOp (Functor Ctl):
                                            Expr 47 [279-288]: UnOp (Functor Adj):
                                                Expr 48 [287-288]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 69 [288-291]: Tuple:
                                            Expr 70 [288-291]: Name: Local(NodeId(66))
                                            Expr 49 [288-291]: Paren: Expr 50 [289-290]: Name: Local(NodeId(28))"#]],
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
            Package 42:
                Item 0 [21-119]:
                    Callable 1 [21-119] (Operation):
                        name: Ident 2 [31-32] "A"
                        input: Pat 3 [32-43]: Paren:
                            Pat 4 [33-42]: Bind:
                                Ident 5 [33-34] "q"
                                Type 6 [37-42]: Prim (Qubit)
                        output: Type 7 [46-50]: Unit
                        functors: Functor Expr 8 [54-57]: Ctl
                        body: Specializations:
                            SpecDecl 9 [68-79] (Body): Impl:
                                Pat 10 [73-76]: Elided
                                Block 11 [77-79]: <empty>
                            SpecDecl 12 [88-113] (Ctl): Impl:
                                Pat 13 [99-110]: Tuple:
                                    Pat 14 [100-104]: Bind:
                                        Ident 15 [100-104] "ctls"
                                    Pat 16 [106-109]: Elided
                                Block 17 [111-113]: <empty>
                Item 18 [124-257]:
                    Callable 19 [124-257] (Operation):
                        name: Ident 20 [134-135] "B"
                        input: Pat 21 [135-146]: Paren:
                            Pat 22 [136-145]: Bind:
                                Ident 23 [136-137] "q"
                                Type 24 [140-145]: Prim (Qubit)
                        output: Type 25 [149-153]: Unit
                        functors: Functor Expr 26 [157-160]: Ctl
                        body: Specializations:
                            SpecDecl 43 [161-257] (Body): Impl:
                                Pat 44 [161-257]: Elided
                                Block 27 [161-257]:
                                    Stmt 28 [171-251]: Expr: Expr 29 [171-251]: Conjugate:
                                        Block 30 [178-207]:
                                            Stmt 31 [192-197]: Semi: Expr 32 [192-196]: Call:
                                                Expr 33 [192-193]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                                Expr 34 [193-196]: Paren: Expr 35 [194-195]: Name: Local(NodeId(23))
                                        Block 36 [222-251]:
                                            Stmt 37 [236-241]: Semi: Expr 38 [236-240]: Call:
                                                Expr 39 [236-237]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                                Expr 40 [237-240]: Paren: Expr 41 [238-239]: Name: Local(NodeId(23))
                            SpecDecl 45 [124-257] (Ctl): Impl:
                                Pat 51 [124-257]: Tuple:
                                    Pat 47 [124-257]: Bind:
                                        Ident 46 [124-257] "ctls"
                                    Pat 52 [124-257]: Elided
                                Block 27 [161-257]:
                                    Stmt 28 [171-251]: Expr: Expr 29 [171-251]: Conjugate:
                                        Block 30 [178-207]:
                                            Stmt 31 [192-197]: Semi: Expr 32 [192-196]: Call:
                                                Expr 33 [192-193]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                                Expr 34 [193-196]: Paren: Expr 35 [194-195]: Name: Local(NodeId(23))
                                        Block 36 [222-251]:
                                            Stmt 37 [236-241]: Semi: Expr 38 [236-240]: Call:
                                                Expr 48 [236-237]: UnOp (Functor Ctl):
                                                    Expr 39 [236-237]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                                Expr 49 [237-240]: Tuple:
                                                    Expr 50 [237-240]: Name: Local(NodeId(46))
                                                    Expr 40 [237-240]: Paren: Expr 41 [238-239]: Name: Local(NodeId(23))"#]],
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
            Package 28:
                Item 0 [21-45]:
                    Callable 1 [21-45] (Function):
                        name: Ident 2 [30-33] "Foo"
                        input: Pat 3 [33-35]: Unit
                        output: Type 4 [38-42]: Unit
                        body: Block: Block 5 [43-45]: <empty>
                Item 6 [50-80]:
                    Callable 7 [50-80] (Operation):
                        name: Ident 8 [60-61] "A"
                        input: Pat 9 [61-63]: Unit
                        output: Type 10 [66-70]: Unit
                        functors: Functor Expr 11 [74-77]: Ctl
                        body: Specializations:
                            SpecDecl 29 [78-80] (Body): Impl:
                                Pat 30 [78-80]: Elided
                                Block 12 [78-80]: <empty>
                            SpecDecl 31 [50-80] (Ctl): Impl:
                                Pat 37 [50-80]: Tuple:
                                    Pat 36 [50-80]: Bind:
                                        Ident 35 [50-80] "ctls"
                                    Pat 38 [50-80]: Elided
                                Block 12 [78-80]: <empty>
                Item 13 [85-148]:
                    Callable 14 [85-148] (Operation):
                        name: Ident 15 [95-96] "B"
                        input: Pat 16 [96-98]: Unit
                        output: Type 17 [101-105]: Unit
                        functors: Functor Expr 18 [109-112]: Ctl
                        body: Specializations:
                            SpecDecl 32 [113-148] (Body): Impl:
                                Pat 33 [113-148]: Elided
                                Block 19 [113-148]:
                                    Stmt 20 [123-129]: Semi: Expr 21 [123-128]: Call:
                                        Expr 22 [123-126]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 23 [126-128]: Unit
                                    Stmt 24 [138-142]: Semi: Expr 25 [138-141]: Call:
                                        Expr 26 [138-139]: Name: Def(DefId { package: None, def: PackageDefId(1) })
                                        Expr 27 [139-141]: Unit
                            SpecDecl 34 [85-148] (Ctl): Impl:
                                Pat 44 [85-148]: Tuple:
                                    Pat 40 [85-148]: Bind:
                                        Ident 39 [85-148] "ctls"
                                    Pat 45 [85-148]: Elided
                                Block 19 [113-148]:
                                    Stmt 20 [123-129]: Semi: Expr 21 [123-128]: Call:
                                        Expr 22 [123-126]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 23 [126-128]: Unit
                                    Stmt 24 [138-142]: Semi: Expr 25 [138-141]: Call:
                                        Expr 41 [138-139]: UnOp (Functor Ctl):
                                            Expr 26 [138-139]: Name: Def(DefId { package: None, def: PackageDefId(1) })
                                        Expr 42 [139-141]: Tuple:
                                            Expr 43 [139-141]: Name: Local(NodeId(39))
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
            Package 33:
                Item 0 [21-62]:
                    Callable 1 [21-62] (Operation):
                        name: Ident 2 [31-32] "B"
                        input: Pat 3 [32-45]: Paren:
                            Pat 4 [33-44]: Bind:
                                Ident 5 [33-38] "input"
                                Type 6 [41-44]: Prim (Int)
                        output: Type 7 [48-52]: Unit
                        functors: Functor Expr 8 [56-59]: Adj
                        body: Specializations:
                            SpecDecl 34 [60-62] (Body): Impl:
                                Pat 35 [60-62]: Elided
                                Block 9 [60-62]: <empty>
                            SpecDecl 36 [21-62] (Adj): Gen: Invert
                Item 10 [67-166]:
                    Callable 11 [67-166] (Operation):
                        name: Ident 12 [77-78] "A"
                        input: Pat 13 [78-89]: Paren:
                            Pat 14 [79-88]: Bind:
                                Ident 15 [79-80] "q"
                                Type 16 [83-88]: Prim (Qubit)
                        output: Type 17 [92-96]: Unit
                        functors: Functor Expr 18 [100-103]: Adj
                        body: Specializations:
                            SpecDecl 19 [114-138] (Body): Impl:
                                Pat 20 [119-122]: Elided
                                Block 21 [123-138]:
                                    Stmt 22 [125-130]: Semi: Expr 23 [125-129]: Call:
                                        Expr 24 [125-126]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 25 [126-129]: Paren: Expr 26 [127-128]: Lit: Int(1)
                                    Stmt 27 [131-136]: Semi: Expr 28 [131-135]: Call:
                                        Expr 29 [131-132]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 30 [132-135]: Paren: Expr 31 [133-134]: Lit: Int(2)
                            SpecDecl 32 [147-160] (Adj): Impl:
                                Pat 20 [119-122]: Elided
                                Block 21 [123-138]:
                                    Stmt 22 [125-130]: Semi: Expr 23 [125-129]: Call:
                                        Expr 24 [125-126]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 25 [126-129]: Paren: Expr 26 [127-128]: Lit: Int(1)
                                    Stmt 27 [131-136]: Semi: Expr 28 [131-135]: Call:
                                        Expr 29 [131-132]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 30 [132-135]: Paren: Expr 31 [133-134]: Lit: Int(2)"#]],
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
            Package 37:
                Item 0 [21-68]:
                    Callable 1 [21-68] (Operation):
                        name: Ident 2 [31-32] "B"
                        input: Pat 3 [32-45]: Paren:
                            Pat 4 [33-44]: Bind:
                                Ident 5 [33-38] "input"
                                Type 6 [41-44]: Prim (Int)
                        output: Type 7 [48-52]: Unit
                        functors: Functor Expr 8 [56-65]: BinOp Union: (Functor Expr 9 [56-59]: Ctl) (Functor Expr 10 [62-65]: Adj)
                        body: Specializations:
                            SpecDecl 38 [66-68] (Body): Impl:
                                Pat 39 [66-68]: Elided
                                Block 11 [66-68]: <empty>
                            SpecDecl 40 [21-68] (Adj): Gen: Invert
                            SpecDecl 41 [21-68] (Ctl): Impl:
                                Pat 47 [21-68]: Tuple:
                                    Pat 46 [21-68]: Bind:
                                        Ident 45 [21-68] "ctls"
                                    Pat 48 [21-68]: Elided
                                Block 11 [66-68]: <empty>
                            SpecDecl 42 [21-68] (CtlAdj): Gen: Distribute
                Item 12 [73-178]:
                    Callable 13 [73-178] (Operation):
                        name: Ident 14 [83-84] "A"
                        input: Pat 15 [84-95]: Paren:
                            Pat 16 [85-94]: Bind:
                                Ident 17 [85-86] "q"
                                Type 18 [89-94]: Prim (Qubit)
                        output: Type 19 [98-102]: Unit
                        functors: Functor Expr 20 [106-115]: BinOp Union: (Functor Expr 21 [106-109]: Ctl) (Functor Expr 22 [112-115]: Adj)
                        body: Specializations:
                            SpecDecl 23 [126-150] (Body): Impl:
                                Pat 24 [131-134]: Elided
                                Block 25 [135-150]:
                                    Stmt 26 [137-142]: Semi: Expr 27 [137-141]: Call:
                                        Expr 28 [137-138]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 29 [138-141]: Paren: Expr 30 [139-140]: Lit: Int(1)
                                    Stmt 31 [143-148]: Semi: Expr 32 [143-147]: Call:
                                        Expr 33 [143-144]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 34 [144-147]: Paren: Expr 35 [145-146]: Lit: Int(2)
                            SpecDecl 36 [159-172] (Adj): Impl:
                                Pat 24 [131-134]: Elided
                                Block 25 [135-150]:
                                    Stmt 26 [137-142]: Semi: Expr 27 [137-141]: Call:
                                        Expr 28 [137-138]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 29 [138-141]: Paren: Expr 30 [139-140]: Lit: Int(1)
                                    Stmt 31 [143-148]: Semi: Expr 32 [143-147]: Call:
                                        Expr 33 [143-144]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 34 [144-147]: Paren: Expr 35 [145-146]: Lit: Int(2)
                            SpecDecl 43 [73-178] (Ctl): Impl:
                                Pat 57 [73-178]: Tuple:
                                    Pat 50 [73-178]: Bind:
                                        Ident 49 [73-178] "ctls"
                                    Pat 58 [73-178]: Elided
                                Block 25 [135-150]:
                                    Stmt 26 [137-142]: Semi: Expr 27 [137-141]: Call:
                                        Expr 51 [137-138]: UnOp (Functor Ctl):
                                            Expr 28 [137-138]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 52 [138-141]: Tuple:
                                            Expr 53 [138-141]: Name: Local(NodeId(49))
                                            Expr 29 [138-141]: Paren: Expr 30 [139-140]: Lit: Int(1)
                                    Stmt 31 [143-148]: Semi: Expr 32 [143-147]: Call:
                                        Expr 54 [143-144]: UnOp (Functor Ctl):
                                            Expr 33 [143-144]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 55 [144-147]: Tuple:
                                            Expr 56 [144-147]: Name: Local(NodeId(49))
                                            Expr 34 [144-147]: Paren: Expr 35 [145-146]: Lit: Int(2)
                            SpecDecl 44 [73-178] (CtlAdj): Impl:
                                Pat 57 [73-178]: Tuple:
                                    Pat 50 [73-178]: Bind:
                                        Ident 49 [73-178] "ctls"
                                    Pat 58 [73-178]: Elided
                                Block 25 [135-150]:
                                    Stmt 26 [137-142]: Semi: Expr 27 [137-141]: Call:
                                        Expr 51 [137-138]: UnOp (Functor Ctl):
                                            Expr 28 [137-138]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 52 [138-141]: Tuple:
                                            Expr 53 [138-141]: Name: Local(NodeId(49))
                                            Expr 29 [138-141]: Paren: Expr 30 [139-140]: Lit: Int(1)
                                    Stmt 31 [143-148]: Semi: Expr 32 [143-147]: Call:
                                        Expr 54 [143-144]: UnOp (Functor Ctl):
                                            Expr 33 [143-144]: Name: Def(DefId { package: None, def: PackageDefId(0) })
                                        Expr 55 [144-147]: Tuple:
                                            Expr 56 [144-147]: Name: Local(NodeId(49))
                                            Expr 34 [144-147]: Paren: Expr 35 [145-146]: Lit: Int(2)"#]],
    );
}
