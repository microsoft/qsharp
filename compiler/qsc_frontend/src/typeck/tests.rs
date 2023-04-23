// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, PackageStore};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{Block, Expr, NodeId, Pat, QubitInit, Ty},
    visit::{self, Visitor},
};
use std::fmt::Write;

struct TyCollector<'a> {
    tys: Vec<(NodeId, Span, &'a Ty)>,
}

impl<'a> Visitor<'a> for TyCollector<'a> {
    fn visit_block(&mut self, block: &'a Block) {
        self.tys.push((block.id, block.span, &block.ty));
        visit::walk_block(self, block);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        self.tys.push((expr.id, expr.span, &expr.ty));
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        self.tys.push((pat.id, pat.span, &pat.ty));
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        self.tys.push((init.id, init.span, &init.ty));
        visit::walk_qubit_init(self, init);
    }
}

fn check(source: &str, entry_expr: &str, expect: &Expect) {
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], [source], entry_expr);
    let mut tys = TyCollector { tys: Vec::new() };
    tys.visit_package(&unit.package);

    let mut actual = String::new();
    for (id, span, ty) in tys.tys {
        let (index, offset) = unit.context.source(span.lo);
        let code = &[source, entry_expr][index.0][span.lo - offset..span.hi - offset];
        writeln!(actual, "#{id} {}-{} {code:?} : {ty}", span.lo, span.hi)
            .expect("writing type to string should succeed");
    }

    for error in unit.context.errors() {
        writeln!(actual, "{error:?}").expect("writing error to string should succeed");
    }

    expect.assert_eq(&actual);
}

#[test]
fn empty_callable() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Unit {}
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : ()
            #7 40-42 "{}" : ()
        "##]],
    );
}

#[test]
fn return_constant() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int { 4 }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : ()
            #7 39-44 "{ 4 }" : Int
            #9 41-42 "4" : Int
        "##]],
    );
}

#[test]
fn return_wrong_type() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int { true }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : ()
            #7 39-47 "{ true }" : Bool
            #9 41-45 "true" : Bool
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 39, hi: 47 }))))
        "##]],
    );
}

#[test]
fn return_semi() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int { 4; }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : ()
            #7 39-45 "{ 4; }" : ()
            #9 41-42 "4" : Int
            Error(Type(Error(TypeMismatch(Prim(Int), Tuple([]), Span { lo: 39, hi: 45 }))))
        "##]],
    );
}

#[test]
fn return_var() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    let x = 4;
                    x
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : ()
            #7 39-75 "{\n        let x = 4;\n        x\n    }" : Int
            #9 53-54 "x" : Int
            #11 57-58 "4" : Int
            #13 68-69 "x" : Int
        "##]],
    );
}

#[test]
fn call_function() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Int) : Int { x }
                function Bar() : Int { Foo(4) }
            }
        "},
        "",
        &expect![[r##"
            #6 30-39 "(x : Int)" : Int
            #7 31-38 "x : Int" : Int
            #9 46-51 "{ x }" : Int
            #11 48-49 "x" : Int
            #15 68-70 "()" : ()
            #16 77-87 "{ Foo(4) }" : Int
            #18 79-85 "Foo(4)" : Int
            #19 79-82 "Foo" : (Int -> Int)
            #20 82-85 "(4)" : Int
            #21 83-84 "4" : Int
        "##]],
    );
}

#[test]
fn call_generic_identity() {
    check(
        indoc! {"
            namespace A {
                function Identity<'T>(x : 'T) : 'T { x }
                function Foo() : Int { Identity(4) }
            }
        "},
        "",
        &expect![[r##"
            #7 39-47 "(x : 'T)" : 'T
            #8 40-46 "x : 'T" : 'T
            #10 53-58 "{ x }" : 'T
            #12 55-56 "x" : 'T
            #16 75-77 "()" : ()
            #17 84-99 "{ Identity(4) }" : Int
            #19 86-97 "Identity(4)" : Int
            #20 86-94 "Identity" : (Int -> Int)
            #21 94-97 "(4)" : Int
            #22 95-96 "4" : Int
        "##]],
    );
}

#[test]
fn call_generic_length() {
    check(
        "",
        "Length([true, false, true])",
        &expect![[r##"
            #1 0-27 "Length([true, false, true])" : Int
            #2 0-6 "Length" : ((Bool)[] -> Int)
            #3 6-27 "([true, false, true])" : (Bool)[]
            #4 7-26 "[true, false, true]" : (Bool)[]
            #5 8-12 "true" : Bool
            #6 14-19 "false" : Bool
            #7 21-25 "true" : Bool
        "##]],
    );
}

#[test]
fn add_wrong_types() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Unit { 1 + [2]; }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : ()
            #7 40-52 "{ 1 + [2]; }" : ()
            #9 42-49 "1 + [2]" : Int
            #10 42-43 "1" : Int
            #11 46-49 "[2]" : (Int)[]
            #12 47-48 "2" : Int
            Error(Type(Error(TypeMismatch(Prim(Int), Array(Prim(Int)), Span { lo: 42, hi: 49 }))))
        "##]],
    );
}

#[test]
fn int_as_double_error() {
    check(
        "",
        "Microsoft.Quantum.Convert.IntAsDouble(false)",
        &expect![[r##"
            #1 0-44 "Microsoft.Quantum.Convert.IntAsDouble(false)" : Double
            #2 0-37 "Microsoft.Quantum.Convert.IntAsDouble" : (Int -> Double)
            #3 37-44 "(false)" : Bool
            #4 38-43 "false" : Bool
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 0, hi: 44 }))))
        "##]],
    );
}

