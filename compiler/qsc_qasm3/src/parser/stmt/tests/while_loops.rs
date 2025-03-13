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
                    block: Block [20-42]:
                        Stmt [30-36]:
                            annotations: <empty>
                            kind: AssignStmt [30-36]:
                                lhs: IndexedIdent [30-31]:
                                    name: Ident [30-31] "a"
                                    indices: <empty>
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
                    block: Block [13-15]: <empty>"#]],
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
                    block: Block [28-34]:
                        Stmt [28-34]:
                            annotations: <empty>
                            kind: AssignStmt [28-34]:
                                lhs: IndexedIdent [28-29]:
                                    name: Ident [28-29] "a"
                                    indices: <empty>
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
                    block: Block [20-60]:
                        Stmt [30-36]:
                            annotations: <empty>
                            kind: AssignStmt [30-36]:
                                lhs: IndexedIdent [30-31]:
                                    name: Ident [30-31] "a"
                                    indices: <empty>
                                rhs: Expr [34-35]: Lit: Int(0)
                        Stmt [45-54]:
                            annotations: <empty>
                            kind: ContinueStmt [45-54]"#]],
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
                    block: Block [20-57]:
                        Stmt [30-36]:
                            annotations: <empty>
                            kind: AssignStmt [30-36]:
                                lhs: IndexedIdent [30-31]:
                                    name: Ident [30-31] "a"
                                    indices: <empty>
                                rhs: Expr [34-35]: Lit: Int(0)
                        Stmt [45-51]:
                            annotations: <empty>
                            kind: BreakStmt [45-51]"#]],
    );
}
