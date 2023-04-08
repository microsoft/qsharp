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
fn call_generic_function() {
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
