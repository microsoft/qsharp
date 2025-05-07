// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! These unit tests check six properties for each target type.
//! Let's call the type we are casting from `T` and the type we are casting to `Q`.
//! We want to test that for each type `Q` we correctly:
//!   1. cast from T to Q.
//!   2. cast from T to Q[n].
//!   3. cast from T[n] to Q.
//!   4. cast from T[n] to Q[n].
//!   5. cast from T[n] to Q[m] when n > m; a truncating cast.
//!   6. cast from T[n] to Q[m] when n < m; an expanding cast.

use crate::semantic::tests::check_stmt_kinds as check;
use expect_test::expect;

//===============
// Casts to bool
//===============

#[test]
fn bool_to_bool() {
    let source = "
        bool a;
        bool(a);
    ";
    check(
        source,
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

//===================
// Casts to duration
//===================

#[test]
fn bool_to_duration_fails() {
    let source = "
        bool a;
        duration(a);
    ";
    check(
        source,
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
         2 |         bool a;
         3 |         duration(a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

//=========================
// Casts to int and int[n]
//=========================

#[test]
fn bool_to_int() {
    let source = "
        bool a;
        int(a);
    ";
    check(
        source,
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
                kind: Cast [29-30]:
                    ty: Int(None, false)
                    expr: Expr [29-30]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bool_to_sized_int() {
    let source = "
        bool a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: Bool(true)
                kind: Lit: Bool(false)
        ExprStmt [25-36]:
            expr: Expr [33-34]:
                ty: Int(Some(32), false)
                kind: Cast [33-34]:
                    ty: Int(Some(32), false)
                    expr: Expr [33-34]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn bool_to_uint() {
    let source = "
        bool a;
        uint(a);
    ";
    check(
        source,
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
                kind: Cast [30-31]:
                    ty: UInt(None, false)
                    expr: Expr [30-31]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bool_to_sized_uint() {
    let source = "
        bool a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: Bool(true)
                kind: Lit: Bool(false)
        ExprStmt [25-37]:
            expr: Expr [34-35]:
                ty: UInt(Some(32), false)
                kind: Cast [34-35]:
                    ty: UInt(Some(32), false)
                    expr: Expr [34-35]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn bool_to_float() {
    let source = "
        bool a;
        float(a);
    ";
    check(
        source,
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
                kind: Cast [31-32]:
                    ty: Float(None, false)
                    expr: Expr [31-32]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bool_to_sized_float() {
    let source = "
        bool a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: Bool(true)
                kind: Lit: Bool(false)
        ExprStmt [25-38]:
            expr: Expr [35-36]:
                ty: Float(Some(32), false)
                kind: Cast [35-36]:
                    ty: Float(Some(32), false)
                    expr: Expr [35-36]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn bool_to_angle_fails() {
    let source = "
        bool a;
        angle(a);
    ";
    check(
        source,
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
         2 |         bool a;
         3 |         angle(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn bool_to_sized_angle_fails() {
    let source = "
        bool a;
        angle[32](a);
    ";
    check(
        source,
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
                Stmt [25-38]:
                    annotations: <empty>
                    kind: ExprStmt [25-38]:
                        expr: Expr [35-36]:
                            ty: Bool(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bool(false) to type Angle(Some(32), false)
           ,-[test:3:19]
         2 |         bool a;
         3 |         angle[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

//=================================
// Casts to complex and complex[n]
//=================================

#[test]
fn bool_to_complex_fails() {
    let source = "
        bool a;
        complex(a);
    ";
    check(
        source,
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
         2 |         bool a;
         3 |         complex(a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn bool_to_sized_complex_fails() {
    let source = "
        bool a;
        complex[float[32]](a);
    ";
    check(
        source,
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
                Stmt [25-47]:
                    annotations: <empty>
                    kind: ExprStmt [25-47]:
                        expr: Expr [44-45]:
                            ty: Bool(false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Bool(false) to type Complex(Some(32),
          | false)
           ,-[test:3:28]
         2 |         bool a;
         3 |         complex[float[32]](a);
           :                            ^
         4 |     
           `----
        ]"#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn bool_to_bit() {
    let source = "
        bool a;
        bit(a);
    ";
    check(
        source,
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
                kind: Cast [29-30]:
                    ty: Bit(false)
                    expr: Expr [29-30]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn bool_to_bitarray() {
    let source = "
        bool a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-16]:
            symbol_id: 8
            ty_span: [9-13]
            init_expr: Expr [0-0]:
                ty: Bool(true)
                kind: Lit: Bool(false)
        ExprStmt [25-36]:
            expr: Expr [33-34]:
                ty: BitArray(32, false)
                kind: Cast [33-34]:
                    ty: BitArray(32, false)
                    expr: Expr [33-34]:
                        ty: Bool(false)
                        kind: SymbolId(8)
    "#]],
    );
}
