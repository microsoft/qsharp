// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, PackageStore, SourceMap};
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
    let sources = SourceMap::new([("test".into(), source.into())], Some(entry_expr.into()));
    let unit = compile(&store, [std], sources);
    let mut tys = TyCollector { tys: Vec::new() };
    tys.visit_package(&unit.package);

    let mut actual = String::new();
    for (id, span, ty) in tys.tys {
        let source = unit.sources.find_offset(span.lo);
        let code = &source.contents[span.lo - source.offset..span.hi - source.offset];
        writeln!(actual, "#{id} {}-{} {code:?} : {ty}", span.lo, span.hi)
            .expect("writing type to string should succeed");
    }

    for error in &unit.errors {
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
            #2 30-32 "()" : ()
            #3 40-42 "{}" : ()
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
            #2 30-32 "()" : ()
            #3 39-44 "{ 4 }" : Int
            #5 41-42 "4" : Int
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
            #2 30-32 "()" : ()
            #3 39-47 "{ true }" : Bool
            #5 41-45 "true" : Bool
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
            #2 30-32 "()" : ()
            #3 39-45 "{ 4; }" : ()
            #5 41-42 "4" : Int
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
            #2 30-32 "()" : ()
            #3 39-75 "{\n        let x = 4;\n        x\n    }" : Int
            #5 53-54 "x" : Int
            #7 57-58 "4" : Int
            #9 68-69 "x" : Int
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
            #2 31-38 "x : Int" : Int
            #4 46-51 "{ x }" : Int
            #6 48-49 "x" : Int
            #9 68-70 "()" : ()
            #10 77-87 "{ Foo(4) }" : Int
            #12 79-85 "Foo(4)" : Int
            #13 79-82 "Foo" : (Int -> Int)
            #14 83-84 "4" : Int
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
            #3 40-46 "x : 'T" : 'T
            #5 53-58 "{ x }" : 'T
            #7 55-56 "x" : 'T
            #10 75-77 "()" : ()
            #11 84-99 "{ Identity(4) }" : Int
            #13 86-97 "Identity(4)" : Int
            #14 86-94 "Identity" : (Int -> Int)
            #15 95-96 "4" : Int
        "##]],
    );
}

