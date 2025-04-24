// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kinds;

#[test]
fn simple_array_slice_has_correct_size() {
    let input = "
        bit[8] a = 16;
        a[1:3];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-39]:
                expr: Expr [32-38]:
                    ty: BitArray(3, false)
                    kind: IndexExpr [32-38]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-37]:
                                start: Expr [34-35]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(1)
                                step: <none>
                                end: Expr [36-37]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(3)
        "#]],
    );
}

#[test]
fn array_slice_with_negative_start_has_correct_size() {
    let input = "
        bit[8] a = 16;
        a[-7:6];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-40]:
                expr: Expr [32-39]:
                    ty: BitArray(6, false)
                    kind: IndexExpr [32-39]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-38]:
                                start: Expr [35-36]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(-7)
                                step: <none>
                                end: Expr [37-38]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(6)
        "#]],
    );
}

#[test]
fn array_slice_with_negative_end_has_correct_size() {
    let input = "
        bit[8] a = 16;
        a[1:-2];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-40]:
                expr: Expr [32-39]:
                    ty: BitArray(6, false)
                    kind: IndexExpr [32-39]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-38]:
                                start: Expr [34-35]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(1)
                                step: <none>
                                end: Expr [37-38]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(-2)
        "#]],
    );
}

#[test]
fn array_slice_with_non_exact_divisor_step_has_correct_size() {
    let input = "
        bit[8] a = 16;
        a[0:3:7];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-41]:
                expr: Expr [32-40]:
                    ty: BitArray(3, false)
                    kind: IndexExpr [32-40]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-39]:
                                start: Expr [34-35]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(0)
                                step: Expr [36-37]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(3)
                                end: Expr [38-39]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(7)
        "#]],
    );
}

#[test]
fn array_slice_with_exact_divisor_step_has_correct_size() {
    let input = "
        bit[8] a = 16;
        a[0:3:6];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-41]:
                expr: Expr [32-40]:
                    ty: BitArray(3, false)
                    kind: IndexExpr [32-40]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-39]:
                                start: Expr [34-35]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(0)
                                step: Expr [36-37]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(3)
                                end: Expr [38-39]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(6)
        "#]],
    );
}

#[test]
fn array_slice_with_negative_step_has_correct_size() {
    let input = "
        bit[8] a = 16;
        a[6:-3:0];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-42]:
                expr: Expr [32-41]:
                    ty: BitArray(3, false)
                    kind: IndexExpr [32-41]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-40]:
                                start: Expr [34-35]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(6)
                                step: Expr [37-38]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(-3)
                                end: Expr [39-40]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(0)
        "#]],
    );
}

#[test]
fn array_slice_with_zero_step_errors() {
    let input = "
        bit[8] a = 16;
        a[0:0:b];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [20-22]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-41]:
                expr: Expr [32-40]:
                    ty: BitArray(0, false)
                    kind: IndexExpr [32-40]:
                        collection: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
                        indices:
                            Range [34-39]:
                                start: Expr [34-35]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(0)
                                step: Expr [36-37]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(0)
                                end: Expr [38-39]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(6)
        "#]],
    );
}
