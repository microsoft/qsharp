// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn bit_to_bool() {
    let input = "
        bit x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Bit(true)
                kind: Lit: Bit(0)
        ExprStmt [24-32]:
            expr: Expr [29-30]:
                ty: Bool(false)
                kind: Cast [0-0]:
                    ty: Bool(false)
                    expr: Expr [29-30]:
                        ty: Bit(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bit_to_int() {
    let input = "
        bit x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Bit(true)
                kind: Lit: Bit(0)
        ExprStmt [24-31]:
            expr: Expr [28-29]:
                ty: Int(None, false)
                kind: Cast [0-0]:
                    ty: Int(None, false)
                    expr: Expr [28-29]:
                        ty: Bit(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bit_to_uint() {
    let input = "
        bit x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Bit(true)
                kind: Lit: Bit(0)
        ExprStmt [24-32]:
            expr: Expr [29-30]:
                ty: UInt(None, false)
                kind: Cast [0-0]:
                    ty: UInt(None, false)
                    expr: Expr [29-30]:
                        ty: Bit(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bit_to_float() {
    let input = "
        bit x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Bit(true)
                    kind: Lit: Bit(0)
            ExprStmt [24-33]:
                expr: Expr [30-31]:
                    ty: Float(None, false)
                    kind: Cast [0-0]:
                        ty: Float(None, false)
                        expr: Expr [30-31]:
                            ty: Bit(false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bit_to_angle_fails() {
    let input = "
        bit x;
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
                                ty: Bit(true)
                                kind: Lit: Bit(0)
                    Stmt [24-33]:
                        annotations: <empty>
                        kind: ExprStmt [24-33]:
                            expr: Expr [30-31]:
                                ty: Bit(false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Angle(None, false)
               ,-[test:3:15]
             2 |         bit x;
             3 |         angle(x);
               :               ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn bit_to_bit() {
    let input = "
        bit x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Bit(true)
                kind: Lit: Bit(0)
        ExprStmt [24-31]:
            expr: Expr [28-29]:
                ty: Bit(false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bit_to_bitarray() {
    let input = "
        bit x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-15]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [0-0]:
                    ty: Bit(true)
                    kind: Lit: Bit(0)
            ExprStmt [24-34]:
                expr: Expr [31-32]:
                    ty: BitArray(8, false)
                    kind: Cast [0-0]:
                        ty: BitArray(8, false)
                        expr: Expr [31-32]:
                            ty: Bit(false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bit_to_duration_fails() {
    let input = "
        bit x;
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
                            ty: Bit(true)
                            kind: Lit: Bit(0)
                Stmt [24-36]:
                    annotations: <empty>
                    kind: ExprStmt [24-36]:
                        expr: Expr [33-34]:
                            ty: Bit(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bit(false) to type Duration(false)
           ,-[test:3:18]
         2 |         bit x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn bit_to_complex_fails() {
    let input = "
        bit x;
        complex(x);
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
                            ty: Bit(true)
                            kind: Lit: Bit(0)
                Stmt [24-35]:
                    annotations: <empty>
                    kind: ExprStmt [24-35]:
                        expr: Expr [32-33]:
                            ty: Bit(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bit(false) to type Complex(None, false)
           ,-[test:3:17]
         2 |         bit x;
         3 |         complex(x);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}
