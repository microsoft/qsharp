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
            Stmt [5-42]
                StmtKind: WhileLoop [5-42]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [30-36]
                    StmtKind: ExprStmt [30-36]: Expr [30-35]: Assign:
                        Expr [30-31]: Ident [30-31] "a"
                        Expr [34-35]: Lit: Int(0)"#]],
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
            Stmt [5-34]
                StmtKind: WhileLoop [5-34]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [28-34]
                    StmtKind: ExprStmt [28-34]: Expr [28-33]: Assign:
                        Expr [28-29]: Ident [28-29] "a"
                        Expr [32-33]: Lit: Int(0)"#]],
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
            Stmt [5-60]
                StmtKind: WhileLoop [5-60]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [30-36]
                    StmtKind: ExprStmt [30-36]: Expr [30-35]: Assign:
                        Expr [30-31]: Ident [30-31] "a"
                        Expr [34-35]: Lit: Int(0)
                Stmt [45-54]
                    StmtKind: Continue [45-54]"#]],
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
            Stmt [5-57]
                StmtKind: WhileLoop [5-57]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [30-36]
                    StmtKind: ExprStmt [30-36]: Expr [30-35]: Assign:
                        Expr [30-31]: Ident [30-31] "a"
                        Expr [34-35]: Lit: Int(0)
                Stmt [45-51]
                    StmtKind: Break [45-51]"#]],
    );
}
