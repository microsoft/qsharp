// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn bool_to_bool() {
    let input = "
        bool x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            ExprStmt [25-33]:
                expr: Expr [30-31]:
                    ty: Bool(false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bool_to_int() {
    let input = "
        bool x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            ExprStmt [25-32]:
                expr: Expr [29-30]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [29-30]:
                            ty: Bool(false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bool_to_uint() {
    let input = "
        bool x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            ExprStmt [25-33]:
                expr: Expr [30-31]:
                    ty: UInt(None, false)
                    kind: Cast [0-0]:
                        ty: UInt(None, false)
                        expr: Expr [30-31]:
                            ty: Bool(false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bool_to_float() {
    let input = "
        bool x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: Bool(true)
                kind: Lit: Bool(false)
        ExprStmt [25-34]:
            expr: Expr [31-32]:
                ty: Float(None, false)
                kind: Cast [0-0]:
                    ty: Float(None, false)
                    expr: Expr [31-32]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bool_to_angle_fails() {
    let input = "
        bool x;
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
                            ty: Bool(true)
                            kind: Lit: Bool(false)
                Stmt [25-34]:
                    annotations: <empty>
                    kind: ExprStmt [25-34]:
                        expr: Expr [31-32]:
                            ty: Bool(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bool(false) to type Angle(None, false)
           ,-[test:3:15]
         2 |         bool x;
         3 |         angle(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn bool_to_bit() {
    let input = "
        bool x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: Bool(true)
                kind: Lit: Bool(false)
        ExprStmt [25-32]:
            expr: Expr [29-30]:
                ty: Bit(false)
                kind: Cast [0-0]:
                    ty: Bit(false)
                    expr: Expr [29-30]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bool_to_bitarray() {
    let input = "
        bool x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: Bool(true)
                    kind: Lit: Bool(false)
            ExprStmt [25-35]:
                expr: Expr [32-33]:
                    ty: BitArray(8, false)
                    kind: Cast [0-0]:
                        ty: BitArray(8, false)
                        expr: Expr [32-33]:
                            ty: Bool(false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bool_to_duration_fails() {
    let input = "
        bool x;
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
                            ty: Bool(true)
                            kind: Lit: Bool(false)
                Stmt [25-37]:
                    annotations: <empty>
                    kind: ExprStmt [25-37]:
                        expr: Expr [34-35]:
                            ty: Bool(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bool(false) to type Duration(false)
           ,-[test:3:18]
         2 |         bool x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn bool_to_complex_fails() {
    let input = "
        bool x;
        complex(x);
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
                            ty: Bool(true)
                            kind: Lit: Bool(false)
                Stmt [25-36]:
                    annotations: <empty>
                    kind: ExprStmt [25-36]:
                        expr: Expr [33-34]:
                            ty: Bool(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bool(false) to type Complex(None, false)
           ,-[test:3:17]
         2 |         bool x;
         3 |         complex(x);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}
