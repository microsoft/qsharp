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
fn int_to_bool() {
    let source = "
        int a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-32]:
            expr: Expr [29-30]:
                ty: Bool(false)
                kind: Cast [29-30]:
                    ty: Bool(false)
                    expr: Expr [29-30]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_bool() {
    let source = "
        int[32] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-36]:
            expr: Expr [33-34]:
                ty: Bool(false)
                kind: Cast [33-34]:
                    ty: Bool(false)
                    expr: Expr [33-34]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn int_to_duration_fails() {
    let source = "
        int a;
        duration(a);
    ";
    check(
        source,
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
                            ty: Int(None, true)
                            kind: Lit: Int(0)
                Stmt [24-36]:
                    annotations: <empty>
                    kind: ExprStmt [24-36]:
                        expr: Expr [33-34]:
                            ty: Int(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(None, false) to type Duration(false)
           ,-[test:3:18]
         2 |         int a;
         3 |         duration(a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_int_to_duration_fails() {
    let source = "
        int[32] a;
        duration(a);
    ";
    check(
        source,
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
                            ty: Int(Some(32), true)
                            kind: Lit: Int(0)
                Stmt [28-40]:
                    annotations: <empty>
                    kind: ExprStmt [28-40]:
                        expr: Expr [37-38]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(Some(32), false) to type
          | Duration(false)
           ,-[test:3:18]
         2 |         int[32] a;
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
fn int_to_int() {
    let source = "
        int a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-31]:
            expr: Expr [28-29]:
                ty: Int(None, false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn int_to_sized_int() {
    let source = "
        int a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-35]:
            expr: Expr [32-33]:
                ty: Int(Some(32), false)
                kind: Cast [32-33]:
                    ty: Int(Some(32), false)
                    expr: Expr [32-33]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_int() {
    let source = "
        int[32] a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-35]:
            expr: Expr [32-33]:
                ty: Int(None, false)
                kind: Cast [32-33]:
                    ty: Int(None, false)
                    expr: Expr [32-33]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_int() {
    let source = "
        int[32] a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: Int(Some(32), false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_int_truncating() {
    let source = "
        int[32] a;
        int[16](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: Int(Some(16), false)
                kind: Cast [36-37]:
                    ty: Int(Some(16), false)
                    expr: Expr [36-37]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_int_expanding() {
    let source = "
        int[32] a;
        int[64](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: Int(Some(64), false)
                kind: Cast [36-37]:
                    ty: Int(Some(64), false)
                    expr: Expr [36-37]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn int_to_uint() {
    let source = "
        int a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-32]:
            expr: Expr [29-30]:
                ty: UInt(None, false)
                kind: Cast [29-30]:
                    ty: UInt(None, false)
                    expr: Expr [29-30]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn int_to_sized_uint() {
    let source = "
        int a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-36]:
            expr: Expr [33-34]:
                ty: UInt(Some(32), false)
                kind: Cast [33-34]:
                    ty: UInt(Some(32), false)
                    expr: Expr [33-34]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_uint() {
    let source = "
        int[32] a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-36]:
            expr: Expr [33-34]:
                ty: UInt(None, false)
                kind: Cast [33-34]:
                    ty: UInt(None, false)
                    expr: Expr [33-34]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_uint() {
    let source = "
        int[32] a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-40]:
            expr: Expr [37-38]:
                ty: UInt(Some(32), false)
                kind: Cast [37-38]:
                    ty: UInt(Some(32), false)
                    expr: Expr [37-38]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_uint_truncating() {
    let source = "
        int[32] a;
        uint[16](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-40]:
            expr: Expr [37-38]:
                ty: UInt(Some(16), false)
                kind: Cast [37-38]:
                    ty: UInt(Some(16), false)
                    expr: Expr [37-38]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_uint_expanding() {
    let source = "
        int[32] a;
        uint[64](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-40]:
            expr: Expr [37-38]:
                ty: UInt(Some(64), false)
                kind: Cast [37-38]:
                    ty: UInt(Some(64), false)
                    expr: Expr [37-38]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn int_to_float() {
    let source = "
        int a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-33]:
            expr: Expr [30-31]:
                ty: Float(None, false)
                kind: Cast [30-31]:
                    ty: Float(None, false)
                    expr: Expr [30-31]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn int_to_sized_float() {
    let source = "
        int a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-37]:
            expr: Expr [34-35]:
                ty: Float(Some(32), false)
                kind: Cast [34-35]:
                    ty: Float(Some(32), false)
                    expr: Expr [34-35]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_float() {
    let source = "
        int[32] a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-37]:
            expr: Expr [34-35]:
                ty: Float(None, false)
                kind: Cast [34-35]:
                    ty: Float(None, false)
                    expr: Expr [34-35]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_float() {
    let source = "
        int[32] a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-41]:
            expr: Expr [38-39]:
                ty: Float(Some(32), false)
                kind: Cast [38-39]:
                    ty: Float(Some(32), false)
                    expr: Expr [38-39]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_float_truncating() {
    let source = "
        int[32] a;
        float[16](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-41]:
            expr: Expr [38-39]:
                ty: Float(Some(16), false)
                kind: Cast [38-39]:
                    ty: Float(Some(16), false)
                    expr: Expr [38-39]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_float_expanding() {
    let source = "
        int[32] a;
        float[64](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-41]:
            expr: Expr [38-39]:
                ty: Float(Some(64), false)
                kind: Cast [38-39]:
                    ty: Float(Some(64), false)
                    expr: Expr [38-39]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn int_to_angle_fails() {
    let source = "
        int a;
        angle(a);
    ";
    check(
        source,
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
                            ty: Int(None, true)
                            kind: Lit: Int(0)
                Stmt [24-33]:
                    annotations: <empty>
                    kind: ExprStmt [24-33]:
                        expr: Expr [30-31]:
                            ty: Int(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(None, false) to type Angle(None, false)
           ,-[test:3:15]
         2 |         int a;
         3 |         angle(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn int_to_sized_angle_fails() {
    let source = "
        int a;
        angle[32](a);
    ";
    check(
        source,
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
                            ty: Int(None, true)
                            kind: Lit: Int(0)
                Stmt [24-37]:
                    annotations: <empty>
                    kind: ExprStmt [24-37]:
                        expr: Expr [34-35]:
                            ty: Int(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(None, false) to type Angle(Some(32),
          | false)
           ,-[test:3:19]
         2 |         int a;
         3 |         angle[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_int_to_angle_fails() {
    let source = "
        int[32] a;
        angle(a);
    ";
    check(
        source,
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
                            ty: Int(Some(32), true)
                            kind: Lit: Int(0)
                Stmt [28-37]:
                    annotations: <empty>
                    kind: ExprStmt [28-37]:
                        expr: Expr [34-35]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(Some(32), false) to type Angle(None,
          | false)
           ,-[test:3:15]
         2 |         int[32] a;
         3 |         angle(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_int_to_sized_angle_fails() {
    let source = "
        int[32] a;
        angle[32](a);
    ";
    check(
        source,
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
                            ty: Int(Some(32), true)
                            kind: Lit: Int(0)
                Stmt [28-41]:
                    annotations: <empty>
                    kind: ExprStmt [28-41]:
                        expr: Expr [38-39]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(Some(32), false) to type
          | Angle(Some(32), false)
           ,-[test:3:19]
         2 |         int[32] a;
         3 |         angle[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_int_to_sized_angle_truncating_fails() {
    let source = "
        int[32] a;
        angle[16](a);
    ";
    check(
        source,
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
                            ty: Int(Some(32), true)
                            kind: Lit: Int(0)
                Stmt [28-41]:
                    annotations: <empty>
                    kind: ExprStmt [28-41]:
                        expr: Expr [38-39]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(Some(32), false) to type
          | Angle(Some(16), false)
           ,-[test:3:19]
         2 |         int[32] a;
         3 |         angle[16](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_int_to_sized_angle_expanding_fails() {
    let source = "
        int[32] a;
        angle[64](a);
    ";
    check(
        source,
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
                            ty: Int(Some(32), true)
                            kind: Lit: Int(0)
                Stmt [28-41]:
                    annotations: <empty>
                    kind: ExprStmt [28-41]:
                        expr: Expr [38-39]:
                            ty: Int(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Int(Some(32), false) to type
          | Angle(Some(64), false)
           ,-[test:3:19]
         2 |         int[32] a;
         3 |         angle[64](a);
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
fn int_to_complex() {
    let source = "
        int a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-35]:
            expr: Expr [32-33]:
                ty: Complex(None, false)
                kind: Cast [32-33]:
                    ty: Complex(None, false)
                    expr: Expr [32-33]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn int_to_sized_complex() {
    let source = "
        int a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-46]:
            expr: Expr [43-44]:
                ty: Complex(Some(32), false)
                kind: Cast [43-44]:
                    ty: Complex(Some(32), false)
                    expr: Expr [43-44]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_complex() {
    let source = "
        int[32] a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: Complex(None, false)
                kind: Cast [36-37]:
                    ty: Complex(None, false)
                    expr: Expr [36-37]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_complex() {
    let source = "
        int[32] a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-50]:
            expr: Expr [47-48]:
                ty: Complex(Some(32), false)
                kind: Cast [47-48]:
                    ty: Complex(Some(32), false)
                    expr: Expr [47-48]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_complex_truncating() {
    let source = "
        int[32] a;
        complex[float[16]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-50]:
            expr: Expr [47-48]:
                ty: Complex(Some(16), false)
                kind: Cast [47-48]:
                    ty: Complex(Some(16), false)
                    expr: Expr [47-48]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_sized_complex_expanding() {
    let source = "
        int[32] a;
        complex[float[64]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-50]:
            expr: Expr [47-48]:
                ty: Complex(Some(64), false)
                kind: Cast [47-48]:
                    ty: Complex(Some(64), false)
                    expr: Expr [47-48]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn int_to_bit() {
    let source = "
        int a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-15]:
            symbol_id: 8
            ty_span: [9-12]
            init_expr: Expr [0-0]:
                ty: Int(None, true)
                kind: Lit: Int(0)
        ExprStmt [24-31]:
            expr: Expr [28-29]:
                ty: Bit(false)
                kind: Cast [28-29]:
                    ty: Bit(false)
                    expr: Expr [28-29]:
                        ty: Int(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn int_to_bitarray_fails() {
    let source = "
        int a;
        bit[32](a);
    ";
    check(
        source,
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
                                ty: Int(None, true)
                                kind: Lit: Int(0)
                    Stmt [24-35]:
                        annotations: <empty>
                        kind: ExprStmt [24-35]:
                            expr: Expr [32-33]:
                                ty: Int(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(None, false) to type BitArray(32,
              | false)
               ,-[test:3:17]
             2 |         int a;
             3 |         bit[32](a);
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_int_to_bit() {
    let source = "
        int[32] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-35]:
            expr: Expr [32-33]:
                ty: Bit(false)
                kind: Cast [32-33]:
                    ty: Bit(false)
                    expr: Expr [32-33]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_bitarray() {
    let source = "
        int[32] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Int(Some(32), true)
                kind: Lit: Int(0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: BitArray(32, false)
                kind: Cast [36-37]:
                    ty: BitArray(32, false)
                    expr: Expr [36-37]:
                        ty: Int(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_int_to_bitarray_truncating_fails() {
    let source = "
        int[32] a;
        bit[16](a);
    ";
    check(
        source,
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
                                ty: Int(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [28-39]:
                        annotations: <empty>
                        kind: ExprStmt [28-39]:
                            expr: Expr [36-37]:
                                ty: Int(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(Some(32), false) to type BitArray(16,
              | false)
               ,-[test:3:17]
             2 |         int[32] a;
             3 |         bit[16](a);
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_int_to_bitarray_expanding_fails() {
    let source = "
        int[32] a;
        bit[64](a);
    ";
    check(
        source,
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
                                ty: Int(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [28-39]:
                        annotations: <empty>
                        kind: ExprStmt [28-39]:
                            expr: Expr [36-37]:
                                ty: Int(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Int(Some(32), false) to type BitArray(64,
              | false)
               ,-[test:3:17]
             2 |         int[32] a;
             3 |         bit[64](a);
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}
