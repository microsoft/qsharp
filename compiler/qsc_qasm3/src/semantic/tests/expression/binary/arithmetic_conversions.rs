// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

#[test]
fn int_idents_without_width_can_be_multiplied() {
    let input = "
        int x = 5;
        int y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: Int(None, false)
                    kind: Lit: Int(5)
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Lit: Int(3)
            ExprStmt [47-53]:
                expr: Expr [47-52]:
                    ty: Int(None, false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [47-48]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
                        rhs: Expr [51-52]:
                            ty: Int(None, false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn int_idents_with_same_width_can_be_multiplied() {
    let input = "
        int[32] x = 5;
        int[32] y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-16]
                init_expr: Expr [21-22]:
                    ty: Int(Some(32), true)
                    kind: Lit: Int(5)
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(32), true)
                    kind: Lit: Int(3)
            ExprStmt [55-61]:
                expr: Expr [55-60]:
                    ty: Int(Some(32), false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [55-56]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(8)
                        rhs: Expr [59-60]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn int_idents_with_different_width_can_be_multiplied() {
    let input = "
        int[32] x = 5;
        int[64] y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-16]
                init_expr: Expr [21-22]:
                    ty: Int(Some(32), true)
                    kind: Lit: Int(5)
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(64), true)
                    kind: Lit: Int(3)
            ExprStmt [55-61]:
                expr: Expr [55-60]:
                    ty: Int(Some(64), false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [55-56]:
                            ty: Int(Some(64), false)
                            kind: Cast [0-0]:
                                ty: Int(Some(64), false)
                                expr: Expr [55-56]:
                                    ty: Int(Some(32), false)
                                    kind: SymbolId(8)
                        rhs: Expr [59-60]:
                            ty: Int(Some(64), false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multiplying_int_idents_with_different_width_result_in_higher_width_result() {
    let input = "
        int[32] x = 5;
        int[64] y = 3;
        int[64] z = x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-16]
                init_expr: Expr [21-22]:
                    ty: Int(Some(32), true)
                    kind: Lit: Int(5)
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(64), true)
                    kind: Lit: Int(3)
            ClassicalDeclarationStmt [55-73]:
                symbol_id: 10
                ty_span: [55-62]
                init_expr: Expr [67-72]:
                    ty: Int(Some(64), false)
                    kind: BinaryOpExpr:
                        op: Mul
                        lhs: Expr [67-68]:
                            ty: Int(Some(64), false)
                            kind: Cast [0-0]:
                                ty: Int(Some(64), false)
                                expr: Expr [67-68]:
                                    ty: Int(Some(32), false)
                                    kind: SymbolId(8)
                        rhs: Expr [71-72]:
                            ty: Int(Some(64), false)
                            kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multiplying_int_idents_with_different_width_result_in_no_width_result() {
    let input = "
        int[32] x = 5;
        int[64] y = 3;
        int z = x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-16]
                init_expr: Expr [21-22]:
                    ty: Int(Some(32), true)
                    kind: Lit: Int(5)
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(64), true)
                    kind: Lit: Int(3)
            ClassicalDeclarationStmt [55-69]:
                symbol_id: 10
                ty_span: [55-58]
                init_expr: Expr [63-68]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [63-68]:
                            ty: Int(Some(64), false)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [63-64]:
                                    ty: Int(Some(64), false)
                                    kind: Cast [0-0]:
                                        ty: Int(Some(64), false)
                                        expr: Expr [63-64]:
                                            ty: Int(Some(32), false)
                                            kind: SymbolId(8)
                                rhs: Expr [67-68]:
                                    ty: Int(Some(64), false)
                                    kind: SymbolId(9)
        "#]],
    );
}

#[test]
fn multiplying_int_idents_with_width_greater_than_64_result_in_bigint_result() {
    let input = "
        int[32] x = 5;
        int[64] y = 3;
        int[67] z = x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-16]
                init_expr: Expr [21-22]:
                    ty: Int(Some(32), true)
                    kind: Lit: Int(5)
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(64), true)
                    kind: Lit: Int(3)
            ClassicalDeclarationStmt [55-73]:
                symbol_id: 10
                ty_span: [55-62]
                init_expr: Expr [67-72]:
                    ty: Int(Some(67), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(67), false)
                        expr: Expr [67-72]:
                            ty: Int(Some(64), false)
                            kind: BinaryOpExpr:
                                op: Mul
                                lhs: Expr [67-68]:
                                    ty: Int(Some(64), false)
                                    kind: Cast [0-0]:
                                        ty: Int(Some(64), false)
                                        expr: Expr [67-68]:
                                            ty: Int(Some(32), false)
                                            kind: SymbolId(8)
                                rhs: Expr [71-72]:
                                    ty: Int(Some(64), false)
                                    kind: SymbolId(9)
        "#]],
    );
}
