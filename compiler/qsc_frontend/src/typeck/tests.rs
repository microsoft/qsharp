// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    compile::{self, Offsetter},
    resolve::{self, Resolver},
    typeck::Checker,
};
use expect_test::{expect, Expect};
use indoc::indoc;
use qsc_ast::{
    assigner::Assigner as AstAssigner,
    ast::{Block, Expr, NodeId, Package, Pat, QubitInit},
    mut_visit::MutVisitor,
    visit::{self, Visitor},
};
use qsc_data_structures::{index_map::IndexMap, span::Span};
use qsc_hir::{assigner::Assigner as HirAssigner, ty::Ty};
use std::fmt::Write;

struct TyCollector<'a> {
    tys: &'a IndexMap<NodeId, Ty>,
    nodes: Vec<(NodeId, Span, Option<&'a Ty>)>,
}

impl<'a> Visitor<'a> for TyCollector<'a> {
    fn visit_block(&mut self, block: &'a Block) {
        let ty = self.tys.get(block.id);
        self.nodes.push((block.id, block.span, ty));
        visit::walk_block(self, block);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        let ty = self.tys.get(expr.id);
        self.nodes.push((expr.id, expr.span, ty));
        visit::walk_expr(self, expr);
    }

    fn visit_pat(&mut self, pat: &'a Pat) {
        let ty = self.tys.get(pat.id);
        self.nodes.push((pat.id, pat.span, ty));
        visit::walk_pat(self, pat);
    }

    fn visit_qubit_init(&mut self, init: &'a QubitInit) {
        let ty = self.tys.get(init.id);
        self.nodes.push((init.id, init.span, ty));
        visit::walk_qubit_init(self, init);
    }
}

fn check(input: &str, entry_expr: &str, expect: &Expect) {
    let (package, tys, errors) = compile(input, entry_expr);
    let mut collector = TyCollector {
        tys: &tys.terms,
        nodes: Vec::new(),
    };
    collector.visit_package(&package);
    let mut actual = String::new();

    for (id, span, ty) in collector.nodes {
        let source = if (span.lo as usize) < input.len() {
            &input[span.lo as usize..span.hi as usize]
        } else {
            &entry_expr[span.lo as usize - input.len()..span.hi as usize - input.len()]
        };
        let ty = ty.unwrap_or(&Ty::Err);

        writeln!(actual, "#{id} {}-{} {source:?} : {ty}", span.lo, span.hi)
            .expect("string should be writable");
    }

    for error in errors {
        writeln!(actual, "{error:?}").expect("writing error to string should succeed");
    }

    expect.assert_eq(&actual);
}

fn compile(input: &str, entry_expr: &str) -> (Package, super::Table, Vec<compile::Error>) {
    let mut package = parse(input, entry_expr);
    AstAssigner::new().visit_package(&mut package);
    let mut assigner = HirAssigner::new();

    let mut globals = resolve::GlobalTable::new();
    let mut errors = globals.add_local_package(&mut assigner, &package);
    let mut resolver = Resolver::new(globals);
    resolver.with(&mut assigner).visit_package(&package);
    let (names, mut resolve_errors) = resolver.into_names();
    errors.append(&mut resolve_errors);

    let mut checker = Checker::new(super::GlobalTable::new());
    checker.check_package(&names, &package);
    let (tys, ty_errors) = checker.into_table();

    let errors = errors
        .into_iter()
        .map(Into::into)
        .chain(ty_errors.into_iter().map(Into::into))
        .map(compile::Error)
        .collect();

    (package, tys, errors)
}

fn parse(input: &str, entry_expr: &str) -> Package {
    let (namespaces, errors) = qsc_parse::namespaces(input);
    assert!(errors.is_empty(), "parsing input failed: {errors:#?}");

    let entry = if entry_expr.is_empty() {
        None
    } else {
        let (mut entry, errors) = qsc_parse::expr(entry_expr);
        let offset = input
            .len()
            .try_into()
            .expect("input length should fit into offset");
        assert!(errors.is_empty(), "parsing entry failed: {errors:#?}");
        Offsetter(offset).visit_expr(&mut entry);
        Some(entry)
    };

    Package {
        id: NodeId::default(),
        namespaces: namespaces.into_boxed_slice(),
        entry,
    }
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
            #6 30-32 "()" : Unit
            #10 40-42 "{}" : Unit
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
            #6 30-32 "()" : Unit
            #10 39-44 "{ 4 }" : Int
            #12 41-42 "4" : Int
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
            #6 30-32 "()" : Unit
            #10 39-47 "{ true }" : Bool
            #12 41-45 "true" : Bool
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Bool), Span { lo: 41, hi: 45 }))))
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
            #6 30-32 "()" : Unit
            #10 39-45 "{ 4; }" : Unit
            #12 41-42 "4" : Int
            Error(Type(Error(TyMismatch(Prim(Int), Tuple([]), Span { lo: 41, hi: 43 }))))
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
            #6 30-32 "()" : Unit
            #10 39-75 "{\n        let x = 4;\n        x\n    }" : Int
            #12 53-54 "x" : Int
            #14 57-58 "4" : Int
            #16 68-69 "x" : Int
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
            #15 46-51 "{ x }" : Int
            #17 48-49 "x" : Int
            #23 68-70 "()" : Unit
            #27 77-87 "{ Foo(4) }" : Int
            #29 79-85 "Foo(4)" : Int
            #30 79-82 "Foo" : (Int -> Int)
            #33 82-85 "(4)" : Int
            #34 83-84 "4" : Int
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
            #7 39-47 "(x : 'T)" : 0
            #8 40-46 "x : 'T" : 0
            #14 53-58 "{ x }" : 0
            #16 55-56 "x" : 0
            #22 75-77 "()" : Unit
            #26 84-99 "{ Identity(4) }" : Int
            #28 86-97 "Identity(4)" : Int
            #29 86-94 "Identity" : (Int -> Int)
            #32 94-97 "(4)" : Int
            #33 95-96 "4" : Int
        "##]],
    );
}

#[test]
fn call_generic_length() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Core {
                function Length<'T>(xs : 'T[]) : Int { body intrinsic; }
            }
        "},
        "Length([true, false, true])",
        &expect![[r##"
            #7 58-69 "(xs : 'T[])" : ?
            #8 59-68 "xs : 'T[]" : ?
            #17 98-125 "Length([true, false, true])" : Int
            #18 98-104 "Length" : ((Bool)[] -> Int)
            #21 104-125 "([true, false, true])" : (Bool)[]
            #22 105-124 "[true, false, true]" : (Bool)[]
            #23 106-110 "true" : Bool
            #24 112-117 "false" : Bool
            #25 119-123 "true" : Bool
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
            #6 30-32 "()" : Unit
            #10 40-52 "{ 1 + [2]; }" : Unit
            #12 42-49 "1 + [2]" : Int
            #13 42-43 "1" : Int
            #14 46-49 "[2]" : (Int)[]
            #15 47-48 "2" : Int
            Error(Type(Error(TyMismatch(Prim(Int), Array(Prim(Int)), Span { lo: 42, hi: 49 }))))
        "##]],
    );
}

#[test]
fn int_as_double_error() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Convert {
                function IntAsDouble(a : Int) : Double { body intrinsic; }
            }
        "},
        "Microsoft.Quantum.Convert.IntAsDouble(false)",
        &expect![[r##"
            #6 62-71 "(a : Int)" : ?
            #7 63-70 "a : Int" : ?
            #16 103-147 "Microsoft.Quantum.Convert.IntAsDouble(false)" : Double
            #17 103-140 "Microsoft.Quantum.Convert.IntAsDouble" : (Int -> Double)
            #21 140-147 "(false)" : Bool
            #22 141-146 "false" : Bool
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Bool), Span { lo: 103, hi: 147 }))))
        "##]],
    );
}

#[test]
fn length_type_error() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Core {
                function Length<'T>(xs : 'T[]) : Int { body intrinsic; }
            }
        "},
        "Length((1, 2, 3))",
        &expect![[r##"
            #7 58-69 "(xs : 'T[])" : ?
            #8 59-68 "xs : 'T[]" : ?
            #17 98-115 "Length((1, 2, 3))" : Int
            #18 98-104 "Length" : ((?0)[] -> Int)
            #21 104-115 "((1, 2, 3))" : (Int, Int, Int)
            #22 105-114 "(1, 2, 3)" : (Int, Int, Int)
            #23 106-107 "1" : Int
            #24 109-110 "2" : Int
            #25 112-113 "3" : Int
            Error(Type(Error(TyMismatch(Array(Infer(InferTyId(0))), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 98, hi: 115 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 98, hi: 104 }))))
        "##]],
    );
}

