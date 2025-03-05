// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_if_stmt() {
    check(
        parse,
        "
    if (x == y) {
        a = 0;
    } else {
        a = 1;
    }
    ",
        &expect![[r#"
            Stmt [5-67]:
                annotations: <empty>
                kind: IfStmt [5-67]:
                    condition: Expr [9-15]: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [9-10]: Ident [9-10] "x"
                        rhs: Expr [14-15]: Ident [14-15] "y"
                    if_block: 
                        Stmt [27-33]:
                            annotations: <empty>
                            kind: ExprStmt [27-33]:
                                expr: Expr [27-32]: AssignExpr:
                                    lhs: Expr [27-28]: Ident [27-28] "a"
                                    rhs: Expr [31-32]: Lit: Int(0)
                    else_block: 
                        Stmt [55-61]:
                            annotations: <empty>
                            kind: ExprStmt [55-61]:
                                expr: Expr [55-60]: AssignExpr:
                                    lhs: Expr [55-56]: Ident [55-56] "a"
                                    rhs: Expr [59-60]: Lit: Int(1)"#]],
    );
}

#[test]
fn if_stmt_missing_else() {
    check(
        parse,
        "
    if (x == y) {
        a = 0;
    }
    ",
        &expect![[r#"
            Stmt [5-39]:
                annotations: <empty>
                kind: IfStmt [5-39]:
                    condition: Expr [9-15]: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [9-10]: Ident [9-10] "x"
                        rhs: Expr [14-15]: Ident [14-15] "y"
                    if_block: 
                        Stmt [27-33]:
                            annotations: <empty>
                            kind: ExprStmt [27-33]:
                                expr: Expr [27-32]: AssignExpr:
                                    lhs: Expr [27-28]: Ident [27-28] "a"
                                    rhs: Expr [31-32]: Lit: Int(0)
                    else_block: <none>"#]],
    );
}

#[test]
fn nested_if_stmts() {
    check(
        parse,
        "
    if (x == y) {
        if (x1 == y1) {
            a = 0;
        } else {
            a = 1;
        }
    } else {
        if (x2 == y2) {
            a = 2;
        } else {
            a = 3;
        }
    }
    ",
        &expect![[r#"
            Stmt [5-215]:
                annotations: <empty>
                kind: IfStmt [5-215]:
                    condition: Expr [9-15]: BinaryOpExpr:
                        op: Eq
                        lhs: Expr [9-10]: Ident [9-10] "x"
                        rhs: Expr [14-15]: Ident [14-15] "y"
                    if_block: 
                        Stmt [27-107]:
                            annotations: <empty>
                            kind: IfStmt [27-107]:
                                condition: Expr [31-39]: BinaryOpExpr:
                                    op: Eq
                                    lhs: Expr [31-33]: Ident [31-33] "x1"
                                    rhs: Expr [37-39]: Ident [37-39] "y1"
                                if_block: 
                                    Stmt [55-61]:
                                        annotations: <empty>
                                        kind: ExprStmt [55-61]:
                                            expr: Expr [55-60]: AssignExpr:
                                                lhs: Expr [55-56]: Ident [55-56] "a"
                                                rhs: Expr [59-60]: Lit: Int(0)
                                else_block: 
                                    Stmt [91-97]:
                                        annotations: <empty>
                                        kind: ExprStmt [91-97]:
                                            expr: Expr [91-96]: AssignExpr:
                                                lhs: Expr [91-92]: Ident [91-92] "a"
                                                rhs: Expr [95-96]: Lit: Int(1)
                    else_block: 
                        Stmt [129-209]:
                            annotations: <empty>
                            kind: IfStmt [129-209]:
                                condition: Expr [133-141]: BinaryOpExpr:
                                    op: Eq
                                    lhs: Expr [133-135]: Ident [133-135] "x2"
                                    rhs: Expr [139-141]: Ident [139-141] "y2"
                                if_block: 
                                    Stmt [157-163]:
                                        annotations: <empty>
                                        kind: ExprStmt [157-163]:
                                            expr: Expr [157-162]: AssignExpr:
                                                lhs: Expr [157-158]: Ident [157-158] "a"
                                                rhs: Expr [161-162]: Lit: Int(2)
                                else_block: 
                                    Stmt [193-199]:
                                        annotations: <empty>
                                        kind: ExprStmt [193-199]:
                                            expr: Expr [193-198]: AssignExpr:
                                                lhs: Expr [193-194]: Ident [193-194] "a"
                                                rhs: Expr [197-198]: Lit: Int(3)"#]],
    );
}
