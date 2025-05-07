// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn complex_to_bool_fails() {
    let input = "
        complex x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-36]:
                    annotations: <empty>
                    kind: ExprStmt [28-36]:
                        expr: Expr [33-34]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Bool(false)
           ,-[test:3:14]
         2 |         complex x;
         3 |         bool(x);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_int_fails() {
    let input = "
        complex x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-35]:
                    annotations: <empty>
                    kind: ExprStmt [28-35]:
                        expr: Expr [32-33]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Int(None,
          | false)
           ,-[test:3:13]
         2 |         complex x;
         3 |         int(x);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_uint_fails() {
    let input = "
        complex x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-36]:
                    annotations: <empty>
                    kind: ExprStmt [28-36]:
                        expr: Expr [33-34]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type UInt(None,
          | false)
           ,-[test:3:14]
         2 |         complex x;
         3 |         uint(x);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_float_fails() {
    let input = "
        complex x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-37]:
                    annotations: <empty>
                    kind: ExprStmt [28-37]:
                        expr: Expr [34-35]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Float(None,
          | false)
           ,-[test:3:15]
         2 |         complex x;
         3 |         float(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_angle_fails() {
    let input = "
        complex x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-37]:
                    annotations: <empty>
                    kind: ExprStmt [28-37]:
                        expr: Expr [34-35]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Angle(None,
          | false)
           ,-[test:3:15]
         2 |         complex x;
         3 |         angle(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_bit_fails() {
    let input = "
        complex x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-35]:
                    annotations: <empty>
                    kind: ExprStmt [28-35]:
                        expr: Expr [32-33]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Bit(false)
           ,-[test:3:13]
         2 |         complex x;
         3 |         bit(x);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_bitarray_fails() {
    let input = "
        complex x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-38]:
                    annotations: <empty>
                    kind: ExprStmt [28-38]:
                        expr: Expr [35-36]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type BitArray(8,
          | false)
           ,-[test:3:16]
         2 |         complex x;
         3 |         bit[8](x);
           :                ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_duration_fails() {
    let input = "
        complex x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-19]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-19]:
                        symbol_id: 8
                        ty_span: [9-16]
                        init_expr: Expr [0-0]:
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-40]:
                    annotations: <empty>
                    kind: ExprStmt [28-40]:
                        expr: Expr [37-38]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type
          | Duration(false)
           ,-[test:3:18]
         2 |         complex x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_complex() {
    let input = "
        complex x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Complex(None, true)
                kind: Lit: Complex(0.0, 0.0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: Complex(None, false)
                kind: SymbolId(8)
    "#]],
    );
}
