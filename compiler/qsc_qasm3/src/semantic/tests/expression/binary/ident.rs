// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;
#[test]
fn mutable_int_idents_without_width_can_be_multiplied() {
    let input = "
        int x = 5;
        int y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 6
            ty_span: [9-12]
            init_expr: Expr [17-18]:
                ty: Int(None, true)
                kind: Lit: Int(5)
        ClassicalDeclarationStmt [28-38]:
            symbol_id: 7
            ty_span: [28-31]
            init_expr: Expr [36-37]:
                ty: Int(None, true)
                kind: Lit: Int(3)
        ExprStmt [47-53]:
            expr: Expr [47-52]:
                ty: Int(None, false)
                kind: BinaryOpExpr:
                    op: Mul
                    lhs: Expr [47-48]:
                        ty: Int(None, false)
                        kind: SymbolId(6)
                    rhs: Expr [51-52]:
                        ty: Int(None, false)
                        kind: SymbolId(7)
    "#]],
    );
}

#[test]
fn const_int_idents_without_width_can_be_multiplied() {
    let input = "
        const int x = 5;
        const int y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-25]:
            symbol_id: 6
            ty_span: [15-18]
            init_expr: Expr [23-24]:
                ty: Int(None, true)
                kind: Lit: Int(5)
        ClassicalDeclarationStmt [34-50]:
            symbol_id: 7
            ty_span: [40-43]
            init_expr: Expr [48-49]:
                ty: Int(None, true)
                kind: Lit: Int(3)
        ExprStmt [59-65]:
            expr: Expr [59-64]:
                ty: Int(None, true)
                kind: BinaryOpExpr:
                    op: Mul
                    lhs: Expr [59-60]:
                        ty: Int(None, true)
                        kind: SymbolId(6)
                    rhs: Expr [63-64]:
                        ty: Int(None, true)
                        kind: SymbolId(7)
    "#]],
    );
}

#[test]
fn const_and_mut_int_idents_without_width_can_be_multiplied() {
    let input = "
        int x = 5;
        const int y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 6
            ty_span: [9-12]
            init_expr: Expr [17-18]:
                ty: Int(None, true)
                kind: Lit: Int(5)
        ClassicalDeclarationStmt [28-44]:
            symbol_id: 7
            ty_span: [34-37]
            init_expr: Expr [42-43]:
                ty: Int(None, true)
                kind: Lit: Int(3)
        ExprStmt [53-59]:
            expr: Expr [53-58]:
                ty: Int(None, false)
                kind: BinaryOpExpr:
                    op: Mul
                    lhs: Expr [53-54]:
                        ty: Int(None, false)
                        kind: SymbolId(6)
                    rhs: Expr [57-58]:
                        ty: Int(None, true)
                        kind: SymbolId(7)
    "#]],
    );
}

#[test]
fn const_int_idents_widthless_lhs_can_be_multiplied_by_explicit_width_int() {
    let input = "
        const int[32] x = 5;
        const int y = 3;
        x * y;
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-29]:
            symbol_id: 6
            ty_span: [15-22]
            init_expr: Expr [27-28]:
                ty: Int(Some(32), true)
                kind: Lit: Int(5)
        ClassicalDeclarationStmt [38-54]:
            symbol_id: 7
            ty_span: [44-47]
            init_expr: Expr [52-53]:
                ty: Int(None, true)
                kind: Lit: Int(3)
        ExprStmt [63-69]:
            expr: Expr [63-68]:
                ty: Int(None, true)
                kind: BinaryOpExpr:
                    op: Mul
                    lhs: Expr [63-64]:
                        ty: Int(None, true)
                        kind: Cast [0-0]:
                            ty: Int(None, true)
                            expr: Expr [63-64]:
                                ty: Int(Some(32), true)
                                kind: SymbolId(6)
                    rhs: Expr [67-68]:
                        ty: Int(None, true)
                        kind: SymbolId(7)
    "#]],
    );
}
