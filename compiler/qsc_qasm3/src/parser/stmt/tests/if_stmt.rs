use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_if_stmt() {
    check(
        parse,
        "
    if (x == y) {
        int a = 0;
    } else {
        int a = 1;
    }
    ",
        &expect![[r#"
            Stmt [5-75]
                StmtKind: IfStmt [5-75]: Expr [8-16]: Paren:
                    Expr [9-15]: BinOp (Eq):
                        Expr [9-10]: Ident [9-10] "x"
                        Expr [14-15]: Ident [14-15] "y"
                Stmt [27-37]
                    StmtKind: ClassicalDeclarationStmt [27-37]: ClassicalType [27-30]: IntType [27-30], Ident [31-32] "a", ValueExpression ExprStmt [35-36]: Expr [35-36]: Lit: Int(0)
                Else:
                Stmt [59-69]
                    StmtKind: ClassicalDeclarationStmt [59-69]: ClassicalType [59-62]: IntType [59-62], Ident [63-64] "a", ValueExpression ExprStmt [67-68]: Expr [67-68]: Lit: Int(1)"#]],
    );
}

#[test]
fn if_stmt_missing_else() {
    check(
        parse,
        "
    if (x == y) {
        int a = 0;
    }
    ",
        &expect![[r#"
            Stmt [5-43]
                StmtKind: IfStmt [5-43]: Expr [8-16]: Paren:
                    Expr [9-15]: BinOp (Eq):
                        Expr [9-10]: Ident [9-10] "x"
                        Expr [14-15]: Ident [14-15] "y"
                Stmt [27-37]
                    StmtKind: ClassicalDeclarationStmt [27-37]: ClassicalType [27-30]: IntType [27-30], Ident [31-32] "a", ValueExpression ExprStmt [35-36]: Expr [35-36]: Lit: Int(0)"#]],
    );
}

#[test]
fn nested_if_stmts() {
    check(
        parse,
        "
    if (x == y) {
        if (x1 == y1) {
            int a = 0;
        } else {
            int a = 1;
        }
    } else {
        if (x2 == y2) {
            int a = 2;
        } else {
            int a = 3;
        }
    }
    ",
        &expect![[r#"
            Stmt [5-231]
                StmtKind: IfStmt [5-231]: Expr [8-16]: Paren:
                    Expr [9-15]: BinOp (Eq):
                        Expr [9-10]: Ident [9-10] "x"
                        Expr [14-15]: Ident [14-15] "y"
                Stmt [27-115]
                    StmtKind: IfStmt [27-115]: Expr [30-40]: Paren:
                        Expr [31-39]: BinOp (Eq):
                            Expr [31-33]: Ident [31-33] "x1"
                            Expr [37-39]: Ident [37-39] "y1"
                    Stmt [55-65]
                        StmtKind: ClassicalDeclarationStmt [55-65]: ClassicalType [55-58]: IntType [55-58], Ident [59-60] "a", ValueExpression ExprStmt [63-64]: Expr [63-64]: Lit: Int(0)
                    Else:
                    Stmt [95-105]
                        StmtKind: ClassicalDeclarationStmt [95-105]: ClassicalType [95-98]: IntType [95-98], Ident [99-100] "a", ValueExpression ExprStmt [103-104]: Expr [103-104]: Lit: Int(1)
                Else:
                Stmt [137-225]
                    StmtKind: IfStmt [137-225]: Expr [140-150]: Paren:
                        Expr [141-149]: BinOp (Eq):
                            Expr [141-143]: Ident [141-143] "x2"
                            Expr [147-149]: Ident [147-149] "y2"
                    Stmt [165-175]
                        StmtKind: ClassicalDeclarationStmt [165-175]: ClassicalType [165-168]: IntType [165-168], Ident [169-170] "a", ValueExpression ExprStmt [173-174]: Expr [173-174]: Lit: Int(2)
                    Else:
                    Stmt [205-215]
                        StmtKind: ClassicalDeclarationStmt [205-215]: ClassicalType [205-208]: IntType [205-208], Ident [209-210] "a", ValueExpression ExprStmt [213-214]: Expr [213-214]: Lit: Int(3)"#]],
    );
}
