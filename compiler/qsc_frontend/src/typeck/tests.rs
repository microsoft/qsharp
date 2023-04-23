// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::compile::{self, compile, PackageStore};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_data_structures::span::Span;
use qsc_hir::{
    hir::{
        self, Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, NodeId, Pat, QubitInit,
        SpecDecl, Stmt, TyDef,
    },
    visit::{self, Visitor},
};
use std::{collections::HashMap, fmt::Write};

struct SpanCollector(HashMap<NodeId, Span>);

impl Visitor<'_> for SpanCollector {
    fn visit_attr(&mut self, attr: &Attr) {
        self.0.insert(attr.id, attr.span);
        visit::walk_attr(self, attr);
    }

    fn visit_ty_def(&mut self, def: &TyDef) {
        self.0.insert(def.id, def.span);
        visit::walk_ty_def(self, def);
    }

    fn visit_callable_decl(&mut self, decl: &CallableDecl) {
        self.0.insert(decl.id, decl.span);
        visit::walk_callable_decl(self, decl);
    }

    fn visit_spec_decl(&mut self, decl: &SpecDecl) {
        self.0.insert(decl.id, decl.span);
        visit::walk_spec_decl(self, decl);
    }

    fn visit_functor_expr(&mut self, expr: &FunctorExpr) {
        self.0.insert(expr.id, expr.span);
        visit::walk_functor_expr(self, expr);
    }

    fn visit_ty(&mut self, ty: &hir::Ty) {
        self.0.insert(ty.id, ty.span);
        visit::walk_ty(self, ty);
    }

    fn visit_block(&mut self, block: &Block) {
        self.0.insert(block.id, block.span);
        visit::walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        self.0.insert(stmt.id, stmt.span);
        visit::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        self.0.insert(expr.id, expr.span);
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &Pat) {
        self.0.insert(pat.id, pat.span);
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &QubitInit) {
        self.0.insert(init.id, init.span);
        visit::walk_qubit_init(self, init);
    }

    fn visit_ident(&mut self, ident: &Ident) {
        self.0.insert(ident.id, ident.span);
    }
}