#[test]
fn length_type_error() {
    check(
        "",
        "Length((1, 2, 3))",
        &expect![[r##"
            #1 0-17 "Length((1, 2, 3))" : Int
            #2 0-6 "Length" : ((?0)[] -> Int)
            #3 6-17 "((1, 2, 3))" : (Int, Int, Int)
            #4 7-16 "(1, 2, 3)" : (Int, Int, Int)
            #5 8-9 "1" : Int
            #6 11-12 "2" : Int
            #7 14-15 "3" : Int
            Error(Type(Error(TypeMismatch(Array(Infer(InferTy(0))), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 17 }))))
        "##]],
    );
}

#[test]
fn single_arg_for_tuple() {
    check(
        "",
        indoc! {"
            {
                use q = Qubit();
                Ry(q);
            }
        "},
        &expect![[r##"
            #1 0-35 "{\n    use q = Qubit();\n    Ry(q);\n}" : ()
            #2 0-35 "{\n    use q = Qubit();\n    Ry(q);\n}" : ()
            #4 10-11 "q" : Qubit
            #6 14-21 "Qubit()" : Qubit
            #8 27-32 "Ry(q)" : ()
            #9 27-29 "Ry" : ((Double, Qubit) => () is Adj + Ctl)
            #10 29-32 "(q)" : Qubit
            #11 30-31 "q" : Qubit
            Error(Type(Error(TypeMismatch(Tuple([Prim(Double), Prim(Qubit)]), Prim(Qubit), Span { lo: 27, hi: 32 }))))
        "##]],
    );
}

#[test]
fn array_index_error() {
    check(
        "",
        "[1, 2, 3][false]",
        &expect![[r##"
            #1 0-16 "[1, 2, 3][false]" : ?0
            #2 0-9 "[1, 2, 3]" : (Int)[]
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 10-15 "false" : Bool
            Error(Type(Error(MissingClass(HasIndex { container: Array(Prim(Int)), index: Prim(Bool), item: Infer(InferTy(0)) }, Span { lo: 0, hi: 16 }))))
        "##]],
    );
}

#[test]
fn array_repeat_error() {
    check(
        "",
        "[4, size = true]",
        &expect![[r##"
            #1 0-16 "[4, size = true]" : (Int)[]
            #2 1-2 "4" : Int
            #3 11-15 "true" : Bool
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 11, hi: 15 }))))
        "##]],
    );
}

#[test]
fn assignop_error() {
    check(
        "",
        indoc! {"
            {
                mutable x = false;
                set x += 1;
                x
            }
        "},
        &expect![[r##"
            #1 0-48 "{\n    mutable x = false;\n    set x += 1;\n    x\n}" : Bool
            #2 0-48 "{\n    mutable x = false;\n    set x += 1;\n    x\n}" : Bool
            #4 14-15 "x" : Bool
            #6 18-23 "false" : Bool
            #8 29-39 "set x += 1" : ()
            #9 33-34 "x" : Bool
            #10 38-39 "1" : Int
            #12 45-46 "x" : Bool
            Error(Type(Error(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 29, hi: 39 }))))
            Error(Type(Error(MissingClass(Add(Prim(Bool)), Span { lo: 33, hi: 34 }))))
        "##]],
    );
}

#[test]
fn binop_add_invalid() {
    check(
        "",
        "(1, 3) + 5.4",
        &expect![[r##"
            #1 0-12 "(1, 3) + 5.4" : (Int, Int)
            #2 0-6 "(1, 3)" : (Int, Int)
            #3 1-2 "1" : Int
            #4 4-5 "3" : Int
            #5 9-12 "5.4" : Double
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Prim(Double), Span { lo: 0, hi: 12 }))))
            Error(Type(Error(MissingClass(Add(Tuple([Prim(Int), Prim(Int)])), Span { lo: 0, hi: 6 }))))
        "##]],
    );
}

#[test]
fn binop_add_mismatch() {
    check(
        "",
        "1 + 5.4",
        &expect![[r##"
            #1 0-7 "1 + 5.4" : Int
            #2 0-1 "1" : Int
            #3 4-7 "5.4" : Double
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Double), Span { lo: 0, hi: 7 }))))
        "##]],
    );
}

