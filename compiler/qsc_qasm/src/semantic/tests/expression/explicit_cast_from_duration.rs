// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn duration_to_bool_fails() {
    let input = "
        duration x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-37]:
                    annotations: <empty>
                    kind: ExprStmt [29-37]:
                        expr: Expr [34-35]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         bool(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type Bool(false)
           ,-[test:3:14]
         2 |         duration x;
         3 |         bool(x);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_int_fails() {
    let input = "
        duration x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-36]:
                    annotations: <empty>
                    kind: ExprStmt [29-36]:
                        expr: Expr [33-34]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         int(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type Int(None, false)
           ,-[test:3:13]
         2 |         duration x;
         3 |         int(x);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_uint_fails() {
    let input = "
        duration x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-37]:
                    annotations: <empty>
                    kind: ExprStmt [29-37]:
                        expr: Expr [34-35]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         uint(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type UInt(None, false)
           ,-[test:3:14]
         2 |         duration x;
         3 |         uint(x);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_float_fails() {
    let input = "
        duration x;
        float(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-38]:
                    annotations: <empty>
                    kind: ExprStmt [29-38]:
                        expr: Expr [35-36]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         float(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type Float(None, false)
           ,-[test:3:15]
         2 |         duration x;
         3 |         float(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_angle_fails() {
    let input = "
        duration x;
        angle(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-38]:
                    annotations: <empty>
                    kind: ExprStmt [29-38]:
                        expr: Expr [35-36]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         angle(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type Angle(None, false)
           ,-[test:3:15]
         2 |         duration x;
         3 |         angle(x);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_bit_fails() {
    let input = "
        duration x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-36]:
                    annotations: <empty>
                    kind: ExprStmt [29-36]:
                        expr: Expr [33-34]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         bit(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type Bit(false)
           ,-[test:3:13]
         2 |         duration x;
         3 |         bit(x);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_bitarray_fails() {
    let input = "
        duration x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-39]:
                    annotations: <empty>
                    kind: ExprStmt [29-39]:
                        expr: Expr [36-37]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         bit[8](x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type BitArray(8, false)
           ,-[test:3:16]
         2 |         duration x;
         3 |         bit[8](x);
           :                ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_duration() {
    let input = "
        duration x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-41]:
                    annotations: <empty>
                    kind: ExprStmt [29-41]:
                        expr: Expr [38-39]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         duration(x);
           `----
        ]"#]],
    );
}

#[test]
fn duration_to_complex_fails() {
    let input = "
        duration x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-20]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-20]:
                        symbol_id: 8
                        ty_span: [9-17]
                        init_expr: Expr [0-0]:
                            ty: Duration(true)
                            kind: Lit: Duration(0.0, Ns)
                Stmt [29-40]:
                    annotations: <empty>
                    kind: ExprStmt [29-40]:
                        expr: Expr [37-38]:
                            ty: Duration(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.NotSupported

          x duration type values are not supported
           ,-[test:2:9]
         1 | 
         2 |         duration x;
           :         ^^^^^^^^
         3 |         complex(x);
           `----
        , Qasm.Lowerer.CannotCast

          x cannot cast expression of type Duration(false) to type Complex(None,
          | false)
           ,-[test:3:17]
         2 |         duration x;
         3 |         complex(x);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}
