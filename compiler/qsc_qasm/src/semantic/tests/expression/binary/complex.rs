// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn addition() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = a + b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            InputDeclaration [41-64]:
                symbol_id: 9
            ClassicalDeclarationStmt [73-91]:
                symbol_id: 10
                ty_span: [73-80]
                init_expr: Expr [85-90]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [85-86]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [89-90]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn addition_assign_op() {
    let input = "
        input complex[float] a;
        complex x = 0.0;
        x += a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            ClassicalDeclarationStmt [41-57]:
                symbol_id: 9
                ty_span: [41-48]
                init_expr: Expr [53-56]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(0.0, 0.0)
            AssignOpStmt [66-73]:
                symbol_id: 9
                indices: <empty>
                op: Add
                lhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
                rhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn subtraction() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = a - b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            InputDeclaration [41-64]:
                symbol_id: 9
            ClassicalDeclarationStmt [73-91]:
                symbol_id: 10
                ty_span: [73-80]
                init_expr: Expr [85-90]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [85-86]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [89-90]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn subtraction_assign_op() {
    let input = "
        input complex[float] a;
        complex x = 0.0;
        x -= a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            ClassicalDeclarationStmt [41-57]:
                symbol_id: 9
                ty_span: [41-48]
                init_expr: Expr [53-56]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(0.0, 0.0)
            AssignOpStmt [66-73]:
                symbol_id: 9
                indices: <empty>
                op: Sub
                lhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
                rhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn multiplication() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = a * b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            InputDeclaration [41-64]:
                symbol_id: 9
            ClassicalDeclarationStmt [73-91]:
                symbol_id: 10
                ty_span: [73-80]
                init_expr: Expr [85-90]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [85-86]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [89-90]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multiplication_assign_op() {
    let input = "
        input complex[float] a;
        complex x = 0.0;
        x *= a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            ClassicalDeclarationStmt [41-57]:
                symbol_id: 9
                ty_span: [41-48]
                init_expr: Expr [53-56]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(0.0, 0.0)
            AssignOpStmt [66-73]:
                symbol_id: 9
                indices: <empty>
                op: Mul
                lhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
                rhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn division() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = a / b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            InputDeclaration [41-64]:
                symbol_id: 9
            ClassicalDeclarationStmt [73-91]:
                symbol_id: 10
                ty_span: [73-80]
                init_expr: Expr [85-90]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Div
                        lhs: Expr [85-86]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [89-90]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn division_assign_op() {
    let input = "
        input complex[float] a;
        complex x = 0.0;
        x /= a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            ClassicalDeclarationStmt [41-57]:
                symbol_id: 9
                ty_span: [41-48]
                init_expr: Expr [53-56]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(0.0, 0.0)
            AssignOpStmt [66-73]:
                symbol_id: 9
                indices: <empty>
                op: Div
                lhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
                rhs: Expr [71-72]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn power() {
    let input = "
        input complex[float] a;
        input complex[float] b;
        complex x = a ** b;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            InputDeclaration [41-64]:
                symbol_id: 9
            ClassicalDeclarationStmt [73-92]:
                symbol_id: 10
                ty_span: [73-80]
                init_expr: Expr [85-91]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Exp
                        lhs: Expr [85-86]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [90-91]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn power_assign_op() {
    let input = "
        input complex[float] a;
        complex x = 0.0;
        x **= a;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            InputDeclaration [9-32]:
                symbol_id: 8
            ClassicalDeclarationStmt [41-57]:
                symbol_id: 9
                ty_span: [41-48]
                init_expr: Expr [53-56]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(0.0, 0.0)
            AssignOpStmt [66-74]:
                symbol_id: 9
                indices: <empty>
                op: Exp
                lhs: Expr [72-73]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
                rhs: Expr [72-73]:
                    ty: Complex(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}
