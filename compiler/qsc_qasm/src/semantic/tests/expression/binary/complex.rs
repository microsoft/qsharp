// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn implicit_cast_from_int() {
    let input = "
        complex x = 2 + 3 im;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [21-29]:
                ty: Complex(None, false)
                kind: BinaryOpExpr:
                    op: Add
                    lhs: Expr [21-22]:
                        ty: Complex(None, true)
                        kind: Lit: Complex(2.0, 0.0)
                    rhs: Expr [25-29]:
                        ty: Complex(None, true)
                        kind: Lit: Complex(0.0, 3.0)
    "#]],
    );
}

#[test]
fn implicit_cast_from_float() {
    let input = "
        complex x = 2.0 + 3.0 im;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-34]:
                symbol_id: 8
                ty_span: [9-16]
                init_expr: Expr [21-33]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [21-24]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(2.0, 0.0)
                        rhs: Expr [27-33]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 3.0)
        "#]],
    );
}

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
            AssignStmt [66-73]:
                symbol_id: 9
                lhs_span: [66-67]
                rhs: Expr [66-73]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Add
                        lhs: Expr [66-67]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
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
            AssignStmt [66-73]:
                symbol_id: 9
                lhs_span: [66-67]
                rhs: Expr [66-73]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Sub
                        lhs: Expr [66-67]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
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
            AssignStmt [66-73]:
                symbol_id: 9
                lhs_span: [66-67]
                rhs: Expr [66-73]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [66-67]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
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
            AssignStmt [66-73]:
                symbol_id: 9
                lhs_span: [66-67]
                rhs: Expr [66-73]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Div
                        lhs: Expr [66-67]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
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
            AssignStmt [66-74]:
                symbol_id: 9
                lhs_span: [66-67]
                rhs: Expr [66-74]:
                    ty: Complex(None, false)
                    kind: BinaryOpExpr:
                        op: Exp
                        lhs: Expr [66-67]:
                            ty: Complex(None, false)
                            kind: SymbolId(9)
                        rhs: Expr [72-73]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}
