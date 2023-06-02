// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, PackageStore, SourceMap};
use expect_test::{expect, Expect};
use indoc::indoc;

fn check_hir(input: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), input.into())], None);
    let unit = compile(&PackageStore::new(compile::core()), &[], sources);
    expect.assert_eq(&unit.package.to_string());
}

fn check_errors(input: &str, expect: &Expect) {
    let sources = SourceMap::new([("test".into(), input.into())], None);
    let unit = compile(&PackageStore::new(compile::core()), &[], sources);

    let lower_errors: Vec<_> = unit
        .errors
        .into_iter()
        .filter_map(try_into_lower_error)
        .collect();

    expect.assert_debug_eq(&lower_errors);
}

fn try_into_lower_error(error: compile::Error) -> Option<super::Error> {
    if let compile::ErrorKind::Lower(error) = error.0 {
        Some(error)
    } else {
        None
    }
}

#[test]
fn test_entrypoint_attr_allowed() {
    check_errors(
        indoc! {"
            namespace input {
                @EntryPoint()
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            []
        "#]],
    );
}

#[test]
fn test_entrypoint_attr_wrong_args() {
    check_errors(
        indoc! {r#"
            namespace input {
                @EntryPoint("Bar")
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "#},
        &expect![[r#"
            [
                InvalidAttrArgs(
                    "()",
                    Span {
                        lo: 33,
                        hi: 40,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn test_unknown_attr() {
    check_errors(
        indoc! {"
            namespace input {
                @Bar()
                operation Foo() : Unit {
                    body ... {}
                }
            }
        "},
        &expect![[r#"
            [
                UnknownAttr(
                    "Bar",
                    Span {
                        lo: 23,
                        hi: 26,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn lift_local_function() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(x : Int) : Int {
                    function Bar(y : Int) : Int { y + 1 }
                    Bar(x + 2)
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-120] (Public):
                    Namespace (Ident 25 [10-11] "A"): Item 1
                Item 1 [18-118] (Public):
                    Parent: 0
                    Callable 0 [18-118] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-38] [Type Int]: Bind: Ident 3 [31-32] "x"
                        output: Int
                        functors: empty set
                        body: SpecDecl 4 [18-118] (Body): Impl:
                            Pat 5 [18-118] [Type Int]: Elided
                            Block 6 [46-118] [Type Int]:
                                Stmt 7 [56-93]: Item: 2
                                Stmt 19 [102-112]: Expr: Expr 20 [102-112] [Type Int]: Call:
                                    Expr 21 [102-105] [Type (Int -> Int)]: Var: Item 2
                                    Expr 22 [106-111] [Type Int]: BinOp (Add):
                                        Expr 23 [106-107] [Type Int]: Var: Local 3
                                        Expr 24 [110-111] [Type Int]: Lit: Int(2)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [56-93] (Internal):
                    Parent: 1
                    Callable 8 [56-93] (Function):
                        name: Ident 9 [65-68] "Bar"
                        input: Pat 10 [69-76] [Type Int]: Bind: Ident 11 [69-70] "y"
                        output: Int
                        functors: empty set
                        body: SpecDecl 12 [56-93] (Body): Impl:
                            Pat 13 [56-93] [Type Int]: Elided
                            Block 14 [84-93] [Type Int]:
                                Stmt 15 [86-91]: Expr: Expr 16 [86-91] [Type Int]: BinOp (Add):
                                    Expr 17 [86-87] [Type Int]: Var: Local 11
                                    Expr 18 [90-91] [Type Int]: Lit: Int(1)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lift_local_operation() {
    check_hir(
        indoc! {"
            namespace A {
                operation Foo() : Result {
                    operation Bar(q : Qubit) : Result { Zero }
                    use q = Qubit();
                    Bar(q)
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-143] (Public):
                    Namespace (Ident 24 [10-11] "A"): Item 1
                Item 1 [18-141] (Public):
                    Parent: 0
                    Callable 0 [18-141] (Operation):
                        name: Ident 1 [28-31] "Foo"
                        input: Pat 2 [31-33] [Type Unit]: Unit
                        output: Result
                        functors: empty set
                        body: SpecDecl 3 [18-141] (Body): Impl:
                            Pat 4 [18-141] [Type Unit]: Elided
                            Block 5 [43-141] [Type Result]:
                                Stmt 6 [53-95]: Item: 2
                                Stmt 16 [104-120]: Qubit (Fresh)
                                    Pat 17 [108-109] [Type Qubit]: Bind: Ident 18 [108-109] "q"
                                    QubitInit 19 [112-119] [Type Qubit]: Single
                                Stmt 20 [129-135]: Expr: Expr 21 [129-135] [Type Result]: Call:
                                    Expr 22 [129-132] [Type (Qubit => Result)]: Var: Item 2
                                    Expr 23 [133-134] [Type Qubit]: Var: Local 18
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [53-95] (Internal):
                    Parent: 1
                    Callable 7 [53-95] (Operation):
                        name: Ident 8 [63-66] "Bar"
                        input: Pat 9 [67-76] [Type Qubit]: Bind: Ident 10 [67-68] "q"
                        output: Result
                        functors: empty set
                        body: SpecDecl 11 [53-95] (Body): Impl:
                            Pat 12 [53-95] [Type Qubit]: Elided
                            Block 13 [87-95] [Type Result]:
                                Stmt 14 [89-93]: Expr: Expr 15 [89-93] [Type Result]: Lit: Result(Zero)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lift_local_newtype() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    newtype Bar = Int;
                    let x = Bar(5);
                    x!
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-110] (Public):
                    Namespace (Ident 17 [10-11] "A"): Item 1
                Item 1 [18-108] (Public):
                    Parent: 0
                    Callable 0 [18-108] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [18-108] (Body): Impl:
                            Pat 4 [18-108] [Type Unit]: Elided
                            Block 5 [39-108] [Type Int]:
                                Stmt 6 [49-67]: Item: 2
                                Stmt 8 [76-91]: Local (Immutable):
                                    Pat 9 [80-81] [Type UDT<Item 2>]: Bind: Ident 10 [80-81] "x"
                                    Expr 11 [84-90] [Type UDT<Item 2>]: Call:
                                        Expr 12 [84-87] [Type (Int -> UDT<Item 2>)]: Var: Item 2
                                        Expr 13 [88-89] [Type Int]: Lit: Int(5)
                                Stmt 14 [100-102]: Expr: Expr 15 [100-102] [Type Int]: UnOp (Unwrap):
                                    Expr 16 [100-101] [Type UDT<Item 2>]: Var: Local 10
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [49-67] (Internal):
                    Parent: 1
                    Type (Ident 7 [57-60] "Bar"): Udt:
                        base: Int
                        fields:"#]],
    );
}

#[test]
fn lambda_function_empty_closure() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    let f = x -> x + 1;
                    f(1)
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-89] (Public):
                    Namespace (Ident 26 [10-11] "A"): Item 1
                Item 1 [18-87] (Public):
                    Parent: 0
                    Callable 0 [18-87] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [18-87] (Body): Impl:
                            Pat 4 [18-87] [Type Unit]: Elided
                            Block 5 [39-87] [Type Int]:
                                Stmt 6 [49-68]: Local (Immutable):
                                    Pat 7 [53-54] [Type (Int -> Int is f?0)]: Bind: Ident 8 [53-54] "f"
                                    Expr 9 [57-67] [Type (Int -> Int is f?0)]: Closure([], 2)
                                Stmt 22 [77-81]: Expr: Expr 23 [77-81] [Type Int]: Call:
                                    Expr 24 [77-78] [Type (Int -> Int is f?0)]: Var: Local 8
                                    Expr 25 [79-80] [Type Int]: Lit: Int(1)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [57-67] (Internal):
                    Parent: 1
                    Callable 16 [57-67] (Function):
                        name: Ident 17 [57-67] "lambda"
                        input: Pat 15 [57-67] [Type (Int,)]: Tuple:
                            Pat 10 [57-58] [Type Int]: Bind: Ident 11 [57-58] "x"
                        output: Int
                        functors: f?0
                        body: SpecDecl 18 [62-67] (Body): Impl:
                            Pat 19 [57-67] [Type (Int,)]: Elided
                            Block 20 [62-67] [Type Int]:
                                Stmt 21 [62-67]: Expr: Expr 12 [62-67] [Type Int]: BinOp (Add):
                                    Expr 13 [62-63] [Type Int]: Var: Local 11
                                    Expr 14 [66-67] [Type Int]: Lit: Int(1)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_function_empty_closure_passed() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(f : Int -> Int) : Int { f(2) }
                function Bar() : Int { Foo(x -> x + 1) }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-108] (Public):
                    Namespace (Ident 33 [10-11] "A"): Item 1, Item 2
                Item 1 [18-61] (Public):
                    Parent: 0
                    Callable 0 [18-61] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-45] [Type (Int -> Int)]: Bind: Ident 3 [31-32] "f"
                        output: Int
                        functors: empty set
                        body: SpecDecl 4 [18-61] (Body): Impl:
                            Pat 5 [18-61] [Type (Int -> Int)]: Elided
                            Block 6 [53-61] [Type Int]:
                                Stmt 7 [55-59]: Expr: Expr 8 [55-59] [Type Int]: Call:
                                    Expr 9 [55-56] [Type (Int -> Int)]: Var: Local 3
                                    Expr 10 [57-58] [Type Int]: Lit: Int(2)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [66-106] (Public):
                    Parent: 0
                    Callable 11 [66-106] (Function):
                        name: Ident 12 [75-78] "Bar"
                        input: Pat 13 [78-80] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 14 [66-106] (Body): Impl:
                            Pat 15 [66-106] [Type Unit]: Elided
                            Block 16 [87-106] [Type Int]:
                                Stmt 17 [89-104]: Expr: Expr 18 [89-104] [Type Int]: Call:
                                    Expr 19 [89-92] [Type ((Int -> Int) -> Int)]: Var: Item 1
                                    Expr 20 [93-103] [Type (Int -> Int)]: Closure([], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [93-103] (Internal):
                    Parent: 2
                    Callable 27 [93-103] (Function):
                        name: Ident 28 [93-103] "lambda"
                        input: Pat 26 [93-103] [Type (Int,)]: Tuple:
                            Pat 21 [93-94] [Type Int]: Bind: Ident 22 [93-94] "x"
                        output: Int
                        functors: empty set
                        body: SpecDecl 29 [98-103] (Body): Impl:
                            Pat 30 [93-103] [Type (Int,)]: Elided
                            Block 31 [98-103] [Type Int]:
                                Stmt 32 [98-103]: Expr: Expr 23 [98-103] [Type Int]: BinOp (Add):
                                    Expr 24 [98-99] [Type Int]: Var: Local 22
                                    Expr 25 [102-103] [Type Int]: Lit: Int(1)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_function_closure() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    let x = 5;
                    let f = y -> x + y;
                    f(2)
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-108] (Public):
                    Namespace (Ident 32 [10-11] "A"): Item 1
                Item 1 [18-106] (Public):
                    Parent: 0
                    Callable 0 [18-106] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [18-106] (Body): Impl:
                            Pat 4 [18-106] [Type Unit]: Elided
                            Block 5 [39-106] [Type Int]:
                                Stmt 6 [49-59]: Local (Immutable):
                                    Pat 7 [53-54] [Type Int]: Bind: Ident 8 [53-54] "x"
                                    Expr 9 [57-58] [Type Int]: Lit: Int(5)
                                Stmt 10 [68-87]: Local (Immutable):
                                    Pat 11 [72-73] [Type (Int -> Int is f?0)]: Bind: Ident 12 [72-73] "f"
                                    Expr 13 [76-86] [Type (Int -> Int is f?0)]: Closure([8], 2)
                                Stmt 28 [96-100]: Expr: Expr 29 [96-100] [Type Int]: Call:
                                    Expr 30 [96-97] [Type (Int -> Int is f?0)]: Var: Local 12
                                    Expr 31 [98-99] [Type Int]: Lit: Int(2)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [76-86] (Internal):
                    Parent: 1
                    Callable 22 [76-86] (Function):
                        name: Ident 23 [76-86] "lambda"
                        input: Pat 20 [76-86] [Type (Int, Int)]: Tuple:
                            Pat 21 [76-86] [Type Int]: Bind: Ident 19 [76-86] "closed"
                            Pat 14 [76-77] [Type Int]: Bind: Ident 15 [76-77] "y"
                        output: Int
                        functors: f?0
                        body: SpecDecl 24 [81-86] (Body): Impl:
                            Pat 25 [76-86] [Type (Int, Int)]: Elided
                            Block 26 [81-86] [Type Int]:
                                Stmt 27 [81-86]: Expr: Expr 16 [81-86] [Type Int]: BinOp (Add):
                                    Expr 17 [81-82] [Type Int]: Var: Local 19
                                    Expr 18 [85-86] [Type Int]: Var: Local 15
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_function_closure_repeated_var() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    let x = 5;
                    let f = y -> x + x + y;
                    f(2)
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-112] (Public):
                    Namespace (Ident 34 [10-11] "A"): Item 1
                Item 1 [18-110] (Public):
                    Parent: 0
                    Callable 0 [18-110] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 3 [18-110] (Body): Impl:
                            Pat 4 [18-110] [Type Unit]: Elided
                            Block 5 [39-110] [Type Int]:
                                Stmt 6 [49-59]: Local (Immutable):
                                    Pat 7 [53-54] [Type Int]: Bind: Ident 8 [53-54] "x"
                                    Expr 9 [57-58] [Type Int]: Lit: Int(5)
                                Stmt 10 [68-91]: Local (Immutable):
                                    Pat 11 [72-73] [Type (Int -> Int is f?0)]: Bind: Ident 12 [72-73] "f"
                                    Expr 13 [76-90] [Type (Int -> Int is f?0)]: Closure([8], 2)
                                Stmt 30 [100-104]: Expr: Expr 31 [100-104] [Type Int]: Call:
                                    Expr 32 [100-101] [Type (Int -> Int is f?0)]: Var: Local 12
                                    Expr 33 [102-103] [Type Int]: Lit: Int(2)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [76-90] (Internal):
                    Parent: 1
                    Callable 24 [76-90] (Function):
                        name: Ident 25 [76-90] "lambda"
                        input: Pat 22 [76-90] [Type (Int, Int)]: Tuple:
                            Pat 23 [76-90] [Type Int]: Bind: Ident 21 [76-90] "closed"
                            Pat 14 [76-77] [Type Int]: Bind: Ident 15 [76-77] "y"
                        output: Int
                        functors: f?0
                        body: SpecDecl 26 [81-90] (Body): Impl:
                            Pat 27 [76-90] [Type (Int, Int)]: Elided
                            Block 28 [81-90] [Type Int]:
                                Stmt 29 [81-90]: Expr: Expr 16 [81-90] [Type Int]: BinOp (Add):
                                    Expr 17 [81-86] [Type Int]: BinOp (Add):
                                        Expr 18 [81-82] [Type Int]: Var: Local 21
                                        Expr 19 [85-86] [Type Int]: Var: Local 21
                                    Expr 20 [89-90] [Type Int]: Var: Local 15
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_function_closure_passed() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(f : Int -> Int) : Int { f(2) }
                function Bar() : Int {
                    let x = 5;
                    Foo(y -> x + y)
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-139] (Public):
                    Namespace (Ident 39 [10-11] "A"): Item 1, Item 2
                Item 1 [18-61] (Public):
                    Parent: 0
                    Callable 0 [18-61] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-45] [Type (Int -> Int)]: Bind: Ident 3 [31-32] "f"
                        output: Int
                        functors: empty set
                        body: SpecDecl 4 [18-61] (Body): Impl:
                            Pat 5 [18-61] [Type (Int -> Int)]: Elided
                            Block 6 [53-61] [Type Int]:
                                Stmt 7 [55-59]: Expr: Expr 8 [55-59] [Type Int]: Call:
                                    Expr 9 [55-56] [Type (Int -> Int)]: Var: Local 3
                                    Expr 10 [57-58] [Type Int]: Lit: Int(2)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [66-137] (Public):
                    Parent: 0
                    Callable 11 [66-137] (Function):
                        name: Ident 12 [75-78] "Bar"
                        input: Pat 13 [78-80] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 14 [66-137] (Body): Impl:
                            Pat 15 [66-137] [Type Unit]: Elided
                            Block 16 [87-137] [Type Int]:
                                Stmt 17 [97-107]: Local (Immutable):
                                    Pat 18 [101-102] [Type Int]: Bind: Ident 19 [101-102] "x"
                                    Expr 20 [105-106] [Type Int]: Lit: Int(5)
                                Stmt 21 [116-131]: Expr: Expr 22 [116-131] [Type Int]: Call:
                                    Expr 23 [116-119] [Type ((Int -> Int) -> Int)]: Var: Item 1
                                    Expr 24 [120-130] [Type (Int -> Int)]: Closure([19], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [120-130] (Internal):
                    Parent: 2
                    Callable 33 [120-130] (Function):
                        name: Ident 34 [120-130] "lambda"
                        input: Pat 31 [120-130] [Type (Int, Int)]: Tuple:
                            Pat 32 [120-130] [Type Int]: Bind: Ident 30 [120-130] "closed"
                            Pat 25 [120-121] [Type Int]: Bind: Ident 26 [120-121] "y"
                        output: Int
                        functors: empty set
                        body: SpecDecl 35 [125-130] (Body): Impl:
                            Pat 36 [120-130] [Type (Int, Int)]: Elided
                            Block 37 [125-130] [Type Int]:
                                Stmt 38 [125-130]: Expr: Expr 27 [125-130] [Type Int]: BinOp (Add):
                                    Expr 28 [125-126] [Type Int]: Var: Local 30
                                    Expr 29 [129-130] [Type Int]: Var: Local 26
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_function_nested_closure() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(f : Int -> Int -> Int) : Int { f(2)(3) }
                function Bar() : Int {
                    let a = 5;
                    Foo(b -> {
                        let c = 1;
                        d -> a + b + c + d
                    })
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-209] (Public):
                    Namespace (Ident 68 [10-11] "A"): Item 1, Item 2
                Item 1 [18-71] (Public):
                    Parent: 0
                    Callable 0 [18-71] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-52] [Type (Int -> (Int -> Int))]: Bind: Ident 3 [31-32] "f"
                        output: Int
                        functors: empty set
                        body: SpecDecl 4 [18-71] (Body): Impl:
                            Pat 5 [18-71] [Type (Int -> (Int -> Int))]: Elided
                            Block 6 [60-71] [Type Int]:
                                Stmt 7 [62-69]: Expr: Expr 8 [62-69] [Type Int]: Call:
                                    Expr 9 [62-66] [Type (Int -> Int)]: Call:
                                        Expr 10 [62-63] [Type (Int -> (Int -> Int))]: Var: Local 3
                                        Expr 11 [64-65] [Type Int]: Lit: Int(2)
                                    Expr 12 [67-68] [Type Int]: Lit: Int(3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [76-207] (Public):
                    Parent: 0
                    Callable 13 [76-207] (Function):
                        name: Ident 14 [85-88] "Bar"
                        input: Pat 15 [88-90] [Type Unit]: Unit
                        output: Int
                        functors: empty set
                        body: SpecDecl 16 [76-207] (Body): Impl:
                            Pat 17 [76-207] [Type Unit]: Elided
                            Block 18 [97-207] [Type Int]:
                                Stmt 19 [107-117]: Local (Immutable):
                                    Pat 20 [111-112] [Type Int]: Bind: Ident 21 [111-112] "a"
                                    Expr 22 [115-116] [Type Int]: Lit: Int(5)
                                Stmt 23 [126-201]: Expr: Expr 24 [126-201] [Type Int]: Call:
                                    Expr 25 [126-129] [Type ((Int -> (Int -> Int)) -> Int)]: Var: Item 1
                                    Expr 26 [130-200] [Type (Int -> (Int -> Int))]: Closure([21], 4)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [172-190] (Internal):
                    Parent: 2
                    Callable 53 [172-190] (Function):
                        name: Ident 54 [172-190] "lambda"
                        input: Pat 49 [172-190] [Type (Int, Int, Int, Int)]: Tuple:
                            Pat 50 [172-190] [Type Int]: Bind: Ident 46 [172-190] "closed"
                            Pat 51 [172-190] [Type Int]: Bind: Ident 47 [172-190] "closed"
                            Pat 52 [172-190] [Type Int]: Bind: Ident 48 [172-190] "closed"
                            Pat 37 [172-173] [Type Int]: Bind: Ident 38 [172-173] "d"
                        output: Int
                        functors: empty set
                        body: SpecDecl 55 [177-190] (Body): Impl:
                            Pat 56 [172-190] [Type (Int, Int, Int, Int)]: Elided
                            Block 57 [177-190] [Type Int]:
                                Stmt 58 [177-190]: Expr: Expr 39 [177-190] [Type Int]: BinOp (Add):
                                    Expr 40 [177-186] [Type Int]: BinOp (Add):
                                        Expr 41 [177-182] [Type Int]: BinOp (Add):
                                            Expr 42 [177-178] [Type Int]: Var: Local 46
                                            Expr 43 [181-182] [Type Int]: Var: Local 47
                                        Expr 44 [185-186] [Type Int]: Var: Local 48
                                    Expr 45 [189-190] [Type Int]: Var: Local 38
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 4 [130-200] (Internal):
                    Parent: 2
                    Callable 62 [130-200] (Function):
                        name: Ident 63 [130-200] "lambda"
                        input: Pat 60 [130-200] [Type (Int, Int)]: Tuple:
                            Pat 61 [130-200] [Type Int]: Bind: Ident 59 [130-200] "closed"
                            Pat 27 [130-131] [Type Int]: Bind: Ident 28 [130-131] "b"
                        output: (Int -> Int)
                        functors: empty set
                        body: SpecDecl 64 [135-200] (Body): Impl:
                            Pat 65 [130-200] [Type (Int, Int)]: Elided
                            Block 66 [135-200] [Type (Int -> Int)]:
                                Stmt 67 [135-200]: Expr: Expr 29 [135-200] [Type (Int -> Int)]: Expr Block: Block 30 [135-200] [Type (Int -> Int)]:
                                    Stmt 31 [149-159]: Local (Immutable):
                                        Pat 32 [153-154] [Type Int]: Bind: Ident 33 [153-154] "c"
                                        Expr 34 [157-158] [Type Int]: Lit: Int(1)
                                    Stmt 35 [172-190]: Expr: Expr 36 [172-190] [Type (Int -> Int)]: Closure([59, 28, 33], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_operation_empty_closure() {
    check_hir(
        indoc! {"
            namespace A {
                operation Foo(op : Qubit => ()) : () {
                    use q = Qubit();
                    op(q)
                }
                operation Bar() : Result { Foo(q => ()) }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-149] (Public):
                    Namespace (Ident 35 [10-11] "A"): Item 1, Item 2
                Item 1 [18-101] (Public):
                    Parent: 0
                    Callable 0 [18-101] (Operation):
                        name: Ident 1 [28-31] "Foo"
                        input: Pat 2 [32-48] [Type (Qubit => Unit)]: Bind: Ident 3 [32-34] "op"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 4 [18-101] (Body): Impl:
                            Pat 5 [18-101] [Type (Qubit => Unit)]: Elided
                            Block 6 [55-101] [Type Unit]:
                                Stmt 7 [65-81]: Qubit (Fresh)
                                    Pat 8 [69-70] [Type Qubit]: Bind: Ident 9 [69-70] "q"
                                    QubitInit 10 [73-80] [Type Qubit]: Single
                                Stmt 11 [90-95]: Expr: Expr 12 [90-95] [Type Unit]: Call:
                                    Expr 13 [90-92] [Type (Qubit => Unit)]: Var: Local 3
                                    Expr 14 [93-94] [Type Qubit]: Var: Local 9
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [106-147] (Public):
                    Parent: 0
                    Callable 15 [106-147] (Operation):
                        name: Ident 16 [116-119] "Bar"
                        input: Pat 17 [119-121] [Type Unit]: Unit
                        output: Result
                        functors: empty set
                        body: SpecDecl 18 [106-147] (Body): Impl:
                            Pat 19 [106-147] [Type Unit]: Elided
                            Block 20 [131-147] [Type Result]:
                                Stmt 21 [133-145]: Expr: Expr 22 [133-145] [Type Result]: Call:
                                    Expr 23 [133-136] [Type ((Qubit => Unit) => Unit)]: Var: Item 1
                                    Expr 24 [137-144] [Type (Qubit => Unit)]: Closure([], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [137-144] (Internal):
                    Parent: 2
                    Callable 29 [137-144] (Operation):
                        name: Ident 30 [137-144] "lambda"
                        input: Pat 28 [137-144] [Type (Qubit,)]: Tuple:
                            Pat 25 [137-138] [Type Qubit]: Bind: Ident 26 [137-138] "q"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 31 [142-144] (Body): Impl:
                            Pat 32 [137-144] [Type (Qubit,)]: Elided
                            Block 33 [142-144] [Type Unit]:
                                Stmt 34 [142-144]: Expr: Expr 27 [142-144] [Type Unit]: Unit
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_operation_closure() {
    check_hir(
        indoc! {"
            namespace A {
                operation MResetZ(q : Qubit) : Result { body intrinsic; }
                operation Foo(op : () => Result) : Result { op() }
                operation Bar() : Result {
                    use q = Qubit();
                    Foo(() => MResetZ(q))
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-224] (Public):
                    Namespace (Ident 43 [10-11] "A"): Item 1, Item 2, Item 3
                Item 1 [18-75] (Public):
                    Parent: 0
                    Callable 0 [18-75] (Operation):
                        name: Ident 1 [28-35] "MResetZ"
                        input: Pat 2 [36-45] [Type Qubit]: Bind: Ident 3 [36-37] "q"
                        output: Result
                        functors: empty set
                        body: SpecDecl 4 [58-73] (Body): Gen: Intrinsic
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [80-130] (Public):
                    Parent: 0
                    Callable 5 [80-130] (Operation):
                        name: Ident 6 [90-93] "Foo"
                        input: Pat 7 [94-111] [Type (Unit => Result)]: Bind: Ident 8 [94-96] "op"
                        output: Result
                        functors: empty set
                        body: SpecDecl 9 [80-130] (Body): Impl:
                            Pat 10 [80-130] [Type (Unit => Result)]: Elided
                            Block 11 [122-130] [Type Result]:
                                Stmt 12 [124-128]: Expr: Expr 13 [124-128] [Type Result]: Call:
                                    Expr 14 [124-126] [Type (Unit => Result)]: Var: Local 8
                                    Expr 15 [126-128] [Type Unit]: Unit
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [135-222] (Public):
                    Parent: 0
                    Callable 16 [135-222] (Operation):
                        name: Ident 17 [145-148] "Bar"
                        input: Pat 18 [148-150] [Type Unit]: Unit
                        output: Result
                        functors: empty set
                        body: SpecDecl 19 [135-222] (Body): Impl:
                            Pat 20 [135-222] [Type Unit]: Elided
                            Block 21 [160-222] [Type Result]:
                                Stmt 22 [170-186]: Qubit (Fresh)
                                    Pat 23 [174-175] [Type Qubit]: Bind: Ident 24 [174-175] "q"
                                    QubitInit 25 [178-185] [Type Qubit]: Single
                                Stmt 26 [195-216]: Expr: Expr 27 [195-216] [Type Result]: Call:
                                    Expr 28 [195-198] [Type ((Unit => Result) => Result)]: Var: Item 2
                                    Expr 29 [199-215] [Type (Unit => Result)]: Closure([24], 4)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 4 [199-215] (Internal):
                    Parent: 3
                    Callable 37 [199-215] (Operation):
                        name: Ident 38 [199-215] "lambda"
                        input: Pat 35 [199-215] [Type (Qubit, Unit)]: Tuple:
                            Pat 36 [199-215] [Type Qubit]: Bind: Ident 34 [199-215] "closed"
                            Pat 30 [199-201] [Type Unit]: Unit
                        output: Result
                        functors: empty set
                        body: SpecDecl 39 [205-215] (Body): Impl:
                            Pat 40 [199-215] [Type (Qubit, Unit)]: Elided
                            Block 41 [205-215] [Type Result]:
                                Stmt 42 [205-215]: Expr: Expr 31 [205-215] [Type Result]: Call:
                                    Expr 32 [205-212] [Type (Qubit => Result)]: Var: Item 1
                                    Expr 33 [213-214] [Type Qubit]: Var: Local 34
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn lambda_adj() {
    check_hir(
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
                        adj: <none>
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
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_one_hole() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(x : Int, y : Int) : Int { x + y }
                function Bar() : () { let f = Foo(_, 2); }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-113] (Public):
                    Namespace (Ident 48 [10-11] "A"): Item 1, Item 2
                Item 1 [18-64] (Public):
                    Parent: 0
                    Callable 0 [18-64] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-48] [Type (Int, Int)]: Tuple:
                            Pat 3 [31-38] [Type Int]: Bind: Ident 4 [31-32] "x"
                            Pat 5 [40-47] [Type Int]: Bind: Ident 6 [40-41] "y"
                        output: Int
                        functors: empty set
                        body: SpecDecl 7 [18-64] (Body): Impl:
                            Pat 8 [18-64] [Type (Int, Int)]: Elided
                            Block 9 [55-64] [Type Int]:
                                Stmt 10 [57-62]: Expr: Expr 11 [57-62] [Type Int]: BinOp (Add):
                                    Expr 12 [57-58] [Type Int]: Var: Local 4
                                    Expr 13 [61-62] [Type Int]: Var: Local 6
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [69-111] (Public):
                    Parent: 0
                    Callable 14 [69-111] (Function):
                        name: Ident 15 [78-81] "Bar"
                        input: Pat 16 [81-83] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 17 [69-111] (Body): Impl:
                            Pat 18 [69-111] [Type Unit]: Elided
                            Block 19 [89-111] [Type Unit]:
                                Stmt 20 [91-109]: Local (Immutable):
                                    Pat 21 [95-96] [Type (Int -> Int)]: Bind: Ident 22 [95-96] "f"
                                    Expr 23 [99-108] [Type (Int -> Int)]: Expr Block: Block 45 [99-108] [Type (Int -> Int)]:
                                        Stmt 32 [106-107]: Local (Immutable):
                                            Pat 31 [106-107] [Type Int]: Bind: Ident 29 [106-107] "arg"
                                            Expr 28 [106-107] [Type Int]: Lit: Int(2)
                                        Stmt 46 [99-108]: Expr: Expr 47 [99-108] [Type (Int -> Int)]: Closure([29], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [99-108] (Internal):
                    Parent: 2
                    Callable 39 [99-108] (Function):
                        name: Ident 40 [99-108] "lambda"
                        input: Pat 37 [99-108] [Type (Int, Int)]: Tuple:
                            Pat 38 [99-108] [Type Int]: Bind: Ident 36 [99-108] "closed"
                            Pat 26 [103-104] [Type Int]: Bind: Ident 25 [103-104] "hole"
                        output: Int
                        functors: empty set
                        body: SpecDecl 41 [99-108] (Body): Impl:
                            Pat 42 [99-108] [Type (Int, Int)]: Elided
                            Block 43 [99-108] [Type Int]:
                                Stmt 44 [99-108]: Expr: Expr 35 [99-108] [Type Int]: Call:
                                    Expr 24 [99-102] [Type ((Int, Int) -> Int)]: Var: Item 1
                                    Expr 34 [102-108] [Type (Int, Int)]: Tuple:
                                        Expr 27 [103-104] [Type Int]: Var: Local 25
                                        Expr 30 [106-107] [Type Int]: Var: Local 36
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_two_holes() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(x : Int, y : Int) : Int { x + y }
                function Bar() : () { let f = Foo(_, _); }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-113] (Public):
                    Namespace (Ident 44 [10-11] "A"): Item 1, Item 2
                Item 1 [18-64] (Public):
                    Parent: 0
                    Callable 0 [18-64] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-48] [Type (Int, Int)]: Tuple:
                            Pat 3 [31-38] [Type Int]: Bind: Ident 4 [31-32] "x"
                            Pat 5 [40-47] [Type Int]: Bind: Ident 6 [40-41] "y"
                        output: Int
                        functors: empty set
                        body: SpecDecl 7 [18-64] (Body): Impl:
                            Pat 8 [18-64] [Type (Int, Int)]: Elided
                            Block 9 [55-64] [Type Int]:
                                Stmt 10 [57-62]: Expr: Expr 11 [57-62] [Type Int]: BinOp (Add):
                                    Expr 12 [57-58] [Type Int]: Var: Local 4
                                    Expr 13 [61-62] [Type Int]: Var: Local 6
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [69-111] (Public):
                    Parent: 0
                    Callable 14 [69-111] (Function):
                        name: Ident 15 [78-81] "Bar"
                        input: Pat 16 [81-83] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 17 [69-111] (Body): Impl:
                            Pat 18 [69-111] [Type Unit]: Elided
                            Block 19 [89-111] [Type Unit]:
                                Stmt 20 [91-109]: Local (Immutable):
                                    Pat 21 [95-96] [Type ((Int, Int) -> Int)]: Bind: Ident 22 [95-96] "f"
                                    Expr 23 [99-108] [Type ((Int, Int) -> Int)]: Expr Block: Block 41 [99-108] [Type ((Int, Int) -> Int)]:
                                        Stmt 42 [99-108]: Expr: Expr 43 [99-108] [Type ((Int, Int) -> Int)]: Closure([], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [99-108] (Internal):
                    Parent: 2
                    Callable 35 [99-108] (Function):
                        name: Ident 36 [99-108] "lambda"
                        input: Pat 34 [99-108] [Type ((Int, Int),)]: Tuple:
                            Pat 32 [102-108] [Type (Int, Int)]: Tuple:
                                Pat 26 [103-104] [Type Int]: Bind: Ident 25 [103-104] "hole"
                                Pat 29 [106-107] [Type Int]: Bind: Ident 28 [106-107] "hole"
                        output: Int
                        functors: empty set
                        body: SpecDecl 37 [99-108] (Body): Impl:
                            Pat 38 [99-108] [Type ((Int, Int),)]: Elided
                            Block 39 [99-108] [Type Int]:
                                Stmt 40 [99-108]: Expr: Expr 33 [99-108] [Type Int]: Call:
                                    Expr 24 [99-102] [Type ((Int, Int) -> Int)]: Var: Item 1
                                    Expr 31 [102-108] [Type (Int, Int)]: Tuple:
                                        Expr 27 [103-104] [Type Int]: Var: Local 25
                                        Expr 30 [106-107] [Type Int]: Var: Local 28
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_nested_tuple() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}
                function Bar() : () { let f = Foo(_, (_, 1.0, _), _); }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-157] (Public):
                    Namespace (Ident 63 [10-11] "A"): Item 1, Item 2
                Item 1 [18-95] (Public):
                    Parent: 0
                    Callable 0 [18-95] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-87] [Type (Int, (Bool, Double, String), Result)]: Tuple:
                            Pat 3 [31-38] [Type Int]: Bind: Ident 4 [31-32] "a"
                            Pat 5 [40-74] [Type (Bool, Double, String)]: Tuple:
                                Pat 6 [41-49] [Type Bool]: Bind: Ident 7 [41-42] "b"
                                Pat 8 [51-61] [Type Double]: Bind: Ident 9 [51-52] "c"
                                Pat 10 [63-73] [Type String]: Bind: Ident 11 [63-64] "d"
                            Pat 12 [76-86] [Type Result]: Bind: Ident 13 [76-77] "e"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 14 [18-95] (Body): Impl:
                            Pat 15 [18-95] [Type (Int, (Bool, Double, String), Result)]: Elided
                            Block 16 [93-95]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [100-155] (Public):
                    Parent: 0
                    Callable 17 [100-155] (Function):
                        name: Ident 18 [109-112] "Bar"
                        input: Pat 19 [112-114] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 20 [100-155] (Body): Impl:
                            Pat 21 [100-155] [Type Unit]: Elided
                            Block 22 [120-155] [Type Unit]:
                                Stmt 23 [122-153]: Local (Immutable):
                                    Pat 24 [126-127] [Type ((Int, (Bool, String), Result) -> Unit)]: Bind: Ident 25 [126-127] "f"
                                    Expr 26 [130-152] [Type ((Int, (Bool, String), Result) -> Unit)]: Expr Block: Block 60 [130-152] [Type ((Int, (Bool, String), Result) -> Unit)]:
                                        Stmt 38 [141-144]: Local (Immutable):
                                            Pat 37 [141-144] [Type Double]: Bind: Ident 35 [141-144] "arg"
                                            Expr 34 [141-144] [Type Double]: Lit: Double(1)
                                        Stmt 61 [130-152]: Expr: Expr 62 [130-152] [Type ((Int, (Bool, String), Result) -> Unit)]: Closure([35], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [130-152] (Internal):
                    Parent: 2
                    Callable 54 [130-152] (Function):
                        name: Ident 55 [130-152] "lambda"
                        input: Pat 52 [130-152] [Type (Double, (Int, (Bool, String), Result))]: Tuple:
                            Pat 53 [130-152] [Type Double]: Bind: Ident 51 [130-152] "closed"
                            Pat 49 [133-152] [Type (Int, (Bool, String), Result)]: Tuple:
                                Pat 29 [134-135] [Type Int]: Bind: Ident 28 [134-135] "hole"
                                Pat 44 [137-148] [Type (Bool, String)]: Tuple:
                                    Pat 32 [138-139] [Type Bool]: Bind: Ident 31 [138-139] "hole"
                                    Pat 41 [146-147] [Type String]: Bind: Ident 40 [146-147] "hole"
                                Pat 46 [150-151] [Type Result]: Bind: Ident 45 [150-151] "hole"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 56 [130-152] (Body): Impl:
                            Pat 57 [130-152] [Type (Double, (Int, (Bool, String), Result))]: Elided
                            Block 58 [130-152] [Type Unit]:
                                Stmt 59 [130-152]: Expr: Expr 50 [130-152] [Type Unit]: Call:
                                    Expr 27 [130-133] [Type ((Int, (Bool, Double, String), Result) -> Unit)]: Var: Item 1
                                    Expr 48 [133-152] [Type (Int, (Bool, Double, String), Result)]: Tuple:
                                        Expr 30 [134-135] [Type Int]: Var: Local 28
                                        Expr 43 [137-148] [Type (Bool, Double, String)]: Tuple:
                                            Expr 33 [138-139] [Type Bool]: Var: Local 31
                                            Expr 36 [141-144] [Type Double]: Var: Local 51
                                            Expr 42 [146-147] [Type String]: Var: Local 40
                                        Expr 47 [150-151] [Type Result]: Var: Local 45
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_nested_tuple_singleton_unwrap() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}
                function Bar() : () { let f = Foo(_, (true, 1.0, _), _); }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-160] (Public):
                    Namespace (Ident 67 [10-11] "A"): Item 1, Item 2
                Item 1 [18-95] (Public):
                    Parent: 0
                    Callable 0 [18-95] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-87] [Type (Int, (Bool, Double, String), Result)]: Tuple:
                            Pat 3 [31-38] [Type Int]: Bind: Ident 4 [31-32] "a"
                            Pat 5 [40-74] [Type (Bool, Double, String)]: Tuple:
                                Pat 6 [41-49] [Type Bool]: Bind: Ident 7 [41-42] "b"
                                Pat 8 [51-61] [Type Double]: Bind: Ident 9 [51-52] "c"
                                Pat 10 [63-73] [Type String]: Bind: Ident 11 [63-64] "d"
                            Pat 12 [76-86] [Type Result]: Bind: Ident 13 [76-77] "e"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 14 [18-95] (Body): Impl:
                            Pat 15 [18-95] [Type (Int, (Bool, Double, String), Result)]: Elided
                            Block 16 [93-95]: <empty>
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [100-158] (Public):
                    Parent: 0
                    Callable 17 [100-158] (Function):
                        name: Ident 18 [109-112] "Bar"
                        input: Pat 19 [112-114] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 20 [100-158] (Body): Impl:
                            Pat 21 [100-158] [Type Unit]: Elided
                            Block 22 [120-158] [Type Unit]:
                                Stmt 23 [122-156]: Local (Immutable):
                                    Pat 24 [126-127] [Type ((Int, String, Result) -> Unit)]: Bind: Ident 25 [126-127] "f"
                                    Expr 26 [130-155] [Type ((Int, String, Result) -> Unit)]: Expr Block: Block 64 [130-155] [Type ((Int, String, Result) -> Unit)]:
                                        Stmt 35 [138-142]: Local (Immutable):
                                            Pat 34 [138-142] [Type Bool]: Bind: Ident 32 [138-142] "arg"
                                            Expr 31 [138-142] [Type Bool]: Lit: Bool(true)
                                        Stmt 41 [144-147]: Local (Immutable):
                                            Pat 40 [144-147] [Type Double]: Bind: Ident 38 [144-147] "arg"
                                            Expr 37 [144-147] [Type Double]: Lit: Double(1)
                                        Stmt 65 [130-155]: Expr: Expr 66 [130-155] [Type ((Int, String, Result) -> Unit)]: Closure([32, 38], 3)
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 3 [130-155] (Internal):
                    Parent: 2
                    Callable 58 [130-155] (Function):
                        name: Ident 59 [130-155] "lambda"
                        input: Pat 55 [130-155] [Type (Bool, Double, (Int, String, Result))]: Tuple:
                            Pat 56 [130-155] [Type Bool]: Bind: Ident 53 [130-155] "closed"
                            Pat 57 [130-155] [Type Double]: Bind: Ident 54 [130-155] "closed"
                            Pat 51 [133-155] [Type (Int, String, Result)]: Tuple:
                                Pat 29 [134-135] [Type Int]: Bind: Ident 28 [134-135] "hole"
                                Pat 44 [149-150] [Type String]: Bind: Ident 43 [149-150] "hole"
                                Pat 48 [153-154] [Type Result]: Bind: Ident 47 [153-154] "hole"
                        output: Unit
                        functors: empty set
                        body: SpecDecl 60 [130-155] (Body): Impl:
                            Pat 61 [130-155] [Type (Bool, Double, (Int, String, Result))]: Elided
                            Block 62 [130-155] [Type Unit]:
                                Stmt 63 [130-155]: Expr: Expr 52 [130-155] [Type Unit]: Call:
                                    Expr 27 [130-133] [Type ((Int, (Bool, Double, String), Result) -> Unit)]: Var: Item 1
                                    Expr 50 [133-155] [Type (Int, (Bool, Double, String), Result)]: Tuple:
                                        Expr 30 [134-135] [Type Int]: Var: Local 28
                                        Expr 46 [137-151] [Type (Bool, Double, String)]: Tuple:
                                            Expr 33 [138-142] [Type Bool]: Var: Local 53
                                            Expr 39 [144-147] [Type Double]: Var: Local 54
                                            Expr 45 [149-150] [Type String]: Var: Local 43
                                        Expr 49 [153-154] [Type Result]: Var: Local 47
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn body_missing_should_fail() {
    check_errors(
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
fn duplicate_specialization() {
    check_errors(
        indoc! {"
        namespace test {
            operation Foo() : Unit {
                body ... {}
                body ... {}
            }
        }
        "},
        &expect![[r#"
            [
                DuplicateSpec(
                    Span {
                        lo: 54,
                        hi: 65,
                    },
                ),
                DuplicateSpec(
                    Span {
                        lo: 74,
                        hi: 85,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn duplicate_specialization_with_gen() {
    check_errors(
        indoc! {"
        namespace test {
            operation Foo() : Unit {
                body ... {}
                body auto;
                body intrinsic;
            }
        }
        "},
        &expect![[r#"
            [
                DuplicateSpec(
                    Span {
                        lo: 54,
                        hi: 65,
                    },
                ),
                DuplicateSpec(
                    Span {
                        lo: 74,
                        hi: 84,
                    },
                ),
                DuplicateSpec(
                    Span {
                        lo: 93,
                        hi: 108,
                    },
                ),
            ]
        "#]],
    );
}

#[test]
fn partial_app_unknown_callable() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo() : () { let f = Unknown(true, _, _); }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-72] (Public):
                    Namespace (Ident 15 [10-11] "A"): Item 1
                Item 1 [18-70] (Public):
                    Parent: 0
                    Callable 0 [18-70] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 3 [18-70] (Body): Impl:
                            Pat 4 [18-70] [Type Unit]: Elided
                            Block 5 [38-70] [Type Unit]:
                                Stmt 6 [40-68]: Local (Immutable):
                                    Pat 7 [44-45] [Type ?0]: Bind: Ident 8 [44-45] "f"
                                    Expr 9 [48-67] [Type ?0]: Call:
                                        Expr 10 [48-55] [Type ?]: Var: Err
                                        Expr 11 [55-67] [Type (Bool, ?1, ?2)]: Tuple:
                                            Expr 12 [56-60] [Type Bool]: Lit: Bool(true)
                                            Expr 13 [62-63] [Type ?1]: Hole
                                            Expr 14 [65-66] [Type ?2]: Hole
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_too_many_args() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(x : Int) : Int { x }
                function Bar() : () { let f = Foo(1, _, _); }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-103] (Public):
                    Namespace (Ident 24 [10-11] "A"): Item 1, Item 2
                Item 1 [18-51] (Public):
                    Parent: 0
                    Callable 0 [18-51] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-38] [Type Int]: Bind: Ident 3 [31-32] "x"
                        output: Int
                        functors: empty set
                        body: SpecDecl 4 [18-51] (Body): Impl:
                            Pat 5 [18-51] [Type Int]: Elided
                            Block 6 [46-51] [Type Int]:
                                Stmt 7 [48-49]: Expr: Expr 8 [48-49] [Type Int]: Var: Local 3
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [56-101] (Public):
                    Parent: 0
                    Callable 9 [56-101] (Function):
                        name: Ident 10 [65-68] "Bar"
                        input: Pat 11 [68-70] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 12 [56-101] (Body): Impl:
                            Pat 13 [56-101] [Type Unit]: Elided
                            Block 14 [76-101] [Type Unit]:
                                Stmt 15 [78-99]: Local (Immutable):
                                    Pat 16 [82-83] [Type Int]: Bind: Ident 17 [82-83] "f"
                                    Expr 18 [86-98] [Type Int]: Call:
                                        Expr 19 [86-89] [Type (Int -> Int)]: Var: Item 1
                                        Expr 20 [89-98] [Type (Int, ?1, ?2)]: Tuple:
                                            Expr 21 [90-91] [Type Int]: Lit: Int(1)
                                            Expr 22 [93-94] [Type ?1]: Hole
                                            Expr 23 [96-97] [Type ?2]: Hole
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_bound_to_non_arrow_ty() {
    check_hir(
        indoc! {"
            namespace A {
                function Foo(x : Int, y : Int) : Int { x + y }
                function Bar() : () {
                    let f : Int = Foo(1, _);
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-131] (Public):
                    Namespace (Ident 28 [10-11] "A"): Item 1, Item 2
                Item 1 [18-64] (Public):
                    Parent: 0
                    Callable 0 [18-64] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-48] [Type (Int, Int)]: Tuple:
                            Pat 3 [31-38] [Type Int]: Bind: Ident 4 [31-32] "x"
                            Pat 5 [40-47] [Type Int]: Bind: Ident 6 [40-41] "y"
                        output: Int
                        functors: empty set
                        body: SpecDecl 7 [18-64] (Body): Impl:
                            Pat 8 [18-64] [Type (Int, Int)]: Elided
                            Block 9 [55-64] [Type Int]:
                                Stmt 10 [57-62]: Expr: Expr 11 [57-62] [Type Int]: BinOp (Add):
                                    Expr 12 [57-58] [Type Int]: Var: Local 4
                                    Expr 13 [61-62] [Type Int]: Var: Local 6
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>
                Item 2 [69-129] (Public):
                    Parent: 0
                    Callable 14 [69-129] (Function):
                        name: Ident 15 [78-81] "Bar"
                        input: Pat 16 [81-83] [Type Unit]: Unit
                        output: Unit
                        functors: empty set
                        body: SpecDecl 17 [69-129] (Body): Impl:
                            Pat 18 [69-129] [Type Unit]: Elided
                            Block 19 [89-129] [Type Unit]:
                                Stmt 20 [99-123]: Local (Immutable):
                                    Pat 21 [103-110] [Type Int]: Bind: Ident 22 [103-104] "f"
                                    Expr 23 [113-122] [Type Int]: Call:
                                        Expr 24 [113-116] [Type ((Int, Int) -> Int)]: Var: Item 1
                                        Expr 25 [116-122] [Type (Int, Int)]: Tuple:
                                            Expr 26 [117-118] [Type Int]: Lit: Int(1)
                                            Expr 27 [120-121] [Type Int]: Hole
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}

#[test]
fn partial_app_hole_as_callee() {
    check_hir(
        indoc! {"
            namespace A {
                @EntryPoint()
                operation Main() : Result[] {
                    let f = _(_);
                    let res = f(4);
                    return [res];
                }
            }
        "},
        &expect![[r#"
            Package:
                Item 0 [0-141] (Public):
                    Namespace (Ident 22 [10-11] "A"): Item 1
                Item 1 [18-139] (Public):
                    Parent: 0
                    EntryPoint
                    Callable 0 [36-139] (Operation):
                        name: Ident 1 [46-50] "Main"
                        input: Pat 2 [50-52] [Type Unit]: Unit
                        output: (Result)[]
                        functors: empty set
                        body: SpecDecl 3 [36-139] (Body): Impl:
                            Pat 4 [36-139] [Type Unit]: Elided
                            Block 5 [64-139] [Type (Result)[]]:
                                Stmt 6 [74-87]: Local (Immutable):
                                    Pat 7 [78-79] [Type ?0]: Bind: Ident 8 [78-79] "f"
                                    Expr 9 [82-86] [Type ?0]: Call:
                                        Expr 10 [82-83] [Type ?1]: Hole
                                        Expr 11 [84-85] [Type ?2]: Hole
                                Stmt 12 [96-111]: Local (Immutable):
                                    Pat 13 [100-103] [Type Result]: Bind: Ident 14 [100-103] "res"
                                    Expr 15 [106-110] [Type Result]: Call:
                                        Expr 16 [106-107] [Type ?0]: Var: Local 8
                                        Expr 17 [108-109] [Type Int]: Lit: Int(4)
                                Stmt 18 [120-133]: Semi: Expr 19 [120-132] [Type ?6]: Return: Expr 20 [127-132] [Type (Result)[]]: Array:
                                    Expr 21 [128-131] [Type Result]: Var: Local 14
                        adj: <none>
                        ctl: <none>
                        ctl-adj: <none>"#]],
    );
}
