// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn int_to_bool() {
    let input = "
        int x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-32]:
                expr: Expr [29-30]:
                    ty: Bool(false)
                    kind: Cast [29-30]:
                        ty: Bool(false)
                        expr: Expr [29-30]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn int_to_int() {
    let input = "
        int x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-31]:
                expr: Expr [28-29]:
                    ty: Int(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn int_to_uint() {
    let input = "
        int x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-32]:
                expr: Expr [29-30]:
                    ty: UInt(None, false)
                    kind: Cast [29-30]:
                        ty: UInt(None, false)
                        expr: Expr [29-30]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn int_to_float() {
    let input = "
        int x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-33]:
                expr: Expr [30-31]:
                    ty: Float(None, false)
                    kind: Cast [30-31]:
                        ty: Float(None, false)
                        expr: Expr [30-31]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn int_to_angle_fails() {
    let input = "
        int x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-15]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-15]:
                            symbol_id: 8
                            ty_span: [9-12]
                            init_expr: Expr [0-0]:
                                ty: Int(None, true)
                                kind: Lit: Int(0)
                    Stmt [24-33]:
                        annotations: <empty>
                        kind: ExprStmt [24-33]:
                            expr: Expr [30-31]:
                                ty: Int(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(None, false) to type Angle(None, false)
               ,-[test:3:15]
             2 |         int x;
             3 |         angle(x);
               :               ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn int_to_bit() {
    let input = "
        int x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-31]:
                expr: Expr [28-29]:
                    ty: Bit(false)
                    kind: Cast [28-29]:
                        ty: Bit(false)
                        expr: Expr [28-29]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn int_to_bitarray() {
    let input = "
        int x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-34]:
                expr: Expr [31-32]:
                    ty: BitArray(8, false)
                    kind: Cast [31-32]:
                        ty: BitArray(8, false)
                        expr: Expr [31-32]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn int_to_duration_fails() {
    let input = "
        int x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-15]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-15]:
                            symbol_id: 8
                            ty_span: [9-12]
                            init_expr: Expr [0-0]:
                                ty: Int(None, true)
                                kind: Lit: Int(0)
                    Stmt [24-36]:
                        annotations: <empty>
                        kind: ExprStmt [24-36]:
                            expr: Expr [33-34]:
                                ty: Int(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(None, false) to type Duration(false)
               ,-[test:3:18]
             2 |         int x;
             3 |         duration(x);
               :                  ^
             4 |     
               `----
            ]"#]],
    );
}

/// Even though the spec doesn't say it, we need to allow
/// casting from int to complex, else this kind of expression
/// would be invalid: 2 + 1i.
#[test]
fn int_to_complex() {
    let input = "
        int x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            ExprStmt [24-35]:
                expr: Expr [32-33]:
                    ty: Complex(None, false)
                    kind: Cast [32-33]:
                        ty: Complex(None, false)
                        expr: Expr [32-33]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}
