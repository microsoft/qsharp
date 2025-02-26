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
            Stmt [5-67]
                StmtKind: IfStmt [5-67]: Expr [8-16]: Paren:
                    Expr [9-15]: BinOp (Eq):
                        Expr [9-10]: Ident [9-10] "x"
                        Expr [14-15]: Ident [14-15] "y"
                Stmt [27-33]
                    StmtKind: ExprStmt [27-33]: Expr [27-32]: Assign:
                        Expr [27-28]: Ident [27-28] "a"
                        Expr [31-32]: Lit: Int(0)
                Else:
                Stmt [55-61]
                    StmtKind: ExprStmt [55-61]: Expr [55-60]: Assign:
                        Expr [55-56]: Ident [55-56] "a"
                        Expr [59-60]: Lit: Int(1)"#]],
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
            Stmt [5-39]
                StmtKind: IfStmt [5-39]: Expr [8-16]: Paren:
                    Expr [9-15]: BinOp (Eq):
                        Expr [9-10]: Ident [9-10] "x"
                        Expr [14-15]: Ident [14-15] "y"
                Stmt [27-33]
                    StmtKind: ExprStmt [27-33]: Expr [27-32]: Assign:
                        Expr [27-28]: Ident [27-28] "a"
                        Expr [31-32]: Lit: Int(0)"#]],
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
            Stmt [5-215]
                StmtKind: IfStmt [5-215]: Expr [8-16]: Paren:
                    Expr [9-15]: BinOp (Eq):
                        Expr [9-10]: Ident [9-10] "x"
                        Expr [14-15]: Ident [14-15] "y"
                Stmt [27-107]
                    StmtKind: IfStmt [27-107]: Expr [30-40]: Paren:
                        Expr [31-39]: BinOp (Eq):
                            Expr [31-33]: Ident [31-33] "x1"
                            Expr [37-39]: Ident [37-39] "y1"
                    Stmt [55-61]
                        StmtKind: ExprStmt [55-61]: Expr [55-60]: Assign:
                            Expr [55-56]: Ident [55-56] "a"
                            Expr [59-60]: Lit: Int(0)
                    Else:
                    Stmt [91-97]
                        StmtKind: ExprStmt [91-97]: Expr [91-96]: Assign:
                            Expr [91-92]: Ident [91-92] "a"
                            Expr [95-96]: Lit: Int(1)
                Else:
                Stmt [129-209]
                    StmtKind: IfStmt [129-209]: Expr [132-142]: Paren:
                        Expr [133-141]: BinOp (Eq):
                            Expr [133-135]: Ident [133-135] "x2"
                            Expr [139-141]: Ident [139-141] "y2"
                    Stmt [157-163]
                        StmtKind: ExprStmt [157-163]: Expr [157-162]: Assign:
                            Expr [157-158]: Ident [157-158] "a"
                            Expr [161-162]: Lit: Int(2)
                    Else:
                    Stmt [193-199]
                        StmtKind: ExprStmt [193-199]: Expr [193-198]: Assign:
                            Expr [193-194]: Ident [193-194] "a"
                            Expr [197-198]: Lit: Int(3)"#]],
    );
}
