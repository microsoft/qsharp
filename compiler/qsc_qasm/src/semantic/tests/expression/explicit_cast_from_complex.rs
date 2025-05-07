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
fn complex_to_bool_fails() {
    let source = "
        complex a;
        bool(a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-36]:
                    annotations: <empty>
                    kind: ExprStmt [28-36]:
                        expr: Expr [33-34]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Bool(false)
           ,-[test:3:14]
         2 |         complex a;
         3 |         bool(a);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_bool_fails() {
    let source = "
        complex[float[32]] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-47]:
                    annotations: <empty>
                    kind: ExprStmt [39-47]:
                        expr: Expr [44-45]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Bool(false)
           ,-[test:3:14]
         2 |         complex[float[32]] a;
         3 |         bool(a);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn complex_to_duration_fails() {
    let source = "
        complex a;
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-40]:
                    annotations: <empty>
                    kind: ExprStmt [28-40]:
                        expr: Expr [37-38]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type
          | Duration(false)
           ,-[test:3:18]
         2 |         complex a;
         3 |         duration(a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_duration_fails() {
    let source = "
        complex[float[32]] a;
        duration(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-51]:
                    annotations: <empty>
                    kind: ExprStmt [39-51]:
                        expr: Expr [48-49]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Duration(false)
           ,-[test:3:18]
         2 |         complex[float[32]] a;
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
fn complex_to_int_fails() {
    let source = "
        complex a;
        int(a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-35]:
                    annotations: <empty>
                    kind: ExprStmt [28-35]:
                        expr: Expr [32-33]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Int(None,
          | false)
           ,-[test:3:13]
         2 |         complex a;
         3 |         int(a);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_sized_int_fails() {
    let source = "
        complex a;
        int[32](a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-39]:
                    annotations: <empty>
                    kind: ExprStmt [28-39]:
                        expr: Expr [36-37]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Int(Some(32),
          | false)
           ,-[test:3:17]
         2 |         complex a;
         3 |         int[32](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_int_fails() {
    let source = "
        complex[float[32]] a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-46]:
                    annotations: <empty>
                    kind: ExprStmt [39-46]:
                        expr: Expr [43-44]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type Int(None,
          | false)
           ,-[test:3:13]
         2 |         complex[float[32]] a;
         3 |         int(a);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_int_fails() {
    let source = "
        complex[float[32]] a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-50]:
                    annotations: <empty>
                    kind: ExprStmt [39-50]:
                        expr: Expr [47-48]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Int(Some(32), false)
           ,-[test:3:17]
         2 |         complex[float[32]] a;
         3 |         int[32](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_int_truncating_fails() {
    let source = "
        complex[float[32]] a;
        int[16](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-50]:
                    annotations: <empty>
                    kind: ExprStmt [39-50]:
                        expr: Expr [47-48]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Int(Some(16), false)
           ,-[test:3:17]
         2 |         complex[float[32]] a;
         3 |         int[16](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_int_expanding_fails() {
    let source = "
        complex[float[32]] a;
        int[64](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-50]:
                    annotations: <empty>
                    kind: ExprStmt [39-50]:
                        expr: Expr [47-48]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Int(Some(64), false)
           ,-[test:3:17]
         2 |         complex[float[32]] a;
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
fn complex_to_uint_fails() {
    let source = "
        complex a;
        uint(a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-36]:
                    annotations: <empty>
                    kind: ExprStmt [28-36]:
                        expr: Expr [33-34]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type UInt(None,
          | false)
           ,-[test:3:14]
         2 |         complex a;
         3 |         uint(a);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_sized_uint_fails() {
    let source = "
        complex a;
        uint[32](a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-40]:
                    annotations: <empty>
                    kind: ExprStmt [28-40]:
                        expr: Expr [37-38]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type UInt(Some(32),
          | false)
           ,-[test:3:18]
         2 |         complex a;
         3 |         uint[32](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_uint_fails() {
    let source = "
        complex[float[32]] a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-47]:
                    annotations: <empty>
                    kind: ExprStmt [39-47]:
                        expr: Expr [44-45]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type UInt(None,
          | false)
           ,-[test:3:14]
         2 |         complex[float[32]] a;
         3 |         uint(a);
           :              ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_uint_fails() {
    let source = "
        complex[float[32]] a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-51]:
                    annotations: <empty>
                    kind: ExprStmt [39-51]:
                        expr: Expr [48-49]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | UInt(Some(32), false)
           ,-[test:3:18]
         2 |         complex[float[32]] a;
         3 |         uint[32](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_uint_truncating_fails() {
    let source = "
        complex[float[32]] a;
        uint[16](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-51]:
                    annotations: <empty>
                    kind: ExprStmt [39-51]:
                        expr: Expr [48-49]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | UInt(Some(16), false)
           ,-[test:3:18]
         2 |         complex[float[32]] a;
         3 |         uint[16](a);
           :                  ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_uint_expanding_fails() {
    let source = "
        complex[float[32]] a;
        uint[64](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-51]:
                    annotations: <empty>
                    kind: ExprStmt [39-51]:
                        expr: Expr [48-49]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | UInt(Some(64), false)
           ,-[test:3:18]
         2 |         complex[float[32]] a;
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
fn complex_to_float_fails() {
    let source = "
        complex a;
        float(a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-37]:
                    annotations: <empty>
                    kind: ExprStmt [28-37]:
                        expr: Expr [34-35]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Float(None,
          | false)
           ,-[test:3:15]
         2 |         complex a;
         3 |         float(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_sized_float_fails() {
    let source = "
        complex a;
        float[32](a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-41]:
                    annotations: <empty>
                    kind: ExprStmt [28-41]:
                        expr: Expr [38-39]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type
          | Float(Some(32), false)
           ,-[test:3:19]
         2 |         complex a;
         3 |         float[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_float_fails() {
    let source = "
        complex[float[32]] a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-48]:
                    annotations: <empty>
                    kind: ExprStmt [39-48]:
                        expr: Expr [45-46]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Float(None, false)
           ,-[test:3:15]
         2 |         complex[float[32]] a;
         3 |         float(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_float_fails() {
    let source = "
        complex[float[32]] a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-52]:
                    annotations: <empty>
                    kind: ExprStmt [39-52]:
                        expr: Expr [49-50]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Float(Some(32), false)
           ,-[test:3:19]
         2 |         complex[float[32]] a;
         3 |         float[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_float_truncating_fails() {
    let source = "
        complex[float[32]] a;
        float[16](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-52]:
                    annotations: <empty>
                    kind: ExprStmt [39-52]:
                        expr: Expr [49-50]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Float(Some(16), false)
           ,-[test:3:19]
         2 |         complex[float[32]] a;
         3 |         float[16](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_float_expanding_fails() {
    let source = "
        complex[float[32]] a;
        float[64](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-52]:
                    annotations: <empty>
                    kind: ExprStmt [39-52]:
                        expr: Expr [49-50]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Float(Some(64), false)
           ,-[test:3:19]
         2 |         complex[float[32]] a;
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
fn complex_to_angle_fails() {
    let source = "
        complex a;
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-37]:
                    annotations: <empty>
                    kind: ExprStmt [28-37]:
                        expr: Expr [34-35]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Angle(None,
          | false)
           ,-[test:3:15]
         2 |         complex a;
         3 |         angle(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_sized_angle_fails() {
    let source = "
        complex a;
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-41]:
                    annotations: <empty>
                    kind: ExprStmt [28-41]:
                        expr: Expr [38-39]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type
          | Angle(Some(32), false)
           ,-[test:3:19]
         2 |         complex a;
         3 |         angle[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_angle_fails() {
    let source = "
        complex[float[32]] a;
        angle(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-48]:
                    annotations: <empty>
                    kind: ExprStmt [39-48]:
                        expr: Expr [45-46]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Angle(None, false)
           ,-[test:3:15]
         2 |         complex[float[32]] a;
         3 |         angle(a);
           :               ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_angle_fails() {
    let source = "
        complex[float[32]] a;
        angle[32](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-52]:
                    annotations: <empty>
                    kind: ExprStmt [39-52]:
                        expr: Expr [49-50]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Angle(Some(32), false)
           ,-[test:3:19]
         2 |         complex[float[32]] a;
         3 |         angle[32](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_angle_truncating_fails() {
    let source = "
        complex[float[32]] a;
        angle[16](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-52]:
                    annotations: <empty>
                    kind: ExprStmt [39-52]:
                        expr: Expr [49-50]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Angle(Some(16), false)
           ,-[test:3:19]
         2 |         complex[float[32]] a;
         3 |         angle[16](a);
           :                   ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_sized_angle_expanding_fails() {
    let source = "
        complex[float[32]] a;
        angle[64](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-52]:
                    annotations: <empty>
                    kind: ExprStmt [39-52]:
                        expr: Expr [49-50]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | Angle(Some(64), false)
           ,-[test:3:19]
         2 |         complex[float[32]] a;
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
fn complex_to_complex() {
    let source = "
        complex a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Complex(None, true)
                kind: Lit: Complex(0.0, 0.0)
        ExprStmt [28-39]:
            expr: Expr [36-37]:
                ty: Complex(None, false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn complex_to_sized_complex() {
    let source = "
        complex a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-19]:
            symbol_id: 8
            ty_span: [9-16]
            init_expr: Expr [0-0]:
                ty: Complex(None, true)
                kind: Lit: Complex(0.0, 0.0)
        ExprStmt [28-50]:
            expr: Expr [47-48]:
                ty: Complex(Some(32), false)
                kind: Cast [47-48]:
                    ty: Complex(Some(32), false)
                    expr: Expr [47-48]:
                        ty: Complex(None, false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_complex_to_complex() {
    let source = "
        complex[float[32]] a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [9-27]
            init_expr: Expr [0-0]:
                ty: Complex(Some(32), true)
                kind: Lit: Complex(0.0, 0.0)
        ExprStmt [39-50]:
            expr: Expr [47-48]:
                ty: Complex(None, false)
                kind: Cast [47-48]:
                    ty: Complex(None, false)
                    expr: Expr [47-48]:
                        ty: Complex(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_complex_to_sized_complex() {
    let source = "
        complex[float[32]] a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [9-27]
            init_expr: Expr [0-0]:
                ty: Complex(Some(32), true)
                kind: Lit: Complex(0.0, 0.0)
        ExprStmt [39-61]:
            expr: Expr [58-59]:
                ty: Complex(Some(32), false)
                kind: SymbolId(8)
    "#]],
    );
}

#[test]
fn sized_complex_to_sized_complex_truncating() {
    let source = "
        complex[float[32]] a;
        complex[float[16]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-30]:
                symbol_id: 8
                ty_span: [9-27]
                init_expr: Expr [0-0]:
                    ty: Complex(Some(32), true)
                    kind: Lit: Complex(0.0, 0.0)
            ExprStmt [39-61]:
                expr: Expr [58-59]:
                    ty: Complex(Some(32), false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_complex_to_sized_complex_expanding() {
    let source = "
        complex[float[32]] a;
        complex[float[64]](a);
    ";
    check(
        source,
        &expect![[r#"
        ClassicalDeclarationStmt [9-30]:
            symbol_id: 8
            ty_span: [9-27]
            init_expr: Expr [0-0]:
                ty: Complex(Some(32), true)
                kind: Lit: Complex(0.0, 0.0)
        ExprStmt [39-61]:
            expr: Expr [58-59]:
                ty: Complex(Some(64), false)
                kind: Cast [58-59]:
                    ty: Complex(Some(64), false)
                    expr: Expr [58-59]:
                        ty: Complex(Some(32), false)
                        kind: SymbolId(8)
    "#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn complex_to_bit_fails() {
    let source = "
        complex a;
        bit(a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-35]:
                    annotations: <empty>
                    kind: ExprStmt [28-35]:
                        expr: Expr [32-33]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type Bit(false)
           ,-[test:3:13]
         2 |         complex a;
         3 |         bit(a);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn complex_to_bitarray_fails() {
    let source = "
        complex a;
        bit[32](a);
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
                            ty: Complex(None, true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [28-39]:
                    annotations: <empty>
                    kind: ExprStmt [28-39]:
                        expr: Expr [36-37]:
                            ty: Complex(None, false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(None, false) to type BitArray(32,
          | false)
           ,-[test:3:17]
         2 |         complex a;
         3 |         bit[32](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_bit_fails() {
    let source = "
        complex[float[32]] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-46]:
                    annotations: <empty>
                    kind: ExprStmt [39-46]:
                        expr: Expr [43-44]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type Bit(false)
           ,-[test:3:13]
         2 |         complex[float[32]] a;
         3 |         bit(a);
           :             ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_bitarray_fails() {
    let source = "
        complex[float[32]] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-50]:
                    annotations: <empty>
                    kind: ExprStmt [39-50]:
                        expr: Expr [47-48]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | BitArray(32, false)
           ,-[test:3:17]
         2 |         complex[float[32]] a;
         3 |         bit[32](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_bitarray_truncating_fails() {
    let source = "
        complex[float[32]] a;
        bit[16](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-50]:
                    annotations: <empty>
                    kind: ExprStmt [39-50]:
                        expr: Expr [47-48]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | BitArray(16, false)
           ,-[test:3:17]
         2 |         complex[float[32]] a;
         3 |         bit[16](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}

#[test]
fn sized_complex_to_bitarray_expanding_fails() {
    let source = "
        complex[float[32]] a;
        bit[64](a);
    ";
    check(
        source,
        &expect![[r#"
        Program:
            version: <none>
            statements:
                Stmt [9-30]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [9-30]:
                        symbol_id: 8
                        ty_span: [9-27]
                        init_expr: Expr [0-0]:
                            ty: Complex(Some(32), true)
                            kind: Lit: Complex(0.0, 0.0)
                Stmt [39-50]:
                    annotations: <empty>
                    kind: ExprStmt [39-50]:
                        expr: Expr [47-48]:
                            ty: Complex(Some(32), false)
                            kind: SymbolId(8)

        [Qasm.Lowerer.CannotCast

          x cannot cast expression of type Complex(Some(32), false) to type
          | BitArray(64, false)
           ,-[test:3:17]
         2 |         complex[float[32]] a;
         3 |         bit[64](a);
           :                 ^
         4 |     
           `----
        ]"#]],
    );
}
