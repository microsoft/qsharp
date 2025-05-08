// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn float_to_bool() {
    let input = "
        float x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-34]:
                expr: Expr [31-32]:
                    ty: Bool(false)
                    kind: Cast [31-32]:
                        ty: Bool(false)
                        expr: Expr [31-32]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_int() {
    let input = "
        float x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-33]:
                expr: Expr [30-31]:
                    ty: Int(None, false)
                    kind: Cast [30-31]:
                        ty: Int(None, false)
                        expr: Expr [30-31]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_uint() {
    let input = "
        float x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-34]:
                expr: Expr [31-32]:
                    ty: UInt(None, false)
                    kind: Cast [31-32]:
                        ty: UInt(None, false)
                        expr: Expr [31-32]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_float() {
    let input = "
        float x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-17]:
            symbol_id: 8
            ty_span: [9-14]
            init_expr: Expr [0-0]:
                ty: Float(None, true)
                kind: Lit: Float(0.0)
        ExprStmt [26-35]:
            expr: Expr [32-33]:
                ty: Float(None, false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn float_to_angle() {
    let input = "
        float x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-35]:
                expr: Expr [32-33]:
                    ty: Angle(None, false)
                    kind: Cast [32-33]:
                        ty: Angle(None, false)
                        expr: Expr [32-33]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_bit() {
    let input = "
        float x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-33]:
                expr: Expr [30-31]:
                    ty: Bit(false)
                    kind: Cast [30-31]:
                        ty: Bit(false)
                        expr: Expr [30-31]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_bitarray_fails() {
    let input = "
        float x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-17]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-17]:
                        symbol_id: 8
                        ty_span: [9-14]
                        init_expr: Expr [0-0]:
                            ty: Float(None, true)
                            kind: Lit: Float(0.0)
                Stmt [26-36]:
                    annotations: <empty>
                    kind: ExprStmt [26-36]:
                        expr: Expr [33-34]:
                            ty: Float(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Float(None, false) to type BitArray(8,
          | false)
           ,-[test:3:16]
         2 |         float x;
         3 |         bit[8](x);
           :                ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn float_to_duration_fails() {
    let input = "
        float x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-17]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-17]:
                        symbol_id: 8
                        ty_span: [9-14]
                        init_expr: Expr [0-0]:
                            ty: Float(None, true)
                            kind: Lit: Float(0.0)
                Stmt [26-38]:
                    annotations: <empty>
                    kind: ExprStmt [26-38]:
                        expr: Expr [35-36]:
                            ty: Float(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Float(None, false) to type Duration(false)
           ,-[test:3:18]
         2 |         float x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

/// Even though the spec doesn't say it, we need to allow
/// casting from float to complex, else this kind of expression
/// would be invalid: 2.0 + sin(pi) + 1.0i.
#[test]
fn float_to_complex() {
    let input = "
        float x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-37]:
                expr: Expr [34-35]:
                    ty: Complex(None, false)
                    kind: Cast [34-35]:
                        ty: Complex(None, false)
                        expr: Expr [34-35]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}
