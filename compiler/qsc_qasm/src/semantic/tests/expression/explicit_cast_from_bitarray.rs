// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

#[test]
fn bitarray_to_bool() {
    let input = "
        bit[8] x;
        bool(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-18]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00000000")
            ExprStmt [27-35]:
                expr: Expr [32-33]:
                    ty: Bool(false)
                    kind: Cast [0-0]:
                        ty: Bool(false)
                        expr: Expr [32-33]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bitarray_to_int() {
    let input = "
        bit[8] x;
        int(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-18]:
            symbol_id: 8
            ty_span: [9-15]
            init_expr: Expr [0-0]:
                ty: BitArray(8, true)
                kind: Lit: Bitstring("00000000")
        ExprStmt [27-34]:
            expr: Expr [31-32]:
                ty: Int(None, false)
                kind: Cast [0-0]:
                    ty: Int(None, false)
                    expr: Expr [31-32]:
                        ty: BitArray(8, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bitarray_to_uint() {
    let input = "
        bit[8] x;
        uint(x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-18]:
            symbol_id: 8
            ty_span: [9-15]
            init_expr: Expr [0-0]:
                ty: BitArray(8, true)
                kind: Lit: Bitstring("00000000")
        ExprStmt [27-35]:
            expr: Expr [32-33]:
                ty: UInt(None, false)
                kind: Cast [0-0]:
                    ty: UInt(None, false)
                    expr: Expr [32-33]:
                        ty: BitArray(8, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bitarray_to_float_fails() {
    let input = "
        bit[8] x;
        float[8](x);
    ";
    check(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-18]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-18]:
                            symbol_id: 8
                            ty_span: [9-15]
                            init_expr: Expr [0-0]:
                                ty: BitArray(8, true)
                                kind: Lit: Bitstring("00000000")
                    Stmt [27-39]:
                        annotations: <empty>
                        kind: ExprStmt [27-39]:
                            expr: Expr [36-37]:
                                ty: BitArray(8, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type BitArray(8, false) to type Float(Some(8),
              | false)
               ,-[test:3:18]
             2 |         bit[8] x;
             3 |         float[8](x);
               :                  ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn bitarray_to_angle() {
    let input = "
        bit[8] x;
        angle[8](x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-18]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00000000")
            ExprStmt [27-39]:
                expr: Expr [36-37]:
                    ty: Angle(Some(8), false)
                    kind: Cast [0-0]:
                        ty: Angle(Some(8), false)
                        expr: Expr [36-37]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bitarray_to_bit() {
    let input = "
        bit[8] x;
        bit(x);
    ";
    check(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-18]:
                symbol_id: 8
                ty_span: [9-15]
                init_expr: Expr [0-0]:
                    ty: BitArray(8, true)
                    kind: Lit: Bitstring("00000000")
            ExprStmt [27-34]:
                expr: Expr [31-32]:
                    ty: Bit(false)
                    kind: Cast [0-0]:
                        ty: Bit(false)
                        expr: Expr [31-32]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn bitarray_to_bitarray() {
    let input = "
        bit[8] x;
        bit[8](x);
    ";
    check(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-18]:
            symbol_id: 8
            ty_span: [9-15]
            init_expr: Expr [0-0]:
                ty: BitArray(8, true)
                kind: Lit: Bitstring("00000000")
        ExprStmt [27-37]:
            expr: Expr [34-35]:
                ty: BitArray(8, false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bitarray_to_duration_fails() {
    let input = "
        bit[8] x;
        duration(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-18]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-18]:
                        symbol_id: 8
                        ty_span: [9-15]
                        init_expr: Expr [0-0]:
                            ty: BitArray(8, true)
                            kind: Lit: Bitstring("00000000")
                Stmt [27-39]:
                    annotations: <empty>
                    kind: ExprStmt [27-39]:
                        expr: Expr [36-37]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type BitArray(8, false) to type Duration(false)
           ,-[test:3:18]
         2 |         bit[8] x;
         3 |         duration(x);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn bitarray_to_complex_fails() {
    let input = "
        bit[8] x;
        complex(x);
    ";
    check(
        input,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-18]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-18]:
                        symbol_id: 8
                        ty_span: [9-15]
                        init_expr: Expr [0-0]:
                            ty: BitArray(8, true)
                            kind: Lit: Bitstring("00000000")
                Stmt [27-38]:
                    annotations: <empty>
                    kind: ExprStmt [27-38]:
                        expr: Expr [35-36]:
                            ty: BitArray(8, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type BitArray(8, false) to type Complex(None,
          | false)
           ,-[test:3:17]
         2 |         bit[8] x;
         3 |         complex(x);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}
