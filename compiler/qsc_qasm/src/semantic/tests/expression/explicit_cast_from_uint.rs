// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn uint_to_bool() {
    let input = "
        uint x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: UInt(None, true)
                kind: Lit: Int(0)
        ExprStmt [25-33]:
            expr: Expr [30-31]:
                ty: Bool(false)
                kind: Cast [0-0]:
                    ty: Bool(false)
                    expr: Expr [30-31]:
                        ty: UInt(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn uint_to_int() {
    let input = "
        uint x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: UInt(None, true)
                kind: Lit: Int(0)
        ExprStmt [25-32]:
            expr: Expr [29-30]:
                ty: Int(None, false)
                kind: Cast [0-0]:
                    ty: Int(None, false)
                    expr: Expr [29-30]:
                        ty: UInt(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn uint_to_uint() {
    let input = "
        uint x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: UInt(None, true)
                kind: Lit: Int(0)
        ExprStmt [25-33]:
            expr: Expr [30-31]:
                ty: UInt(None, false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn uint_to_float() {
    let input = "
        uint x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: UInt(None, true)
                kind: Lit: Int(0)
        ExprStmt [25-34]:
            expr: Expr [31-32]:
                ty: Float(None, false)
                kind: Cast [0-0]:
                    ty: Float(None, false)
                    expr: Expr [31-32]:
                        ty: UInt(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn uint_to_angle_fails() {
    let input = "
        uint x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-16]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-16]:
                        symbol_id: 8
                        ty_span: [9-13]
                        init_expr: Expr [0-0]:
                            ty: UInt(None, true)
                            kind: Lit: Int(0)
                Stmt [25-34]:
                    annotations: <empty>
                    kind: ExprStmt [25-34]:
                        expr: Expr [31-32]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type UInt(None, false) to type Angle(None,
          | false)
           ,-[test:3:15]
         2 |         uint x;
         3 |         angle(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn uint_to_bit() {
    let input = "
        uint x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: UInt(None, true)
                kind: Lit: Int(0)
        ExprStmt [25-32]:
            expr: Expr [29-30]:
                ty: Bit(false)
                kind: Cast [0-0]:
                    ty: Bit(false)
                    expr: Expr [29-30]:
                        ty: UInt(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn uint_to_bitarray() {
    let input = "
        uint x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: UInt(None, true)
                kind: Lit: Int(0)
        ExprStmt [25-35]:
            expr: Expr [32-33]:
                ty: BitArray(8, false)
                kind: Cast [0-0]:
                    ty: BitArray(8, false)
                    expr: Expr [32-33]:
                        ty: UInt(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn uint_to_duration_fails() {
    let input = "
        uint x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-16]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-16]:
                        symbol_id: 8
                        ty_span: [9-13]
                        init_expr: Expr [0-0]:
                            ty: UInt(None, true)
                            kind: Lit: Int(0)
                Stmt [25-37]:
                    annotations: <empty>
                    kind: ExprStmt [25-37]:
                        expr: Expr [34-35]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type UInt(None, false) to type Duration(false)
           ,-[test:3:18]
         2 |         uint x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn uint_to_complex() {
    let input = "
        uint x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-36]:
                expr: Expr [33-34]:
                    ty: Complex(None, false)
                    kind: Cast [0-0]:
                        ty: Complex(None, false)
                        expr: Expr [33-34]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}
