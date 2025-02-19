use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_for_loop() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        int a = 0;
    }",
        &expect![[r#"
            Stmt [5-54]
                StmtKind: ForStmt [5-54]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", DiscreteSet [18-27]:
                    Expr [19-20]: Lit: Int(1)
                    Expr [22-23]: Lit: Int(2)
                    Expr [25-26]: Lit: Int(3)
                Stmt [38-48]
                    StmtKind: ClassicalDeclarationStmt [38-48]: ClassicalType [38-41]: IntType [38-41], Ident [42-43] "a", ValueExpression ExprStmt [46-47]: Expr [46-47]: Lit: Int(0)"#]],
    );
}

#[test]
fn simple_for_loop_stmt_body() {
    check(
        parse,
        "
    for int x in {1, 2, 3}
        int a = 0;
    ",
        &expect![[r#"
            Stmt [5-46]
                StmtKind: ForStmt [5-46]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", DiscreteSet [18-27]:
                    Expr [19-20]: Lit: Int(1)
                    Expr [22-23]: Lit: Int(2)
                    Expr [25-26]: Lit: Int(3)
                Stmt [36-46]
                    StmtKind: ClassicalDeclarationStmt [36-46]: ClassicalType [36-39]: IntType [36-39], Ident [40-41] "a", ValueExpression ExprStmt [44-45]: Expr [44-45]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_range() {
    check(
        parse,
        "
    for int x in [0:2:7] {
        int a = 0;
    }",
        &expect![[r#"
            Stmt [5-52]
                StmtKind: ForStmt [5-52]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", Range: [18-25]
                    Expr [19-20]: Lit: Int(0)
                    Expr [21-22]: Lit: Int(2)
                    Expr [23-24]: Lit: Int(7)
                Stmt [36-46]
                    StmtKind: ClassicalDeclarationStmt [36-46]: ClassicalType [36-39]: IntType [36-39], Ident [40-41] "a", ValueExpression ExprStmt [44-45]: Expr [44-45]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_range_no_step() {
    check(
        parse,
        "
    for int x in [0:7] {
        int a = 0;
    }",
        &expect![[r#"
            Stmt [5-50]
                StmtKind: ForStmt [5-50]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", Range: [18-23]
                    Expr [19-20]: Lit: Int(0)
                    <no step>
                    Expr [21-22]: Lit: Int(7)
                Stmt [34-44]
                    StmtKind: ClassicalDeclarationStmt [34-44]: ClassicalType [34-37]: IntType [34-37], Ident [38-39] "a", ValueExpression ExprStmt [42-43]: Expr [42-43]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_expr() {
    check(
        parse,
        "
    for int x in xs {
        int a = 0;
    }",
        &expect![[r#"
            Stmt [5-47]
                StmtKind: ForStmt [5-47]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", Expr [18-20]: Ident [18-20] "xs"
                Stmt [31-41]
                    StmtKind: ClassicalDeclarationStmt [31-41]: ClassicalType [31-34]: IntType [31-34], Ident [35-36] "a", ValueExpression ExprStmt [39-40]: Expr [39-40]: Lit: Int(0)"#]],
    );
}
