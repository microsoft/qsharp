// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn creg_decl() {
    check(
        parse,
        "creg c;",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: ClassicalDeclarationStmt [0-7]: ClassicalType [0-7]: BitType, Ident [5-6] "c""#]],
    );
}

#[test]
fn creg_array_decl() {
    check(
        parse,
        "creg c[n];",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: ClassicalDeclarationStmt [0-10]: ClassicalType [0-10]: BitType [0-10]: Expr [7-8]: Ident [7-8] "n", Ident [5-6] "c""#]],
    );
}

#[test]
fn qreg_decl() {
    check(
        parse,
        "qreg q;",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: QubitDeclaration [0-7]: Ident [5-6] "q""#]],
    );
}

#[test]
fn qreg_array_decl() {
    check(
        parse,
        "qreg q[n];",
        &expect![[r#"
            Stmt [0-10]
                StmtKind: QubitDeclaration [0-10]: Ident [5-6] "q", Expr [7-8]: Ident [7-8] "n""#]],
    );
}
