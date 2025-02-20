use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn simple_while() {
    check(
        parse,
        "
    while (x != 2) {
        int a = 0;
    }",
        &expect![[r#"
            Stmt [5-46]
                StmtKind: WhileLoop [5-46]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [30-40]
                    StmtKind: ClassicalDeclarationStmt [30-40]: ClassicalType [30-33]: IntType [30-33], Ident [34-35] "a", ValueExpression ExprStmt [38-39]: Expr [38-39]: Lit: Int(0)"#]],
    );
}

#[test]
fn while_stmt_body() {
    check(
        parse,
        "
    while (x != 2)
        int a = 0;",
        &expect![[r#"
            Stmt [5-38]
                StmtKind: WhileLoop [5-38]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [28-38]
                    StmtKind: ClassicalDeclarationStmt [28-38]: ClassicalType [28-31]: IntType [28-31], Ident [32-33] "a", ValueExpression ExprStmt [36-37]: Expr [36-37]: Lit: Int(0)"#]],
    );
}

#[test]
fn while_loop_with_continue_stmt() {
    check(
        parse,
        "
    while (x != 2) {
        int a = 0;
        continue;
    }",
        &expect![[r#"
            Stmt [5-64]
                StmtKind: WhileLoop [5-64]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [30-40]
                    StmtKind: ClassicalDeclarationStmt [30-40]: ClassicalType [30-33]: IntType [30-33], Ident [34-35] "a", ValueExpression ExprStmt [38-39]: Expr [38-39]: Lit: Int(0)
                Stmt [49-58]
                    StmtKind: Continue [49-58]"#]],
    );
}

#[test]
fn while_loop_with_break_stmt() {
    check(
        parse,
        "
    while (x != 2) {
        int a = 0;
        break;
    }",
        &expect![[r#"
            Stmt [5-61]
                StmtKind: WhileLoop [5-61]: Expr [11-19]: Paren:
                    Expr [12-18]: BinOp (Neq):
                        Expr [12-13]: Ident [12-13] "x"
                        Expr [17-18]: Lit: Int(2)
                Stmt [30-40]
                    StmtKind: ClassicalDeclarationStmt [30-40]: ClassicalType [30-33]: IntType [30-33], Ident [34-35] "a", ValueExpression ExprStmt [38-39]: Expr [38-39]: Lit: Int(0)
                Stmt [49-55]
                    StmtKind: Break [49-55]"#]],
    );
}