#[test]
fn single_arg_for_tuple() {
    check(
        indoc! {"
            namespace Microsoft.Quantum.Intrinsic {
                operation Ry(theta : Double, qubit : Qubit) : () is Adj + Ctl {}
            }
        "},
        indoc! {"{
            use q = Qubit();
            Ry(q);
        }"},
        &expect![[r##"
            #6 56-87 "(theta : Double, qubit : Qubit)" : (Double, Qubit)
            #7 57-71 "theta : Double" : Double
            #12 73-86 "qubit : Qubit" : Qubit
            #21 106-108 "{}" : Unit
            #22 111-146 "{\n    use q = Qubit();\n    Ry(q);\n}" : Unit
            #23 111-146 "{\n    use q = Qubit();\n    Ry(q);\n}" : Unit
            #25 121-122 "q" : Qubit
            #27 125-132 "Qubit()" : Qubit
            #29 138-143 "Ry(q)" : Unit
            #30 138-140 "Ry" : ((Double, Qubit) => Unit is Adj + Ctl)
            #33 140-143 "(q)" : Qubit
            #34 141-142 "q" : Qubit
            Error(Type(Error(TyMismatch(Tuple([Prim(Double), Prim(Qubit)]), Prim(Qubit), Span { lo: 138, hi: 143 }))))
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
            Error(Type(Error(MissingClassHasIndex(Array(Prim(Int)), Prim(Bool), Span { lo: 0, hi: 16 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 16 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Bool), Span { lo: 11, hi: 15 }))))
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
            #8 29-39 "set x += 1" : Unit
            #9 33-34 "x" : Bool
            #12 38-39 "1" : Int
            #14 45-46 "x" : Bool
            Error(Type(Error(TyMismatch(Prim(Bool), Prim(Int), Span { lo: 29, hi: 39 }))))
            Error(Type(Error(MissingClassAdd(Prim(Bool), Span { lo: 33, hi: 34 }))))
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
            Error(Type(Error(TyMismatch(Tuple([Prim(Int), Prim(Int)]), Prim(Double), Span { lo: 0, hi: 12 }))))
            Error(Type(Error(MissingClassAdd(Tuple([Prim(Int), Prim(Int)]), Span { lo: 0, hi: 6 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Double), Span { lo: 0, hi: 7 }))))
        "##]],
    );
}

#[test]
fn binop_andb_invalid() {
    check(
        "",
        "2.8 &&& 5.4",
        &expect![[r##"
            #1 0-11 "2.8 &&& 5.4" : Double
            #2 0-3 "2.8" : Double
            #3 8-11 "5.4" : Double
            Error(Type(Error(MissingClassInteger(Prim(Double), Span { lo: 0, hi: 3 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 }))))
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
            #6 31-33 "()" : Unit
            #10 41-43 "{}" : Unit
            #14 58-60 "()" : Unit
            #18 68-70 "{}" : Unit
            #19 73-89 "Test.A == Test.B" : Bool
            #20 73-79 "Test.A" : (Unit -> Unit)
            #24 83-89 "Test.B" : (Unit -> Unit)
            Error(Type(Error(MissingClassEq(Arrow(Arrow { kind: Function, input: Tuple([]), output: Tuple([]), functors: Value(Empty) }), Span { lo: 73, hi: 79 }))))
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
            Error(Type(Error(TyMismatch(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 25 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Result), Span { lo: 0, hi: 25 }))))
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
            Error(Type(Error(TyMismatch(Prim(BigInt), Prim(Int), Span { lo: 0, hi: 9 }))))
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
            Error(Type(Error(TyMismatch(Prim(BigInt), Prim(Int), Span { lo: 0, hi: 9 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Result), Span { lo: 0, hi: 25 }))))
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
            Error(Type(Error(TyMismatch(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 0, hi: 25 }))))
        "##]],
    );
}

#[test]
fn binop_orb_invalid() {
    check(
        "",
        "2.8 ||| 5.4",
        &expect![[r##"
            #1 0-11 "2.8 ||| 5.4" : Double
            #2 0-3 "2.8" : Double
            #3 8-11 "5.4" : Double
            Error(Type(Error(MissingClassInteger(Prim(Double), Span { lo: 0, hi: 3 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn binop_xorb_invalid() {
    check(
        "",
        "2.8 ^^^ 5.4",
        &expect![[r##"
            #1 0-11 "2.8 ^^^ 5.4" : Double
            #2 0-3 "2.8" : Double
            #3 8-11 "5.4" : Double
            Error(Type(Error(MissingClassInteger(Prim(Double), Span { lo: 0, hi: 3 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Prim(BigInt), Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn let_tuple_arity_error() {
    check(
        "",
        "{ let (x, y, z) = (0, 1); }",
        &expect![[r##"
            #1 0-27 "{ let (x, y, z) = (0, 1); }" : Unit
            #2 0-27 "{ let (x, y, z) = (0, 1); }" : Unit
            #4 6-15 "(x, y, z)" : (Int, Int, ?2)
            #5 7-8 "x" : Int
            #7 10-11 "y" : Int
            #9 13-14 "z" : ?2
            #11 18-24 "(0, 1)" : (Int, Int)
            #12 19-20 "0" : Int
            #13 22-23 "1" : Int
            Error(Type(Error(TyMismatch(Tuple([Infer(InferTyId(0)), Infer(InferTyId(1)), Infer(InferTyId(2))]), Tuple([Prim(Int), Prim(Int)]), Span { lo: 18, hi: 24 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 13, hi: 14 }))))
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
            #13 35-57 "set (x, y) = (1, 2, 3)" : Unit
            #14 39-45 "(x, y)" : (Int, Int)
            #15 40-41 "x" : Int
            #18 43-44 "y" : Int
            #21 48-57 "(1, 2, 3)" : (Int, Int, Int)
            #22 49-50 "1" : Int
            #23 52-53 "2" : Int
            #24 55-56 "3" : Int
            #26 63-64 "x" : Int
            Error(Type(Error(TyMismatch(Tuple([Prim(Int), Prim(Int)]), Tuple([Prim(Int), Prim(Int), Prim(Int)]), Span { lo: 39, hi: 45 }))))
        "##]],
    );
}

#[test]
fn qubit_array_length_error() {
    check(
        "",
        "{ use q = Qubit[false]; }",
        &expect![[r##"
            #1 0-25 "{ use q = Qubit[false]; }" : Unit
            #2 0-25 "{ use q = Qubit[false]; }" : Unit
            #4 6-7 "q" : (Qubit)[]
            #6 10-22 "Qubit[false]" : (Qubit)[]
            #7 16-21 "false" : Bool
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Bool), Span { lo: 16, hi: 21 }))))
        "##]],
    );
}

#[test]
fn qubit_tuple_arity_error() {
    check(
        "",
        "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }",
        &expect![[r##"
            #1 0-47 "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }" : Unit
            #2 0-47 "{ use (q, q1) = (Qubit[3], Qubit(), Qubit()); }" : Unit
            #4 6-13 "(q, q1)" : ((Qubit)[], Qubit)
            #5 7-8 "q" : (Qubit)[]
            #7 10-12 "q1" : Qubit
            #9 16-44 "(Qubit[3], Qubit(), Qubit())" : ((Qubit)[], Qubit, Qubit)
            #10 17-25 "Qubit[3]" : (Qubit)[]
            #11 23-24 "3" : Int
            #12 27-34 "Qubit()" : Qubit
            #13 36-43 "Qubit()" : Qubit
            Error(Type(Error(TyMismatch(Tuple([Array(Prim(Qubit)), Prim(Qubit), Prim(Qubit)]), Tuple([Infer(InferTyId(0)), Infer(InferTyId(1))]), Span { lo: 6, hi: 13 }))))
        "##]],
    );
}

#[test]
fn for_loop_not_iterable() {
    check(
        "",
        "for i in (1, true, One) {}",
        &expect![[r##"
            #1 0-26 "for i in (1, true, One) {}" : Unit
            #2 4-5 "i" : ?0
            #4 9-23 "(1, true, One)" : (Int, Bool, Result)
            #5 10-11 "1" : Int
            #6 13-17 "true" : Bool
            #7 19-22 "One" : Result
            #8 24-26 "{}" : Unit
            Error(Type(Error(MissingClassIterable(Tuple([Prim(Int), Prim(Bool), Prim(Result)]), Span { lo: 9, hi: 23 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 4, hi: 5 }))))
        "##]],
    );
}

#[test]
fn if_cond_error() {
    check(
        "",
        "if 4 {}",
        &expect![[r##"
            #1 0-7 "if 4 {}" : Unit
            #2 3-4 "4" : Int
            #3 5-7 "{}" : Unit
            Error(Type(Error(TyMismatch(Prim(Bool), Prim(Int), Span { lo: 3, hi: 4 }))))
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
            Error(Type(Error(TyMismatch(Prim(Int), Tuple([]), Span { lo: 0, hi: 13 }))))
        "##]],
    );
}

#[test]
fn if_else_fail() {
    check(
        "",
        r#"if false {} else { fail "error"; }"#,
        &expect![[r##"
            #1 0-34 "if false {} else { fail \"error\"; }" : Unit
            #2 3-8 "false" : Bool
            #3 9-11 "{}" : Unit
            #4 12-34 "else { fail \"error\"; }" : Unit
            #5 17-34 "{ fail \"error\"; }" : Unit
            #7 19-31 "fail \"error\"" : Unit
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
            #6 28-30 "()" : Unit
            #10 37-154 "{\n        if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }\n    }" : Int
            #12 47-148 "if fail \"error\" {\n            \"this type doesn't matter\"\n        } else {\n            \"foo\"\n        }" : Int
            #13 50-62 "fail \"error\"" : Bool
            #14 55-62 "\"error\"" : String
            #15 63-113 "{\n            \"this type doesn't matter\"\n        }" : String
            #17 77-103 "\"this type doesn't matter\"" : String
            #18 114-148 "else {\n            \"foo\"\n        }" : String
            #19 119-148 "{\n            \"foo\"\n        }" : String
            #21 133-138 "\"foo\"" : String
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
            #6 28-30 "()" : Unit
            #10 37-145 "{\n        if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }\n    }" : Int
            #12 47-139 "if fail \"cond\" {\n            fail \"true\"\n        } else {\n            fail \"false\"\n        }" : Int
            #13 50-61 "fail \"cond\"" : Bool
            #14 55-61 "\"cond\"" : String
            #15 62-97 "{\n            fail \"true\"\n        }" : Int
            #17 76-87 "fail \"true\"" : Int
            #18 81-87 "\"true\"" : String
            #19 98-139 "else {\n            fail \"false\"\n        }" : Int
            #20 103-139 "{\n            fail \"false\"\n        }" : Int
            #22 117-129 "fail \"false\"" : Int
            #23 122-129 "\"false\"" : String
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
            Error(Type(Error(TyMismatch(Prim(Bool), Prim(Int), Span { lo: 0, hi: 1 }))))
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
            Error(Type(Error(MissingClassHasIndex(Tuple([Prim(Int), Prim(Int), Prim(Int)]), Prim(Int), Span { lo: 0, hi: 19 }))))
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
            Error(Type(Error(MissingClassHasIndex(Array(Prim(Int)), Prim(Bool), Span { lo: 0, hi: 23 }))))
        "##]],
    );
}

#[test]
fn ternop_update_array_index_var() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let xs = [2];
                    let i = 0;
                    let ys = xs w/ i <- 3;
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-117 "{\n        let xs = [2];\n        let i = 0;\n        let ys = xs w/ i <- 3;\n    }" : Unit
            #10 52-54 "xs" : (Int)[]
            #12 57-60 "[2]" : (Int)[]
            #13 58-59 "2" : Int
            #15 74-75 "i" : Int
            #17 78-79 "0" : Int
            #19 93-95 "ys" : (Int)[]
            #21 98-110 "xs w/ i <- 3" : (Int)[]
            #22 98-100 "xs" : (Int)[]
            #25 104-105 "i" : Int
            #28 109-110 "3" : Int
        "##]],
    );
}

#[test]
fn ternop_update_array_index_expr() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let xs = [2];
                    let i = 0;
                    let ys = xs w/ i + 1 <- 3;
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-121 "{\n        let xs = [2];\n        let i = 0;\n        let ys = xs w/ i + 1 <- 3;\n    }" : Unit
            #10 52-54 "xs" : (Int)[]
            #12 57-60 "[2]" : (Int)[]
            #13 58-59 "2" : Int
            #15 74-75 "i" : Int
            #17 78-79 "0" : Int
            #19 93-95 "ys" : (Int)[]
            #21 98-114 "xs w/ i + 1 <- 3" : (Int)[]
            #22 98-100 "xs" : (Int)[]
            #25 104-109 "i + 1" : Int
            #26 104-105 "i" : Int
            #29 108-109 "1" : Int
            #30 113-114 "3" : Int
        "##]],
    );
}

#[test]
fn ternop_update_udt_known_field_name() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ First <- 3;
                }
            }
        "},
        "",
        &expect![[r##"
            #19 79-81 "()" : Unit
            #21 87-155 "{\n        let p = Pair(1, 2);\n        let q = p w/ First <- 3;\n    }" : Unit
            #23 101-102 "p" : UDT<Item 1>
            #25 105-115 "Pair(1, 2)" : UDT<Item 1>
            #26 105-109 "Pair" : ((Int, Int) -> UDT<Item 1>)
            #29 109-115 "(1, 2)" : (Int, Int)
            #30 110-111 "1" : Int
            #31 113-114 "2" : Int
            #33 129-130 "q" : UDT<Item 1>
            #35 133-148 "p w/ First <- 3" : UDT<Item 1>
            #36 133-134 "p" : UDT<Item 1>
            #39 138-143 "First" : ?
            #42 147-148 "3" : Int
        "##]],
    );
}

#[test]
fn ternop_update_udt_known_field_name_expr() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ First + 1 <- 3;
                }
            }
        "},
        "",
        &expect![[r##"
            #19 79-81 "()" : Unit
            #21 87-159 "{\n        let p = Pair(1, 2);\n        let q = p w/ First + 1 <- 3;\n    }" : Unit
            #23 101-102 "p" : UDT<Item 1>
            #25 105-115 "Pair(1, 2)" : UDT<Item 1>
            #26 105-109 "Pair" : ((Int, Int) -> UDT<Item 1>)
            #29 109-115 "(1, 2)" : (Int, Int)
            #30 110-111 "1" : Int
            #31 113-114 "2" : Int
            #33 129-130 "q" : UDT<Item 1>
            #35 133-152 "p w/ First + 1 <- 3" : UDT<Item 1>
            #36 133-134 "p" : UDT<Item 1>
            #39 138-147 "First + 1" : ?
            #40 138-143 "First" : ?
            #43 146-147 "1" : Int
            #44 151-152 "3" : Int
            Error(Resolve(NotFound("First", Span { lo: 138, hi: 143 })))
        "##]],
    );
}

#[test]
fn ternop_update_udt_unknown_field_name() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ Third <- 3;
                }
            }
        "},
        "",
        &expect![[r##"
            #19 79-81 "()" : Unit
            #21 87-155 "{\n        let p = Pair(1, 2);\n        let q = p w/ Third <- 3;\n    }" : Unit
            #23 101-102 "p" : UDT<Item 1>
            #25 105-115 "Pair(1, 2)" : UDT<Item 1>
            #26 105-109 "Pair" : ((Int, Int) -> UDT<Item 1>)
            #29 109-115 "(1, 2)" : (Int, Int)
            #30 110-111 "1" : Int
            #31 113-114 "2" : Int
            #33 129-130 "q" : UDT<Item 1>
            #35 133-148 "p w/ Third <- 3" : UDT<Item 1>
            #36 133-134 "p" : UDT<Item 1>
            #39 138-143 "Third" : ?
            #42 147-148 "3" : Int
            Error(Type(Error(MissingClassHasField(Udt(Item(ItemId { package: None, item: LocalItemId(1) })), "Third", Span { lo: 133, hi: 148 }))))
        "##]],
    );
}

#[test]
fn ternop_update_udt_unknown_field_name_known_global() {
    check(
        indoc! {"
            namespace A {
                newtype Pair = (First : Int, Second : Int);

                function Third() : () {}

                function Foo() : () {
                    let p = Pair(1, 2);
                    let q = p w/ Third <- 3;
                }
            }
        "},
        "",
        &expect![[r##"
            #19 81-83 "()" : Unit
            #21 89-91 "{}" : Unit
            #25 109-111 "()" : Unit
            #27 117-185 "{\n        let p = Pair(1, 2);\n        let q = p w/ Third <- 3;\n    }" : Unit
            #29 131-132 "p" : UDT<Item 1>
            #31 135-145 "Pair(1, 2)" : UDT<Item 1>
            #32 135-139 "Pair" : ((Int, Int) -> UDT<Item 1>)
            #35 139-145 "(1, 2)" : (Int, Int)
            #36 140-141 "1" : Int
            #37 143-144 "2" : Int
            #39 159-160 "q" : UDT<Item 1>
            #41 163-178 "p w/ Third <- 3" : UDT<Item 1>
            #42 163-164 "p" : UDT<Item 1>
            #45 168-173 "Third" : ?
            #48 177-178 "3" : Int
            Error(Type(Error(MissingClassHasField(Udt(Item(ItemId { package: None, item: LocalItemId(1) })), "Third", Span { lo: 163, hi: 178 }))))
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
            Error(Type(Error(MissingClassNum(Prim(Bool), Span { lo: 3, hi: 8 }))))
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
            Error(Type(Error(TyMismatch(Prim(Bool), Prim(Int), Span { lo: 4, hi: 5 }))))
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
            Error(Type(Error(MissingClassNum(Prim(Bool), Span { lo: 1, hi: 6 }))))
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
            Error(Type(Error(MissingClassNum(Prim(Bool), Span { lo: 1, hi: 6 }))))
        "##]],
    );
}

#[test]
fn while_cond_error() {
    check(
        "",
        "while Zero {}",
        &expect![[r##"
            #1 0-13 "while Zero {}" : Unit
            #2 6-10 "Zero" : Result
            #3 11-13 "{}" : Unit
            Error(Type(Error(TyMismatch(Prim(Bool), Prim(Result), Span { lo: 6, hi: 10 }))))
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
            #17 72-75 "..." : Qubit
            #18 76-78 "{}" : Unit
            #20 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #21 99-101 "cs" : (Qubit)[]
            #23 103-106 "..." : Qubit
            #24 108-110 "{}" : Unit
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
            #17 72-75 "..." : Qubit
            #18 76-78 "{}" : Unit
            #20 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #21 99-101 "cs" : (Qubit)[]
            #23 103-106 "..." : Qubit
            #24 108-110 "{}" : Unit
            #25 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : Unit
            #26 119-198 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    Controlled A.Foo([q1], q2);\n}" : Unit
            #28 129-131 "q1" : Qubit
            #30 134-141 "Qubit()" : Qubit
            #32 151-153 "q2" : Qubit
            #34 156-163 "Qubit()" : Qubit
            #36 169-195 "Controlled A.Foo([q1], q2)" : Unit
            #37 169-185 "Controlled A.Foo" : (((Qubit)[], Qubit) => Unit is Ctl)
            #38 180-185 "A.Foo" : (Qubit => Unit is Ctl)
            #42 185-195 "([q1], q2)" : ((Qubit)[], Qubit)
            #43 186-190 "[q1]" : (Qubit)[]
            #44 187-189 "q1" : Qubit
            #47 192-194 "q2" : Qubit
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
            #17 72-75 "..." : Qubit
            #18 76-78 "{}" : Unit
            #20 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #21 99-101 "cs" : (Qubit)[]
            #23 103-106 "..." : Qubit
            #24 108-110 "{}" : Unit
            #25 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : Unit
            #26 119-239 "{\n    use q1 = Qubit();\n    use q2 = Qubit();\n    use q3 = Qubit();\n    Controlled Controlled A.Foo([q1], ([q2], q3));\n}" : Unit
            #28 129-131 "q1" : Qubit
            #30 134-141 "Qubit()" : Qubit
            #32 151-153 "q2" : Qubit
            #34 156-163 "Qubit()" : Qubit
            #36 173-175 "q3" : Qubit
            #38 178-185 "Qubit()" : Qubit
            #40 191-236 "Controlled Controlled A.Foo([q1], ([q2], q3))" : Unit
            #41 191-218 "Controlled Controlled A.Foo" : (((Qubit)[], ((Qubit)[], Qubit)) => Unit is Ctl)
            #42 202-218 "Controlled A.Foo" : (((Qubit)[], Qubit) => Unit is Ctl)
            #43 213-218 "A.Foo" : (Qubit => Unit is Ctl)
            #47 218-236 "([q1], ([q2], q3))" : ((Qubit)[], ((Qubit)[], Qubit))
            #48 219-223 "[q1]" : (Qubit)[]
            #49 220-222 "q1" : Qubit
            #52 225-235 "([q2], q3)" : ((Qubit)[], Qubit)
            #53 226-230 "[q2]" : (Qubit)[]
            #54 227-229 "q2" : Qubit
            #57 232-234 "q3" : Qubit
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
            #17 72-75 "..." : Qubit
            #18 76-78 "{}" : Unit
            #20 98-107 "(cs, ...)" : ((Qubit)[], Qubit)
            #21 99-101 "cs" : (Qubit)[]
            #23 103-106 "..." : Qubit
            #24 108-110 "{}" : Unit
            #25 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : Unit
            #26 119-173 "{\n    use q = Qubit();\n    Controlled A.Foo([1], q);\n}" : Unit
            #28 129-130 "q" : Qubit
            #30 133-140 "Qubit()" : Qubit
            #32 146-170 "Controlled A.Foo([1], q)" : Unit
            #33 146-162 "Controlled A.Foo" : (((Qubit)[], Qubit) => Unit is Ctl)
            #34 157-162 "A.Foo" : (Qubit => Unit is Ctl)
            #38 162-170 "([1], q)" : ((Int)[], Qubit)
            #39 163-166 "[1]" : (Int)[]
            #40 164-165 "1" : Int
            #41 168-169 "q" : Qubit
            Error(Type(Error(TyMismatch(Prim(Qubit), Prim(Int), Span { lo: 146, hi: 170 }))))
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
            #6 31-33 "()" : Unit
            #11 47-52 "{ 1 }" : Int
            #13 49-50 "1" : Int
            Error(Type(Error(TyMismatch(Tuple([]), Prim(Int), Span { lo: 36, hi: 39 }))))
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
            #6 31-33 "()" : Unit
            #11 47-52 "{ 1 }" : Int
            #13 49-50 "1" : Int
            Error(Type(Error(TyMismatch(Tuple([]), Prim(Int), Span { lo: 36, hi: 39 }))))
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
            #6 31-33 "()" : Unit
            #13 53-58 "{ 1 }" : Int
            #15 55-56 "1" : Int
            Error(Type(Error(TyMismatch(Tuple([]), Prim(Int), Span { lo: 36, hi: 39 }))))
        "##]],
    );
}

#[test]
fn adj_non_adj() {
    check(
        indoc! {"
            namespace A {
                operation Foo() : () is Ctl {}
            }
        "},
        "Adjoint A.Foo",
        &expect![[r##"
            #6 31-33 "()" : Unit
            #9 46-48 "{}" : Unit
            #10 51-64 "Adjoint A.Foo" : (Unit => Unit is Ctl)
            #11 59-64 "A.Foo" : (Unit => Unit is Ctl)
            Error(Type(Error(MissingFunctor(Value(Adj), Value(Ctl), Span { lo: 59, hi: 64 }))))
        "##]],
    );
}

#[test]
fn ctl_non_ctl() {
    check(
        indoc! {"
            namespace A {
                operation Foo() : () is Adj {}
            }
        "},
        "Controlled A.Foo",
        &expect![[r##"
            #6 31-33 "()" : Unit
            #9 46-48 "{}" : Unit
            #10 51-67 "Controlled A.Foo" : (((Qubit)[], Unit) => Unit is Adj)
            #11 62-67 "A.Foo" : (Unit => Unit is Adj)
            Error(Type(Error(MissingFunctor(Value(Ctl), Value(Adj), Span { lo: 62, hi: 67 }))))
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
            #15 47-153 "{\n        let x = if x {\n            return 1\n        } else {\n            true\n        };\n        2\n    }" : Int
            #17 61-62 "x" : Bool
            #19 65-136 "if x {\n            return 1\n        } else {\n            true\n        }" : Bool
            #20 68-69 "x" : Bool
            #23 70-102 "{\n            return 1\n        }" : Bool
            #25 84-92 "return 1" : Bool
            #26 91-92 "1" : Int
            #27 103-136 "else {\n            true\n        }" : Bool
            #28 108-136 "{\n            true\n        }" : Bool
            #30 122-126 "true" : Bool
            #32 146-147 "2" : Int
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
            #15 47-132 "{\n        let x = {\n            return 1;\n            true\n        };\n        x\n    }" : Int
            #17 61-62 "x" : Unit
            #19 65-115 "{\n            return 1;\n            true\n        }" : Unit
            #20 65-115 "{\n            return 1;\n            true\n        }" : Unit
            #22 79-87 "return 1" : Unit
            #23 86-87 "1" : Int
            #25 101-105 "true" : Bool
            #27 125-126 "x" : Unit
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
            #15 47-75 "{\n        return true;\n    }" : Int
            #17 57-68 "return true" : Unit
            #18 64-68 "true" : Bool
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Bool), Span { lo: 64, hi: 68 }))))
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
            #6 30-43 "(x : Qubit[])" : (Qubit)[]
            #7 31-42 "x : Qubit[]" : (Qubit)[]
            #16 50-73 "{\n        x::Size\n    }" : Int
            #18 60-67 "x::Size" : Int
            #19 60-61 "x" : (Qubit)[]
            Error(Type(Error(MissingClassHasField(Array(Prim(Qubit)), "Size", Span { lo: 60, hi: 67 }))))
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
            #6 30-41 "(r : Range)" : Range
            #7 31-40 "r : Range" : Range
            #22 60-103 "{\n        (r::Start, r::Step, r::End)\n    }" : (Int, Int, Int)
            #24 70-97 "(r::Start, r::Step, r::End)" : (Int, Int, Int)
            #25 71-79 "r::Start" : Int
            #26 71-72 "r" : Range
            #30 81-88 "r::Step" : Int
            #31 81-82 "r" : Range
            #35 90-96 "r::End" : Int
            #36 90-91 "r" : Range
        "##]],
    );
}

#[test]
fn range_to_field_start() {
    check(
        "",
        "(...2..8)::Start",
        &expect![[r##"
            #1 0-16 "(...2..8)::Start" : ?0
            #2 0-9 "(...2..8)" : RangeTo
            #3 1-8 "...2..8" : RangeTo
            #4 4-5 "2" : Int
            #5 7-8 "8" : Int
            Error(Type(Error(MissingClassHasField(Prim(RangeTo), "Start", Span { lo: 0, hi: 16 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 16 }))))
        "##]],
    );
}

#[test]
fn range_to_field_step() {
    check(
        "",
        "(...2..8)::Step",
        &expect![[r##"
            #1 0-15 "(...2..8)::Step" : Int
            #2 0-9 "(...2..8)" : RangeTo
            #3 1-8 "...2..8" : RangeTo
            #4 4-5 "2" : Int
            #5 7-8 "8" : Int
        "##]],
    );
}

#[test]
fn range_to_field_end() {
    check(
        "",
        "(...2..8)::End",
        &expect![[r##"
            #1 0-14 "(...2..8)::End" : Int
            #2 0-9 "(...2..8)" : RangeTo
            #3 1-8 "...2..8" : RangeTo
            #4 4-5 "2" : Int
            #5 7-8 "8" : Int
        "##]],
    );
}

#[test]
fn range_from_field_start() {
    check(
        "",
        "(0..2...)::Start",
        &expect![[r##"
            #1 0-16 "(0..2...)::Start" : Int
            #2 0-9 "(0..2...)" : RangeFrom
            #3 1-8 "0..2..." : RangeFrom
            #4 1-2 "0" : Int
            #5 4-5 "2" : Int
        "##]],
    );
}

#[test]
fn range_from_field_step() {
    check(
        "",
        "(0..2...)::Step",
        &expect![[r##"
            #1 0-15 "(0..2...)::Step" : Int
            #2 0-9 "(0..2...)" : RangeFrom
            #3 1-8 "0..2..." : RangeFrom
            #4 1-2 "0" : Int
            #5 4-5 "2" : Int
        "##]],
    );
}

#[test]
fn range_from_field_end() {
    check(
        "",
        "(0..2...)::End",
        &expect![[r##"
            #1 0-14 "(0..2...)::End" : ?0
            #2 0-9 "(0..2...)" : RangeFrom
            #3 1-8 "0..2..." : RangeFrom
            #4 1-2 "0" : Int
            #5 4-5 "2" : Int
            Error(Type(Error(MissingClassHasField(Prim(RangeFrom), "End", Span { lo: 0, hi: 14 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 14 }))))
        "##]],
    );
}

#[test]
fn range_full_field_start() {
    check(
        "",
        "...::Start",
        &expect![[r##"
            #1 0-10 "...::Start" : ?0
            #2 0-3 "..." : RangeFull
            Error(Type(Error(MissingClassHasField(Prim(RangeFull), "Start", Span { lo: 0, hi: 10 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 10 }))))
        "##]],
    );
}

#[test]
fn range_full_implicit_step() {
    check(
        "",
        "...::Step",
        &expect![[r##"
            #1 0-9 "...::Step" : Int
            #2 0-3 "..." : RangeFull
        "##]],
    );
}

#[test]
fn range_full_explicit_step() {
    check(
        "",
        "(...2...)::Step",
        &expect![[r##"
            #1 0-15 "(...2...)::Step" : Int
            #2 0-9 "(...2...)" : RangeFull
            #3 1-8 "...2..." : RangeFull
            #4 4-5 "2" : Int
        "##]],
    );
}

#[test]
fn range_full_field_end() {
    check(
        "",
        "...::End",
        &expect![[r##"
            #1 0-8 "...::End" : ?0
            #2 0-3 "..." : RangeFull
            Error(Type(Error(MissingClassHasField(Prim(RangeFull), "End", Span { lo: 0, hi: 8 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 8 }))))
        "##]],
    );
}

#[test]
fn interpolate_int() {
    check(
        "",
        r#"$"{4}""#,
        &expect![[r##"
            #1 0-6 "$\"{4}\"" : String
            #2 3-4 "4" : Int
        "##]],
    );
}

#[test]
fn interpolate_string() {
    check(
        "",
        r#"$"{"foo"}""#,
        &expect![[r##"
            #1 0-10 "$\"{\"foo\"}\"" : String
            #2 3-8 "\"foo\"" : String
        "##]],
    );
}

#[test]
fn interpolate_qubit() {
    check(
        "",
        r#"{ use q = Qubit(); $"{q}" }"#,
        &expect![[r##"
            #1 0-27 "{ use q = Qubit(); $\"{q}\" }" : String
            #2 0-27 "{ use q = Qubit(); $\"{q}\" }" : String
            #4 6-7 "q" : Qubit
            #6 10-17 "Qubit()" : Qubit
            #8 19-25 "$\"{q}\"" : String
            #9 22-23 "q" : Qubit
        "##]],
    );
}

#[test]
fn interpolate_function() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {}
            }
        "},
        r#"$"{A.Foo}""#,
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-40 "{}" : Unit
            #9 43-53 "$\"{A.Foo}\"" : String
            #10 46-51 "A.Foo" : (Unit -> Unit)
            Error(Type(Error(MissingClassShow(Arrow(Arrow { kind: Function, input: Tuple([]), output: Tuple([]), functors: Value(Empty) }), Span { lo: 46, hi: 51 }))))
        "##]],
    );
}

#[test]
fn interpolate_operation() {
    check(
        indoc! {"
            namespace A {
                operation Foo() : () {}
            }
        "},
        r#"$"{A.Foo}""#,
        &expect![[r##"
            #6 31-33 "()" : Unit
            #8 39-41 "{}" : Unit
            #9 44-54 "$\"{A.Foo}\"" : String
            #10 47-52 "A.Foo" : (Unit => Unit)
            Error(Type(Error(MissingClassShow(Arrow(Arrow { kind: Operation, input: Tuple([]), output: Tuple([]), functors: Value(Empty) }), Span { lo: 47, hi: 52 }))))
        "##]],
    );
}

#[test]
fn interpolate_int_array() {
    check(
        "",
        r#"$"{[1, 2, 3]}""#,
        &expect![[r##"
            #1 0-14 "$\"{[1, 2, 3]}\"" : String
            #2 3-12 "[1, 2, 3]" : (Int)[]
            #3 4-5 "1" : Int
            #4 7-8 "2" : Int
            #5 10-11 "3" : Int
        "##]],
    );
}

#[test]
fn interpolate_function_array() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {}
                function Bar() : () {}
            }
        "},
        r#"$"{[A.Foo, A.Bar]}""#,
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-40 "{}" : Unit
            #12 57-59 "()" : Unit
            #14 65-67 "{}" : Unit
            #15 70-89 "$\"{[A.Foo, A.Bar]}\"" : String
            #16 73-87 "[A.Foo, A.Bar]" : ((Unit -> Unit))[]
            #17 74-79 "A.Foo" : (Unit -> Unit)
            #21 81-86 "A.Bar" : (Unit -> Unit)
            Error(Type(Error(MissingClassShow(Arrow(Arrow { kind: Function, input: Tuple([]), output: Tuple([]), functors: Value(Empty) }), Span { lo: 73, hi: 87 }))))
        "##]],
    );
}

#[test]
fn interpolate_int_string_tuple() {
    check(
        "",
        r#"$"{(1, "foo")}""#,
        &expect![[r##"
            #1 0-15 "$\"{(1, \"foo\")}\"" : String
            #2 3-13 "(1, \"foo\")" : (Int, String)
            #3 4-5 "1" : Int
            #4 7-12 "\"foo\"" : String
        "##]],
    );
}

#[test]
fn interpolate_int_function_tuple() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {}
            }
        "},
        r#"$"{(1, A.Foo)}""#,
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-40 "{}" : Unit
            #9 43-58 "$\"{(1, A.Foo)}\"" : String
            #10 46-56 "(1, A.Foo)" : (Int, (Unit -> Unit))
            #11 47-48 "1" : Int
            #12 50-55 "A.Foo" : (Unit -> Unit)
            Error(Type(Error(MissingClassShow(Arrow(Arrow { kind: Function, input: Tuple([]), output: Tuple([]), functors: Value(Empty) }), Span { lo: 46, hi: 56 }))))
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
            #12 56-58 "()" : Unit
            #16 68-81 "{ NewInt(5) }" : UDT<Item 1>
            #18 70-79 "NewInt(5)" : UDT<Item 1>
            #19 70-76 "NewInt" : (Int -> UDT<Item 1>)
            #22 76-79 "(5)" : Int
            #23 77-78 "5" : Int
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
            #12 56-58 "()" : Unit
            #16 68-83 "{ NewInt(5.0) }" : UDT<Item 1>
            #18 70-81 "NewInt(5.0)" : UDT<Item 1>
            #19 70-76 "NewInt" : (Int -> UDT<Item 1>)
            #22 76-81 "(5.0)" : Double
            #23 77-80 "5.0" : Double
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Double), Span { lo: 70, hi: 81 }))))
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
            #12 56-58 "()" : Unit
            #16 65-78 "{ NewInt(5) }" : UDT<Item 1>
            #18 67-76 "NewInt(5)" : UDT<Item 1>
            #19 67-73 "NewInt" : (Int -> UDT<Item 1>)
            #22 73-76 "(5)" : Int
            #23 74-75 "5" : Int
            Error(Type(Error(TyMismatch(Prim(Int), Udt(Item(ItemId { package: None, item: LocalItemId(1) })), Span { lo: 67, hi: 76 }))))
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
            #18 84-86 "()" : Unit
            #22 97-111 "{ NewInt1(5) }" : UDT<Item 1>
            #24 99-109 "NewInt1(5)" : UDT<Item 1>
            #25 99-106 "NewInt1" : (Int -> UDT<Item 1>)
            #28 106-109 "(5)" : Int
            #29 107-108 "5" : Int
            Error(Type(Error(TyMismatch(Udt(Item(ItemId { package: None, item: LocalItemId(2) })), Udt(Item(ItemId { package: None, item: LocalItemId(1) })), Span { lo: 99, hi: 109 }))))
        "##]],
    );
}

#[test]
fn newtype_unwrap() {
    check(
        indoc! {"
            namespace A {
                newtype Foo = (Int, Bool);
                function Bar(x : Foo) : () {
                    let y = x!;
                }
            }
        "},
        "",
        &expect![[r##"
            #17 61-70 "(x : Foo)" : UDT<Item 1>
            #18 62-69 "x : Foo" : UDT<Item 1>
            #24 76-103 "{\n        let y = x!;\n    }" : Unit
            #26 90-91 "y" : (Int, Bool)
            #28 94-96 "x!" : (Int, Bool)
            #29 94-95 "x" : UDT<Item 1>
        "##]],
    );
}

#[test]
fn newtype_field() {
    check(
        indoc! {"
            namespace A {
                newtype Foo = Bar : Int;
                function Baz(x : Foo) : () {
                    let y = x::Bar;
                }
            }
        "},
        "",
        &expect![[r##"
            #13 59-68 "(x : Foo)" : UDT<Item 1>
            #14 60-67 "x : Foo" : UDT<Item 1>
            #20 74-105 "{\n        let y = x::Bar;\n    }" : Unit
            #22 88-89 "y" : Int
            #24 92-98 "x::Bar" : Int
            #25 92-93 "x" : UDT<Item 1>
        "##]],
    );
}

#[test]
fn newtype_field_invalid() {
    check(
        indoc! {"
            namespace A {
                newtype Foo = Bar : Int;
                function Baz(x : Foo) : () {
                    let y = x::Nope;
                }
            }
        "},
        "",
        &expect![[r##"
            #13 59-68 "(x : Foo)" : UDT<Item 1>
            #14 60-67 "x : Foo" : UDT<Item 1>
            #20 74-106 "{\n        let y = x::Nope;\n    }" : Unit
            #22 88-89 "y" : ?1
            #24 92-99 "x::Nope" : ?1
            #25 92-93 "x" : UDT<Item 1>
            Error(Type(Error(MissingClassHasField(Udt(Item(ItemId { package: None, item: LocalItemId(1) })), "Nope", Span { lo: 92, hi: 99 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 92, hi: 99 }))))
        "##]],
    );
}

#[test]
fn unknown_name_fits_any_ty() {
    check(
        "",
        "{ let x : Int = foo; let y : Qubit = foo; }",
        &expect![[r##"
            #1 0-43 "{ let x : Int = foo; let y : Qubit = foo; }" : Unit
            #2 0-43 "{ let x : Int = foo; let y : Qubit = foo; }" : Unit
            #4 6-13 "x : Int" : Int
            #9 16-19 "foo" : ?
            #13 25-34 "y : Qubit" : Qubit
            #18 37-40 "foo" : ?
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
            #1 0-18 "{ foo(); foo + 1 }" : ?
            #2 0-18 "{ foo(); foo + 1 }" : ?
            #4 2-7 "foo()" : ?0
            #5 2-5 "foo" : ?
            #8 5-7 "()" : Unit
            #10 9-16 "foo + 1" : ?
            #11 9-12 "foo" : ?
            #14 15-16 "1" : Int
            Error(Resolve(NotFound("foo", Span { lo: 2, hi: 5 })))
            Error(Resolve(NotFound("foo", Span { lo: 9, hi: 12 })))
            Error(Type(Error(AmbiguousTy(Span { lo: 2, hi: 7 }))))
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
            #6 30-32 "()" : Unit
            #10 39-99 "{\n        function Bar() : Int { 2 }\n        Bar() + 1\n    }" : Int
            #15 61-63 "()" : Unit
            #19 70-75 "{ 2 }" : Int
            #21 72-73 "2" : Int
            #23 84-93 "Bar() + 1" : Int
            #24 84-89 "Bar()" : Int
            #25 84-87 "Bar" : (Unit -> Int)
            #28 87-89 "()" : Unit
            #29 92-93 "1" : Int
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
            #6 30-32 "()" : Unit
            #10 39-97 "{\n        function Bar() : Int { 2.0 }\n        Bar()\n    }" : Int
            #15 61-63 "()" : Unit
            #19 70-77 "{ 2.0 }" : Double
            #21 72-75 "2.0" : Double
            #23 86-91 "Bar()" : Int
            #24 86-89 "Bar" : (Unit -> Int)
            #27 89-91 "()" : Unit
            Error(Type(Error(TyMismatch(Prim(Int), Prim(Double), Span { lo: 72, hi: 75 }))))
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
            #6 30-32 "()" : Unit
            #8 38-91 "{\n        Bar();\n        function Bar() : () {}\n    }" : Unit
            #10 48-53 "Bar()" : Unit
            #11 48-51 "Bar" : (Unit -> Unit)
            #14 51-53 "()" : Unit
            #19 75-77 "()" : Unit
            #21 83-85 "{}" : Unit
        "##]],
    );
}

#[test]
fn local_function_last_stmt_is_unit_block() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Int {
                    Bar();
                    function Bar() : Int { 4 }
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #10 39-96 "{\n        Bar();\n        function Bar() : Int { 4 }\n    }" : Unit
            #12 49-54 "Bar()" : Int
            #13 49-52 "Bar" : (Unit -> Int)
            #16 52-54 "()" : Unit
            #21 76-78 "()" : Unit
            #25 85-90 "{ 4 }" : Int
            #27 87-88 "4" : Int
            Error(Type(Error(TyMismatch(Prim(Int), Tuple([]), Span { lo: 64, hi: 90 }))))
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
            #6 30-32 "()" : Unit
            #8 38-96 "{\n        newtype Bar = Int;\n        let x = Bar(5);\n    }" : Unit
            #17 79-80 "x" : UDT<Item 2>
            #19 83-89 "Bar(5)" : UDT<Item 2>
            #20 83-86 "Bar" : (Int -> UDT<Item 2>)
            #23 86-89 "(5)" : Int
            #24 87-88 "5" : Int
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
            #6 26-28 "()" : Unit
            #8 34-52 "{ open B; Bar(); }" : Unit
            #13 44-49 "Bar()" : Unit
            #14 44-47 "Bar" : (Unit -> Unit)
            #17 47-49 "()" : Unit
            #23 81-83 "()" : Unit
            #25 89-91 "{}" : Unit
        "##]],
    );
}

#[test]
fn infinite() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let x = invalid;
                    let xs = [x, [x]];
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-97 "{\n        let x = invalid;\n        let xs = [x, [x]];\n    }" : Unit
            #10 52-53 "x" : ?0
            #12 56-63 "invalid" : ?
            #16 77-79 "xs" : (?0)[]
            #18 82-90 "[x, [x]]" : (?0)[]
            #19 83-84 "x" : ?0
            #22 86-89 "[x]" : (?0)[]
            #23 87-88 "x" : ?0
            Error(Resolve(NotFound("invalid", Span { lo: 56, hi: 63 })))
            Error(Type(Error(TyMismatch(Infer(InferTyId(0)), Array(Infer(InferTyId(0))), Span { lo: 86, hi: 89 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 52, hi: 53 }))))
        "##]],
    );
}

#[test]
fn lambda_adj() {
    check(
        indoc! {"
            namespace A {
                operation Foo(op : () => () is Adj) : () {}
                operation Bar() : () { Foo(() => ()) }
            }
        "},
        "",
        &expect![[r##"
            #6 31-53 "(op : () => () is Adj)" : (Unit => Unit is Adj)
            #7 32-52 "op : () => () is Adj" : (Unit => Unit is Adj)
            #14 59-61 "{}" : Unit
            #18 79-81 "()" : Unit
            #20 87-104 "{ Foo(() => ()) }" : Unit
            #22 89-102 "Foo(() => ())" : Unit
            #23 89-92 "Foo" : ((Unit => Unit is Adj) => Unit)
            #26 92-102 "(() => ())" : (Unit => Unit is Adj)
            #27 93-101 "() => ()" : (Unit => Unit is Adj)
            #28 93-95 "()" : Unit
            #29 99-101 "()" : Unit
        "##]],
    );
}

#[test]
fn lambda_ctl() {
    check(
        indoc! {"
            namespace A {
                operation Foo(op : () => () is Ctl) : () {}
                operation Bar() : () { Foo(() => ()) }
            }
        "},
        "",
        &expect![[r##"
            #6 31-53 "(op : () => () is Ctl)" : (Unit => Unit is Ctl)
            #7 32-52 "op : () => () is Ctl" : (Unit => Unit is Ctl)
            #14 59-61 "{}" : Unit
            #18 79-81 "()" : Unit
            #20 87-104 "{ Foo(() => ()) }" : Unit
            #22 89-102 "Foo(() => ())" : Unit
            #23 89-92 "Foo" : ((Unit => Unit is Ctl) => Unit)
            #26 92-102 "(() => ())" : (Unit => Unit is Ctl)
            #27 93-101 "() => ()" : (Unit => Unit is Ctl)
            #28 93-95 "()" : Unit
            #29 99-101 "()" : Unit
        "##]],
    );
}

#[test]
fn lambda_adj_ctl() {
    check(
        indoc! {"
            namespace A {
                operation Foo(op : () => () is Adj + Ctl) : () {}
                operation Bar() : () { Foo(() => ()) }
            }
        "},
        "",
        &expect![[r##"
            #6 31-59 "(op : () => () is Adj + Ctl)" : (Unit => Unit is Adj + Ctl)
            #7 32-58 "op : () => () is Adj + Ctl" : (Unit => Unit is Adj + Ctl)
            #16 65-67 "{}" : Unit
            #20 85-87 "()" : Unit
            #22 93-110 "{ Foo(() => ()) }" : Unit
            #24 95-108 "Foo(() => ())" : Unit
            #25 95-98 "Foo" : ((Unit => Unit is Adj + Ctl) => Unit)
            #28 98-108 "(() => ())" : (Unit => Unit is Adj + Ctl)
            #29 99-107 "() => ()" : (Unit => Unit is Adj + Ctl)
            #30 99-101 "()" : Unit
            #31 105-107 "()" : Unit
        "##]],
    );
}

#[test]
fn lambda_functors_let_binding() {
    check(
        indoc! {"
            namespace A {
                function Foo() : () {
                    let op : Qubit => Unit is Adj = q => ();
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #8 38-94 "{\n        let op : Qubit => Unit is Adj = q => ();\n    }" : Unit
            #10 52-77 "op : Qubit => Unit is Adj" : (Qubit => Unit is Adj)
            #20 80-87 "q => ()" : (Qubit => Unit is Adj)
            #21 80-81 "q" : Qubit
            #23 85-87 "()" : Unit
        "##]],
    );
}

#[test]
fn lambda_adjoint_before_functors_inferred() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Qubit => Unit is Adj {
                    let op = q => ();
                    Adjoint op
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #15 56-108 "{\n        let op = q => ();\n        Adjoint op\n    }" : (Qubit => Unit is Adj)
            #17 70-72 "op" : (Qubit => Unit is Adj)
            #19 75-82 "q => ()" : (Qubit => Unit is Adj)
            #20 75-76 "q" : Qubit
            #22 80-82 "()" : Unit
            #24 92-102 "Adjoint op" : (Qubit => Unit is Adj)
            #25 100-102 "op" : (Qubit => Unit is Adj)
        "##]],
    );
}

#[test]
fn lambda_invalid_adjoint_before_functors_inferred() {
    check(
        indoc! {"
            namespace A {
                function Foo() : Qubit => Unit is Ctl {
                    let op = q => ();
                    Adjoint op
                }
            }
        "},
        "",
        &expect![[r##"
            #6 30-32 "()" : Unit
            #15 56-108 "{\n        let op = q => ();\n        Adjoint op\n    }" : (Qubit => Unit is Ctl)
            #17 70-72 "op" : (Qubit => Unit is Ctl)
            #19 75-82 "q => ()" : (Qubit => Unit is Ctl)
            #20 75-76 "q" : Qubit
            #22 80-82 "()" : Unit
            #24 92-102 "Adjoint op" : (Qubit => Unit is Ctl)
            #25 100-102 "op" : (Qubit => Unit is Ctl)
            Error(Type(Error(MissingFunctor(Value(Adj), Value(Ctl), Span { lo: 92, hi: 102 }))))
        "##]],
    );
}

#[test]
fn lambda_multiple_uses_functors_inferred() {
    check(
        indoc! {"
            namespace A {
                operation TakeAdj(op : Qubit => () is Adj) : () {}
                operation TakeAdjCtl(op : Qubit => () is Adj + Ctl) : () {}
                operation Foo() : () {
                    let op = q => ();
                    TakeAdj(op);
                    TakeAdjCtl(op);
                    let opCtl = Controlled op;
                }
            }
        "},
        "",
        &expect![[r##"
            #6 35-60 "(op : Qubit => () is Adj)" : (Qubit => Unit is Adj)
            #7 36-59 "op : Qubit => () is Adj" : (Qubit => Unit is Adj)
            #16 66-68 "{}" : Unit
            #20 93-124 "(op : Qubit => () is Adj + Ctl)" : (Qubit => Unit is Adj + Ctl)
            #21 94-123 "op : Qubit => () is Adj + Ctl" : (Qubit => Unit is Adj + Ctl)
            #32 130-132 "{}" : Unit
            #36 150-152 "()" : Unit
            #38 158-271 "{\n        let op = q => ();\n        TakeAdj(op);\n        TakeAdjCtl(op);\n        let opCtl = Controlled op;\n    }" : Unit
            #40 172-174 "op" : (Qubit => Unit is Adj + Ctl)
            #42 177-184 "q => ()" : (Qubit => Unit is Adj + Ctl)
            #43 177-178 "q" : Qubit
            #45 182-184 "()" : Unit
            #47 194-205 "TakeAdj(op)" : Unit
            #48 194-201 "TakeAdj" : ((Qubit => Unit is Adj + Ctl) => Unit)
            #51 201-205 "(op)" : (Qubit => Unit is Adj + Ctl)
            #52 202-204 "op" : (Qubit => Unit is Adj + Ctl)
            #56 215-229 "TakeAdjCtl(op)" : Unit
            #57 215-225 "TakeAdjCtl" : ((Qubit => Unit is Adj + Ctl) => Unit)
            #60 225-229 "(op)" : (Qubit => Unit is Adj + Ctl)
            #61 226-228 "op" : (Qubit => Unit is Adj + Ctl)
            #65 243-248 "opCtl" : (((Qubit)[], Qubit) => Unit is Adj + Ctl)
            #67 251-264 "Controlled op" : (((Qubit)[], Qubit) => Unit is Adj + Ctl)
            #68 262-264 "op" : (Qubit => Unit is Adj + Ctl)
        "##]],
    );
}

#[test]
fn partial_app_one_hole() {
    check(
        "",
        "{
            function Foo(x : Int) : Int { x }
            let f = Foo(_);
        }",
        &expect![[r##"
            #1 0-85 "{\n            function Foo(x : Int) : Int { x }\n            let f = Foo(_);\n        }" : Unit
            #2 0-85 "{\n            function Foo(x : Int) : Int { x }\n            let f = Foo(_);\n        }" : Unit
            #7 26-35 "(x : Int)" : Int
            #8 27-34 "x : Int" : Int
            #16 42-47 "{ x }" : Int
            #18 44-45 "x" : Int
            #22 64-65 "f" : (Int -> Int)
            #24 68-74 "Foo(_)" : (Int -> Int)
            #25 68-71 "Foo" : (Int -> Int)
            #28 71-74 "(_)" : Int
            #29 72-73 "_" : Int
        "##]],
    );
}

#[test]
fn partial_app_one_given_one_hole() {
    check(
        "",
        indoc! {"{
            function Foo(x : Int, y : Int) : Int { x + y }
            let f = Foo(2, _);
        }"},
        &expect![[r##"
            #1 0-77 "{\n    function Foo(x : Int, y : Int) : Int { x + y }\n    let f = Foo(2, _);\n}" : Unit
            #2 0-77 "{\n    function Foo(x : Int, y : Int) : Int { x + y }\n    let f = Foo(2, _);\n}" : Unit
            #7 18-36 "(x : Int, y : Int)" : (Int, Int)
            #8 19-26 "x : Int" : Int
            #13 28-35 "y : Int" : Int
            #21 43-52 "{ x + y }" : Int
            #23 45-50 "x + y" : Int
            #24 45-46 "x" : Int
            #27 49-50 "y" : Int
            #31 61-62 "f" : (Int -> Int)
            #33 65-74 "Foo(2, _)" : (Int -> Int)
            #34 65-68 "Foo" : ((Int, Int) -> Int)
            #37 68-74 "(2, _)" : (Int, Int)
            #38 69-70 "2" : Int
            #39 72-73 "_" : Int
        "##]],
    );
}

#[test]
fn partial_app_two_holes() {
    check(
        "",
        indoc! {"{
            function Foo(x : Int, y : Int) : Int { x + y }
            let f = Foo(_, _);
        }"},
        &expect![[r##"
            #1 0-77 "{\n    function Foo(x : Int, y : Int) : Int { x + y }\n    let f = Foo(_, _);\n}" : Unit
            #2 0-77 "{\n    function Foo(x : Int, y : Int) : Int { x + y }\n    let f = Foo(_, _);\n}" : Unit
            #7 18-36 "(x : Int, y : Int)" : (Int, Int)
            #8 19-26 "x : Int" : Int
            #13 28-35 "y : Int" : Int
            #21 43-52 "{ x + y }" : Int
            #23 45-50 "x + y" : Int
            #24 45-46 "x" : Int
            #27 49-50 "y" : Int
            #31 61-62 "f" : ((Int, Int) -> Int)
            #33 65-74 "Foo(_, _)" : ((Int, Int) -> Int)
            #34 65-68 "Foo" : ((Int, Int) -> Int)
            #37 68-74 "(_, _)" : (Int, Int)
            #38 69-70 "_" : Int
            #39 72-73 "_" : Int
        "##]],
    );
}

#[test]
fn partial_app_nested_tuple() {
    check(
        "",
        indoc! {"{
            function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}
            let f = Foo(_, (_, 1.0, _), _);
        }"},
        &expect![[r##"
            #1 0-121 "{\n    function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}\n    let f = Foo(_, (_, 1.0, _), _);\n}" : Unit
            #2 0-121 "{\n    function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}\n    let f = Foo(_, (_, 1.0, _), _);\n}" : Unit
            #7 18-75 "(a : Int, (b : Bool, c : Double, d : String), e : Result)" : (Int, (Bool, Double, String), Result)
            #8 19-26 "a : Int" : Int
            #13 28-62 "(b : Bool, c : Double, d : String)" : (Bool, Double, String)
            #14 29-37 "b : Bool" : Bool
            #19 39-49 "c : Double" : Double
            #24 51-61 "d : String" : String
            #29 64-74 "e : Result" : Result
            #35 81-83 "{}" : Unit
            #37 92-93 "f" : ((Int, (Bool, String), Result) -> Unit)
            #39 96-118 "Foo(_, (_, 1.0, _), _)" : ((Int, (Bool, String), Result) -> Unit)
            #40 96-99 "Foo" : ((Int, (Bool, Double, String), Result) -> Unit)
            #43 99-118 "(_, (_, 1.0, _), _)" : (Int, (Bool, Double, String), Result)
            #44 100-101 "_" : Int
            #45 103-114 "(_, 1.0, _)" : (Bool, Double, String)
            #46 104-105 "_" : Bool
            #47 107-110 "1.0" : Double
            #48 112-113 "_" : String
            #49 116-117 "_" : Result
        "##]],
    );
}

#[test]
fn partial_app_nested_tuple_singleton_unwrap() {
    check(
        "",
        indoc! {"{
            function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}
            let f = Foo(_, (true, 1.0, _), _);
        }"},
        &expect![[r##"
            #1 0-124 "{\n    function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}\n    let f = Foo(_, (true, 1.0, _), _);\n}" : Unit
            #2 0-124 "{\n    function Foo(a : Int, (b : Bool, c : Double, d : String), e : Result) : () {}\n    let f = Foo(_, (true, 1.0, _), _);\n}" : Unit
            #7 18-75 "(a : Int, (b : Bool, c : Double, d : String), e : Result)" : (Int, (Bool, Double, String), Result)
            #8 19-26 "a : Int" : Int
            #13 28-62 "(b : Bool, c : Double, d : String)" : (Bool, Double, String)
            #14 29-37 "b : Bool" : Bool
            #19 39-49 "c : Double" : Double
            #24 51-61 "d : String" : String
            #29 64-74 "e : Result" : Result
            #35 81-83 "{}" : Unit
            #37 92-93 "f" : ((Int, String, Result) -> Unit)
            #39 96-121 "Foo(_, (true, 1.0, _), _)" : ((Int, String, Result) -> Unit)
            #40 96-99 "Foo" : ((Int, (Bool, Double, String), Result) -> Unit)
            #43 99-121 "(_, (true, 1.0, _), _)" : (Int, (Bool, Double, String), Result)
            #44 100-101 "_" : Int
            #45 103-117 "(true, 1.0, _)" : (Bool, Double, String)
            #46 104-108 "true" : Bool
            #47 110-113 "1.0" : Double
            #48 115-116 "_" : String
            #49 119-120 "_" : Result
        "##]],
    );
}

#[test]
fn partial_app_too_many_args() {
    check(
        "",
        indoc! {"{
            function Foo(x : Int) : Int { x }
            let f = Foo(1, _, _);
        }"},
        &expect![[r##"
            #1 0-67 "{\n    function Foo(x : Int) : Int { x }\n    let f = Foo(1, _, _);\n}" : Unit
            #2 0-67 "{\n    function Foo(x : Int) : Int { x }\n    let f = Foo(1, _, _);\n}" : Unit
            #7 18-27 "(x : Int)" : Int
            #8 19-26 "x : Int" : Int
            #16 34-39 "{ x }" : Int
            #18 36-37 "x" : Int
            #22 48-49 "f" : Int
            #24 52-64 "Foo(1, _, _)" : Int
            #25 52-55 "Foo" : (Int -> Int)
            #28 55-64 "(1, _, _)" : (Int, ?1, ?2)
            #29 56-57 "1" : Int
            #30 59-60 "_" : ?1
            #31 62-63 "_" : ?2
            Error(Type(Error(TyMismatch(Prim(Int), Tuple([Prim(Int), Infer(InferTyId(1)), Infer(InferTyId(2))]), Span { lo: 52, hi: 64 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 59, hi: 60 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 62, hi: 63 }))))
        "##]],
    );
}

#[test]
fn typed_hole_error_concrete_type() {
    check(
        "",
        "_ + 3",
        &expect![[r##"
            #1 0-5 "_ + 3" : Int
            #2 0-1 "_" : Int
            #3 4-5 "3" : Int
            Error(Type(Error(TyHole(Prim(Int), Span { lo: 0, hi: 1 }))))
        "##]],
    );
}

#[test]
fn typed_hole_error_ambiguous_type() {
    check(
        "",
        "_(3)",
        &expect![[r##"
            #1 0-4 "_(3)" : ?1
            #2 0-1 "_" : ?0
            #3 1-4 "(3)" : Int
            #4 2-3 "3" : Int
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 1 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 0, hi: 4 }))))
            Error(Type(Error(TyHole(Infer(InferTyId(0)), Span { lo: 0, hi: 1 }))))
        "##]],
    );
}

#[test]
fn functors_in_arg_superset_of_empty() {
    check(
        "",
        "{
            operation Foo(op : Qubit => ()) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-137 "{\n            operation Foo(op : Qubit => ()) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            Foo(Bar);\n        }" : Unit
            #2 0-137 "{\n            operation Foo(op : Qubit => ()) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            Foo(Bar);\n        }" : Unit
            #7 27-45 "(op : Qubit => ())" : (Qubit => Unit)
            #8 28-44 "op : Qubit => ()" : (Qubit => Unit)
            #16 51-53 "{}" : Unit
            #21 79-90 "(q : Qubit)" : Qubit
            #22 80-89 "q : Qubit" : Qubit
            #29 103-105 "{}" : Unit
            #31 118-126 "Foo(Bar)" : Unit
            #32 118-121 "Foo" : ((Qubit => Unit is Adj) => Unit)
            #35 121-126 "(Bar)" : (Qubit => Unit is Adj)
            #36 122-125 "Bar" : (Qubit => Unit is Adj)
        "##]],
    );
}

#[test]
fn functors_in_arg_superset_of_adj() {
    check(
        "",
        "{
            operation Foo(op : Qubit => () is Adj) : () {}
            operation Bar(q : Qubit) : () is Adj + Ctl {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-150 "{\n            operation Foo(op : Qubit => () is Adj) : () {}\n            operation Bar(q : Qubit) : () is Adj + Ctl {}\n            Foo(Bar);\n        }" : Unit
            #2 0-150 "{\n            operation Foo(op : Qubit => () is Adj) : () {}\n            operation Bar(q : Qubit) : () is Adj + Ctl {}\n            Foo(Bar);\n        }" : Unit
            #7 27-52 "(op : Qubit => () is Adj)" : (Qubit => Unit is Adj)
            #8 28-51 "op : Qubit => () is Adj" : (Qubit => Unit is Adj)
            #17 58-60 "{}" : Unit
            #22 86-97 "(q : Qubit)" : Qubit
            #23 87-96 "q : Qubit" : Qubit
            #32 116-118 "{}" : Unit
            #34 131-139 "Foo(Bar)" : Unit
            #35 131-134 "Foo" : ((Qubit => Unit is Adj + Ctl) => Unit)
            #38 134-139 "(Bar)" : (Qubit => Unit is Adj + Ctl)
            #39 135-138 "Bar" : (Qubit => Unit is Adj + Ctl)
        "##]],
    );
}

#[test]
fn functors_in_arg_subset_of_ctl_adj() {
    check(
        "",
        "{
            operation Foo(op : Qubit => () is Adj + Ctl) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-150 "{\n            operation Foo(op : Qubit => () is Adj + Ctl) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            Foo(Bar);\n        }" : Unit
            #2 0-150 "{\n            operation Foo(op : Qubit => () is Adj + Ctl) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            Foo(Bar);\n        }" : Unit
            #7 27-58 "(op : Qubit => () is Adj + Ctl)" : (Qubit => Unit is Adj + Ctl)
            #8 28-57 "op : Qubit => () is Adj + Ctl" : (Qubit => Unit is Adj + Ctl)
            #19 64-66 "{}" : Unit
            #24 92-103 "(q : Qubit)" : Qubit
            #25 93-102 "q : Qubit" : Qubit
            #32 116-118 "{}" : Unit
            #34 131-139 "Foo(Bar)" : Unit
            #35 131-134 "Foo" : ((Qubit => Unit is Adj) => Unit)
            #38 134-139 "(Bar)" : (Qubit => Unit is Adj)
            #39 135-138 "Bar" : (Qubit => Unit is Adj)
            Error(Type(Error(MissingFunctor(Value(CtlAdj), Value(Adj), Span { lo: 131, hi: 139 }))))
        "##]],
    );
}

#[test]
fn functors_in_arg_eq_to_adj() {
    check(
        "",
        "{
            operation Foo(op : Qubit => () is Adj) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-144 "{\n            operation Foo(op : Qubit => () is Adj) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            Foo(Bar);\n        }" : Unit
            #2 0-144 "{\n            operation Foo(op : Qubit => () is Adj) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            Foo(Bar);\n        }" : Unit
            #7 27-52 "(op : Qubit => () is Adj)" : (Qubit => Unit is Adj)
            #8 28-51 "op : Qubit => () is Adj" : (Qubit => Unit is Adj)
            #17 58-60 "{}" : Unit
            #22 86-97 "(q : Qubit)" : Qubit
            #23 87-96 "q : Qubit" : Qubit
            #30 110-112 "{}" : Unit
            #32 125-133 "Foo(Bar)" : Unit
            #33 125-128 "Foo" : ((Qubit => Unit is Adj) => Unit)
            #36 128-133 "(Bar)" : (Qubit => Unit is Adj)
            #37 129-132 "Bar" : (Qubit => Unit is Adj)
        "##]],
    );
}

#[test]
fn functors_in_arg_nested_arrow_superset_of_adj() {
    check(
        "",
        "{
            operation Foo(op : (Qubit => () is Adj) => ()) : () {}
            operation Bar(op : Qubit => () is Adj + Ctl) : () {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-165 "{\n            operation Foo(op : (Qubit => () is Adj) => ()) : () {}\n            operation Bar(op : Qubit => () is Adj + Ctl) : () {}\n            Foo(Bar);\n        }" : Unit
            #2 0-165 "{\n            operation Foo(op : (Qubit => () is Adj) => ()) : () {}\n            operation Bar(op : Qubit => () is Adj + Ctl) : () {}\n            Foo(Bar);\n        }" : Unit
            #7 27-60 "(op : (Qubit => () is Adj) => ())" : ((Qubit => Unit is Adj) => Unit)
            #8 28-59 "op : (Qubit => () is Adj) => ()" : ((Qubit => Unit is Adj) => Unit)
            #20 66-68 "{}" : Unit
            #25 94-125 "(op : Qubit => () is Adj + Ctl)" : (Qubit => Unit is Adj + Ctl)
            #26 95-124 "op : Qubit => () is Adj + Ctl" : (Qubit => Unit is Adj + Ctl)
            #37 131-133 "{}" : Unit
            #39 146-154 "Foo(Bar)" : Unit
            #40 146-149 "Foo" : (((Qubit => Unit is Adj) => Unit) => Unit)
            #43 149-154 "(Bar)" : ((Qubit => Unit is Adj) => Unit)
            #44 150-153 "Bar" : ((Qubit => Unit is Adj) => Unit)
            Error(Type(Error(MissingFunctor(Value(CtlAdj), Value(Adj), Span { lo: 146, hi: 154 }))))
        "##]],
    );
}

#[test]
fn functors_in_arg_nested_arrow_subset_of_adj() {
    check(
        "",
        "{
            operation Foo(op : (Qubit => () is Adj) => ()) : () {}
            operation Bar(op : Qubit => ()) : () {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-152 "{\n            operation Foo(op : (Qubit => () is Adj) => ()) : () {}\n            operation Bar(op : Qubit => ()) : () {}\n            Foo(Bar);\n        }" : Unit
            #2 0-152 "{\n            operation Foo(op : (Qubit => () is Adj) => ()) : () {}\n            operation Bar(op : Qubit => ()) : () {}\n            Foo(Bar);\n        }" : Unit
            #7 27-60 "(op : (Qubit => () is Adj) => ())" : ((Qubit => Unit is Adj) => Unit)
            #8 28-59 "op : (Qubit => () is Adj) => ()" : ((Qubit => Unit is Adj) => Unit)
            #20 66-68 "{}" : Unit
            #25 94-112 "(op : Qubit => ())" : (Qubit => Unit)
            #26 95-111 "op : Qubit => ()" : (Qubit => Unit)
            #34 118-120 "{}" : Unit
            #36 133-141 "Foo(Bar)" : Unit
            #37 133-136 "Foo" : (((Qubit => Unit is Adj) => Unit) => Unit)
            #40 136-141 "(Bar)" : ((Qubit => Unit is Adj) => Unit)
            #41 137-140 "Bar" : ((Qubit => Unit is Adj) => Unit)
        "##]],
    );
}

#[test]
fn functors_in_arg_nested_arrow_eq_to_adj() {
    check(
        "",
        "{
            operation Foo(op : (Qubit => () is Adj) => ()) : () {}
            operation Bar(op : Qubit => () is Adj) : () {}
            Foo(Bar);
        }",
        &expect![[r##"
            #1 0-159 "{\n            operation Foo(op : (Qubit => () is Adj) => ()) : () {}\n            operation Bar(op : Qubit => () is Adj) : () {}\n            Foo(Bar);\n        }" : Unit
            #2 0-159 "{\n            operation Foo(op : (Qubit => () is Adj) => ()) : () {}\n            operation Bar(op : Qubit => () is Adj) : () {}\n            Foo(Bar);\n        }" : Unit
            #7 27-60 "(op : (Qubit => () is Adj) => ())" : ((Qubit => Unit is Adj) => Unit)
            #8 28-59 "op : (Qubit => () is Adj) => ()" : ((Qubit => Unit is Adj) => Unit)
            #20 66-68 "{}" : Unit
            #25 94-119 "(op : Qubit => () is Adj)" : (Qubit => Unit is Adj)
            #26 95-118 "op : Qubit => () is Adj" : (Qubit => Unit is Adj)
            #35 125-127 "{}" : Unit
            #37 140-148 "Foo(Bar)" : Unit
            #38 140-143 "Foo" : (((Qubit => Unit is Adj) => Unit) => Unit)
            #41 143-148 "(Bar)" : ((Qubit => Unit is Adj) => Unit)
            #42 144-147 "Bar" : ((Qubit => Unit is Adj) => Unit)
        "##]],
    );
}

#[test]
fn functors_in_arg_array_superset_of_adj() {
    check(
        "",
        "{
            operation Foo(ops : (Qubit => () is Adj)[]) : () {}
            operation Bar(q : Qubit) : () is Adj + Ctl {}
            Foo([Bar]);
        }",
        &expect![[r##"
            #1 0-157 "{\n            operation Foo(ops : (Qubit => () is Adj)[]) : () {}\n            operation Bar(q : Qubit) : () is Adj + Ctl {}\n            Foo([Bar]);\n        }" : Unit
            #2 0-157 "{\n            operation Foo(ops : (Qubit => () is Adj)[]) : () {}\n            operation Bar(q : Qubit) : () is Adj + Ctl {}\n            Foo([Bar]);\n        }" : Unit
            #7 27-57 "(ops : (Qubit => () is Adj)[])" : ((Qubit => Unit is Adj))[]
            #8 28-56 "ops : (Qubit => () is Adj)[]" : ((Qubit => Unit is Adj))[]
            #19 63-65 "{}" : Unit
            #24 91-102 "(q : Qubit)" : Qubit
            #25 92-101 "q : Qubit" : Qubit
            #34 121-123 "{}" : Unit
            #36 136-146 "Foo([Bar])" : Unit
            #37 136-139 "Foo" : (((Qubit => Unit is Adj + Ctl))[] => Unit)
            #40 139-146 "([Bar])" : ((Qubit => Unit is Adj + Ctl))[]
            #41 140-145 "[Bar]" : ((Qubit => Unit is Adj + Ctl))[]
            #42 141-144 "Bar" : (Qubit => Unit is Adj + Ctl)
        "##]],
    );
}

#[test]
fn functors_in_arg_array_subset_of_adj() {
    check(
        "",
        "{
            operation Foo(ops : (Qubit => () is Adj)[]) : () {}
            operation Bar(q : Qubit) : () {}
            Foo([Bar]);
        }",
        &expect![[r##"
            #1 0-144 "{\n            operation Foo(ops : (Qubit => () is Adj)[]) : () {}\n            operation Bar(q : Qubit) : () {}\n            Foo([Bar]);\n        }" : Unit
            #2 0-144 "{\n            operation Foo(ops : (Qubit => () is Adj)[]) : () {}\n            operation Bar(q : Qubit) : () {}\n            Foo([Bar]);\n        }" : Unit
            #7 27-57 "(ops : (Qubit => () is Adj)[])" : ((Qubit => Unit is Adj))[]
            #8 28-56 "ops : (Qubit => () is Adj)[]" : ((Qubit => Unit is Adj))[]
            #19 63-65 "{}" : Unit
            #24 91-102 "(q : Qubit)" : Qubit
            #25 92-101 "q : Qubit" : Qubit
            #31 108-110 "{}" : Unit
            #33 123-133 "Foo([Bar])" : Unit
            #34 123-126 "Foo" : (((Qubit => Unit))[] => Unit)
            #37 126-133 "([Bar])" : ((Qubit => Unit))[]
            #38 127-132 "[Bar]" : ((Qubit => Unit))[]
            #39 128-131 "Bar" : (Qubit => Unit)
            Error(Type(Error(MissingFunctor(Value(Adj), Value(Empty), Span { lo: 123, hi: 133 }))))
        "##]],
    );
}

#[test]
fn functors_in_array_all_same() {
    check(
        "",
        "{
            operation Foo(q : Qubit) : () is Adj {}
            let ops = [Foo, Foo, Foo];
        }",
        &expect![[r##"
            #1 0-102 "{\n            operation Foo(q : Qubit) : () is Adj {}\n            let ops = [Foo, Foo, Foo];\n        }" : Unit
            #2 0-102 "{\n            operation Foo(q : Qubit) : () is Adj {}\n            let ops = [Foo, Foo, Foo];\n        }" : Unit
            #7 27-38 "(q : Qubit)" : Qubit
            #8 28-37 "q : Qubit" : Qubit
            #15 51-53 "{}" : Unit
            #17 70-73 "ops" : ((Qubit => Unit is Adj))[]
            #19 76-91 "[Foo, Foo, Foo]" : ((Qubit => Unit is Adj))[]
            #20 77-80 "Foo" : (Qubit => Unit is Adj)
            #23 82-85 "Foo" : (Qubit => Unit is Adj)
            #26 87-90 "Foo" : (Qubit => Unit is Adj)
        "##]],
    );
}

#[test]
fn functors_in_array_mixed() {
    check(
        "",
        "{
            operation Foo(q : Qubit) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            operation Baz(q : Qubit) : () is Adj + Ctl {}
            let ops = [Foo, Bar, Baz];
        }",
        &expect![[r##"
            #1 0-205 "{\n            operation Foo(q : Qubit) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let ops = [Foo, Bar, Baz];\n        }" : Unit
            #2 0-205 "{\n            operation Foo(q : Qubit) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let ops = [Foo, Bar, Baz];\n        }" : Unit
            #7 27-38 "(q : Qubit)" : Qubit
            #8 28-37 "q : Qubit" : Qubit
            #14 44-46 "{}" : Unit
            #19 72-83 "(q : Qubit)" : Qubit
            #20 73-82 "q : Qubit" : Qubit
            #27 96-98 "{}" : Unit
            #32 124-135 "(q : Qubit)" : Qubit
            #33 125-134 "q : Qubit" : Qubit
            #42 154-156 "{}" : Unit
            #44 173-176 "ops" : ((Qubit => Unit))[]
            #46 179-194 "[Foo, Bar, Baz]" : ((Qubit => Unit))[]
            #47 180-183 "Foo" : (Qubit => Unit)
            #50 185-188 "Bar" : (Qubit => Unit is Adj)
            #53 190-193 "Baz" : (Qubit => Unit is Adj + Ctl)
            Error(Type(Error(FunctorMismatch(Value(Empty), Value(Adj), Span { lo: 185, hi: 188 }))))
            Error(Type(Error(FunctorMismatch(Value(Empty), Value(CtlAdj), Span { lo: 190, hi: 193 }))))
        "##]],
    );
}

#[test]
fn functors_in_array_mixed_lambda_all_empty() {
    check(
        "",
        "{
            operation Foo(q : Qubit) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            operation Baz(q : Qubit) : () is Adj + Ctl {}
            let ops = [Foo, q => Bar(q), q => Baz(q)];
        }",
        &expect![[r##"
            #1 0-221 "{\n            operation Foo(q : Qubit) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let ops = [Foo, q => Bar(q), q => Baz(q)];\n        }" : Unit
            #2 0-221 "{\n            operation Foo(q : Qubit) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let ops = [Foo, q => Bar(q), q => Baz(q)];\n        }" : Unit
            #7 27-38 "(q : Qubit)" : Qubit
            #8 28-37 "q : Qubit" : Qubit
            #14 44-46 "{}" : Unit
            #19 72-83 "(q : Qubit)" : Qubit
            #20 73-82 "q : Qubit" : Qubit
            #27 96-98 "{}" : Unit
            #32 124-135 "(q : Qubit)" : Qubit
            #33 125-134 "q : Qubit" : Qubit
            #42 154-156 "{}" : Unit
            #44 173-176 "ops" : ((Qubit => Unit))[]
            #46 179-210 "[Foo, q => Bar(q), q => Baz(q)]" : ((Qubit => Unit))[]
            #47 180-183 "Foo" : (Qubit => Unit)
            #50 185-196 "q => Bar(q)" : (Qubit => Unit)
            #51 185-186 "q" : Qubit
            #53 190-196 "Bar(q)" : Unit
            #54 190-193 "Bar" : (Qubit => Unit is Adj)
            #57 193-196 "(q)" : Qubit
            #58 194-195 "q" : Qubit
            #61 198-209 "q => Baz(q)" : (Qubit => Unit)
            #62 198-199 "q" : Qubit
            #64 203-209 "Baz(q)" : Unit
            #65 203-206 "Baz" : (Qubit => Unit is Adj + Ctl)
            #68 206-209 "(q)" : Qubit
            #69 207-208 "q" : Qubit
        "##]],
    );
}

#[test]
fn functors_in_array_mixed_lambda_all_ctl_adj() {
    check(
        "",
        "{
            operation Foo(q : Qubit) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            operation Baz(q : Qubit) : () is Adj + Ctl {}
            let ops = [q => Foo(q), q => Bar(q), Baz];
        }",
        &expect![[r##"
            #1 0-221 "{\n            operation Foo(q : Qubit) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let ops = [q => Foo(q), q => Bar(q), Baz];\n        }" : Unit
            #2 0-221 "{\n            operation Foo(q : Qubit) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let ops = [q => Foo(q), q => Bar(q), Baz];\n        }" : Unit
            #7 27-38 "(q : Qubit)" : Qubit
            #8 28-37 "q : Qubit" : Qubit
            #14 44-46 "{}" : Unit
            #19 72-83 "(q : Qubit)" : Qubit
            #20 73-82 "q : Qubit" : Qubit
            #27 96-98 "{}" : Unit
            #32 124-135 "(q : Qubit)" : Qubit
            #33 125-134 "q : Qubit" : Qubit
            #42 154-156 "{}" : Unit
            #44 173-176 "ops" : ((Qubit => Unit is Adj + Ctl))[]
            #46 179-210 "[q => Foo(q), q => Bar(q), Baz]" : ((Qubit => Unit is Adj + Ctl))[]
            #47 180-191 "q => Foo(q)" : (Qubit => Unit is Adj + Ctl)
            #48 180-181 "q" : Qubit
            #50 185-191 "Foo(q)" : Unit
            #51 185-188 "Foo" : (Qubit => Unit)
            #54 188-191 "(q)" : Qubit
            #55 189-190 "q" : Qubit
            #58 193-204 "q => Bar(q)" : (Qubit => Unit is Adj + Ctl)
            #59 193-194 "q" : Qubit
            #61 198-204 "Bar(q)" : Unit
            #62 198-201 "Bar" : (Qubit => Unit is Adj)
            #65 201-204 "(q)" : Qubit
            #66 202-203 "q" : Qubit
            #69 206-209 "Baz" : (Qubit => Unit is Adj + Ctl)
        "##]],
    );
}

#[test]
fn functors_in_arg_bound_to_let_becomes_monotype() {
    check(
        "",
        "{
            operation Foo(op : Qubit => () is Adj) : () {}
            operation Bar(q : Qubit) : () is Adj {}
            operation Baz(q : Qubit) : () is Adj + Ctl {}
            let foo = Foo;
            foo(Bar);
            foo(Baz);
        }",
        &expect![[r##"
            #1 0-251 "{\n            operation Foo(op : Qubit => () is Adj) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let foo = Foo;\n            foo(Bar);\n            foo(Baz);\n        }" : Unit
            #2 0-251 "{\n            operation Foo(op : Qubit => () is Adj) : () {}\n            operation Bar(q : Qubit) : () is Adj {}\n            operation Baz(q : Qubit) : () is Adj + Ctl {}\n            let foo = Foo;\n            foo(Bar);\n            foo(Baz);\n        }" : Unit
            #7 27-52 "(op : Qubit => () is Adj)" : (Qubit => Unit is Adj)
            #8 28-51 "op : Qubit => () is Adj" : (Qubit => Unit is Adj)
            #17 58-60 "{}" : Unit
            #22 86-97 "(q : Qubit)" : Qubit
            #23 87-96 "q : Qubit" : Qubit
            #30 110-112 "{}" : Unit
            #35 138-149 "(q : Qubit)" : Qubit
            #36 139-148 "q : Qubit" : Qubit
            #45 168-170 "{}" : Unit
            #47 187-190 "foo" : ((Qubit => Unit is Adj) => Unit)
            #49 193-196 "Foo" : ((Qubit => Unit is Adj) => Unit)
            #53 210-218 "foo(Bar)" : Unit
            #54 210-213 "foo" : ((Qubit => Unit is Adj) => Unit)
            #57 213-218 "(Bar)" : (Qubit => Unit is Adj)
            #58 214-217 "Bar" : (Qubit => Unit is Adj)
            #62 232-240 "foo(Baz)" : Unit
            #63 232-235 "foo" : ((Qubit => Unit is Adj) => Unit)
            #66 235-240 "(Baz)" : (Qubit => Unit is Adj + Ctl)
            #67 236-239 "Baz" : (Qubit => Unit is Adj + Ctl)
            Error(Type(Error(FunctorMismatch(Value(Adj), Value(CtlAdj), Span { lo: 232, hi: 240 }))))
        "##]],
    );
}

#[test]
fn duplicate_callable_decls_inferred_and_ignored() {
    check(
        indoc! {"
            namespace Test {
                function Foo() : Bool { true }
                function Foo() : Unit {}
                function Bar() : Unit {
                    let val = Foo();
                }
            }
        "},
        "",
        &expect![[r##"
            #6 33-35 "()" : Unit
            #10 43-51 "{ true }" : Bool
            #12 45-49 "true" : Bool
            #16 68-70 "()" : Unit
            #20 78-80 "{}" : Unit
            #24 97-99 "()" : Unit
            #28 107-139 "{\n        let val = Foo();\n    }" : Unit
            #30 121-124 "val" : Bool
            #32 127-132 "Foo()" : Bool
            #33 127-130 "Foo" : (Unit -> Bool)
            #36 130-132 "()" : Unit
            Error(Resolve(Duplicate("Foo", "Test", Span { lo: 65, hi: 68 })))
        "##]],
    );
}

#[test]
fn duplicate_type_decls_inferred_and_ignored() {
    check(
        indoc! {"
            namespace Test {
                newtype Foo = Bool;
                newtype Foo = Unit;
                function Bar() : Unit {
                    let val = Foo(true);
                }
            }
        "},
        "",
        &expect![[r##"
            #18 81-83 "()" : Unit
            #22 91-127 "{\n        let val = Foo(true);\n    }" : Unit
            #24 105-108 "val" : UDT<Item 1>
            #26 111-120 "Foo(true)" : UDT<Item 1>
            #27 111-114 "Foo" : (Bool -> UDT<Item 1>)
            #30 114-120 "(true)" : Bool
            #31 115-119 "true" : Bool
            Error(Resolve(Duplicate("Foo", "Test", Span { lo: 53, hi: 56 })))
        "##]],
    );
}

#[test]
fn instantiate_duplicate_ty_param_names() {
    check(
        "namespace Test { function Foo<'T, 'T>() : () { let f = Foo; } }",
        "",
        &expect![[r##"
            #8 37-39 "()" : Unit
            #10 45-61 "{ let f = Foo; }" : Unit
            #12 51-52 "f" : (Unit -> Unit)
            #14 55-58 "Foo" : (Unit -> Unit)
            Error(Type(Error(AmbiguousTy(Span { lo: 55, hi: 58 }))))
            Error(Type(Error(AmbiguousTy(Span { lo: 55, hi: 58 }))))
        "##]],
    );
}
#[test]
fn ambiguous_generic() {
    check(
        "namespace Test { 
            function Foo<'T>(x: 'T) : 'T { x }
            function Bar() : () { let x = Foo([]); } 
        }",
        "",
        &expect![[r##"
            #7 46-53 "(x: 'T)" : 0
            #8 47-52 "x: 'T" : 0
            #14 59-64 "{ x }" : 0
            #16 61-62 "x" : 0
            #22 89-91 "()" : Unit
            #24 97-117 "{ let x = Foo([]); }" : Unit
            #26 103-104 "x" : (?2)[]
            #28 107-114 "Foo([])" : (?2)[]
            #29 107-110 "Foo" : ((?2)[] -> (?2)[])
            #32 110-114 "([])" : (?2)[]
            #33 111-113 "[]" : (?2)[]
            Error(Type(Error(AmbiguousTy(Span { lo: 111, hi: 113 }))))
        "##]],
    );
}
#[test]
fn invalid_ident() {
    check(
        r#"namespace NS {
    function Foo() : () {
        let x : 'invalid = 0;
    }
}
        "#,
        "",
        &expect![[r##"
            #6 31-33 "()" : Unit
            #8 39-76 "{\n        let x : 'invalid = 0;\n    }" : Unit
            #10 53-65 "x : 'invalid" : ?
            #14 68-69 "0" : Int
            Error(Resolve(NotFound("'invalid", Span { lo: 57, hi: 65 })))
        "##]],
    );
}
#[test]
fn undeclared_generic_param() {
    check(
        r#"namespace c{operation y(g: 'U): Unit {} }"#,
        "",
        &expect![[r##"
            #6 23-30 "(g: 'U)" : ?
            #7 24-29 "g: 'U" : ?
            #14 37-39 "{}" : Unit
            Error(Resolve(NotFound("'U", Span { lo: 27, hi: 29 })))
        "##]],
    );
}
