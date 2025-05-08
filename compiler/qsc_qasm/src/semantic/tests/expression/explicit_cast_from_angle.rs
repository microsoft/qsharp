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
fn angle_to_bool() {
    let source = "
        angle a;
        bool(a);
    ";
    check(
        source,
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
                kind: Cast [31-32]:
                    ty: Bool(false)
                    expr: Expr [31-32]:
                        ty: Angle(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_bool() {
    let source = "
        angle[32] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-38]:
            expr: Expr [35-36]:
                ty: Bool(false)
                kind: Cast [35-36]:
                    ty: Bool(false)
                    expr: Expr [35-36]:
                        ty: Angle(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn angle_to_duration_fails() {
    let source = "
        angle a;
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
         2 |         angle a;
         3 |         duration(a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_duration_fails() {
    let source = "
        angle[32] a;
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-42]:
                    annotations: <empty>
                    kind: ExprStmt [30-42]:
                        expr: Expr [39-40]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Duration(false)
           ,-[test:3:18]
         2 |         angle[32] a;
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
fn angle_to_int_fails() {
    let source = "
        angle a;
        int(a);
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
         2 |         angle a;
         3 |         int(a);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_sized_int_fails() {
    let source = "
        angle a;
        int[32](a);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-37]:
                    annotations: <empty>
                    kind: ExprStmt [26-37]:
                        expr: Expr [34-35]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type Int(Some(32),
          | false)
           ,-[test:3:17]
         2 |         angle a;
         3 |         int[32](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_int_fails() {
    let source = "
        angle[32] a;
        int(a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-37]:
                    annotations: <empty>
                    kind: ExprStmt [30-37]:
                        expr: Expr [34-35]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type Int(None,
          | false)
           ,-[test:3:13]
         2 |         angle[32] a;
         3 |         int(a);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_int_fails() {
    let source = "
        angle[32] a;
        int[32](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-41]:
                    annotations: <empty>
                    kind: ExprStmt [30-41]:
                        expr: Expr [38-39]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Int(Some(32), false)
           ,-[test:3:17]
         2 |         angle[32] a;
         3 |         int[32](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_int_truncating_fails() {
    let source = "
        angle[32] a;
        int[16](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-41]:
                    annotations: <empty>
                    kind: ExprStmt [30-41]:
                        expr: Expr [38-39]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Int(Some(16), false)
           ,-[test:3:17]
         2 |         angle[32] a;
         3 |         int[16](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_int_expanding_fails() {
    let source = "
        angle[32] a;
        int[64](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-41]:
                    annotations: <empty>
                    kind: ExprStmt [30-41]:
                        expr: Expr [38-39]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Int(Some(64), false)
           ,-[test:3:17]
         2 |         angle[32] a;
         3 |         int[64](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn angle_to_uint_fails() {
    let source = "
        angle a;
        uint(a);
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
         2 |         angle a;
         3 |         uint(a);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_sized_uint_fails() {
    let source = "
        angle a;
        uint[32](a);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-38]:
                    annotations: <empty>
                    kind: ExprStmt [26-38]:
                        expr: Expr [35-36]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type UInt(Some(32),
          | false)
           ,-[test:3:18]
         2 |         angle a;
         3 |         uint[32](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_uint_fails() {
    let source = "
        angle[32] a;
        uint(a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-38]:
                    annotations: <empty>
                    kind: ExprStmt [30-38]:
                        expr: Expr [35-36]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type UInt(None,
          | false)
           ,-[test:3:14]
         2 |         angle[32] a;
         3 |         uint(a);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_uint_fails() {
    let source = "
        angle[32] a;
        uint[32](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-42]:
                    annotations: <empty>
                    kind: ExprStmt [30-42]:
                        expr: Expr [39-40]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | UInt(Some(32), false)
           ,-[test:3:18]
         2 |         angle[32] a;
         3 |         uint[32](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_uint_truncating_fails() {
    let source = "
        angle[32] a;
        uint[16](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-42]:
                    annotations: <empty>
                    kind: ExprStmt [30-42]:
                        expr: Expr [39-40]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | UInt(Some(16), false)
           ,-[test:3:18]
         2 |         angle[32] a;
         3 |         uint[16](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_uint_expanding_fails() {
    let source = "
        angle[32] a;
        uint[64](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-42]:
                    annotations: <empty>
                    kind: ExprStmt [30-42]:
                        expr: Expr [39-40]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | UInt(Some(64), false)
           ,-[test:3:18]
         2 |         angle[32] a;
         3 |         uint[64](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn angle_to_float_fails() {
    let source = "
        angle a;
        float(a);
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
         2 |         angle a;
         3 |         float(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_sized_float_fails() {
    let source = "
        angle a;
        float[32](a);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-39]:
                    annotations: <empty>
                    kind: ExprStmt [26-39]:
                        expr: Expr [36-37]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type Float(Some(32),
          | false)
           ,-[test:3:19]
         2 |         angle a;
         3 |         float[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_float_fails() {
    let source = "
        angle[32] a;
        float(a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-39]:
                    annotations: <empty>
                    kind: ExprStmt [30-39]:
                        expr: Expr [36-37]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type Float(None,
          | false)
           ,-[test:3:15]
         2 |         angle[32] a;
         3 |         float(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_float_fails() {
    let source = "
        angle[32] a;
        float[32](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-43]:
                    annotations: <empty>
                    kind: ExprStmt [30-43]:
                        expr: Expr [40-41]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Float(Some(32), false)
           ,-[test:3:19]
         2 |         angle[32] a;
         3 |         float[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_float_truncating_fails() {
    let source = "
        angle[32] a;
        float[16](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-43]:
                    annotations: <empty>
                    kind: ExprStmt [30-43]:
                        expr: Expr [40-41]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Float(Some(16), false)
           ,-[test:3:19]
         2 |         angle[32] a;
         3 |         float[16](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_float_expanding_fails() {
    let source = "
        angle[32] a;
        float[64](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-43]:
                    annotations: <empty>
                    kind: ExprStmt [30-43]:
                        expr: Expr [40-41]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Float(Some(64), false)
           ,-[test:3:19]
         2 |         angle[32] a;
         3 |         float[64](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn angle_to_angle() {
    let source = "
        angle a;
        angle(a);
    ";
    check(
        source,
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
fn angle_to_sized_angle() {
    let source = "
        angle a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-17]:
            symbol_id: 8
            ty_span: [9-14]
            init_expr: Expr [0-0]:
                ty: Angle(None, true)
                kind: Lit: Angle(0)
        ExprStmt [26-39]:
            expr: Expr [36-37]:
                ty: Angle(Some(32), false)
                kind: Cast [36-37]:
                    ty: Angle(Some(32), false)
                    expr: Expr [36-37]:
                        ty: Angle(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_angle() {
    let source = "
        angle[32] a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-39]:
            expr: Expr [36-37]:
                ty: Angle(None, false)
                kind: Cast [36-37]:
                    ty: Angle(None, false)
                    expr: Expr [36-37]:
                        ty: Angle(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_sized_angle() {
    let source = "
        angle[32] a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-43]:
            expr: Expr [40-41]:
                ty: Angle(Some(32), false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_sized_angle_truncating() {
    let source = "
        angle[32] a;
        angle[16](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-43]:
            expr: Expr [40-41]:
                ty: Angle(Some(16), false)
                kind: Cast [40-41]:
                    ty: Angle(Some(16), false)
                    expr: Expr [40-41]:
                        ty: Angle(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_sized_angle_expanding() {
    let source = "
        angle[32] a;
        angle[64](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-43]:
            expr: Expr [40-41]:
                ty: Angle(Some(64), false)
                kind: Cast [40-41]:
                    ty: Angle(Some(64), false)
                    expr: Expr [40-41]:
                        ty: Angle(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=================================
// Casts to complex and complex[n]
//=================================

#[test]
fn angle_to_complex_fails() {
    let source = "
        angle a;
        complex(a);
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
         2 |         angle a;
         3 |         complex(a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn angle_to_sized_complex_fails() {
    let source = "
        angle a;
        complex[float[32]](a);
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
                            ty: Angle(None, true)
                            kind: Lit: Angle(0)
                Stmt [26-48]:
                    annotations: <empty>
                    kind: ExprStmt [26-48]:
                        expr: Expr [45-46]:
                            ty: Angle(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(None, false) to type
          | Complex(Some(32), false)
           ,-[test:3:28]
         2 |         angle a;
         3 |         complex[float[32]](a);
           :                            ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_complex_fails() {
    let source = "
        angle[32] a;
        complex(a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-41]:
                    annotations: <empty>
                    kind: ExprStmt [30-41]:
                        expr: Expr [38-39]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Complex(None, false)
           ,-[test:3:17]
         2 |         angle[32] a;
         3 |         complex(a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_complex_fails() {
    let source = "
        angle[32] a;
        complex[float[32]](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-52]:
                    annotations: <empty>
                    kind: ExprStmt [30-52]:
                        expr: Expr [49-50]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Complex(Some(32), false)
           ,-[test:3:28]
         2 |         angle[32] a;
         3 |         complex[float[32]](a);
           :                            ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_complex_truncating_fails() {
    let source = "
        angle[32] a;
        complex[float[16]](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-52]:
                    annotations: <empty>
                    kind: ExprStmt [30-52]:
                        expr: Expr [49-50]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Complex(Some(16), false)
           ,-[test:3:28]
         2 |         angle[32] a;
         3 |         complex[float[16]](a);
           :                            ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_angle_to_sized_complex_expanding_fails() {
    let source = "
        angle[32] a;
        complex[float[64]](a);
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
                            ty: Angle(Some(32), true)
                            kind: Lit: Angle(0)
                Stmt [30-52]:
                    annotations: <empty>
                    kind: ExprStmt [30-52]:
                        expr: Expr [49-50]:
                            ty: Angle(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Angle(Some(32), false) to type
          | Complex(Some(64), false)
           ,-[test:3:28]
         2 |         angle[32] a;
         3 |         complex[float[64]](a);
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
fn angle_to_bit() {
    let source = "
        angle a;
        bit(a);
    ";
    check(
        source,
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
                kind: Cast [30-31]:
                    ty: Bit(false)
                    expr: Expr [30-31]:
                        ty: Angle(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn angle_to_bitarray_fails() {
    let source = "
        angle a;
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
                                ty: Angle(None, true)
                                kind: Lit: Angle(0)
                    Stmt [26-37]:
                        annotations: <empty>
                        kind: ExprStmt [26-37]:
                            expr: Expr [34-35]:
                                ty: Angle(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(None, false) to type BitArray(32,
              | false)
               ,-[test:3:17]
             2 |         angle a;
             3 |         bit[32](a);
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_angle_to_bit() {
    let source = "
        angle[32] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-37]:
            expr: Expr [34-35]:
                ty: Bit(false)
                kind: Cast [34-35]:
                    ty: Bit(false)
                    expr: Expr [34-35]:
                        ty: Angle(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_bitarray() {
    let source = "
        angle[32] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-21]:
            symbol_id: 8
            ty_span: [9-18]
            init_expr: Expr [0-0]:
                ty: Angle(Some(32), true)
                kind: Lit: Angle(0)
        ExprStmt [30-41]:
            expr: Expr [38-39]:
                ty: BitArray(32, false)
                kind: Cast [38-39]:
                    ty: BitArray(32, false)
                    expr: Expr [38-39]:
                        ty: Angle(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_angle_to_bitarray_truncating_fails() {
    let source = "
        angle[32] a;
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
                                ty: Angle(Some(32), true)
                                kind: Lit: Angle(0)
                    Stmt [30-41]:
                        annotations: <empty>
                        kind: ExprStmt [30-41]:
                            expr: Expr [38-39]:
                                ty: Angle(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type BitArray(16,
              | false)
               ,-[test:3:17]
             2 |         angle[32] a;
             3 |         bit[16](a);
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_angle_to_bitarray_expanding_fails() {
    let source = "
        angle[32] a;
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
                                ty: Angle(Some(32), true)
                                kind: Lit: Angle(0)
                    Stmt [30-41]:
                        annotations: <empty>
                        kind: ExprStmt [30-41]:
                            expr: Expr [38-39]:
                                ty: Angle(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Angle(Some(32), false) to type BitArray(64,
              | false)
               ,-[test:3:17]
             2 |         angle[32] a;
             3 |         bit[64](a);
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}
