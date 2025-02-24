// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::stmt::parse;
use crate::parser::tests::check;
use expect_test::expect;

#[test]
fn identifier() {
    check(
        parse,
        "H;",
        &expect![[r#"
            Stmt [0-2]
                StmtKind: ExprStmt [0-2]: Expr [0-1]: Ident [0-1] "H""#]],
    );
}

#[test]
fn identifier_plus_number() {
    check(
        parse,
        "H + 2;",
        &expect![[r#"
            Stmt [0-6]
                StmtKind: ExprStmt [0-6]: Expr [0-5]: BinOp (Add):
                    Expr [0-1]: Ident [0-1] "H"
                    Expr [4-5]: Lit: Int(2)"#]],
    );
}

#[test]
fn function_call() {
    check(
        parse,
        "f(2);",
        &expect![[r#"
            Stmt [0-5]
                StmtKind: ExprStmt [0-5]: Expr [0-4]: FunctionCall [0-4]: Ident [0-1] "f"
                    Expr [2-3]: Lit: Int(2)"#]],
    );
}