#[test]
fn call_generic_length() {
    check(
        "",
        "Length([true, false, true])",
        &expect![[r##"
            #0 0-27 "Length([true, false, true])" : Int
            #1 0-6 "Length" : ((Bool)[] -> Int)
            #2 7-26 "[true, false, true]" : (Bool)[]
            #3 8-12 "true" : Bool
            #4 14-19 "false" : Bool
            #5 21-25 "true" : Bool
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
            #2 30-32 "()" : ()
            #3 40-52 "{ 1 + [2]; }" : ()
            #5 42-49 "1 + [2]" : Int
            #6 42-43 "1" : Int
            #7 46-49 "[2]" : (Int)[]
            #8 47-48 "2" : Int
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
            #0 0-44 "Microsoft.Quantum.Convert.IntAsDouble(false)" : Double
            #1 0-37 "Microsoft.Quantum.Convert.IntAsDouble" : (Int -> Double)
            #2 38-43 "false" : Bool
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
            #0 0-17 "Length((1, 2, 3))" : Int
            #1 0-6 "Length" : ((?0)[] -> Int)
            #2 7-16 "(1, 2, 3)" : (Int, Int, Int)
            #3 8-9 "1" : Int
            #4 11-12 "2" : Int
            #5 14-15 "3" : Int
            Error(Type(Error(TypeMismatch(Array(Infer(InferId(0))), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 17 }))))
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
            #0 0-35 "{\n    use q = Qubit();\n    Ry(q);\n}" : ()
            #1 0-35 "{\n    use q = Qubit();\n    Ry(q);\n}" : ()
            #3 10-11 "q" : Qubit
            #5 14-21 "Qubit()" : Qubit
            #7 27-32 "Ry(q)" : ()
            #8 27-29 "Ry" : ((Double, Qubit) => () is Adj + Ctl)
            #9 30-31 "q" : Qubit
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
            #0 0-16 "[1, 2, 3][false]" : ?0
            #1 0-9 "[1, 2, 3]" : (Int)[]
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 10-15 "false" : Bool
            Error(Type(Error(MissingClass(HasIndex { container: Array(Prim(Int)), index: Prim(Bool), item: Infer(InferId(0)) }, Span { lo: 0, hi: 16 }))))
        "##]],
    );
}

#[test]
fn array_repeat_error() {
    check(
        "",
        "[4, size = true]",
        &expect![[r##"
            #0 0-16 "[4, size = true]" : (Int)[]
            #1 1-2 "4" : Int
            #2 11-15 "true" : Bool
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
            #0 0-48 "{\n    mutable x = false;\n    set x += 1;\n    x\n}" : Bool
            #1 0-48 "{\n    mutable x = false;\n    set x += 1;\n    x\n}" : Bool
            #3 14-15 "x" : Bool
            #5 18-23 "false" : Bool
            #7 29-39 "set x += 1" : ()
            #8 33-34 "x" : Bool
            #9 38-39 "1" : Int
            #11 45-46 "x" : Bool
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
            #0 0-12 "(1, 3) + 5.4" : (Int, Int)
            #1 0-6 "(1, 3)" : (Int, Int)
            #2 1-2 "1" : Int
            #3 4-5 "3" : Int
            #4 9-12 "5.4" : Double
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
            #0 0-7 "1 + 5.4" : Int
            #1 0-1 "1" : Int
            #2 4-7 "5.4" : Double
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
            #0 0-10 "28 &&& 54L" : Int
            #1 0-2 "28" : Int
            #2 7-10 "54L" : BigInt
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
            #2 31-33 "()" : ()
            #3 41-43 "{}" : ()
            #6 58-60 "()" : ()
            #7 68-70 "{}" : ()
            #9 73-89 "Test.A == Test.B" : Bool
            #10 73-79 "Test.A" : (() -> ())
            #11 83-89 "Test.B" : (() -> ())
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
            #0 0-25 "(1, 2, 3) == (1, 2, 3, 4)" : Bool
            #1 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 13-25 "(1, 2, 3, 4)" : (Int, Int, Int, Int)
            #6 14-15 "1" : Int
            #7 17-18 "2" : Int
            #8 20-21 "3" : Int
            #9 23-24 "4" : Int
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
            #0 0-25 "(1, 2, 3) == (1, Zero, 3)" : Bool
            #1 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 13-25 "(1, Zero, 3)" : (Int, Result, Int)
            #6 14-15 "1" : Int
            #7 17-21 "Zero" : Result
            #8 23-24 "3" : Int
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
            #0 0-9 "18L == 18" : Bool
            #1 0-3 "18L" : BigInt
            #2 7-9 "18" : Int
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
            #0 0-9 "18L != 18" : Bool
            #1 0-3 "18L" : BigInt
            #2 7-9 "18" : Int
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
            #0 0-25 "(1, 2, 3) != (1, Zero, 3)" : Bool
            #1 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 13-25 "(1, Zero, 3)" : (Int, Result, Int)
            #6 14-15 "1" : Int
            #7 17-21 "Zero" : Result
            #8 23-24 "3" : Int
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
            #0 0-25 "(1, 2, 3) != (1, 2, 3, 4)" : Bool
            #1 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 13-25 "(1, 2, 3, 4)" : (Int, Int, Int, Int)
            #6 14-15 "1" : Int
            #7 17-18 "2" : Int
            #8 20-21 "3" : Int
            #9 23-24 "4" : Int
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
            #0 0-10 "28 ||| 54L" : Int
            #1 0-2 "28" : Int
            #2 7-10 "54L" : BigInt
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
            #0 0-10 "28 ^^^ 54L" : Int
            #1 0-2 "28" : Int
            #2 7-10 "54L" : BigInt
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
            #0 0-27 "{ let (x, y, z) = (0, 1); }" : ()
            #1 0-27 "{ let (x, y, z) = (0, 1); }" : ()
            #3 6-15 "(x, y, z)" : (?0, ?1, ?2)
            #4 7-8 "x" : ?0
            #6 10-11 "y" : ?1
            #8 13-14 "z" : ?2
            #10 18-24 "(0, 1)" : (Int, Int)
            #11 19-20 "0" : Int
            #12 22-23 "1" : Int
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Infer(InferId(0)), Infer(InferId(1)), Infer(InferId(2))]), Span { lo: 6, hi: 15 }))))
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
            #0 0-66 "{\n    mutable (x, y) = (0, 1);\n    set (x, y) = (1, 2, 3);\n    x\n}" : Int
            #1 0-66 "{\n    mutable (x, y) = (0, 1);\n    set (x, y) = (1, 2, 3);\n    x\n}" : Int
            #3 14-20 "(x, y)" : (Int, Int)
            #4 15-16 "x" : Int
            #6 18-19 "y" : Int
            #8 23-29 "(0, 1)" : (Int, Int)
            #9 24-25 "0" : Int
            #10 27-28 "1" : Int
            #12 35-57 "set (x, y) = (1, 2, 3)" : ()
            #13 39-45 "(x, y)" : (Int, Int)
            #14 40-41 "x" : Int
            #15 43-44 "y" : Int
            #16 48-57 "(1, 2, 3)" : (Int, Int, Int)
            #17 49-50 "1" : Int
            #18 52-53 "2" : Int
            #19 55-56 "3" : Int
            #21 63-64 "x" : Int
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
            #0 0-25 "{ use q = Qubit[false]; }" : ()
            #1 0-25 "{ use q = Qubit[false]; }" : ()
            #3 6-7 "q" : (Qubit)[]
            #5 10-22 "Qubit[false]" : (Qubit)[]
            #6 16-21 "false" : Bool
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
            #0 0-47 "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }" : ()
            #1 0-47 "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }" : ()
            #3 6-13 "(q, q1)" : (?0, ?1)
            #4 7-8 "q" : ?0
            #6 10-12 "q1" : ?1
            #8 16-44 "(Qubit[3], Qubit(), Qubit())" : ((Qubit)[], Qubit, Qubit)
            #9 17-25 "Qubit[3]" : (Qubit)[]
            #10 23-24 "3" : Int
            #11 27-34 "Qubit()" : Qubit
            #12 36-43 "Qubit()" : Qubit
            Error(Type(Error(TypeMismatch(Tuple([Array(Prim(Qubit)), Prim(Qubit), Prim(Qubit)]), Tuple([Infer(InferId(0)), Infer(InferId(1))]), Span { lo: 6, hi: 13 }))))
        "##]],
    );
}

