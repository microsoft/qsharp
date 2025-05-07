// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn angle_to_bool() {
    let input = "
        angle x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-17]:
            symbol_id: 8
            ty_span: [9-14]
            init_expr: Expr [0-0]:
                ty: Angle(None, true)
                kind: Lit: Angle(0)
        ExprStmt [26-34]:
            expr: Expr [31-32]:
                ty: Bool(false)
                kind: Cast [0-0]:
                    ty: Bool(false)
                    expr: Expr [31-32]:
                        ty: Angle(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn angle_to_int_fails() {
    let input = "
        angle x;
        int(x);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-33]:
                    annotations: <empty>
                    kind: ExprStmt [26-33]:
                        expr: Expr [30-31]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type Int(None, false)
           ,-[test:3:13]
         2 |         angle x;
         3 |         int(x);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_uint_fails() {
    let input = "
        angle x;
        uint(x);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-34]:
                    annotations: <empty>
                    kind: ExprStmt [26-34]:
                        expr: Expr [31-32]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type UInt(None,
          | false)
           ,-[test:3:14]
         2 |         angle x;
         3 |         uint(x);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_float_fails() {
    let input = "
        angle x;
        float(x);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-35]:
                    annotations: <empty>
                    kind: ExprStmt [26-35]:
                        expr: Expr [32-33]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type Float(None,
          | false)
           ,-[test:3:15]
         2 |         angle x;
         3 |         float(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_angle() {
    let input = "
        angle x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-17]:
            symbol_id: 8
            ty_span: [9-14]
            init_expr: Expr [0-0]:
                ty: Angle(None, true)
                kind: Lit: Angle(0)
        ExprStmt [26-35]:
            expr: Expr [32-33]:
                ty: Angle(None, false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn angle_to_bit() {
    let input = "
        angle x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-17]:
            symbol_id: 8
            ty_span: [9-14]
            init_expr: Expr [0-0]:
                ty: Angle(None, true)
                kind: Lit: Angle(0)
        ExprStmt [26-33]:
            expr: Expr [30-31]:
                ty: Bit(false)
                kind: Cast [0-0]:
                    ty: Bit(false)
                    expr: Expr [30-31]:
                        ty: Angle(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn angle_to_bitarray() {
    let input = "
        angle x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-17]:
            symbol_id: 8
            ty_span: [9-14]
            init_expr: Expr [0-0]:
                ty: Angle(None, true)
                kind: Lit: Angle(0)
        ExprStmt [26-36]:
            expr: Expr [33-34]:
                ty: BitArray(8, false)
                kind: Cast [0-0]:
                    ty: BitArray(8, false)
                    expr: Expr [33-34]:
                        ty: Angle(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn angle_to_duration_fails() {
    let input = "
        angle x;
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-38]:
                    annotations: <empty>
                    kind: ExprStmt [26-38]:
                        expr: Expr [35-36]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type Duration(false)
           ,-[test:3:18]
         2 |         angle x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_complex_fails() {
    let input = "
        angle x;
        complex(x);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-37]:
                    annotations: <empty>
                    kind: ExprStmt [26-37]:
                        expr: Expr [34-35]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type Complex(None,
          | false)
           ,-[test:3:17]
         2 |         angle x;
         3 |         complex(x);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}
