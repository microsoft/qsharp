// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

#[test]
fn subtraction() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a - b);
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        InputDeclaration [9-32]:
            symbol_id: 8
        InputDeclaration [41-64]:
            symbol_id: 9
        ClassicalDeclarationStmt [73-93]:
            symbol_id: 10
            ty_span: [73-80]
            init_expr: Expr [85-92]:
                ty: Complex(None, false)
                kind: Paren Expr [86-91]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [86-87]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [90-91]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
    "#]],
    );
}

#[test]
fn addition() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a + b);
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        InputDeclaration [9-32]:
            symbol_id: 8
        InputDeclaration [41-64]:
            symbol_id: 9
        ClassicalDeclarationStmt [73-93]:
            symbol_id: 10
            ty_span: [73-80]
            init_expr: Expr [85-92]:
                ty: Complex(None, false)
                kind: Paren Expr [86-91]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [86-87]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [90-91]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
    "#]],
    );
}

#[test]
fn multiplication() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a * b);
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        InputDeclaration [9-32]:
            symbol_id: 8
        InputDeclaration [41-64]:
            symbol_id: 9
        ClassicalDeclarationStmt [73-93]:
            symbol_id: 10
            ty_span: [73-80]
            init_expr: Expr [85-92]:
                ty: Complex(None, false)
                kind: Paren Expr [86-91]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [86-87]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [90-91]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
    "#]],
    );
}

#[test]
fn division() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a / b);
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        InputDeclaration [9-32]:
            symbol_id: 8
        InputDeclaration [41-64]:
            symbol_id: 9
        ClassicalDeclarationStmt [73-93]:
            symbol_id: 10
            ty_span: [73-80]
            init_expr: Expr [85-92]:
                ty: Complex(None, false)
                kind: Paren Expr [86-91]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Div
                        lhs: Expr [86-87]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [90-91]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
    "#]],
    );
}

#[test]
fn power() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = (a ** b);
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        InputDeclaration [9-32]:
            symbol_id: 8
        InputDeclaration [41-64]:
            symbol_id: 9
        ClassicalDeclarationStmt [73-94]:
            symbol_id: 10
            ty_span: [73-80]
            init_expr: Expr [85-93]:
                ty: Complex(None, false)
                kind: Paren Expr [86-92]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Exp
                        lhs: Expr [86-87]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [91-92]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
    "#]],
    );
}