#[test]
fn for_loop_not_iterable() {
    check(
        "",
        "for i in (1, true, One) {}",
        &expect![[r##"
            #0 0-26 "for i in (1, true, One) {}" : ()
            #1 4-5 "i" : ?0
            #3 9-23 "(1, true, One)" : (Int, Bool, Result)
            #4 10-11 "1" : Int
            #5 13-17 "true" : Bool
            #6 19-22 "One" : Result
            #7 24-26 "{}" : ()
            Error(Type(Error(MissingClass(Iterable { container: Tuple([Prim(Int), Prim(Bool), Prim(Result)]), item: Infer(InferId(0)) }, Span { lo: 9, hi: 23 }))))
        "##]],
    );
}

#[test]
fn if_cond_error() {
    check(
        "",
        "if 4 {}",
        &expect![[r##"
            #0 0-7 "if 4 {}" : ()
            #1 3-4 "4" : Int
            #2 5-7 "{}" : ()
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
            #0 0-13 "if true { 4 }" : Int
            #1 3-7 "true" : Bool
            #2 8-13 "{ 4 }" : Int
            #4 10-11 "4" : Int
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
            #0 0-34 "if false {} else { fail \"error\"; }" : ()
            #1 3-8 "false" : Bool
            #2 9-11 "{}" : ()
            #3 12-34 "else { fail \"error\"; }" : ()
            #4 17-34 "{ fail \"error\"; }" : ()
            #6 19-31 "fail \"error\"" : ?0
            #7 24-31 "\"error\"" : String
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
            #2 28-30 "()" : ()
            #3 37-154 "{\n        if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }\n    }" : Int
            #5 47-148 "if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }" : Int
            #6 50-62 "fail \"error\"" : Bool
            #7 55-62 "\"error\"" : String
            #8 63-113 "{\n            \"this type doesn't matter\"\n        }" : String
            #10 77-103 "\"this type doesn't matter\"" : String
            #11 114-148 "else {\n            \"foo\"\n        }" : String
            #12 119-148 "{\n            \"foo\"\n        }" : String
            #14 133-138 "\"foo\"" : String
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
            #2 28-30 "()" : ()
            #3 37-145 "{\n        if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }\n    }" : Int
            #5 47-139 "if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }" : Int
            #6 50-61 "fail \"cond\"" : Bool
            #7 55-61 "\"cond\"" : String
            #8 62-97 "{\n            fail \"true\"\n        }" : Int
            #10 76-87 "fail \"true\"" : Int
            #11 81-87 "\"true\"" : String
            #12 98-139 "else {\n            fail \"false\"\n        }" : Int
            #13 103-139 "{\n            fail \"false\"\n        }" : Int
            #15 117-129 "fail \"false\"" : Int
            #16 122-129 "\"false\"" : String
        "##]],
    );
}

#[test]
fn ternop_cond_error() {
    check(
        "",
        "7 ? 1 | 0",
        &expect![[r##"
            #0 0-9 "7 ? 1 | 0" : Int
            #1 0-1 "7" : Int
            #2 4-5 "1" : Int
            #3 8-9 "0" : Int
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
            #0 0-19 "(1, 2, 3) w/ 2 <- 4" : (Int, Int, Int)
            #1 0-9 "(1, 2, 3)" : (Int, Int, Int)
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 13-14 "2" : Int
            #6 18-19 "4" : Int
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
            #0 0-23 "[1, 2, 3] w/ false <- 4" : (Int)[]
            #1 0-9 "[1, 2, 3]" : (Int)[]
            #2 1-2 "1" : Int
            #3 4-5 "2" : Int
            #4 7-8 "3" : Int
            #5 13-18 "false" : Bool
            #6 22-23 "4" : Int
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
            #0 0-8 "~~~false" : Bool
            #1 3-8 "false" : Bool
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
            #0 0-5 "not 0" : Int
            #1 4-5 "0" : Int
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
            #0 0-6 "-false" : Bool
            #1 1-6 "false" : Bool
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
            #0 0-6 "+false" : Bool
            #1 1-6 "false" : Bool
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
            #0 0-13 "while Zero {}" : ()
            #1 6-10 "Zero" : Result
            #2 11-13 "{}" : ()
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
            #2 32-41 "q : Qubit" : Qubit
            #6 72-75 "..." : Qubit
            #7 76-78 "{}" : ()
            #9 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #10 99-101 "cs" : (Qubit)[]
            #12 103-106 "..." : Qubit
            #13 108-110 "{}" : ()
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
            #2 32-41 "q : Qubit" : Qubit
            #6 72-75 "..." : Qubit
            #7 76-78 "{}" : ()
            #9 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #10 99-101 "cs" : (Qubit)[]
            #12 103-106 "..." : Qubit
            #13 108-110 "{}" : ()
            #15 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #16 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #18 129-131 "q1" : Qubit
            #20 134-141 "Qubit()" : Qubit
            #22 151-153 "q2" : Qubit
            #24 156-163 "Qubit()" : Qubit
            #26 169-195 "Controlled A.Foo([q1], q2)" : ()
            #27 169-185 "Controlled A.Foo" : (((Qubit)[], Qubit) => () is Ctl)
            #28 180-185 "A.Foo" : (Qubit => () is Ctl)
            #29 185-195 "([q1], q2)" : ((Qubit)[], Qubit)
            #30 186-190 "[q1]" : (Qubit)[]
            #31 187-189 "q1" : Qubit
            #32 192-194 "q2" : Qubit
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
            #2 32-41 "q : Qubit" : Qubit
            #6 72-75 "..." : Qubit
            #7 76-78 "{}" : ()
            #9 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #10 99-101 "cs" : (Qubit)[]
            #12 103-106 "..." : Qubit
            #13 108-110 "{}" : ()
            #15 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #16 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #18 129-131 "q1" : Qubit
            #20 134-141 "Qubit()" : Qubit
            #22 151-153 "q2" : Qubit
            #24 156-163 "Qubit()" : Qubit
            #26 173-175 "q3" : Qubit
            #28 178-185 "Qubit()" : Qubit
            #30 191-236 "Controlled Controlled A.Foo([q1], ([q2], q3))" : ()
            #31 191-218 "Controlled Controlled A.Foo" : (((Qubit)[], ((Qubit)[], Qubit)) => () is Ctl)
            #32 202-218 "Controlled A.Foo" : (((Qubit)[], Qubit) => () is Ctl)
            #33 213-218 "A.Foo" : (Qubit => () is Ctl)
            #34 218-236 "([q1], ([q2], q3))" : ((Qubit)[], ((Qubit)[], Qubit))
            #35 219-223 "[q1]" : (Qubit)[]
            #36 220-222 "q1" : Qubit
            #37 225-235 "([q2], q3)" : ((Qubit)[], Qubit)
            #38 226-230 "[q2]" : (Qubit)[]
            #39 227-229 "q2" : Qubit
            #40 232-234 "q3" : Qubit
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
            #2 32-41 "q : Qubit" : Qubit
            #6 72-75 "..." : Qubit
            #7 76-78 "{}" : ()
            #9 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #10 99-101 "cs" : (Qubit)[]
            #12 103-106 "..." : Qubit
            #13 108-110 "{}" : ()
            #15 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #16 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #18 129-130 "q" : Qubit
            #20 133-140 "Qubit()" : Qubit
            #22 146-170 "Controlled A.Foo([1], q)" : ()
            #23 146-162 "Controlled A.Foo" : (((Qubit)[], Qubit) => () is Ctl)
            #24 157-162 "A.Foo" : (Qubit => () is Ctl)
            #25 162-170 "([1], q)" : ((Int)[], Qubit)
            #26 163-166 "[1]" : (Int)[]
            #27 164-165 "1" : Int
            #28 168-169 "q" : Qubit
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
            #2 31-33 "()" : ()
            #4 47-52 "{ 1 }" : Int
            #6 49-50 "1" : Int
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
            #2 31-33 "()" : ()
            #4 47-52 "{ 1 }" : Int
            #6 49-50 "1" : Int
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
            #2 31-33 "()" : ()
            #6 53-58 "{ 1 }" : Int
            #8 55-56 "1" : Int
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
            #0 0-42 "if true {\n    fail \"true\"\n} else {\n    4\n}" : Int
            #1 3-7 "true" : Bool
            #2 8-27 "{\n    fail \"true\"\n}" : Int
            #4 14-25 "fail \"true\"" : Int
            #5 19-25 "\"true\"" : String
            #6 28-42 "else {\n    4\n}" : Int
            #7 33-42 "{\n    4\n}" : Int
            #9 39-40 "4" : Int
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
            #2 31-39 "x : Bool" : Bool
            #4 47-153 "{\n        let x = if x {\n            return 1\n        } else {\n            true\n        };\n        2\n    }" : Int
            #6 61-62 "x" : Bool
            #8 65-136 "if x {\n            return 1\n        } else {\n            true\n        }" : Bool
            #9 68-69 "x" : Bool
            #10 70-102 "{\n            return 1\n        }" : Bool
            #12 84-92 "return 1" : Bool
            #13 91-92 "1" : Int
            #14 103-136 "else {\n            true\n        }" : Bool
            #15 108-136 "{\n            true\n        }" : Bool
            #17 122-126 "true" : Bool
            #19 146-147 "2" : Int
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
            #2 31-39 "x : Bool" : Bool
            #4 47-132 "{\n        let x = {\n            return 1;\n            true\n        };\n        x\n    }" : Int
            #6 61-62 "x" : ?0
            #8 65-115 "{\n            return 1;\n            true\n        }" : ?0
            #9 65-115 "{\n            return 1;\n            true\n        }" : ?0
            #11 79-87 "return 1" : ?1
            #12 86-87 "1" : Int
            #14 101-105 "true" : Bool
            #16 125-126 "x" : ?0
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
            #2 31-39 "x : Bool" : Bool
            #4 47-75 "{\n        return true;\n    }" : Int
            #6 57-68 "return true" : ?0
            #7 64-68 "true" : Bool
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
            #2 31-42 "x : Qubit[]" : (Qubit)[]
            #4 50-75 "{\n        x::Length\n    }" : Int
            #6 60-69 "x::Length" : Int
            #7 60-61 "x" : (Qubit)[]
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
            #3 38-46 "a : 'T[]" : ('T)[]
            #5 54-79 "{\n        a::Length\n    }" : Int
            #7 64-73 "a::Length" : Int
            #8 64-65 "a" : ('T)[]
            #11 97-108 "x : Qubit[]" : (Qubit)[]
            #13 116-141 "{\n        Length(x)\n    }" : Int
            #15 126-135 "Length(x)" : Int
            #16 126-132 "Length" : ((Qubit)[] -> Int)
            #17 133-134 "x" : (Qubit)[]
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
            #2 31-42 "x : Qubit[]" : (Qubit)[]
            #4 53-84 "{\n        x::Length * 2.0\n    }" : Double
            #6 63-78 "x::Length * 2.0" : Double
            #7 63-72 "x::Length" : Double
            #8 63-64 "x" : (Qubit)[]
            #9 75-78 "2.0" : Double
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
            #2 31-42 "x : Qubit[]" : (Qubit)[]
            #4 50-73 "{\n        x::Size\n    }" : Int
            #6 60-67 "x::Size" : Int
            #7 60-61 "x" : (Qubit)[]
            Error(Type(Error(MissingClass(HasField { record: Array(Prim(Qubit)), name: "Size", item: Infer(InferId(0)) }, Span { lo: 60, hi: 67 }))))
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
            #2 31-40 "r : Range" : Range
            #4 60-103 "{\n        (r::Start, r::Step, r::End)\n    }" : (Int, Int, Int)
            #6 70-97 "(r::Start, r::Step, r::End)" : (Int, Int, Int)
            #7 71-79 "r::Start" : Int
            #8 71-72 "r" : Range
            #9 81-88 "r::Step" : Int
            #10 81-82 "r" : Range
            #11 90-96 "r::End" : Int
            #12 90-91 "r" : Range
        "##]],
    );
}

#[test]
fn range_to_field_start() {
    check(
        "",
        "(...2..8)::Start",
        &expect![[r##"
            #0 0-16 "(...2..8)::Start" : ?0
            #1 1-8 "...2..8" : RangeTo
            #2 4-5 "2" : Int
            #3 7-8 "8" : Int
            Error(Type(Error(MissingClass(HasField { record: Prim(RangeTo), name: "Start", item: Infer(InferId(0)) }, Span { lo: 0, hi: 16 }))))
        "##]],
    );
}

#[test]
fn range_to_field_step() {
    check(
        "",
        "(...2..8)::Step",
        &expect![[r##"
            #0 0-15 "(...2..8)::Step" : Int
            #1 1-8 "...2..8" : RangeTo
            #2 4-5 "2" : Int
            #3 7-8 "8" : Int
        "##]],
    );
}

#[test]
fn range_to_field_end() {
    check(
        "",
        "(...2..8)::End",
        &expect![[r##"
            #0 0-14 "(...2..8)::End" : Int
            #1 1-8 "...2..8" : RangeTo
            #2 4-5 "2" : Int
            #3 7-8 "8" : Int
        "##]],
    );
}

#[test]
fn range_from_field_start() {
    check(
        "",
        "(0..2...)::Start",
        &expect![[r##"
            #0 0-16 "(0..2...)::Start" : Int
            #1 1-8 "0..2..." : RangeFrom
            #2 1-2 "0" : Int
            #3 4-5 "2" : Int
        "##]],
    );
}

#[test]
fn range_from_field_step() {
    check(
        "",
        "(0..2...)::Step",
        &expect![[r##"
            #0 0-15 "(0..2...)::Step" : Int
            #1 1-8 "0..2..." : RangeFrom
            #2 1-2 "0" : Int
            #3 4-5 "2" : Int
        "##]],
    );
}

#[test]
fn range_from_field_end() {
    check(
        "",
        "(0..2...)::End",
        &expect![[r##"
            #0 0-14 "(0..2...)::End" : ?0
            #1 1-8 "0..2..." : RangeFrom
            #2 1-2 "0" : Int
            #3 4-5 "2" : Int
            Error(Type(Error(MissingClass(HasField { record: Prim(RangeFrom), name: "End", item: Infer(InferId(0)) }, Span { lo: 0, hi: 14 }))))
        "##]],
    );
}

#[test]
fn range_full_field_start() {
    check(
        "",
        "...::Start",
        &expect![[r##"
            #0 0-10 "...::Start" : ?0
            #1 0-3 "..." : RangeFull
            Error(Type(Error(MissingClass(HasField { record: Prim(RangeFull), name: "Start", item: Infer(InferId(0)) }, Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn range_full_implicit_step() {
    check(
        "",
        "...::Step",
        &expect![[r##"
            #0 0-9 "...::Step" : Int
            #1 0-3 "..." : RangeFull
        "##]],
    );
}

#[test]
fn range_full_explicit_step() {
    check(
        "",
        "(...2...)::Step",
        &expect![[r##"
            #0 0-15 "(...2...)::Step" : Int
            #1 1-8 "...2..." : RangeFull
            #2 4-5 "2" : Int
        "##]],
    );
}

#[test]
fn range_full_field_end() {
    check(
        "",
        "...::End",
        &expect![[r##"
            #0 0-8 "...::End" : ?0
            #1 0-3 "..." : RangeFull
            Error(Type(Error(MissingClass(HasField { record: Prim(RangeFull), name: "End", item: Infer(InferId(0)) }, Span { lo: 0, hi: 8 }))))
        "##]],
    );
}

#[test]
fn newtype_cons() {
    check(
        indoc! {"
            namespace A {
                newtype NewInt = Int;
                function Foo() : NewInt { NewInt(5) }
            }
        "},
        "",
        &expect![[r##"
            #4 56-58 "()" : ()
            #5 68-81 "{ NewInt(5) }" : UDT<Item 1>
            #7 70-79 "NewInt(5)" : UDT<Item 1>
            #8 70-76 "NewInt" : (Int -> UDT<Item 1>)
            #9 77-78 "5" : Int
            Error(Validate(NotCurrentlySupported("newtype", Span { lo: 18, hi: 39 })))
        "##]],
    );
}

#[test]
fn newtype_cons_wrong_input() {
    check(
        indoc! {"
            namespace A {
                newtype NewInt = Int;
                function Foo() : NewInt { NewInt(5.0) }
            }
        "},
        "",
        &expect![[r##"
            #4 56-58 "()" : ()
            #5 68-83 "{ NewInt(5.0) }" : UDT<Item 1>
            #7 70-81 "NewInt(5.0)" : UDT<Item 1>
            #8 70-76 "NewInt" : (Int -> UDT<Item 1>)
            #9 77-80 "5.0" : Double
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Double), Span { lo: 70, hi: 81 }))))
            Error(Validate(NotCurrentlySupported("newtype", Span { lo: 18, hi: 39 })))
        "##]],
    );
}

#[test]
fn newtype_does_not_match_base_ty() {
    check(
        indoc! {"
            namespace A {
                newtype NewInt = Int;
                function Foo() : Int { NewInt(5) }
            }
        "},
        "",
        &expect![[r##"
            #4 56-58 "()" : ()
            #5 65-78 "{ NewInt(5) }" : Int
            #7 67-76 "NewInt(5)" : Int
            #8 67-73 "NewInt" : (Int -> UDT<Item 1>)
            #9 74-75 "5" : Int
            Error(Type(Error(TypeMismatch(Udt(Item(ItemId { package: None, item: LocalItemId(1) })), Prim(Int), Span { lo: 67, hi: 76 }))))
            Error(Validate(NotCurrentlySupported("newtype", Span { lo: 18, hi: 39 })))
        "##]],
    );
}

#[test]
fn newtype_does_not_match_other_newtype() {
    check(
        indoc! {"
            namespace A {
                newtype NewInt1 = Int;
                newtype NewInt2 = Int;
                function Foo() : NewInt2 { NewInt1(5) }
            }
        "},
        "",
        &expect![[r##"
            #6 84-86 "()" : ()
            #7 97-111 "{ NewInt1(5) }" : UDT<Item 2>
            #9 99-109 "NewInt1(5)" : UDT<Item 2>
            #10 99-106 "NewInt1" : (Int -> UDT<Item 1>)
            #11 107-108 "5" : Int
            Error(Type(Error(TypeMismatch(Udt(Item(ItemId { package: None, item: LocalItemId(1) })), Udt(Item(ItemId { package: None, item: LocalItemId(2) })), Span { lo: 99, hi: 109 }))))
            Error(Validate(NotCurrentlySupported("newtype", Span { lo: 18, hi: 40 })))
            Error(Validate(NotCurrentlySupported("newtype", Span { lo: 45, hi: 67 })))
        "##]],
    );
}

#[test]
fn unknown_name_fits_any_ty() {
    check(
        "",
        "{ let x : Int = foo; let y : Qubit = foo; }",
        &expect![[r##"
            #0 0-43 "{ let x : Int = foo; let y : Qubit = foo; }" : ()
            #1 0-43 "{ let x : Int = foo; let y : Qubit = foo; }" : ()
            #3 6-13 "x : Int" : Int
            #5 16-19 "foo" : ?
            #7 25-34 "y : Qubit" : Qubit
            #9 37-40 "foo" : ?
            Error(Resolve(NotFound("foo", Span { lo: 16, hi: 19 })))
            Error(Resolve(NotFound("foo", Span { lo: 37, hi: 40 })))
        "##]],
    );
}

#[test]
fn unknown_name_has_any_class() {
    check(
        "",
        "{ foo(); foo + 1 }",
        &expect![[r##"
            #0 0-18 "{ foo(); foo + 1 }" : ?
            #1 0-18 "{ foo(); foo + 1 }" : ?
            #3 2-7 "foo()" : ?0
            #4 2-5 "foo" : ?
            #5 5-7 "()" : ()
            #7 9-16 "foo + 1" : ?
            #8 9-12 "foo" : ?
            #9 15-16 "1" : Int
            Error(Resolve(NotFound("foo", Span { lo: 2, hi: 5 })))
            Error(Resolve(NotFound("foo", Span { lo: 9, hi: 12 })))
        "##]],
    );
}

#[test]
fn local_function() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    function Bar() : Int { 2 }
                    Bar() + 1
                }
            }
        "},
        "",
        &expect![[r##"
            #2 30-32 "()" : ()
            #3 39-99 "{\n        function Bar() : Int { 2 }\n        Bar() + 1\n    }" : Int
            #12 84-93 "Bar() + 1" : Int
            #13 84-89 "Bar()" : Int
            #14 84-87 "Bar" : (() -> Int)
            #15 87-89 "()" : ()
            #16 92-93 "1" : Int
            #7 61-63 "()" : ()
            #8 70-75 "{ 2 }" : Int
            #10 72-73 "2" : Int
        "##]],
    );
}

#[test]
fn local_function_error() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    function Bar() : Int { 2.0 }
                    Bar()
                }
            }
        "},
        "",
        &expect![[r##"
            #2 30-32 "()" : ()
            #3 39-97 "{\n        function Bar() : Int { 2.0 }\n        Bar()\n    }" : Int
            #12 86-91 "Bar()" : Int
            #13 86-89 "Bar" : (() -> Int)
            #14 89-91 "()" : ()
            #7 61-63 "()" : ()
            #8 70-77 "{ 2.0 }" : Double
            #10 72-75 "2.0" : Double
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Double), Span { lo: 70, hi: 77 }))))
        "##]],
    );
}

#[test]
fn local_function_use_before_declare() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    Bar();
                    function Bar() : () {}
                }
            }
        "},
        "",
        &expect![[r##"
            #2 30-32 "()" : ()
            #3 38-91 "{\n        Bar();\n        function Bar() : () {}\n    }" : ()
            #5 48-53 "Bar()" : ()
            #6 48-51 "Bar" : (() -> ())
            #7 51-53 "()" : ()
            #11 75-77 "()" : ()
            #12 83-85 "{}" : ()
        "##]],
    );
}

#[test]
fn local_function_last_stmt_is_unit_block() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    Bar()
                    function Bar() : Int { 4 }
                }
            }
        "},
        "",
        &expect![[r##"
            #2 30-32 "()" : ()
            #3 39-95 "{\n        Bar()\n        function Bar() : Int { 4 }\n    }" : ()
            #5 49-54 "Bar()" : Int
            #6 49-52 "Bar" : (() -> Int)
            #7 52-54 "()" : ()
            #11 75-77 "()" : ()
            #12 84-89 "{ 4 }" : Int
            #14 86-87 "4" : Int
            Error(Type(Error(TypeMismatch(Prim(Int), Tuple([]), Span { lo: 39, hi: 95 }))))
        "##]],
    );
}

#[test]
fn local_type() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    newtype Bar = Int;
                    let x = Bar(5);
                }
            }
        "},
        "",
        &expect![[r##"
            #2 30-32 "()" : ()
            #3 38-96 "{\n        newtype Bar = Int;\n        let x = Bar(5);\n    }" : ()
            #8 79-80 "x" : UDT<Item 2>
            #10 83-89 "Bar(5)" : UDT<Item 2>
            #11 83-86 "Bar" : (Int -> UDT<Item 2>)
            #12 87-88 "5" : Int
            Error(Validate(NotCurrentlySupported("newtype", Span { lo: 48, hi: 66 })))
        "##]],
    );
}

#[test]
fn local_open() {
    check(
        indoc! {"
            namespace A { function Foo() : () { open B; Bar(); } }
            namespace B { function Bar() : () {} }
        "},
        "",
        &expect![[r##"
            #2 26-28 "()" : ()
            #3 34-52 "{ open B; Bar(); }" : ()
            #6 44-49 "Bar()" : ()
            #7 44-47 "Bar" : (() -> ())
            #8 47-49 "()" : ()
            #12 81-83 "()" : ()
            #13 89-91 "{}" : ()
        "##]],
    );
}
