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
                    if_block: Block [17-39]:
                        Stmt [27-33]:
                            annotations: <empty>
                            kind: AssignStmt [27-33]:
                                lhs: IndexedIdent [27-28]:
                                    name: Ident [27-28] "a"
                                    indices: <empty>
                                rhs: Expr [31-32]: Lit: Int(0)
                    else_block: Block [45-67]:
                        Stmt [55-61]:
                            annotations: <empty>
                            kind: AssignStmt [55-61]:
                                lhs: IndexedIdent [55-56]:
                                    name: Ident [55-56] "a"
                                    indices: <empty>
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
                    if_block: Block [17-39]:
                        Stmt [27-33]:
                            annotations: <empty>
                            kind: AssignStmt [27-33]:
                                lhs: IndexedIdent [27-28]:
                                    name: Ident [27-28] "a"
                                    indices: <empty>
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
                    if_block: Block [17-113]:
                        Stmt [27-107]:
                            annotations: <empty>
                            kind: IfStmt [27-107]:
                                condition: Expr [31-39]: BinaryOpExpr:
                                    op: Eq
                                    lhs: Expr [31-33]: Ident [31-33] "x1"
                                    rhs: Expr [37-39]: Ident [37-39] "y1"
                                if_block: Block [41-71]:
                                    Stmt [55-61]:
                                        annotations: <empty>
                                        kind: AssignStmt [55-61]:
                                            lhs: IndexedIdent [55-56]:
                                                name: Ident [55-56] "a"
                                                indices: <empty>
                                            rhs: Expr [59-60]: Lit: Int(0)
                                else_block: Block [77-107]:
                                    Stmt [91-97]:
                                        annotations: <empty>
                                        kind: AssignStmt [91-97]:
                                            lhs: IndexedIdent [91-92]:
                                                name: Ident [91-92] "a"
                                                indices: <empty>
                                            rhs: Expr [95-96]: Lit: Int(1)
                    else_block: Block [119-215]:
                        Stmt [129-209]:
                            annotations: <empty>
                            kind: IfStmt [129-209]:
                                condition: Expr [133-141]: BinaryOpExpr:
                                    op: Eq
                                    lhs: Expr [133-135]: Ident [133-135] "x2"
                                    rhs: Expr [139-141]: Ident [139-141] "y2"
                                if_block: Block [143-173]:
                                    Stmt [157-163]:
                                        annotations: <empty>
                                        kind: AssignStmt [157-163]:
                                            lhs: IndexedIdent [157-158]:
                                                name: Ident [157-158] "a"
                                                indices: <empty>
                                            rhs: Expr [161-162]: Lit: Int(2)
                                else_block: Block [179-209]:
                                    Stmt [193-199]:
                                        annotations: <empty>
                                        kind: AssignStmt [193-199]:
                                            lhs: IndexedIdent [193-194]:
                                                name: Ident [193-194] "a"
                                                indices: <empty>
                                            rhs: Expr [197-198]: Lit: Int(3)"#]],
    );
}
