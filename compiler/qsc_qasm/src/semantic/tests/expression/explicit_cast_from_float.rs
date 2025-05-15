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
fn float_to_bool() {
    let source = "
        float a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-34]:
                expr: Expr [26-33]:
                    ty: Bool(false)
                    kind: Cast [26-33]:
                        ty: Bool(false)
                        expr: Expr [31-32]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_bool() {
    let source = "
        float[32] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-38]:
                expr: Expr [30-37]:
                    ty: Bool(false)
                    kind: Cast [30-37]:
                        ty: Bool(false)
                        expr: Expr [35-36]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn float_to_duration_fails() {
    let source = "
        float a;
        duration(a);
    ";
    check(
        source,
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
                            expr: Expr [26-37]:
                                ty: Float(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(None, false) to type Duration(false)
               ,-[test:3:9]
             2 |         float a;
             3 |         duration(a);
               :         ^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_float_to_duration_fails() {
    let source = "
        float[32] a;
        duration(a);
    ";
    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-21]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-21]:
                            symbol_id: 8
                            ty_span: [9-18]
                            init_expr: Expr [0-0]:
                                ty: Float(Some(32), true)
                                kind: Lit: Float(0.0)
                    Stmt [30-42]:
                        annotations: <empty>
                        kind: ExprStmt [30-42]:
                            expr: Expr [30-41]:
                                ty: Float(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(Some(32), false) to type
              | Duration(false)
               ,-[test:3:9]
             2 |         float[32] a;
             3 |         duration(a);
               :         ^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

//=========================
// Casts to int and int[n]
//=========================

#[test]
fn float_to_int() {
    let source = "
        float a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-33]:
                expr: Expr [26-32]:
                    ty: Int(None, false)
                    kind: Cast [26-32]:
                        ty: Int(None, false)
                        expr: Expr [30-31]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_sized_int() {
    let source = "
        float a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-37]:
                expr: Expr [26-36]:
                    ty: Int(Some(32), false)
                    kind: Cast [26-36]:
                        ty: Int(Some(32), false)
                        expr: Expr [34-35]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_int() {
    let source = "
        float[32] a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-37]:
                expr: Expr [30-36]:
                    ty: Int(None, false)
                    kind: Cast [30-36]:
                        ty: Int(None, false)
                        expr: Expr [34-35]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_int() {
    let source = "
        float[32] a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-41]:
                expr: Expr [30-40]:
                    ty: Int(Some(32), false)
                    kind: Cast [30-40]:
                        ty: Int(Some(32), false)
                        expr: Expr [38-39]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_int_truncating() {
    let source = "
        float[32] a;
        int[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-41]:
                expr: Expr [30-40]:
                    ty: Int(Some(16), false)
                    kind: Cast [30-40]:
                        ty: Int(Some(16), false)
                        expr: Expr [38-39]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_int_expanding() {
    let source = "
        float[32] a;
        int[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-41]:
                expr: Expr [30-40]:
                    ty: Int(Some(64), false)
                    kind: Cast [30-40]:
                        ty: Int(Some(64), false)
                        expr: Expr [38-39]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn float_to_uint() {
    let source = "
        float a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-34]:
                expr: Expr [26-33]:
                    ty: UInt(None, false)
                    kind: Cast [26-33]:
                        ty: UInt(None, false)
                        expr: Expr [31-32]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_sized_uint() {
    let source = "
        float a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-38]:
                expr: Expr [26-37]:
                    ty: UInt(Some(32), false)
                    kind: Cast [26-37]:
                        ty: UInt(Some(32), false)
                        expr: Expr [35-36]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_uint() {
    let source = "
        float[32] a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-38]:
                expr: Expr [30-37]:
                    ty: UInt(None, false)
                    kind: Cast [30-37]:
                        ty: UInt(None, false)
                        expr: Expr [35-36]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_uint() {
    let source = "
        float[32] a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-42]:
                expr: Expr [30-41]:
                    ty: UInt(Some(32), false)
                    kind: Cast [30-41]:
                        ty: UInt(Some(32), false)
                        expr: Expr [39-40]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_uint_truncating() {
    let source = "
        float[32] a;
        uint[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-42]:
                expr: Expr [30-41]:
                    ty: UInt(Some(16), false)
                    kind: Cast [30-41]:
                        ty: UInt(Some(16), false)
                        expr: Expr [39-40]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_uint_expanding() {
    let source = "
        float[32] a;
        uint[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-42]:
                expr: Expr [30-41]:
                    ty: UInt(Some(64), false)
                    kind: Cast [30-41]:
                        ty: UInt(Some(64), false)
                        expr: Expr [39-40]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn float_to_float() {
    let source = "
        float a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-35]:
                expr: Expr [26-34]:
                    ty: Float(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_sized_float() {
    let source = "
        float a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-39]:
                expr: Expr [26-38]:
                    ty: Float(Some(32), false)
                    kind: Cast [26-38]:
                        ty: Float(Some(32), false)
                        expr: Expr [36-37]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_float() {
    let source = "
        float[32] a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-39]:
                expr: Expr [30-38]:
                    ty: Float(None, false)
                    kind: Cast [30-38]:
                        ty: Float(None, false)
                        expr: Expr [36-37]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_float() {
    let source = "
        float[32] a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-43]:
                expr: Expr [30-42]:
                    ty: Float(Some(32), false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_float_truncating() {
    let source = "
        float[32] a;
        float[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-43]:
                expr: Expr [30-42]:
                    ty: Float(Some(16), false)
                    kind: Cast [30-42]:
                        ty: Float(Some(16), false)
                        expr: Expr [40-41]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_float_expanding() {
    let source = "
        float[32] a;
        float[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-43]:
                expr: Expr [30-42]:
                    ty: Float(Some(64), false)
                    kind: Cast [30-42]:
                        ty: Float(Some(64), false)
                        expr: Expr [40-41]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn float_to_angle() {
    let source = "
        float a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-35]:
                expr: Expr [26-34]:
                    ty: Angle(None, false)
                    kind: Cast [26-34]:
                        ty: Angle(None, false)
                        expr: Expr [32-33]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_sized_angle() {
    let source = "
        float a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-39]:
                expr: Expr [26-38]:
                    ty: Angle(Some(32), false)
                    kind: Cast [26-38]:
                        ty: Angle(Some(32), false)
                        expr: Expr [36-37]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_angle() {
    let source = "
        float[32] a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-39]:
                expr: Expr [30-38]:
                    ty: Angle(None, false)
                    kind: Cast [30-38]:
                        ty: Angle(None, false)
                        expr: Expr [36-37]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_angle() {
    let source = "
        float[32] a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-43]:
                expr: Expr [30-42]:
                    ty: Angle(Some(32), false)
                    kind: Cast [30-42]:
                        ty: Angle(Some(32), false)
                        expr: Expr [40-41]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_angle_truncating() {
    let source = "
        float[32] a;
        angle[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-43]:
                expr: Expr [30-42]:
                    ty: Angle(Some(16), false)
                    kind: Cast [30-42]:
                        ty: Angle(Some(16), false)
                        expr: Expr [40-41]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_angle_expanding() {
    let source = "
        float[32] a;
        angle[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-43]:
                expr: Expr [30-42]:
                    ty: Angle(Some(64), false)
                    kind: Cast [30-42]:
                        ty: Angle(Some(64), false)
                        expr: Expr [40-41]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=================================
// Casts to complex and complex[n]
//=================================

#[test]
fn float_to_complex() {
    let source = "
        float a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-37]:
                expr: Expr [26-36]:
                    ty: Complex(None, false)
                    kind: Cast [26-36]:
                        ty: Complex(None, false)
                        expr: Expr [34-35]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_sized_complex() {
    let source = "
        float a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-48]:
                expr: Expr [26-47]:
                    ty: Complex(Some(32), false)
                    kind: Cast [26-47]:
                        ty: Complex(Some(32), false)
                        expr: Expr [45-46]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_complex() {
    let source = "
        float[32] a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-41]:
                expr: Expr [30-40]:
                    ty: Complex(None, false)
                    kind: Cast [30-40]:
                        ty: Complex(None, false)
                        expr: Expr [38-39]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_complex() {
    let source = "
        float[32] a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-52]:
                expr: Expr [30-51]:
                    ty: Complex(Some(32), false)
                    kind: Cast [30-51]:
                        ty: Complex(Some(32), false)
                        expr: Expr [49-50]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_complex_truncating() {
    let source = "
        float[32] a;
        complex[float[16]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-52]:
                expr: Expr [30-51]:
                    ty: Complex(Some(16), false)
                    kind: Cast [30-51]:
                        ty: Complex(Some(16), false)
                        expr: Expr [49-50]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_sized_complex_expanding() {
    let source = "
        float[32] a;
        complex[float[64]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-52]:
                expr: Expr [30-51]:
                    ty: Complex(Some(64), false)
                    kind: Cast [30-51]:
                        ty: Complex(Some(64), false)
                        expr: Expr [49-50]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn float_to_bit() {
    let source = "
        float a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-17]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [0-0]:
                    ty: Float(None, true)
                    kind: Lit: Float(0.0)
            ExprStmt [26-33]:
                expr: Expr [26-32]:
                    ty: Bit(false)
                    kind: Cast [26-32]:
                        ty: Bit(false)
                        expr: Expr [30-31]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn float_to_bitarray_fails() {
    let source = "
        float a;
        bit[32](a);
    ";
    check(
        source,
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
                    Stmt [26-37]:
                        annotations: <empty>
                        kind: ExprStmt [26-37]:
                            expr: Expr [26-36]:
                                ty: Float(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(None, false) to type BitArray(32,
              | false)
               ,-[test:3:9]
             2 |         float a;
             3 |         bit[32](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_float_to_bit() {
    let source = "
        float[32] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-21]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [0-0]:
                    ty: Float(Some(32), true)
                    kind: Lit: Float(0.0)
            ExprStmt [30-37]:
                expr: Expr [30-36]:
                    ty: Bit(false)
                    kind: Cast [30-36]:
                        ty: Bit(false)
                        expr: Expr [34-35]:
                            ty: Float(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_float_to_bitarray_fails() {
    let source = "
        float[32] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-21]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-21]:
                            symbol_id: 8
                            ty_span: [9-18]
                            init_expr: Expr [0-0]:
                                ty: Float(Some(32), true)
                                kind: Lit: Float(0.0)
                    Stmt [30-41]:
                        annotations: <empty>
                        kind: ExprStmt [30-41]:
                            expr: Expr [30-40]:
                                ty: Float(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(Some(32), false) to type BitArray(32,
              | false)
               ,-[test:3:9]
             2 |         float[32] a;
             3 |         bit[32](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_float_to_bitarray_truncating_fails() {
    let source = "
        float[32] a;
        bit[16](a);
    ";
    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-21]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-21]:
                            symbol_id: 8
                            ty_span: [9-18]
                            init_expr: Expr [0-0]:
                                ty: Float(Some(32), true)
                                kind: Lit: Float(0.0)
                    Stmt [30-41]:
                        annotations: <empty>
                        kind: ExprStmt [30-41]:
                            expr: Expr [30-40]:
                                ty: Float(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(Some(32), false) to type BitArray(16,
              | false)
               ,-[test:3:9]
             2 |         float[32] a;
             3 |         bit[16](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_float_to_bitarray_expanding_fails() {
    let source = "
        float[32] a;
        bit[64](a);
    ";
    check(
        source,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-21]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-21]:
                            symbol_id: 8
                            ty_span: [9-18]
                            init_expr: Expr [0-0]:
                                ty: Float(Some(32), true)
                                kind: Lit: Float(0.0)
                    Stmt [30-41]:
                        annotations: <empty>
                        kind: ExprStmt [30-41]:
                            expr: Expr [30-40]:
                                ty: Float(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Float(Some(32), false) to type BitArray(64,
              | false)
               ,-[test:3:9]
             2 |         float[32] a;
             3 |         bit[64](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}