#[test]
fn binop_andb_mismatch() {
    check(
        "",
        "28 &&& 54L",
        &expect![[r##"
            #1 0-10 "28 &&& 54L" : Int
            #2 0-2 "28" : Int
            #3 7-10 "54L" : BigInt
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn binop_equal_callable() {
    check(
        indoc! {"
            namespace Test {
                function A() : Unit {}
                function B() : Unit {}
            }
        "},
        "Test.A == Test.B",
        &expect![[r##"
            #6 31-33 "()" : ()
            #7 41-43 "{}" : ()
            #11 58-60 "()" : ()
            #12 68-70 "{}" : ()
            #13 73-89 "Test.A == Test.B" : Bool
            #14 73-79 "Test.A" : (() -> ())
            #15 83-89 "Test.B" : (() -> ())
            Error(Type(Error(MissingClass(Eq(Arrow(Function, Tuple([]), Tuple([]), {})), Span { lo: 73, hi: 79 }))))
        "##]],
    );
}

#[test]
fn binop_equal_tuple_arity_mismatch() {
    check(
        "",
        "(1, 2, 3) == (1, 2, 3, 4)",
        &expect![[r##"
            #1 0-25 "(1, 2, 3) == (1, 2, 3, 4)" : Bool
            #2 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 13-25 "(1, 2, 3, 4)" : (Int, Int, Int, Int)
            #7 14-15 "1" : Int
            #8 17-18 "2" : Int
            #9 20-21 "3" : Int
            #10 23-24 "4" : Int
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 25 }))))
        "##]],
    );
}

#[test]
fn binop_equal_tuple_type_mismatch() {
    check(
        "",
        "(1, 2, 3) == (1, Zero, 3)",
        &expect![[r##"
            #1 0-25 "(1, 2, 3) == (1, Zero, 3)" : Bool
            #2 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 13-25 "(1, Zero, 3)" : (Int, Result, Int)
            #7 14-15 "1" : Int
            #8 17-21 "Zero" : Result
            #9 23-24 "3" : Int
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Result), Span { lo: 0, hi: 25 }))))
        "##]],
    );
}

#[test]
fn binop_eq_mismatch() {
    check(
        "",
        "18L == 18",
        &expect![[r##"
            #1 0-9 "18L == 18" : Bool
            #2 0-3 "18L" : BigInt
            #3 7-9 "18" : Int
            Error(Type(Error(TypeMismatch(Prim(BigInt), Prim(Int), Span { lo: 0, hi: 9 }))))
        "##]],
    );
}

#[test]
fn binop_neq_mismatch() {
    check(
        "",
        "18L != 18",
        &expect![[r##"
            #1 0-9 "18L != 18" : Bool
            #2 0-3 "18L" : BigInt
            #3 7-9 "18" : Int
            Error(Type(Error(TypeMismatch(Prim(BigInt), Prim(Int), Span { lo: 0, hi: 9 }))))
        "##]],
    );
}

#[test]
fn binop_neq_tuple_type_mismatch() {
    check(
        "",
        "(1, 2, 3) != (1, Zero, 3)",
        &expect![[r##"
            #1 0-25 "(1, 2, 3) != (1, Zero, 3)" : Bool
            #2 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 13-25 "(1, Zero, 3)" : (Int, Result, Int)
            #7 14-15 "1" : Int
            #8 17-21 "Zero" : Result
            #9 23-24 "3" : Int
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Result), Span { lo: 0, hi: 25 }))))
        "##]],
    );
}

#[test]
fn binop_neq_tuple_arity_mismatch() {
    check(
        "",
        "(1, 2, 3) != (1, 2, 3, 4)",
        &expect![[r##"
            #1 0-25 "(1, 2, 3) != (1, 2, 3, 4)" : Bool
            #2 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 13-25 "(1, 2, 3, 4)" : (Int, Int, Int, Int)
            #7 14-15 "1" : Int
            #8 17-18 "2" : Int
            #9 20-21 "3" : Int
            #10 23-24 "4" : Int
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 25 }))))
        "##]],
    );
}

