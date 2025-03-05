// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_while() {
    check(
        parse,
        "
    while (x != 2) {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-42]:
                annotations: <empty>
                kind: WhileLoop [5-42]:
                    condition: Expr [12-18]: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [12-13]: Ident [12-13] "x"
                        rhs: Expr [17-18]: Lit: Int(2)
                    block:
                        Stmt [30-36]:
                            annotations: <empty>
                            kind: ExprStmt [30-36]:
                                expr: Expr [30-35]: AssignExpr:
                                    lhs: Expr [30-31]: Ident [30-31] "a"
                                    rhs: Expr [34-35]: Lit: Int(0)"#]],
    );
}

#[test]
fn empty_while() {
    check(
        parse,
        "while (true) {}",
        &expect![[r#"
            Stmt [0-15]:
                annotations: <empty>
                kind: WhileLoop [0-15]:
                    condition: Expr [7-11]: Lit: Bool(true)
                    block: <empty>"#]],
    );
}

#[test]
fn while_stmt_body() {
    check(
        parse,
        "
    while (x != 2)
        a = 0;",
        &expect![[r#"
            Stmt [5-34]:
                annotations: <empty>
                kind: WhileLoop [5-34]:
                    condition: Expr [12-18]: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [12-13]: Ident [12-13] "x"
                        rhs: Expr [17-18]: Lit: Int(2)
                    block:
                        Stmt [28-34]:
                            annotations: <empty>
                            kind: ExprStmt [28-34]:
                                expr: Expr [28-33]: AssignExpr:
                                    lhs: Expr [28-29]: Ident [28-29] "a"
                                    rhs: Expr [32-33]: Lit: Int(0)"#]],
    );
}

#[test]
fn while_loop_with_continue_stmt() {
    check(
        parse,
        "
    while (x != 2) {
        a = 0;
        continue;
    }",
        &expect![[r#"
            Stmt [5-60]:
                annotations: <empty>
                kind: WhileLoop [5-60]:
                    condition: Expr [12-18]: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [12-13]: Ident [12-13] "x"
                        rhs: Expr [17-18]: Lit: Int(2)
                    block:
                        Stmt [30-36]:
                            annotations: <empty>
                            kind: ExprStmt [30-36]:
                                expr: Expr [30-35]: AssignExpr:
                                    lhs: Expr [30-31]: Ident [30-31] "a"
                                    rhs: Expr [34-35]: Lit: Int(0)
                        Stmt [45-54]:
                            annotations: <empty>
                            kind: Continue [45-54]"#]],
    );
}

#[test]
fn while_loop_with_break_stmt() {
    check(
        parse,
        "
    while (x != 2) {
        a = 0;
        break;
    }",
        &expect![[r#"
            Stmt [5-57]:
                annotations: <empty>
                kind: WhileLoop [5-57]:
                    condition: Expr [12-18]: BinaryOpExpr:
                        op: Neq
                        lhs: Expr [12-13]: Ident [12-13] "x"
                        rhs: Expr [17-18]: Lit: Int(2)
                    block:
                        Stmt [30-36]:
                            annotations: <empty>
                            kind: ExprStmt [30-36]:
                                expr: Expr [30-35]: AssignExpr:
                                    lhs: Expr [30-31]: Ident [30-31] "a"
                                    rhs: Expr [34-35]: Lit: Int(0)
                        Stmt [45-51]:
                            annotations: <empty>
                            kind: Break [45-51]"#]],
    );
}
