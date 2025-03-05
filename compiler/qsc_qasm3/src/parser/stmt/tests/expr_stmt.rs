// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn identifier() {
    check(
        parse,
        "H;",
        &expect![[r#"
            Stmt [0-2]:
                annotations: <empty>
                kind: ExprStmt [0-2]:
                    expr: Expr [0-1]: Ident [0-1] "H""#]],
    );
}

#[test]
fn identifier_plus_number() {
    check(
        parse,
        "H + 2;",
        &expect![[r#"
            Stmt [0-6]:
                annotations: <empty>
                kind: ExprStmt [0-6]:
                    expr: Expr [0-5]: BinaryOpExpr:
                        op: Add
                        lhs: Expr [0-1]: Ident [0-1] "H"
                        rhs: Expr [4-5]: Lit: Int(2)"#]],
    );
}

// These are negative unit tests for gate calls:

#[test]
fn function_call_plus_ident() {
    check(
        parse,
        "Name(2, 3) + a;",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: ExprStmt [0-15]:
                    expr: Expr [0-14]: BinaryOpExpr:
                        op: Add
                        lhs: Expr [0-10]: FunctionCall [0-10]:
                            name: Ident [0-4] "Name"
                            args:
                                Expr [5-6]: Lit: Int(2)
                                Expr [8-9]: Lit: Int(3)
                        rhs: Expr [13-14]: Ident [13-14] "a""#]],
    );
}

#[test]
fn function_call() {
    check(
        parse,
        "Name(2, 3);",
        &expect![[r#"
            Stmt [0-11]:
                annotations: <empty>
                kind: ExprStmt [0-11]:
                    expr: Expr [0-10]: FunctionCall [0-10]:
                        name: Ident [0-4] "Name"
                        args:
                            Expr [5-6]: Lit: Int(2)
                            Expr [8-9]: Lit: Int(3)"#]],
    );
}

#[test]
fn indexed_function_call() {
    check(
        parse,
        "Name(2, 3)[1];",
        &expect![[r#"
            Stmt [0-14]:
                annotations: <empty>
                kind: ExprStmt [0-14]:
                    expr: Expr [0-13]: IndexExpr [10-13]:
                        collection: Expr [0-10]: FunctionCall [0-10]:
                            name: Ident [0-4] "Name"
                            args:
                                Expr [5-6]: Lit: Int(2)
                                Expr [8-9]: Lit: Int(3)
                        index: IndexElement:
                            IndexSetItem Expr [11-12]: Lit: Int(1)"#]],
    );
}

#[test]
fn multi_indexed_function_call() {
    check(
        parse,
        "Name(2, 3)[1, 0];",
        &expect![[r#"
            Stmt [0-17]:
                annotations: <empty>
                kind: ExprStmt [0-17]:
                    expr: Expr [0-16]: IndexExpr [10-16]:
                        collection: Expr [0-10]: FunctionCall [0-10]:
                            name: Ident [0-4] "Name"
                            args:
                                Expr [5-6]: Lit: Int(2)
                                Expr [8-9]: Lit: Int(3)
                        index: IndexElement:
                            IndexSetItem Expr [11-12]: Lit: Int(1)
                            IndexSetItem Expr [14-15]: Lit: Int(0)"#]],
    );
}

#[test]
fn ident() {
    check(
        parse,
        "Name;",
        &expect![[r#"
            Stmt [0-5]:
                annotations: <empty>
                kind: ExprStmt [0-5]:
                    expr: Expr [0-4]: Ident [0-4] "Name""#]],
    );
}

#[test]
fn index_expr() {
    check(
        parse,
        "Name[1];",
        &expect![[r#"
            Stmt [0-8]:
                annotations: <empty>
                kind: ExprStmt [0-8]:
                    expr: Expr [0-7]: IndexExpr [4-7]:
                        collection: Expr [0-4]: Ident [0-4] "Name"
                        index: IndexElement:
                            IndexSetItem Expr [5-6]: Lit: Int(1)"#]],
    );
}

#[test]
fn cast_expr() {
    check(
        parse,
        "bit(0);",
        &expect![[r#"
            Stmt [0-7]:
                annotations: <empty>
                kind: ExprStmt [0-7]:
                    expr: Expr [0-6]: Cast [0-6]:
                        type: ScalarType [0-3]: BitType [0-3]:
                            size: <none>
                        arg: Expr [4-5]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_expr_with_designator() {
    check(
        parse,
        "bit[45](0);",
        &expect![[r#"
            Stmt [0-11]:
                annotations: <empty>
                kind: ExprStmt [0-11]:
                    expr: Expr [0-10]: Cast [0-10]:
                        type: ScalarType [0-7]: BitType [0-7]:
                            size: Expr [4-6]: Lit: Int(45)
                        arg: Expr [8-9]: Lit: Int(0)"#]],
    );
}