#[test]
fn binop_orb_mismatch() {
    check(
        "",
        "28 ||| 54L",
        &expect![[r##"
            #1 0-10 "28 ||| 54L" : Int
            #2 0-2 "28" : Int
            #3 7-10 "54L" : BigInt
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn binop_xorb_mismatch() {
    check(
        "",
        "28 ^^^ 54L",
        &expect![[r##"
            #1 0-10 "28 ^^^ 54L" : Int
            #2 0-2 "28" : Int
            #3 7-10 "54L" : BigInt
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn let_tuple_arity_error() {
    check(
        "",
        "{ let (x, y, z) = (0, 1); }",
        &expect![[r##"
            #1 0-27 "{ let (x, y, z) = (0, 1); }" : ()
            #2 0-27 "{ let (x, y, z) = (0, 1); }" : ()
            #4 6-15 "(x, y, z)" : (?0, ?1, ?2)
            #5 7-8 "x" : ?0
            #7 10-11 "y" : ?1
            #9 13-14 "z" : ?2
            #11 18-24 "(0, 1)" : (Int, Int)
            #12 19-20 "0" : Int
            #13 22-23 "1" : Int
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Infer(InferTy(0)), Infer(InferTy(1)), Infer(InferTy(2))]), Span { lo: 6, hi: 15 }))))
        "##]],
    );
}

#[test]
fn set_tuple_arity_error() {
    check(
        "",
        indoc! {"
            {
                mutable (x, y) = (0, 1);
                set (x, y) = (1, 2, 3);
                x
            }
        "},
        &expect![[r##"
            #1 0-66 "{\n    mutable (x, y) = (0, 1);\n    set (x, y) = (1, 2, 3);\n    x\n}" : Int
            #2 0-66 "{\n    mutable (x, y) = (0, 1);\n    set (x, y) = (1, 2, 3);\n    x\n}" : Int
            #4 14-20 "(x, y)" : (Int, Int)
            #5 15-16 "x" : Int
            #7 18-19 "y" : Int
            #9 23-29 "(0, 1)" : (Int, Int)
            #10 24-25 "0" : Int
            #11 27-28 "1" : Int
            #13 35-57 "set (x, y) = (1, 2, 3)" : ()
            #14 39-45 "(x, y)" : (Int, Int)
            #15 40-41 "x" : Int
            #16 43-44 "y" : Int
            #17 48-57 "(1, 2, 3)" : (Int, Int, Int)
            #18 49-50 "1" : Int
            #19 52-53 "2" : Int
            #20 55-56 "3" : Int
            #22 63-64 "x" : Int
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 39, hi: 45 }))))
        "##]],
    );
}

#[test]
fn qubit_array_length_error() {
    check(
        "",
        "{ use q = Qubit[false]; }",
        &expect![[r##"
            #1 0-25 "{ use q = Qubit[false]; }" : ()
            #2 0-25 "{ use q = Qubit[false]; }" : ()
            #4 6-7 "q" : (Qubit)[]
            #6 10-22 "Qubit[false]" : (Qubit)[]
            #7 16-21 "false" : Bool
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 16, hi: 21 }))))
        "##]],
    );
}