fn check(source: &str, entry_expr: &str, expect: &Expect) {
    let mut store = PackageStore::new();
    let std = store.insert(compile::std());
    let unit = compile(&store, [std], [source], entry_expr);
    let mut spans = SpanCollector(HashMap::new());
    spans.visit_package(&unit.package);

    let mut actual = String::new();
    for (id, ty) in unit.context.tys() {
        let span = spans.0.get(&id).expect("node should have span");
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
            #1 27-30 "Foo" : (()) -> (())
            #2 30-32 "()" : ()
            #4 40-42 "{}" : ()
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
            #1 27-30 "Foo" : (()) -> (Int)
            #2 30-32 "()" : ()
            #4 39-44 "{ 4 }" : Int
            #5 41-42 "4" : Int
            #6 41-42 "4" : Int
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
            #1 27-30 "Foo" : (()) -> (Int)
            #2 30-32 "()" : ()
            #4 39-47 "{ true }" : Bool
            #5 41-45 "true" : Bool
            #6 41-45 "true" : Bool
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
            #1 27-30 "Foo" : (()) -> (Int)
            #2 30-32 "()" : ()
            #4 39-45 "{ 4; }" : ()
            #5 41-43 "4;" : ()
            #6 41-42 "4" : Int
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
            #1 27-30 "Foo" : (()) -> (Int)
            #2 30-32 "()" : ()
            #4 39-75 "{\n        let x = 4;\n        x\n    }" : Int
            #5 49-59 "let x = 4;" : ()
            #6 53-54 "x" : Int
            #7 53-54 "x" : Int
            #8 57-58 "4" : Int
            #9 68-69 "x" : Int
            #10 68-69 "x" : Int
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
            #1 27-30 "Foo" : (Int) -> (Int)
            #2 30-39 "(x : Int)" : Int
            #3 31-38 "x : Int" : Int
            #4 31-32 "x" : Int
            #7 46-51 "{ x }" : Int
            #8 48-49 "x" : Int
            #9 48-49 "x" : Int
            #11 65-68 "Bar" : (()) -> (Int)
            #12 68-70 "()" : ()
            #14 77-87 "{ Foo(4) }" : Int
            #15 79-85 "Foo(4)" : Int
            #16 79-85 "Foo(4)" : Int
            #17 79-82 "Foo" : (Int) -> (Int)
            #18 82-85 "(4)" : Int
            #19 83-84 "4" : Int
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
            #1 27-35 "Identity" : ('T) -> ('T)
            #3 39-47 "(x : 'T)" : 'T
            #4 40-46 "x : 'T" : 'T
            #5 40-41 "x" : 'T
            #10 53-58 "{ x }" : 'T
            #11 55-56 "x" : 'T
            #12 55-56 "x" : 'T
            #14 72-75 "Foo" : (()) -> (Int)
            #15 75-77 "()" : ()
            #17 84-99 "{ Identity(4) }" : Int
            #18 86-97 "Identity(4)" : Int
            #19 86-97 "Identity(4)" : Int
            #20 86-94 "Identity" : (Int) -> (Int)
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
            #0 0-27 "Length([true, false, true])" : Int
            #1 0-6 "Length" : ((Bool)[]) -> (Int)
            #2 6-27 "([true, false, true])" : (Bool)[]
            #3 7-26 "[true, false, true]" : (Bool)[]
            #4 8-12 "true" : Bool
            #5 14-19 "false" : Bool
            #6 21-25 "true" : Bool
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
            #1 27-30 "Foo" : (()) -> (())
            #2 30-32 "()" : ()
            #4 40-52 "{ 1 + [2]; }" : ()
            #5 42-50 "1 + [2];" : ()
            #6 42-49 "1 + [2]" : Int
            #7 42-43 "1" : Int
            #8 46-49 "[2]" : (Int)[]
            #9 47-48 "2" : Int
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
            #1 0-37 "Microsoft.Quantum.Convert.IntAsDouble" : (Int) -> (Double)
            #2 37-44 "(false)" : Bool
            #3 38-43 "false" : Bool
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
            #1 0-6 "Length" : ((?0)[]) -> (Int)
            #2 6-17 "((1, 2, 3))" : (Int, Int, Int)
            #3 7-16 "(1, 2, 3)" : (Int, Int, Int)
            #4 8-9 "1" : Int
            #5 11-12 "2" : Int
            #6 14-15 "3" : Int
            Error(Type(Error(TypeMismatch(Array(Var(Var(0))), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 17 }))))
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
            #2 6-22 "use q = Qubit();" : ()
            #3 10-11 "q" : Qubit
            #4 10-11 "q" : Qubit
            #5 14-21 "Qubit()" : Qubit
            #6 27-33 "Ry(q);" : ()
            #7 27-32 "Ry(q)" : ()
            #8 27-29 "Ry" : ((Double, Qubit)) => (()) is Adj + Ctl
            #9 29-32 "(q)" : Qubit
            #10 30-31 "q" : Qubit
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
            Error(Type(Error(MissingClass(HasIndex { container: Array(Prim(Int)), index: Prim(Bool), item: Var(Var(0)) }, Span { lo: 0, hi: 16 }))))
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
            #2 6-24 "mutable x = false;" : ()
            #3 14-15 "x" : Bool
            #4 14-15 "x" : Bool
            #5 18-23 "false" : Bool
            #6 29-40 "set x += 1;" : ()
            #7 29-39 "set x += 1" : ()
            #8 33-34 "x" : Bool
            #9 38-39 "1" : Int
            #10 45-46 "x" : Bool
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
            #1 30-31 "A" : (()) -> (())
            #2 31-33 "()" : ()
            #4 41-43 "{}" : ()
            #6 57-58 "B" : (()) -> (())
            #7 58-60 "()" : ()
            #9 68-70 "{}" : ()
            #11 73-89 "Test.A == Test.B" : Bool
            #12 73-79 "Test.A" : (()) -> (())
            #13 83-89 "Test.B" : (()) -> (())
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
            #2 2-25 "let (x, y, z) = (0, 1);" : ()
            #3 6-15 "(x, y, z)" : (?0, ?1, ?2)
            #4 7-8 "x" : ?0
            #5 7-8 "x" : ?0
            #6 10-11 "y" : ?1
            #7 10-11 "y" : ?1
            #8 13-14 "z" : ?2
            #9 13-14 "z" : ?2
            #10 18-24 "(0, 1)" : (Int, Int)
            #11 19-20 "0" : Int
            #12 22-23 "1" : Int
            Error(Type(Error(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Var(Var(0)), Var(Var(1)), Var(Var(2))]), Span { lo: 6, hi: 15 }))))
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
            #2 6-30 "mutable (x, y) = (0, 1);" : ()
            #3 14-20 "(x, y)" : (Int, Int)
            #4 15-16 "x" : Int
            #5 15-16 "x" : Int
            #6 18-19 "y" : Int
            #7 18-19 "y" : Int
            #8 23-29 "(0, 1)" : (Int, Int)
            #9 24-25 "0" : Int
            #10 27-28 "1" : Int
            #11 35-58 "set (x, y) = (1, 2, 3);" : ()
            #12 35-57 "set (x, y) = (1, 2, 3)" : ()
            #13 39-45 "(x, y)" : (Int, Int)
            #14 40-41 "x" : Int
            #15 43-44 "y" : Int
            #16 48-57 "(1, 2, 3)" : (Int, Int, Int)
            #17 49-50 "1" : Int
            #18 52-53 "2" : Int
            #19 55-56 "3" : Int
            #20 63-64 "x" : Int
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
            #2 2-23 "use q = Qubit[false];" : ()
            #3 6-7 "q" : (Qubit)[]
            #4 6-7 "q" : (Qubit)[]
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
            #2 2-45 "use (q, q1) = (Qubit[3], Qubit(), Qubit());" : ()
            #3 6-13 "(q, q1)" : (?0, ?1)
            #4 7-8 "q" : ?0
            #5 7-8 "q" : ?0
            #6 10-12 "q1" : ?1
            #7 10-12 "q1" : ?1
            #8 16-44 "(Qubit[3], Qubit(), Qubit())" : ((Qubit)[], Qubit, Qubit)
            #9 17-25 "Qubit[3]" : (Qubit)[]
            #10 23-24 "3" : Int
            #11 27-34 "Qubit()" : Qubit
            #12 36-43 "Qubit()" : Qubit
            Error(Type(Error(TypeMismatch(Tuple([Array(Prim(Qubit)), Prim(Qubit), Prim(Qubit)]), Tuple([Var(Var(0)), Var(Var(1))]), Span { lo: 6, hi: 13 }))))
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
            #2 4-5 "i" : ?0
            #3 9-23 "(1, true, One)" : (Int, Bool, Result)
            #4 10-11 "1" : Int
            #5 13-17 "true" : Bool
            #6 19-22 "One" : Result
            #7 24-26 "{}" : ()
            Error(Type(Error(MissingClass(Iterable { container: Tuple([Prim(Int), Prim(Bool), Prim(Result)]), item: Var(Var(0)) }, Span { lo: 9, hi: 23 }))))
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
            #3 10-11 "4" : Int
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
            #5 19-32 "fail \"error\";" : ()
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
            #1 27-28 "F" : (()) -> (Int)
            #2 28-30 "()" : ()
            #4 37-154 "{\n        if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }\n    }" : Int
            #5 47-148 "if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }" : Int
            #6 47-148 "if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }" : Int
            #7 50-62 "fail \"error\"" : Bool
            #8 55-62 "\"error\"" : String
            #9 63-113 "{\n            \"this type doesn't matter\"\n        }" : String
            #10 77-103 "\"this type doesn't matter\"" : String
            #11 77-103 "\"this type doesn't matter\"" : String
            #12 114-148 "else {\n            \"foo\"\n        }" : String
            #13 119-148 "{\n            \"foo\"\n        }" : String
            #14 133-138 "\"foo\"" : String
            #15 133-138 "\"foo\"" : String
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
            #1 27-28 "F" : (()) -> (Int)
            #2 28-30 "()" : ()
            #4 37-145 "{\n        if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }\n    }" : Int
            #5 47-139 "if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }" : Int
            #6 47-139 "if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }" : Int
            #7 50-61 "fail \"cond\"" : Bool
            #8 55-61 "\"cond\"" : String
            #9 62-97 "{\n            fail \"true\"\n        }" : Int
            #10 76-87 "fail \"true\"" : Int
            #11 76-87 "fail \"true\"" : Int
            #12 81-87 "\"true\"" : String
            #13 98-139 "else {\n            fail \"false\"\n        }" : Int
            #14 103-139 "{\n            fail \"false\"\n        }" : Int
            #15 117-129 "fail \"false\"" : Int
            #16 117-129 "fail \"false\"" : Int
            #17 122-129 "\"false\"" : String
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
            #1 28-31 "Foo" : (Qubit) => (()) is Ctl
            #2 31-42 "(q : Qubit)" : Qubit
            #3 32-41 "q : Qubit" : Qubit
            #4 32-33 "q" : Qubit
            #9 72-75 "..." : Qubit
            #10 76-78 "{}" : ()
            #12 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #13 99-101 "cs" : (Qubit)[]
            #14 99-101 "cs" : (Qubit)[]
            #15 103-106 "..." : Qubit
            #16 108-110 "{}" : ()
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
            #1 28-31 "Foo" : (Qubit) => (()) is Ctl
            #2 31-42 "(q : Qubit)" : Qubit
            #3 32-41 "q : Qubit" : Qubit
            #4 32-33 "q" : Qubit
            #9 72-75 "..." : Qubit
            #10 76-78 "{}" : ()
            #12 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #13 99-101 "cs" : (Qubit)[]
            #14 99-101 "cs" : (Qubit)[]
            #15 103-106 "..." : Qubit
            #16 108-110 "{}" : ()
            #18 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #19 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #20 125-142 "use q1 = Qubit();" : ()
            #21 129-131 "q1" : Qubit
            #22 129-131 "q1" : Qubit
            #23 134-141 "Qubit()" : Qubit
            #24 147-164 "use q2 = Qubit();" : ()
            #25 151-153 "q2" : Qubit
            #26 151-153 "q2" : Qubit
            #27 156-163 "Qubit()" : Qubit
            #28 169-196 "Controlled A.Foo([q1], q2);" : ()
            #29 169-195 "Controlled A.Foo([q1], q2)" : ()
            #30 169-185 "Controlled A.Foo" : (((Qubit)[], Qubit)) => (()) is Ctl
            #31 180-185 "A.Foo" : (Qubit) => (()) is Ctl
            #32 185-195 "([q1], q2)" : ((Qubit)[], Qubit)
            #33 186-190 "[q1]" : (Qubit)[]
            #34 187-189 "q1" : Qubit
            #35 192-194 "q2" : Qubit
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
            #1 28-31 "Foo" : (Qubit) => (()) is Ctl
            #2 31-42 "(q : Qubit)" : Qubit
            #3 32-41 "q : Qubit" : Qubit
            #4 32-33 "q" : Qubit
            #9 72-75 "..." : Qubit
            #10 76-78 "{}" : ()
            #12 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #13 99-101 "cs" : (Qubit)[]
            #14 99-101 "cs" : (Qubit)[]
            #15 103-106 "..." : Qubit
            #16 108-110 "{}" : ()
            #18 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #19 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #20 125-142 "use q1 = Qubit();" : ()
            #21 129-131 "q1" : Qubit
            #22 129-131 "q1" : Qubit
            #23 134-141 "Qubit()" : Qubit
            #24 147-164 "use q2 = Qubit();" : ()
            #25 151-153 "q2" : Qubit
            #26 151-153 "q2" : Qubit
            #27 156-163 "Qubit()" : Qubit
            #28 169-186 "use q3 = Qubit();" : ()
            #29 173-175 "q3" : Qubit
            #30 173-175 "q3" : Qubit
            #31 178-185 "Qubit()" : Qubit
            #32 191-237 "Controlled Controlled A.Foo([q1], ([q2], q3));" : ()
            #33 191-236 "Controlled Controlled A.Foo([q1], ([q2], q3))" : ()
            #34 191-218 "Controlled Controlled A.Foo" : (((Qubit)[], ((Qubit)[], Qubit))) => (()) is Ctl
            #35 202-218 "Controlled A.Foo" : (((Qubit)[], Qubit)) => (()) is Ctl
            #36 213-218 "A.Foo" : (Qubit) => (()) is Ctl
            #37 218-236 "([q1], ([q2], q3))" : ((Qubit)[], ((Qubit)[], Qubit))
            #38 219-223 "[q1]" : (Qubit)[]
            #39 220-222 "q1" : Qubit
            #40 225-235 "([q2], q3)" : ((Qubit)[], Qubit)
            #41 226-230 "[q2]" : (Qubit)[]
            #42 227-229 "q2" : Qubit
            #43 232-234 "q3" : Qubit
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
            #1 28-31 "Foo" : (Qubit) => (()) is Ctl
            #2 31-42 "(q : Qubit)" : Qubit
            #3 32-41 "q : Qubit" : Qubit
            #4 32-33 "q" : Qubit
            #9 72-75 "..." : Qubit
            #10 76-78 "{}" : ()
            #12 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #13 99-101 "cs" : (Qubit)[]
            #14 99-101 "cs" : (Qubit)[]
            #15 103-106 "..." : Qubit
            #16 108-110 "{}" : ()
            #18 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #19 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #20 125-141 "use q = Qubit();" : ()
            #21 129-130 "q" : Qubit
            #22 129-130 "q" : Qubit
            #23 133-140 "Qubit()" : Qubit
            #24 146-171 "Controlled A.Foo([1], q);" : ()
            #25 146-170 "Controlled A.Foo([1], q)" : ()
            #26 146-162 "Controlled A.Foo" : (((Qubit)[], Qubit)) => (()) is Ctl
            #27 157-162 "A.Foo" : (Qubit) => (()) is Ctl
            #28 162-170 "([1], q)" : ((Int)[], Qubit)
            #29 163-166 "[1]" : (Int)[]
            #30 164-165 "1" : Int
            #31 168-169 "q" : Qubit
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
            #1 28-31 "Foo" : (()) => (Int) is Adj
            #2 31-33 "()" : ()
            #5 47-52 "{ 1 }" : Int
            #6 49-50 "1" : Int
            #7 49-50 "1" : Int
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
            #1 28-31 "Foo" : (()) => (Int) is Ctl
            #2 31-33 "()" : ()
            #5 47-52 "{ 1 }" : Int
            #6 49-50 "1" : Int
            #7 49-50 "1" : Int
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
            #1 28-31 "Foo" : (()) => (Int) is Adj + Ctl
            #2 31-33 "()" : ()
            #7 53-58 "{ 1 }" : Int
            #8 55-56 "1" : Int
            #9 55-56 "1" : Int
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
            #3 14-25 "fail \"true\"" : Int
            #4 14-25 "fail \"true\"" : Int
            #5 19-25 "\"true\"" : String
            #6 28-42 "else {\n    4\n}" : Int
            #7 33-42 "{\n    4\n}" : Int
            #8 39-40 "4" : Int
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
            #1 27-30 "Foo" : (Bool) -> (Int)
            #2 30-40 "(x : Bool)" : Bool
            #3 31-39 "x : Bool" : Bool
            #4 31-32 "x" : Bool
            #7 47-153 "{\n        let x = if x {\n            return 1\n        } else {\n            true\n        };\n        2\n    }" : Int
            #8 57-137 "let x = if x {\n            return 1\n        } else {\n            true\n        };" : ()
            #9 61-62 "x" : Bool
            #10 61-62 "x" : Bool
            #11 65-136 "if x {\n            return 1\n        } else {\n            true\n        }" : Bool
            #12 68-69 "x" : Bool
            #13 70-102 "{\n            return 1\n        }" : Bool
            #14 84-92 "return 1" : Bool
            #15 84-92 "return 1" : Bool
            #16 91-92 "1" : Int
            #17 103-136 "else {\n            true\n        }" : Bool
            #18 108-136 "{\n            true\n        }" : Bool
            #19 122-126 "true" : Bool
            #20 122-126 "true" : Bool
            #21 146-147 "2" : Int
            #22 146-147 "2" : Int
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
            #1 27-30 "Foo" : (Bool) -> (Int)
            #2 30-40 "(x : Bool)" : Bool
            #3 31-39 "x : Bool" : Bool
            #4 31-32 "x" : Bool
            #7 47-132 "{\n        let x = {\n            return 1;\n            true\n        };\n        x\n    }" : Int
            #8 57-116 "let x = {\n            return 1;\n            true\n        };" : ?4
            #9 61-62 "x" : ?0
            #10 61-62 "x" : ?0
            #11 65-115 "{\n            return 1;\n            true\n        }" : ?0
            #12 65-115 "{\n            return 1;\n            true\n        }" : ?0
            #13 79-88 "return 1;" : ?2
            #14 79-87 "return 1" : ?1
            #15 86-87 "1" : Int
            #16 101-105 "true" : Bool
            #17 101-105 "true" : Bool
            #18 125-126 "x" : ?0
            #19 125-126 "x" : ?0
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
            #1 27-30 "Foo" : (Bool) -> (Int)
            #2 30-40 "(x : Bool)" : Bool
            #3 31-39 "x : Bool" : Bool
            #4 31-32 "x" : Bool
            #7 47-75 "{\n        return true;\n    }" : Int
            #8 57-69 "return true;" : Int
            #9 57-68 "return true" : ?0
            #10 64-68 "true" : Bool
            Error(Type(Error(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 64, hi: 68 }))))
        "##]],
    );
}
