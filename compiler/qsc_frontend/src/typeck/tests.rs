// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Ty;
use crate::compile::{self, compile, PackageStore};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    ast::{
        self, Attr, Block, CallableDecl, Expr, FunctorExpr, Ident, Item, Namespace, NodeId,
        Package, Pat, Path, QubitInit, Span, SpecDecl, Stmt, TyDef,
    },
    visit::{self, Visitor},
};
use std::fmt::{self, Display, Write};
use std::{collections::HashMap, fmt::Formatter};

struct TypedNode<'a> {
    id: NodeId,
    span: Span,
    source: &'a str,
    ty: Ty,
}

impl Display for TypedNode<'_> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "#{} {}-{} {:?} : {}",
            self.id, self.span.lo, self.span.hi, self.source, self.ty
        )
    }
}

struct SpanCollector(HashMap<NodeId, Span>);

impl Visitor<'_> for SpanCollector {
    fn visit_package(&mut self, package: &Package) {
        visit::walk_package(self, package);
    }

    fn visit_namespace(&mut self, namespace: &Namespace) {
        self.0.insert(namespace.id, namespace.span);
        visit::walk_namespace(self, namespace);
    }

    fn visit_item(&mut self, item: &Item) {
        self.0.insert(item.id, item.span);
        visit::walk_item(self, item);
    }

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

    fn visit_ty(&mut self, ty: &ast::Ty) {
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

    fn visit_path(&mut self, path: &Path) {
        self.0.insert(path.id, path.span);
        visit::walk_path(self, path);
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

    let mut nodes: Vec<_> = unit
        .context
        .tys()
        .iter()
        .map(|(id, ty)| {
            let span = spans.0.get(id).expect("node should have span");
            let excerpt = if span.lo < source.len() {
                &source[span]
            } else {
                let span = Span {
                    lo: span.lo - source.len(),
                    hi: span.hi - source.len(),
                };
                &entry_expr[span]
            };

            TypedNode {
                id: *id,
                span: *span,
                source: excerpt,
                ty: ty.clone(),
            }
        })
        .collect();
    nodes.sort_by_key(|node| node.id);

    let mut actual = String::new();
    for node in nodes {
        writeln!(actual, "{node}").expect("writing node to string should succeed");
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
            #8 40-42 "{}" : ()
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
            #8 39-44 "{ 4 }" : Int
            #9 41-42 "4" : Int
            #10 41-42 "4" : Int
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
            #8 39-47 "{ true }" : Bool
            #9 41-45 "true" : Bool
            #10 41-45 "true" : Bool
            Error(Ty(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 39, hi: 47 })))
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
            #8 39-45 "{ 4; }" : ()
            #9 41-43 "4;" : ()
            #10 41-42 "4" : Int
            Error(Ty(TypeMismatch(Prim(Int), Tuple([]), Span { lo: 39, hi: 45 })))
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
            #8 39-75 "{\n        let x = 4;\n        x\n    }" : Int
            #9 49-59 "let x = 4;" : ()
            #10 53-54 "x" : Int
            #11 53-54 "x" : Int
            #12 57-58 "4" : Int
            #13 68-69 "x" : Int
            #14 68-69 "x" : Int
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
            #8 31-32 "x" : Int
            #11 46-51 "{ x }" : Int
            #12 48-49 "x" : Int
            #13 48-49 "x" : Int
            #19 68-70 "()" : ()
            #21 77-87 "{ Foo(4) }" : Int
            #22 79-85 "Foo(4)" : Int
            #23 79-85 "Foo(4)" : Int
            #24 79-82 "Foo" : (Int) -> (Int)
            #27 82-85 "(4)" : Int
            #28 83-84 "4" : Int
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
            #9 40-41 "x" : 'T
            #14 53-58 "{ x }" : 'T
            #15 55-56 "x" : 'T
            #16 55-56 "x" : 'T
            #22 75-77 "()" : ()
            #24 84-99 "{ Identity(4) }" : Int
            #25 86-97 "Identity(4)" : Int
            #26 86-97 "Identity(4)" : Int
            #27 86-94 "Identity" : (Int) -> (Int)
            #30 94-97 "(4)" : Int
            #31 95-96 "4" : Int
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
            #2 0-6 "Length" : ((Bool)[]) -> (Int)
            #5 6-27 "([true, false, true])" : (Bool)[]
            #6 7-26 "[true, false, true]" : (Bool)[]
            #7 8-12 "true" : Bool
            #8 14-19 "false" : Bool
            #9 21-25 "true" : Bool
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
            #8 40-52 "{ 1 + [2]; }" : ()
            #9 42-50 "1 + [2];" : ()
            #10 42-49 "1 + [2]" : Int
            #11 42-43 "1" : Int
            #12 46-49 "[2]" : Array<Int>
            #13 47-48 "2" : Int
            Error(Ty(TypeMismatch(Prim(Int), App(Prim(Array), [Prim(Int)]), Span { lo: 42, hi: 49 })))
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
            #2 0-37 "Microsoft.Quantum.Convert.IntAsDouble" : (Int) -> (Double)
            #6 37-44 "(false)" : Bool
            #7 38-43 "false" : Bool
            Error(Ty(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 0, hi: 44 })))
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
            #2 0-6 "Length" : ((?0)[]) -> (Int)
            #5 6-17 "((1, 2, 3))" : (Int, Int, Int)
            #6 7-16 "(1, 2, 3)" : (Int, Int, Int)
            #7 8-9 "1" : Int
            #8 11-12 "2" : Int
            #9 14-15 "3" : Int
            Error(Ty(TypeMismatch(App(Prim(Array), [Var(Var(0))]), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 17 })))
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
            #3 6-22 "use q = Qubit();" : ()
            #4 10-11 "q" : Qubit
            #5 10-11 "q" : Qubit
            #6 14-21 "Qubit()" : Qubit
            #7 27-33 "Ry(q);" : ()
            #8 27-32 "Ry(q)" : ()
            #9 27-29 "Ry" : ((Double, Qubit)) => (()) is Adj + Ctl
            #12 29-32 "(q)" : Qubit
            #13 30-31 "q" : Qubit
            Error(Ty(TypeMismatch(Tuple([Prim(Double), Prim(Qubit)]), Prim(Qubit), Span { lo: 27, hi: 32 })))
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
            Error(Ty(MissingClass(HasIndex { container: App(Prim(Array), [Prim(Int)]), index: Prim(Bool), item: Var(Var(0)) }, Span { lo: 0, hi: 16 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 11, hi: 15 })))
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
            #3 6-24 "mutable x = false;" : ()
            #4 14-15 "x" : Bool
            #5 14-15 "x" : Bool
            #6 18-23 "false" : Bool
            #7 29-40 "set x += 1;" : ()
            #8 29-39 "set x += 1" : ()
            #9 33-34 "x" : Bool
            #12 38-39 "1" : Int
            #13 45-46 "x" : Bool
            #14 45-46 "x" : Bool
            Error(Ty(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 29, hi: 39 })))
            Error(Ty(MissingClass(Add(Prim(Bool)), Span { lo: 33, hi: 34 })))
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
            Error(Ty(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Prim(Double), Span { lo: 0, hi: 12 })))
            Error(Ty(MissingClass(Add(Tuple([Prim(Int), Prim(Int)])), Span { lo: 0, hi: 6 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(Double), Span { lo: 0, hi: 7 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 })))
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
            #8 41-43 "{}" : ()
            #12 58-60 "()" : ()
            #14 68-70 "{}" : ()
            #15 73-89 "Test.A == Test.B" : Bool
            #16 73-79 "Test.A" : (()) -> (())
            #20 83-89 "Test.B" : (()) -> (())
            Error(Ty(MissingClass(Eq(Arrow(Function, Tuple([]), Tuple([]), None)), Span { lo: 73, hi: 79 })))
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
            Error(Ty(TypeMismatch(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 25 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(Result), Span { lo: 0, hi: 25 })))
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
            Error(Ty(TypeMismatch(Prim(BigInt), Prim(Int), Span { lo: 0, hi: 9 })))
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
            Error(Ty(TypeMismatch(Prim(BigInt), Prim(Int), Span { lo: 0, hi: 9 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(Result), Span { lo: 0, hi: 25 })))
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
            Error(Ty(TypeMismatch(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 25 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 })))
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
            Error(Ty(TypeMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 })))
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
            #3 2-25 "let (x, y, z) = (0, 1);" : ()
            #4 6-15 "(x, y, z)" : (?0, ?1, ?2)
            #5 7-8 "x" : ?0
            #6 7-8 "x" : ?0
            #7 10-11 "y" : ?1
            #8 10-11 "y" : ?1
            #9 13-14 "z" : ?2
            #10 13-14 "z" : ?2
            #11 18-24 "(0, 1)" : (Int, Int)
            #12 19-20 "0" : Int
            #13 22-23 "1" : Int
            Error(Ty(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Var(Var(0)), Var(Var(1)), Var(Var(2))]), Span { lo: 6, hi: 15 })))
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
            #3 6-30 "mutable (x, y) = (0, 1);" : ()
            #4 14-20 "(x, y)" : (Int, Int)
            #5 15-16 "x" : Int
            #6 15-16 "x" : Int
            #7 18-19 "y" : Int
            #8 18-19 "y" : Int
            #9 23-29 "(0, 1)" : (Int, Int)
            #10 24-25 "0" : Int
            #11 27-28 "1" : Int
            #12 35-58 "set (x, y) = (1, 2, 3);" : ()
            #13 35-57 "set (x, y) = (1, 2, 3)" : ()
            #14 39-45 "(x, y)" : (Int, Int)
            #15 40-41 "x" : Int
            #18 43-44 "y" : Int
            #21 48-57 "(1, 2, 3)" : (Int, Int, Int)
            #22 49-50 "1" : Int
            #23 52-53 "2" : Int
            #24 55-56 "3" : Int
            #25 63-64 "x" : Int
            #26 63-64 "x" : Int
            Error(Ty(TypeMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 39, hi: 45 })))
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
            #3 2-23 "use q = Qubit[false];" : ()
            #4 6-7 "q" : (Qubit)[]
            #5 6-7 "q" : (Qubit)[]
            #6 10-22 "Qubit[false]" : (Qubit)[]
            #7 16-21 "false" : Bool
            Error(Ty(TypeMismatch(Prim(Int), Prim(Bool), Span { lo: 16, hi: 21 })))
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
            #3 2-45 "use (q, q1) = (Qubit[3], Qubit(), Qubit());" : ()
            #4 6-13 "(q, q1)" : (?0, ?1)
            #5 7-8 "q" : ?0
            #6 7-8 "q" : ?0
            #7 10-12 "q1" : ?1
            #8 10-12 "q1" : ?1
            #9 16-44 "(Qubit[3], Qubit(), Qubit())" : ((Qubit)[], Qubit, Qubit)
            #10 17-25 "Qubit[3]" : (Qubit)[]
            #11 23-24 "3" : Int
            #12 27-34 "Qubit()" : Qubit
            #13 36-43 "Qubit()" : Qubit
            Error(Ty(TypeMismatch(Tuple([App(Prim(Array), [Prim(Qubit)]), Prim(Qubit), Prim(Qubit)]), Tuple([Var(Var(0)), Var(Var(1))]), Span { lo: 6, hi: 13 })))
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
            #3 4-5 "i" : ?0
            #4 9-23 "(1, true, One)" : (Int, Bool, Result)
            #5 10-11 "1" : Int
            #6 13-17 "true" : Bool
            #7 19-22 "One" : Result
            #8 24-26 "{}" : ()
            Error(Ty(MissingClass(Iterable { container: Tuple([Prim(Int), Prim(Bool), Prim(Result)]), item: Var(Var(0)) }, Span { lo: 9, hi: 23 })))
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
            Error(Ty(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 3, hi: 4 })))
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
            #4 10-11 "4" : Int
            #5 10-11 "4" : Int
            Error(Ty(TypeMismatch(Prim(Int), Tuple([]), Span { lo: 0, hi: 13 })))
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
            Error(Ty(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 0, hi: 1 })))
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
            Error(Ty(MissingClass(HasIndex { container: Tuple([Prim(Int), Prim(Int), Prim(Int)]), index: Prim(Int), item: Prim(Int) }, Span { lo: 0, hi: 19 })))
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
            Error(Ty(MissingClass(HasIndex { container: App(Prim(Array), [Prim(Int)]), index: Prim(Bool), item: Prim(Int) }, Span { lo: 0, hi: 23 })))
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
            Error(Ty(MissingClass(Num(Prim(Bool)), Span { lo: 3, hi: 8 })))
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
            Error(Ty(TypeMismatch(Prim(Bool), Prim(Int), Span { lo: 4, hi: 5 })))
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
            Error(Ty(MissingClass(Num(Prim(Bool)), Span { lo: 1, hi: 6 })))
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
            Error(Ty(MissingClass(Num(Prim(Bool)), Span { lo: 1, hi: 6 })))
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
            Error(Ty(TypeMismatch(Prim(Bool), Prim(Result), Span { lo: 6, hi: 10 })))
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
            #8 32-33 "q" : Qubit
            #13 72-75 "..." : Qubit
            #14 76-78 "{}" : ()
            #16 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #17 99-101 "cs" : (Qubit)[]
            #18 99-101 "cs" : (Qubit)[]
            #19 103-106 "..." : Qubit
            #20 108-110 "{}" : ()
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
            #8 32-33 "q" : Qubit
            #13 72-75 "..." : Qubit
            #14 76-78 "{}" : ()
            #16 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #17 99-101 "cs" : (Qubit)[]
            #18 99-101 "cs" : (Qubit)[]
            #19 103-106 "..." : Qubit
            #20 108-110 "{}" : ()
            #21 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #22 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : ()
            #23 125-142 "use q1 = Qubit();" : ()
            #24 129-131 "q1" : Qubit
            #25 129-131 "q1" : Qubit
            #26 134-141 "Qubit()" : Qubit
            #27 147-164 "use q2 = Qubit();" : ()
            #28 151-153 "q2" : Qubit
            #29 151-153 "q2" : Qubit
            #30 156-163 "Qubit()" : Qubit
            #31 169-196 "Controlled A.Foo([q1], q2);" : ()
            #32 169-195 "Controlled A.Foo([q1], q2)" : ()
            #33 169-185 "Controlled A.Foo" : (((Qubit)[], Qubit)) => (()) is Ctl
            #34 180-185 "A.Foo" : (Qubit) => (()) is Ctl
            #38 185-195 "([q1], q2)" : ((Qubit)[], Qubit)
            #39 186-190 "[q1]" : (Qubit)[]
            #40 187-189 "q1" : Qubit
            #43 192-194 "q2" : Qubit
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
            #8 32-33 "q" : Qubit
            #13 72-75 "..." : Qubit
            #14 76-78 "{}" : ()
            #16 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #17 99-101 "cs" : (Qubit)[]
            #18 99-101 "cs" : (Qubit)[]
            #19 103-106 "..." : Qubit
            #20 108-110 "{}" : ()
            #21 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #22 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : ()
            #23 125-142 "use q1 = Qubit();" : ()
            #24 129-131 "q1" : Qubit
            #25 129-131 "q1" : Qubit
            #26 134-141 "Qubit()" : Qubit
            #27 147-164 "use q2 = Qubit();" : ()
            #28 151-153 "q2" : Qubit
            #29 151-153 "q2" : Qubit
            #30 156-163 "Qubit()" : Qubit
            #31 169-186 "use q3 = Qubit();" : ()
            #32 173-175 "q3" : Qubit
            #33 173-175 "q3" : Qubit
            #34 178-185 "Qubit()" : Qubit
            #35 191-237 "Controlled Controlled A.Foo([q1], ([q2], q3));" : ()
            #36 191-236 "Controlled Controlled A.Foo([q1], ([q2], q3))" : ()
            #37 191-218 "Controlled Controlled A.Foo" : (((Qubit)[], ((Qubit)[], Qubit))) => (()) is Ctl
            #38 202-218 "Controlled A.Foo" : (((Qubit)[], Qubit)) => (()) is Ctl
            #39 213-218 "A.Foo" : (Qubit) => (()) is Ctl
            #43 218-236 "([q1], ([q2], q3))" : ((Qubit)[], ((Qubit)[], Qubit))
            #44 219-223 "[q1]" : (Qubit)[]
            #45 220-222 "q1" : Qubit
            #48 225-235 "([q2], q3)" : ((Qubit)[], Qubit)
            #49 226-230 "[q2]" : (Qubit)[]
            #50 227-229 "q2" : Qubit
            #53 232-234 "q3" : Qubit
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
            #8 32-33 "q" : Qubit
            #13 72-75 "..." : Qubit
            #14 76-78 "{}" : ()
            #16 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #17 99-101 "cs" : (Qubit)[]
            #18 99-101 "cs" : (Qubit)[]
            #19 103-106 "..." : Qubit
            #20 108-110 "{}" : ()
            #21 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #22 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : ()
            #23 125-141 "use q = Qubit();" : ()
            #24 129-130 "q" : Qubit
            #25 129-130 "q" : Qubit
            #26 133-140 "Qubit()" : Qubit
            #27 146-171 "Controlled A.Foo([1], q);" : ()
            #28 146-170 "Controlled A.Foo([1], q)" : ()
            #29 146-162 "Controlled A.Foo" : (((Qubit)[], Qubit)) => (()) is Ctl
            #30 157-162 "A.Foo" : (Qubit) => (()) is Ctl
            #34 162-170 "([1], q)" : ((Int)[], Qubit)
            #35 163-166 "[1]" : (Int)[]
            #36 164-165 "1" : Int
            #37 168-169 "q" : Qubit
            Error(Ty(TypeMismatch(Prim(Qubit), Prim(Int), Span { lo: 157, hi: 162 })))
        "##]],
    );
}
