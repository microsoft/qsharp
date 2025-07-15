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
                    ty: const bit[8]
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-39]:
                expr: Expr [32-37]:
                    ty: bit[3]
                    kind: IndexedExpr [32-37]:
                        collection: Expr [32-33]:
                            ty: bit[8]
                            kind: SymbolId(8)
                        index: Range [34-37]:
                            start: Expr [34-35]:
                                ty: const int
                                const_value: Int(1)
                                kind: Lit: Int(1)
                            step: <none>
                            end: Expr [36-37]:
                                ty: const int
                                const_value: Int(3)
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
                    ty: const bit[8]
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-40]:
                expr: Expr [32-38]:
                    ty: bit[6]
                    kind: IndexedExpr [32-38]:
                        collection: Expr [32-33]:
                            ty: bit[8]
                            kind: SymbolId(8)
                        index: Range [34-38]:
                            start: Expr [35-36]:
                                ty: const int
                                const_value: Int(-7)
                                kind: Lit: Int(-7)
                            step: <none>
                            end: Expr [37-38]:
                                ty: const int
                                const_value: Int(6)
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
                    ty: const bit[8]
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-40]:
                expr: Expr [32-38]:
                    ty: bit[6]
                    kind: IndexedExpr [32-38]:
                        collection: Expr [32-33]:
                            ty: bit[8]
                            kind: SymbolId(8)
                        index: Range [34-38]:
                            start: Expr [34-35]:
                                ty: const int
                                const_value: Int(1)
                                kind: Lit: Int(1)
                            step: <none>
                            end: Expr [37-38]:
                                ty: const int
                                const_value: Int(-2)
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
                    ty: const bit[8]
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-41]:
                expr: Expr [32-39]:
                    ty: bit[3]
                    kind: IndexedExpr [32-39]:
                        collection: Expr [32-33]:
                            ty: bit[8]
                            kind: SymbolId(8)
                        index: Range [34-39]:
                            start: Expr [34-35]:
                                ty: const int
                                const_value: Int(0)
                                kind: Lit: Int(0)
                            step: Expr [36-37]:
                                ty: const int
                                const_value: Int(3)
                                kind: Lit: Int(3)
                            end: Expr [38-39]:
                                ty: const int
                                const_value: Int(7)
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
                    ty: const bit[8]
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-41]:
                expr: Expr [32-39]:
                    ty: bit[3]
                    kind: IndexedExpr [32-39]:
                        collection: Expr [32-33]:
                            ty: bit[8]
                            kind: SymbolId(8)
                        index: Range [34-39]:
                            start: Expr [34-35]:
                                ty: const int
                                const_value: Int(0)
                                kind: Lit: Int(0)
                            step: Expr [36-37]:
                                ty: const int
                                const_value: Int(3)
                                kind: Lit: Int(3)
                            end: Expr [38-39]:
                                ty: const int
                                const_value: Int(6)
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
                    ty: const bit[8]
                    kind: Lit: Bitstring("00010000")
            ExprStmt [32-42]:
                expr: Expr [32-40]:
                    ty: bit[3]
                    kind: IndexedExpr [32-40]:
                        collection: Expr [32-33]:
                            ty: bit[8]
                            kind: SymbolId(8)
                        index: Range [34-40]:
                            start: Expr [34-35]:
                                ty: const int
                                const_value: Int(6)
                                kind: Lit: Int(6)
                            step: Expr [37-38]:
                                ty: const int
                                const_value: Int(-3)
                                kind: Lit: Int(-3)
                            end: Expr [39-40]:
                                ty: const int
                                const_value: Int(0)
                                kind: Lit: Int(0)
        "#]],
    );
}

#[test]
fn array_slice_with_zero_step_errors() {
    let input = "
        bit[8] a = 16;
        a[:0:];
    ";

    check_stmt_kinds(
        input,
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [9-23]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-23]:
                            symbol_id: 8
                            ty_span: [9-15]
                            init_expr: Expr [20-22]:
                                ty: const bit[8]
                                kind: Lit: Bitstring("00010000")
                    Stmt [32-39]:
                        annotations: <empty>
                        kind: Err

            [Qasm.Lowerer.ZeroStepInRange

              x range step cannot be zero
               ,-[test:3:11]
             2 |         bit[8] a = 16;
             3 |         a[:0:];
               :           ^^^
             4 |     
               `----
            ]"#]],
    );
}
