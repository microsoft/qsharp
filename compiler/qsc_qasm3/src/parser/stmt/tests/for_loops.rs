use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_for_loop() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-50]
                StmtKind: ForStmt [5-50]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", DiscreteSet [18-27]:
                    Expr [19-20]: Lit: Int(1)
                    Expr [22-23]: Lit: Int(2)
                    Expr [25-26]: Lit: Int(3)
                Stmt [38-44]
                    StmtKind: ExprStmt [38-44]: Expr [38-43]: Assign:
                        Expr [38-39]: Ident [38-39] "a"
                        Expr [42-43]: Lit: Int(0)"#]],
    );
}

#[test]
fn simple_for_loop_stmt_body() {
    check(
        parse,
        "
    for int x in {1, 2, 3}
        a = 0;
    ",
        &expect![[r#"
            Stmt [5-42]
                StmtKind: ForStmt [5-42]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", DiscreteSet [18-27]:
                    Expr [19-20]: Lit: Int(1)
                    Expr [22-23]: Lit: Int(2)
                    Expr [25-26]: Lit: Int(3)
                Stmt [36-42]
                    StmtKind: ExprStmt [36-42]: Expr [36-41]: Assign:
                        Expr [36-37]: Ident [36-37] "a"
                        Expr [40-41]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_range() {
    check(
        parse,
        "
    for int x in [0:2:7] {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-48]
                StmtKind: ForStmt [5-48]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", Range: [18-25]
                    start: Expr [19-20]: Lit: Int(0)
                    step: Expr [21-22]: Lit: Int(2)
                    end: Expr [23-24]: Lit: Int(7)
                Stmt [36-42]
                    StmtKind: ExprStmt [36-42]: Expr [36-41]: Assign:
                        Expr [36-37]: Ident [36-37] "a"
                        Expr [40-41]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_range_no_step() {
    check(
        parse,
        "
    for int x in [0:7] {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-46]
                StmtKind: ForStmt [5-46]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", Range: [18-23]
                    start: Expr [19-20]: Lit: Int(0)
                    <no step>
                    end: Expr [21-22]: Lit: Int(7)
                Stmt [34-40]
                    StmtKind: ExprStmt [34-40]: Expr [34-39]: Assign:
                        Expr [34-35]: Ident [34-35] "a"
                        Expr [38-39]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_expr() {
    check(
        parse,
        "
    for int x in xs {
        a = 0;
    }",
        &expect![[r#"
            Stmt [5-43]
                StmtKind: ForStmt [5-43]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", Expr [18-20]: Ident [18-20] "xs"
                Stmt [31-37]
                    StmtKind: ExprStmt [31-37]: Expr [31-36]: Assign:
                        Expr [31-32]: Ident [31-32] "a"
                        Expr [35-36]: Lit: Int(0)"#]],
    );
}

#[test]
fn for_loop_with_continue_stmt() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        a = 0;
        continue;
    }",
        &expect![[r#"
            Stmt [5-68]
                StmtKind: ForStmt [5-68]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", DiscreteSet [18-27]:
                    Expr [19-20]: Lit: Int(1)
                    Expr [22-23]: Lit: Int(2)
                    Expr [25-26]: Lit: Int(3)
                Stmt [38-44]
                    StmtKind: ExprStmt [38-44]: Expr [38-43]: Assign:
                        Expr [38-39]: Ident [38-39] "a"
                        Expr [42-43]: Lit: Int(0)
                Stmt [53-62]
                    StmtKind: Continue [53-62]"#]],
    );
}

#[test]
fn for_loop_with_break_stmt() {
    check(
        parse,
        "
    for int x in {1, 2, 3} {
        a = 0;
        break;
    }",
        &expect![[r#"
            Stmt [5-65]
                StmtKind: ForStmt [5-65]: ClassicalType [9-12]: IntType [9-12], Ident [13-14] "x", DiscreteSet [18-27]:
                    Expr [19-20]: Lit: Int(1)
                    Expr [22-23]: Lit: Int(2)
                    Expr [25-26]: Lit: Int(3)
                Stmt [38-44]
                    StmtKind: ExprStmt [38-44]: Expr [38-43]: Assign:
                        Expr [38-39]: Ident [38-39] "a"
                        Expr [42-43]: Lit: Int(0)
                Stmt [53-59]
                    StmtKind: Break [53-59]"#]],
    );
}