#[test]
fn qubit_tuple_arity_error() {
    check(
        "",
        "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }",
        &expect![[r##"
            #1 0-47 "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }" : ()
            #2 0-47 "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }" : ()
            #4 6-13 "(q, q1)" : (?0, ?1)
            #5 7-8 "q" : ?0
            #7 10-12 "q1" : ?1
            #9 16-44 "(Qubit[3], Qubit(), Qubit())" : ((Qubit)[], Qubit, Qubit)
            #10 17-25 "Qubit[3]" : (Qubit)[]
            #11 23-24 "3" : Int
            #12 27-34 "Qubit()" : Qubit
            #13 36-43 "Qubit()" : Qubit
            Error(Type(Error(TypeMismatch(Tuple([Array(Prim(Qubit)), Prim(Qubit), Prim(Qubit)]), Tuple([Infer(InferTy(0)), Infer(InferTy(1))]), Span { lo: 6, hi: 13 }))))
        "##]],
    );
}

#[test]
fn for_loop_not_iterable() {
    check(
        "",
        "for i in (1, true, One) {}",
        &expect![[r##"
            #1 0-26 "for i in (1, true, One) {}" : ()
            #2 4-5 "i" : ?0
            #4 9-23 "(1, true, One)" : (Int, Bool, Result)
            #5 10-11 "1" : Int
            #6 13-17 "true" : Bool
            #7 19-22 "One" : Result
            #8 24-26 "{}" : ()
            Error(Type(Error(MissingClass(Iterable { container: Tuple([Prim(Int), Prim(Bool), Prim(Result)]), item: Infer(InferTy(0)) }, Span { lo: 9, hi: 23 }))))
        "##]],
    );
}

#[test]
fn if_cond_error() {
    check(
        "",
        "if 4 {}",
        &expect![[r##"
            #1 0-7 "if 4 {}" : ()
            #2 3-4 "4" : Int
            #3 5-7 "{}" : ()
            Error(Type(Error(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 3, hi: 4 }))))
        "##]],
    );
}

#[test]
fn if_no_else_must_be_unit() {
    check(
        "",
        "if true { 4 }",
        &expect![[r##"
            #1 0-13 "if true { 4 }" : Int
            #2 3-7 "true" : Bool
            #3 8-13 "{ 4 }" : Int
            #5 10-11 "4" : Int
            Error(Type(Error(TypeMismatch(Prim(Int), Tuple([]), Span { lo: 0, hi: 13 }))))
        "##]],
    );
}

#[test]
fn if_else_fail() {
    check(
        "",
        r#"if false {} else { fail "error"; }"#,
        &expect![[r##"
            #1 0-34 "if false {} else { fail \"error\"; }" : ()
            #2 3-8 "false" : Bool
            #3 9-11 "{}" : ()
            #4 12-34 "else { fail \"error\"; }" : ()
            #5 17-34 "{ fail \"error\"; }" : ()
            #7 19-31 "fail \"error\"" : ?0
            #8 24-31 "\"error\"" : String
        "##]],
    );
}

#[test]
fn if_cond_fail() {
    check(
        indoc! {r#"
            namespace A {
                function F() : Int {
                    if fail "error" {
                        "this type doesn't matter"
                    } else {
                        "foo"
                    }
                }
            }
        "#},
        "",
        &expect![[r##"
            #6 28-30 "()" : ()
            #7 37-154 "{\n        if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }\n    }" : Int
            #9 47-148 "if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }" : Int
            #10 50-62 "fail \"error\"" : Bool
            #11 55-62 "\"error\"" : String
            #12 63-113 "{\n            \"this type doesn't matter\"\n        }" : String
            #14 77-103 "\"this type doesn't matter\"" : String
            #15 114-148 "else {\n            \"foo\"\n        }" : String
            #16 119-148 "{\n            \"foo\"\n        }" : String
            #18 133-138 "\"foo\"" : String
        "##]],
    );
}

#[test]
fn if_all_diverge() {
    check(
        indoc! {r#"
            namespace A {
                function F() : Int {
                    if fail "cond" {
                        fail "true"
                    } else {
                        fail "false"
                    }
                }
            }
        "#},
        "",
        &expect![[r##"
            #6 28-30 "()" : ()
            #7 37-145 "{\n        if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }\n    }" : Int
            #9 47-139 "if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }" : Int
            #10 50-61 "fail \"cond\"" : Bool
            #11 55-61 "\"cond\"" : String
            #12 62-97 "{\n            fail \"true\"\n        }" : Int
            #14 76-87 "fail \"true\"" : Int
            #15 81-87 "\"true\"" : String
            #16 98-139 "else {\n            fail \"false\"\n        }" : Int
            #17 103-139 "{\n            fail \"false\"\n        }" : Int
            #19 117-129 "fail \"false\"" : Int
            #20 122-129 "\"false\"" : String
        "##]],
    );
}

#[test]
fn ternop_cond_error() {
    check(
        "",
        "7 ? 1 | 0",
        &expect![[r##"
            #1 0-9 "7 ? 1 | 0" : Int
            #2 0-1 "7" : Int
            #3 4-5 "1" : Int
            #4 8-9 "0" : Int
            Error(Type(Error(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 0, hi: 1 }))))
        "##]],
    );
}

#[test]
fn ternop_update_invalid_container() {
    check(
        "",
        "(1, 2, 3) w/ 2 <- 4",
        &expect![[r##"
            #1 0-19 "(1, 2, 3) w/ 2 <- 4" : (Int, Int, Int)
            #2 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 13-14 "2" : Int
            #7 18-19 "4" : Int
            Error(Type(Error(MissingClass(HasIndex { container: Tuple([Prim(Int), Prim(Int), Prim(Int)]), index: Prim(Int), item: Prim(Int) }, Span { lo: 0, hi: 19 }))))
        "##]],
    );
}

#[test]
fn ternop_update_invalid_index() {
    check(
        "",
        "[1, 2, 3] w/ false <- 4",
        &expect![[r##"
            #1 0-23 "[1, 2, 3] w/ false <- 4" : (Int)[]
            #2 0-9 "[1, 2, 3]" : (Int)[]
            #3 1-2 "1" : Int
            #4 4-5 "2" : Int
            #5 7-8 "3" : Int
            #6 13-18 "false" : Bool
            #7 22-23 "4" : Int
            Error(Type(Error(MissingClass(HasIndex { container: Array(Prim(Int)), index: Prim(Bool), item: Prim(Int) }, Span { lo: 0, hi: 23 }))))
        "##]],
    );
}

#[test]
fn unop_bitwise_not_bool() {
    check(
        "",
        "~~~false",
        &expect![[r##"
            #1 0-8 "~~~false" : Bool
            #2 3-8 "false" : Bool
            Error(Type(Error(MissingClass(Num(Prim(Bool)), Span { lo: 3, hi: 8 }))))
        "##]],
    );
}

#[test]
fn unop_not_int() {
    check(
        "",
        "not 0",
        &expect![[r##"
            #1 0-5 "not 0" : Int
            #2 4-5 "0" : Int
            Error(Type(Error(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 4, hi: 5 }))))
        "##]],
    );
}

#[test]
fn unop_neg_bool() {
    check(
        "",
        "-false",
        &expect![[r##"
            #1 0-6 "-false" : Bool
            #2 1-6 "false" : Bool
            Error(Type(Error(MissingClass(Num(Prim(Bool)), Span { lo: 1, hi: 6 }))))
        "##]],
    );
}

#[test]
fn unop_pos_bool() {
    check(
        "",
        "+false",
        &expect![[r##"
            #1 0-6 "+false" : Bool
            #2 1-6 "false" : Bool
            Error(Type(Error(MissingClass(Num(Prim(Bool)), Span { lo: 1, hi: 6 }))))
        "##]],
    );
}

#[test]
fn while_cond_error() {
    check(
        "",
        "while Zero {}",
        &expect![[r##"
            #1 0-13 "while Zero {}" : ()
            #2 6-10 "Zero" : Result
            #3 11-13 "{}" : ()
            Error(Type(Error(TypeMismatch(Prim(Bool), Prim(Result), Span { lo: 6, hi: 10 }))))
        "##]],
    );
}

#[test]
fn controlled_spec_impl() {
    check(
        indoc! {"
            namespace A {
                operation Foo(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (cs, ...) {}
                }
            }
        "},
        "",
        &expect![[r##"
            #6 31-42 "(q : Qubit)" : Qubit
            #7 32-41 "q : Qubit" : Qubit
            #11 72-75 "..." : Qubit
            #12 76-78 "{}" : ()
            #14 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #15 99-101 "cs" : (Qubit)[]
            #17 103-106 "..." : Qubit
            #18 108-110 "{}" : ()
        "##]],
    );
}

#[test]
fn call_controlled() {
    check(
        indoc! {"
            namespace A {
                operation Foo(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (cs, ...) {}
                }
            }
        "},
        indoc! {"
            {
                use q1 = Qubit();
                use q2 = Qubit();
                Controlled A.Foo([q1], q2);
            }
        "},
        &expect![[r##"
            #6 31-42 "(q : Qubit)" : Qubit
            #7 32-41 "q : Qubit" : Qubit
            #11 72-75 "..." : Qubit
            #12 76-78 "{}" : ()
            #14 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #15 99-101 "cs" : (Qubit)[]
            #17 103-106 "..." : Qubit
            #18 108-110 "{}" : ()
            #19 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #20 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #22 129-131 "q1" : Qubit
            #24 134-141 "Qubit()" : Qubit
            #26 151-153 "q2" : Qubit
            #28 156-163 "Qubit()" : Qubit
            #30 169-195 "Controlled A.Foo([q1], q2)" : ()
            #31 169-185 "Controlled A.Foo" : (((Qubit)[], Qubit) => () is Ctl)
            #32 180-185 "A.Foo" : (Qubit => () is Ctl)
            #33 185-195 "([q1], q2)" : ((Qubit)[], Qubit)
            #34 186-190 "[q1]" : (Qubit)[]
            #35 187-189 "q1" : Qubit
            #36 192-194 "q2" : Qubit
        "##]],
    );
}

#[test]
fn call_controlled_nested() {
    check(
        indoc! {"
            namespace A {
                operation Foo(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (cs, ...) {}
                }
            }
        "},
        indoc! {"
            {
                use q1 = Qubit();
                use q2 = Qubit();
                use q3 = Qubit();
                Controlled Controlled A.Foo([q1], ([q2], q3));
            }
        "},
        &expect![[r##"
            #6 31-42 "(q : Qubit)" : Qubit
            #7 32-41 "q : Qubit" : Qubit
            #11 72-75 "..." : Qubit
            #12 76-78 "{}" : ()
            #14 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #15 99-101 "cs" : (Qubit)[]
            #17 103-106 "..." : Qubit
            #18 108-110 "{}" : ()
            #19 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #20 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #22 129-131 "q1" : Qubit
            #24 134-141 "Qubit()" : Qubit
            #26 151-153 "q2" : Qubit
            #28 156-163 "Qubit()" : Qubit
            #30 173-175 "q3" : Qubit
            #32 178-185 "Qubit()" : Qubit
            #34 191-236 "Controlled Controlled A.Foo([q1], ([q2], q3))" : ()
            #35 191-218 "Controlled Controlled A.Foo" : (((Qubit)[], ((Qubit)[], Qubit)) => () is Ctl)
            #36 202-218 "Controlled A.Foo" : (((Qubit)[], Qubit) => () is Ctl)
            #37 213-218 "A.Foo" : (Qubit => () is Ctl)
            #38 218-236 "([q1], ([q2], q3))" : ((Qubit)[], ((Qubit)[], Qubit))
            #39 219-223 "[q1]" : (Qubit)[]
            #40 220-222 "q1" : Qubit
            #41 225-235 "([q2], q3)" : ((Qubit)[], Qubit)
            #42 226-230 "[q2]" : (Qubit)[]
            #43 227-229 "q2" : Qubit
            #44 232-234 "q3" : Qubit
        "##]],
    );
}

#[test]
fn call_controlled_error() {
    check(
        indoc! {"
            namespace A {
                operation Foo(q : Qubit) : Unit is Ctl {
                    body ... {}
                    controlled (cs, ...) {}
                }
            }
        "},
        indoc! {"
            {
                use q = Qubit();
                Controlled A.Foo([1], q);
            }
        "},
        &expect![[r##"
            #6 31-42 "(q : Qubit)" : Qubit
            #7 32-41 "q : Qubit" : Qubit
            #11 72-75 "..." : Qubit
            #12 76-78 "{}" : ()
            #14 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #15 99-101 "cs" : (Qubit)[]
            #17 103-106 "..." : Qubit
            #18 108-110 "{}" : ()
            #19 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #20 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #22 129-130 "q" : Qubit
            #24 133-140 "Qubit()" : Qubit
            #26 146-170 "Controlled A.Foo([1], q)" : ()
            #27 146-162 "Controlled A.Foo" : (((Qubit)[], Qubit) => () is Ctl)
            #28 157-162 "A.Foo" : (Qubit => () is Ctl)
            #29 162-170 "([1], q)" : ((Int)[], Qubit)
            #30 163-166 "[1]" : (Int)[]
            #31 164-165 "1" : Int
            #32 168-169 "q" : Qubit
            Error(Type(Error(TypeMismatch(Prim(Qubit), Prim(Int), Span { lo: 157, hi: 162 }))))
        "##]],
    );
}

#[test]
fn adj_requires_unit_return() {
    check(
        indoc! {"
            namespace A {
                operation Foo() : Int is Adj { 1 }
            }
        "},
        "",
        &expect![[r##"
            #6 31-33 "()" : ()
            #8 47-52 "{ 1 }" : Int
            #10 49-50 "1" : Int
            Error(Type(Error(TypeMismatch(Tuple([]), Prim(Int), Span { lo: 36, hi: 39 }))))
        "##]],
    );
}

#[test]
fn ctl_requires_unit_return() {
    check(
        indoc! {"
            namespace A {
                operation Foo() : Int is Ctl { 1 }
            }
        "},
        "",
        &expect![[r##"
            #6 31-33 "()" : ()
            #8 47-52 "{ 1 }" : Int
            #10 49-50 "1" : Int
            Error(Type(Error(TypeMismatch(Tuple([]), Prim(Int), Span { lo: 36, hi: 39 }))))
        "##]],
    );
}

#[test]
fn adj_ctl_requires_unit_return() {
    check(
        indoc! {"
            namespace A {
                operation Foo() : Int is Adj + Ctl { 1 }
            }
        "},
        "",
        &expect![[r##"
            #6 31-33 "()" : ()
            #10 53-58 "{ 1 }" : Int
            #12 55-56 "1" : Int
            Error(Type(Error(TypeMismatch(Tuple([]), Prim(Int), Span { lo: 36, hi: 39 }))))
        "##]],
    );
}

#[test]
fn fail_diverges() {
    check(
        "",
        indoc! {r#"
            if true {
                fail "true"
            } else {
                4
            }
        "#},
        &expect![[r##"
            #1 0-42 "if true {\n    fail \"true\"\n} else {\n    4\n}" : Int
            #2 3-7 "true" : Bool
            #3 8-27 "{\n    fail \"true\"\n}" : Int
            #5 14-25 "fail \"true\"" : Int
            #6 19-25 "\"true\"" : String
            #7 28-42 "else {\n    4\n}" : Int
            #8 33-42 "{\n    4\n}" : Int
            #10 39-40 "4" : Int
        "##]],
    );
}

#[test]
fn return_diverges() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Bool) : Int {
                    let x = if x {
                        return 1
                    } else {
                        true
                    };
                    2
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-40 "(x : Bool)" : Bool
            #7 31-39 "x : Bool" : Bool
            #9 47-153 "{\n        let x = if x {\n            return 1\n        } else {\n            true\n        };\n        2\n    }" : Int
            #11 61-62 "x" : Bool
            #13 65-136 "if x {\n            return 1\n        } else {\n            true\n        }" : Bool
            #14 68-69 "x" : Bool
            #15 70-102 "{\n            return 1\n        }" : Bool
            #17 84-92 "return 1" : Bool
            #18 91-92 "1" : Int
            #19 103-136 "else {\n            true\n        }" : Bool
            #20 108-136 "{\n            true\n        }" : Bool
            #22 122-126 "true" : Bool
            #24 146-147 "2" : Int
        "##]],
    );
}

#[test]
fn return_diverges_stmt_after() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Bool) : Int {
                    let x = {
                        return 1;
                        true
                    };
                    x
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-40 "(x : Bool)" : Bool
            #7 31-39 "x : Bool" : Bool
            #9 47-132 "{\n        let x = {\n            return 1;\n            true\n        };\n        x\n    }" : Int
            #11 61-62 "x" : ?0
            #13 65-115 "{\n            return 1;\n            true\n        }" : ?0
            #14 65-115 "{\n            return 1;\n            true\n        }" : ?0
            #16 79-87 "return 1" : ?1
            #17 86-87 "1" : Int
            #19 101-105 "true" : Bool
            #21 125-126 "x" : ?0
        "##]],
    );
}

#[test]
fn return_mismatch() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Bool) : Int {
                    return true;
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-40 "(x : Bool)" : Bool
            #7 31-39 "x : Bool" : Bool
            #9 47-75 "{\n        return true;\n    }" : Int
            #11 57-68 "return true" : ?0
            #12 64-68 "true" : Bool
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 64, hi: 68 }))))
        "##]],
    );
}

#[test]
fn array_length_field_is_int() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Qubit[]) : Int {
                    x::Length
                }
            }
        "},
        "",
        &expect![[r##"
            #5 27-30 "Foo" : ((Qubit)[]) -> (Int)
            #6 30-43 "(x : Qubit[])" : (Qubit)[]
            #7 31-42 "x : Qubit[]" : (Qubit)[]
            #8 31-32 "x" : (Qubit)[]
            #12 50-75 "{\n        x::Length\n    }" : Int
            #13 60-69 "x::Length" : Int
            #14 60-69 "x::Length" : Int
            #15 60-61 "x" : (Qubit)[]
        "##]],
    );
}

#[test]
fn array_length_generic_is_int() {
    check(
        indoc! {"
            namespace A {
                function Length<'T>(a : 'T[]) : Int {
                    a::Length
                }
                function Foo(x : Qubit[]) : Int {
                    Length(x)
                }
            }
        "},
        "",
        &expect![[r##"
            #5 27-33 "Length" : (('T)[]) -> (Int)
            #7 37-47 "(a : 'T[])" : ('T)[]
            #8 38-46 "a : 'T[]" : ('T)[]
            #9 38-39 "a" : ('T)[]
            #14 54-79 "{\n        a::Length\n    }" : Int
            #15 64-73 "a::Length" : Int
            #16 64-73 "a::Length" : Int
            #17 64-65 "a" : ('T)[]
            #21 93-96 "Foo" : ((Qubit)[]) -> (Int)
            #22 96-109 "(x : Qubit[])" : (Qubit)[]
            #23 97-108 "x : Qubit[]" : (Qubit)[]
            #24 97-98 "x" : (Qubit)[]
            #28 116-141 "{\n        Length(x)\n    }" : Int
            #29 126-135 "Length(x)" : Int
            #30 126-135 "Length(x)" : Int
            #31 126-132 "Length" : ((Qubit)[]) -> (Int)
            #32 132-135 "(x)" : (Qubit)[]
            #33 133-134 "x" : (Qubit)[]
        "##]],
    );
}

#[test]
fn array_length_field_used_as_double_error() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Qubit[]) : Double {
                    x::Length * 2.0
                }
            }
        "},
        "",
        &expect![[r##"
            #5 27-30 "Foo" : ((Qubit)[]) -> (Double)
            #6 30-43 "(x : Qubit[])" : (Qubit)[]
            #7 31-42 "x : Qubit[]" : (Qubit)[]
            #8 31-32 "x" : (Qubit)[]
            #12 53-84 "{\n        x::Length * 2.0\n    }" : Double
            #13 63-78 "x::Length * 2.0" : Double
            #14 63-78 "x::Length * 2.0" : Double
            #15 63-72 "x::Length" : Double
            #16 63-64 "x" : (Qubit)[]
            #18 75-78 "2.0" : Double
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Double), Span { lo: 63, hi: 72 }))))
        "##]],
    );
}

#[test]
fn array_unknown_field_error() {
    check(
        indoc! {"
            namespace A {
                function Foo(x : Qubit[]) : Int {
                    x::Size
                }
            }
        "},
        "",
        &expect![[r##"
            #5 27-30 "Foo" : ((Qubit)[]) -> (Int)
            #6 30-43 "(x : Qubit[])" : (Qubit)[]
            #7 31-42 "x : Qubit[]" : (Qubit)[]
            #8 31-32 "x" : (Qubit)[]
            #12 50-73 "{\n        x::Size\n    }" : Int
            #13 60-67 "x::Size" : Int
            #14 60-67 "x::Size" : Int
            #15 60-61 "x" : (Qubit)[]
            Error(Type(Error(MissingClass(HasField { record: Array(Prim(Qubit)), name: "Size", item: Var(Var(0)) }, Span { lo: 60, hi: 67 }))))
        "##]],
    );
}

#[test]
fn range_fields_are_int() {
    check(
        indoc! {"
            namespace A {
                function Foo(r : Range) : (Int, Int, Int) {
                    (r::Start, r::Step, r::End)
                }
            }
        "},
        "",
        &expect![[r##"
            #5 27-30 "Foo" : (Range) -> ((Int, Int, Int))
            #6 30-41 "(r : Range)" : Range
            #7 31-40 "r : Range" : Range
            #8 31-32 "r" : Range
            #14 60-103 "{\n        (r::Start, r::Step, r::End)\n    }" : (Int, Int, Int)
            #15 70-97 "(r::Start, r::Step, r::End)" : (Int, Int, Int)
            #16 70-97 "(r::Start, r::Step, r::End)" : (Int, Int, Int)
            #17 71-79 "r::Start" : Int
            #18 71-72 "r" : Range
            #20 81-88 "r::Step" : Int
            #21 81-82 "r" : Range
            #23 90-96 "r::End" : Int
            #24 90-91 "r" : Range
        "##]],
    );
}
