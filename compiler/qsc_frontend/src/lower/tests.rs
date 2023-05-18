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
                    Namespace (Ident 22 [10-11] "A"): Item 1
                Item 1 [18-87] (Public):
                    Parent: 0
                    Callable 0 [18-87] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 3 [39-87] [Type Int]:
                            Stmt 4 [49-68]: Local (Immutable):
                                Pat 5 [53-54] [Type (Int -> Int)]: Bind: Ident 6 [53-54] "f"
                                Expr 7 [57-67] [Type (Int -> Int)]: Closure([], 2)
                            Stmt 18 [77-81]: Expr: Expr 19 [77-81] [Type Int]: Call:
                                Expr 20 [77-78] [Type (Int -> Int)]: Var: Local 6
                                Expr 21 [79-80] [Type Int]: Lit: Int(1)
                Item 2 [57-67] (Internal):
                    Parent: 0
                    Callable 14 [57-67] (Function):
                        name: Ident 15 [57-67] "lambda"
                        input: Pat 13 [57-67] [Type (Int,)]: Tuple:
                            Pat 8 [57-58] [Type Int]: Bind: Ident 9 [57-58] "x"
                        output: Int
                        functors: 
                        body: Block: Block 16 [62-67] [Type Int]:
                            Stmt 17 [62-67]: Expr: Expr 10 [62-67] [Type Int]: BinOp (Add):
                                Expr 11 [62-63] [Type Int]: Var: Local 9
                                Expr 12 [66-67] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 27 [10-11] "A"): Item 1, Item 2
                Item 1 [18-61] (Public):
                    Parent: 0
                    Callable 0 [18-61] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-45] [Type (Int -> Int)]: Bind: Ident 3 [31-32] "f"
                        output: Int
                        functors: 
                        body: Block: Block 4 [53-61] [Type Int]:
                            Stmt 5 [55-59]: Expr: Expr 6 [55-59] [Type Int]: Call:
                                Expr 7 [55-56] [Type (Int -> Int)]: Var: Local 3
                                Expr 8 [57-58] [Type Int]: Lit: Int(2)
                Item 2 [66-106] (Public):
                    Parent: 0
                    Callable 9 [66-106] (Function):
                        name: Ident 10 [75-78] "Bar"
                        input: Pat 11 [78-80] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 12 [87-106] [Type Int]:
                            Stmt 13 [89-104]: Expr: Expr 14 [89-104] [Type Int]: Call:
                                Expr 15 [89-92] [Type ((Int -> Int) -> Int)]: Var: Item 1
                                Expr 16 [93-103] [Type (Int -> Int)]: Closure([], 3)
                Item 3 [93-103] (Internal):
                    Parent: 0
                    Callable 23 [93-103] (Function):
                        name: Ident 24 [93-103] "lambda"
                        input: Pat 22 [93-103] [Type (Int,)]: Tuple:
                            Pat 17 [93-94] [Type Int]: Bind: Ident 18 [93-94] "x"
                        output: Int
                        functors: 
                        body: Block: Block 25 [98-103] [Type Int]:
                            Stmt 26 [98-103]: Expr: Expr 19 [98-103] [Type Int]: BinOp (Add):
                                Expr 20 [98-99] [Type Int]: Var: Local 18
                                Expr 21 [102-103] [Type Int]: Lit: Int(1)"#]],
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
                    Namespace (Ident 28 [10-11] "A"): Item 1
                Item 1 [18-106] (Public):
                    Parent: 0
                    Callable 0 [18-106] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 3 [39-106] [Type Int]:
                            Stmt 4 [49-59]: Local (Immutable):
                                Pat 5 [53-54] [Type Int]: Bind: Ident 6 [53-54] "x"
                                Expr 7 [57-58] [Type Int]: Lit: Int(5)
                            Stmt 8 [68-87]: Local (Immutable):
                                Pat 9 [72-73] [Type (Int -> Int)]: Bind: Ident 10 [72-73] "f"
                                Expr 11 [76-86] [Type (Int -> Int)]: Closure([6], 2)
                            Stmt 24 [96-100]: Expr: Expr 25 [96-100] [Type Int]: Call:
                                Expr 26 [96-97] [Type (Int -> Int)]: Var: Local 10
                                Expr 27 [98-99] [Type Int]: Lit: Int(2)
                Item 2 [76-86] (Internal):
                    Parent: 0
                    Callable 20 [76-86] (Function):
                        name: Ident 21 [76-86] "lambda"
                        input: Pat 18 [76-86] [Type (Int, Int)]: Tuple:
                            Pat 19 [76-86] [Type Int]: Bind: Ident 17 [76-86] "closed"
                            Pat 12 [76-77] [Type Int]: Bind: Ident 13 [76-77] "y"
                        output: Int
                        functors: 
                        body: Block: Block 22 [81-86] [Type Int]:
                            Stmt 23 [81-86]: Expr: Expr 14 [81-86] [Type Int]: BinOp (Add):
                                Expr 15 [81-82] [Type Int]: Var: Local 17
                                Expr 16 [85-86] [Type Int]: Var: Local 13"#]],
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
                    Namespace (Ident 30 [10-11] "A"): Item 1
                Item 1 [18-110] (Public):
                    Parent: 0
                    Callable 0 [18-110] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [30-32] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 3 [39-110] [Type Int]:
                            Stmt 4 [49-59]: Local (Immutable):
                                Pat 5 [53-54] [Type Int]: Bind: Ident 6 [53-54] "x"
                                Expr 7 [57-58] [Type Int]: Lit: Int(5)
                            Stmt 8 [68-91]: Local (Immutable):
                                Pat 9 [72-73] [Type (Int -> Int)]: Bind: Ident 10 [72-73] "f"
                                Expr 11 [76-90] [Type (Int -> Int)]: Closure([6], 2)
                            Stmt 26 [100-104]: Expr: Expr 27 [100-104] [Type Int]: Call:
                                Expr 28 [100-101] [Type (Int -> Int)]: Var: Local 10
                                Expr 29 [102-103] [Type Int]: Lit: Int(2)
                Item 2 [76-90] (Internal):
                    Parent: 0
                    Callable 22 [76-90] (Function):
                        name: Ident 23 [76-90] "lambda"
                        input: Pat 20 [76-90] [Type (Int, Int)]: Tuple:
                            Pat 21 [76-90] [Type Int]: Bind: Ident 19 [76-90] "closed"
                            Pat 12 [76-77] [Type Int]: Bind: Ident 13 [76-77] "y"
                        output: Int
                        functors: 
                        body: Block: Block 24 [81-90] [Type Int]:
                            Stmt 25 [81-90]: Expr: Expr 14 [81-90] [Type Int]: BinOp (Add):
                                Expr 15 [81-86] [Type Int]: BinOp (Add):
                                    Expr 16 [81-82] [Type Int]: Var: Local 19
                                    Expr 17 [85-86] [Type Int]: Var: Local 19
                                Expr 18 [89-90] [Type Int]: Var: Local 13"#]],
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
                    Namespace (Ident 33 [10-11] "A"): Item 1, Item 2
                Item 1 [18-61] (Public):
                    Parent: 0
                    Callable 0 [18-61] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-45] [Type (Int -> Int)]: Bind: Ident 3 [31-32] "f"
                        output: Int
                        functors: 
                        body: Block: Block 4 [53-61] [Type Int]:
                            Stmt 5 [55-59]: Expr: Expr 6 [55-59] [Type Int]: Call:
                                Expr 7 [55-56] [Type (Int -> Int)]: Var: Local 3
                                Expr 8 [57-58] [Type Int]: Lit: Int(2)
                Item 2 [66-137] (Public):
                    Parent: 0
                    Callable 9 [66-137] (Function):
                        name: Ident 10 [75-78] "Bar"
                        input: Pat 11 [78-80] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 12 [87-137] [Type Int]:
                            Stmt 13 [97-107]: Local (Immutable):
                                Pat 14 [101-102] [Type Int]: Bind: Ident 15 [101-102] "x"
                                Expr 16 [105-106] [Type Int]: Lit: Int(5)
                            Stmt 17 [116-131]: Expr: Expr 18 [116-131] [Type Int]: Call:
                                Expr 19 [116-119] [Type ((Int -> Int) -> Int)]: Var: Item 1
                                Expr 20 [120-130] [Type (Int -> Int)]: Closure([15], 3)
                Item 3 [120-130] (Internal):
                    Parent: 0
                    Callable 29 [120-130] (Function):
                        name: Ident 30 [120-130] "lambda"
                        input: Pat 27 [120-130] [Type (Int, Int)]: Tuple:
                            Pat 28 [120-130] [Type Int]: Bind: Ident 26 [120-130] "closed"
                            Pat 21 [120-121] [Type Int]: Bind: Ident 22 [120-121] "y"
                        output: Int
                        functors: 
                        body: Block: Block 31 [125-130] [Type Int]:
                            Stmt 32 [125-130]: Expr: Expr 23 [125-130] [Type Int]: BinOp (Add):
                                Expr 24 [125-126] [Type Int]: Var: Local 26
                                Expr 25 [129-130] [Type Int]: Var: Local 22"#]],
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
                    Namespace (Ident 60 [10-11] "A"): Item 1, Item 2
                Item 1 [18-71] (Public):
                    Parent: 0
                    Callable 0 [18-71] (Function):
                        name: Ident 1 [27-30] "Foo"
                        input: Pat 2 [31-52] [Type (Int -> (Int -> Int))]: Bind: Ident 3 [31-32] "f"
                        output: Int
                        functors: 
                        body: Block: Block 4 [60-71] [Type Int]:
                            Stmt 5 [62-69]: Expr: Expr 6 [62-69] [Type Int]: Call:
                                Expr 7 [62-66] [Type (Int -> Int)]: Call:
                                    Expr 8 [62-63] [Type (Int -> (Int -> Int))]: Var: Local 3
                                    Expr 9 [64-65] [Type Int]: Lit: Int(2)
                                Expr 10 [67-68] [Type Int]: Lit: Int(3)
                Item 2 [76-207] (Public):
                    Parent: 0
                    Callable 11 [76-207] (Function):
                        name: Ident 12 [85-88] "Bar"
                        input: Pat 13 [88-90] [Type Unit]: Unit
                        output: Int
                        functors: 
                        body: Block: Block 14 [97-207] [Type Int]:
                            Stmt 15 [107-117]: Local (Immutable):
                                Pat 16 [111-112] [Type Int]: Bind: Ident 17 [111-112] "a"
                                Expr 18 [115-116] [Type Int]: Lit: Int(5)
                            Stmt 19 [126-201]: Expr: Expr 20 [126-201] [Type Int]: Call:
                                Expr 21 [126-129] [Type ((Int -> (Int -> Int)) -> Int)]: Var: Item 1
                                Expr 22 [130-200] [Type (Int -> (Int -> Int))]: Closure([17], 4)
                Item 3 [172-190] (Internal):
                    Parent: 0
                    Callable 49 [172-190] (Function):
                        name: Ident 50 [172-190] "lambda"
                        input: Pat 45 [172-190] [Type (Int, Int, Int, Int)]: Tuple:
                            Pat 46 [172-190] [Type Int]: Bind: Ident 42 [172-190] "closed"
                            Pat 47 [172-190] [Type Int]: Bind: Ident 43 [172-190] "closed"
                            Pat 48 [172-190] [Type Int]: Bind: Ident 44 [172-190] "closed"
                            Pat 33 [172-173] [Type Int]: Bind: Ident 34 [172-173] "d"
                        output: Int
                        functors: 
                        body: Block: Block 51 [177-190] [Type Int]:
                            Stmt 52 [177-190]: Expr: Expr 35 [177-190] [Type Int]: BinOp (Add):
                                Expr 36 [177-186] [Type Int]: BinOp (Add):
                                    Expr 37 [177-182] [Type Int]: BinOp (Add):
                                        Expr 38 [177-178] [Type Int]: Var: Local 42
                                        Expr 39 [181-182] [Type Int]: Var: Local 43
                                    Expr 40 [185-186] [Type Int]: Var: Local 44
                                Expr 41 [189-190] [Type Int]: Var: Local 34
                Item 4 [130-200] (Internal):
                    Parent: 0
                    Callable 56 [130-200] (Function):
                        name: Ident 57 [130-200] "lambda"
                        input: Pat 54 [130-200] [Type (Int, Int)]: Tuple:
                            Pat 55 [130-200] [Type Int]: Bind: Ident 53 [130-200] "closed"
                            Pat 23 [130-131] [Type Int]: Bind: Ident 24 [130-131] "b"
                        output: (Int -> Int)
                        functors: 
                        body: Block: Block 58 [135-200] [Type (Int -> Int)]:
                            Stmt 59 [135-200]: Expr: Expr 25 [135-200] [Type (Int -> Int)]: Expr Block: Block 26 [135-200] [Type (Int -> Int)]:
                                Stmt 27 [149-159]: Local (Immutable):
                                    Pat 28 [153-154] [Type Int]: Bind: Ident 29 [153-154] "c"
                                    Expr 30 [157-158] [Type Int]: Lit: Int(1)
                                Stmt 31 [172-190]: Expr: Expr 32 [172-190] [Type (Int -> Int)]: Closure([53, 24, 29], 3)"#]],
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
                    Namespace (Ident 29 [10-11] "A"): Item 1, Item 2
                Item 1 [18-101] (Public):
                    Parent: 0
                    Callable 0 [18-101] (Operation):
                        name: Ident 1 [28-31] "Foo"
                        input: Pat 2 [32-48] [Type (Qubit => Unit)]: Bind: Ident 3 [32-34] "op"
                        output: Unit
                        functors: 
                        body: Block: Block 4 [55-101] [Type Unit]:
                            Stmt 5 [65-81]: Qubit (Fresh)
                                Pat 6 [69-70] [Type Qubit]: Bind: Ident 7 [69-70] "q"
                                QubitInit 8 [73-80] [Type Qubit]: Single
                            Stmt 9 [90-95]: Expr: Expr 10 [90-95] [Type Unit]: Call:
                                Expr 11 [90-92] [Type (Qubit => Unit)]: Var: Local 3
                                Expr 12 [93-94] [Type Qubit]: Var: Local 7
                Item 2 [106-147] (Public):
                    Parent: 0
                    Callable 13 [106-147] (Operation):
                        name: Ident 14 [116-119] "Bar"
                        input: Pat 15 [119-121] [Type Unit]: Unit
                        output: Result
                        functors: 
                        body: Block: Block 16 [131-147] [Type Result]:
                            Stmt 17 [133-145]: Expr: Expr 18 [133-145] [Type Result]: Call:
                                Expr 19 [133-136] [Type ((Qubit => Unit) => Unit)]: Var: Item 1
                                Expr 20 [137-144] [Type (Qubit => Unit)]: Closure([], 3)
                Item 3 [137-144] (Internal):
                    Parent: 0
                    Callable 25 [137-144] (Operation):
                        name: Ident 26 [137-144] "lambda"
                        input: Pat 24 [137-144] [Type (Qubit,)]: Tuple:
                            Pat 21 [137-138] [Type Qubit]: Bind: Ident 22 [137-138] "q"
                        output: Unit
                        functors: 
                        body: Block: Block 27 [142-144] [Type Unit]:
                            Stmt 28 [142-144]: Expr: Expr 23 [142-144] [Type Unit]: Unit"#]],
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
                    Namespace (Ident 37 [10-11] "A"): Item 1, Item 2, Item 3
                Item 1 [18-75] (Public):
                    Parent: 0
                    Callable 0 [18-75] (Operation):
                        name: Ident 1 [28-35] "MResetZ"
                        input: Pat 2 [36-45] [Type Qubit]: Bind: Ident 3 [36-37] "q"
                        output: Result
                        functors: 
                        body: Specializations:
                            SpecDecl 4 [58-73] (Body): Gen: Intrinsic
                Item 2 [80-130] (Public):
                    Parent: 0
                    Callable 5 [80-130] (Operation):
                        name: Ident 6 [90-93] "Foo"
                        input: Pat 7 [94-111] [Type (Unit => Result)]: Bind: Ident 8 [94-96] "op"
                        output: Result
                        functors: 
                        body: Block: Block 9 [122-130] [Type Result]:
                            Stmt 10 [124-128]: Expr: Expr 11 [124-128] [Type Result]: Call:
                                Expr 12 [124-126] [Type (Unit => Result)]: Var: Local 8
                                Expr 13 [126-128] [Type Unit]: Unit
                Item 3 [135-222] (Public):
                    Parent: 0
                    Callable 14 [135-222] (Operation):
                        name: Ident 15 [145-148] "Bar"
                        input: Pat 16 [148-150] [Type Unit]: Unit
                        output: Result
                        functors: 
                        body: Block: Block 17 [160-222] [Type Result]:
                            Stmt 18 [170-186]: Qubit (Fresh)
                                Pat 19 [174-175] [Type Qubit]: Bind: Ident 20 [174-175] "q"
                                QubitInit 21 [178-185] [Type Qubit]: Single
                            Stmt 22 [195-216]: Expr: Expr 23 [195-216] [Type Result]: Call:
                                Expr 24 [195-198] [Type ((Unit => Result) => Result)]: Var: Item 2
                                Expr 25 [199-215] [Type (Unit => Result)]: Closure([20], 4)
                Item 4 [199-215] (Internal):
                    Parent: 0
                    Callable 33 [199-215] (Operation):
                        name: Ident 34 [199-215] "lambda"
                        input: Pat 31 [199-215] [Type (Qubit, Unit)]: Tuple:
                            Pat 32 [199-215] [Type Qubit]: Bind: Ident 30 [199-215] "closed"
                            Pat 26 [199-201] [Type Unit]: Unit
                        output: Result
                        functors: 
                        body: Block: Block 35 [205-215] [Type Result]:
                            Stmt 36 [205-215]: Expr: Expr 27 [205-215] [Type Result]: Call:
                                Expr 28 [205-212] [Type (Qubit => Result)]: Var: Item 1
                                Expr 29 [213-214] [Type Qubit]: Var: Local 30"#]],
    );
}

#[test]
fn lambda_mutable_closure() {
    check_errors(
        indoc! {"
            namespace A {
                function Foo() : () {
                    mutable x = 1;
                    let f = y -> x + y;
                }
            }
        "},
        &expect![[r#"
            [
                MutableClosure(
                    Span {
                        lo: 79,
                        hi: 89,
                    },
                ),
            ]
        "#]],
    );
}
