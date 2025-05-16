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
fn uint_to_bool() {
    let source = "
        uint a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-33]:
                expr: Expr [25-32]:
                    ty: Bool(false)
                    kind: Cast [25-32]:
                        ty: Bool(false)
                        expr: Expr [30-31]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_bool() {
    let source = "
        uint[32] a;
        bool(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-37]:
                expr: Expr [29-36]:
                    ty: Bool(false)
                    kind: Cast [29-36]:
                        ty: Bool(false)
                        expr: Expr [34-35]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//===================
// Casts to duration
//===================

#[test]
fn uint_to_duration_fails() {
    let source = "
        uint a;
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
                                ty: UInt(None, true)
                                kind: Lit: Int(0)
                    Stmt [25-37]:
                        annotations: <empty>
                        kind: ExprStmt [25-37]:
                            expr: Expr [25-36]:
                                ty: UInt(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(None, false) to type Duration(false)
               ,-[test:3:9]
             2 |         uint a;
             3 |         duration(a);
               :         ^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_duration_fails() {
    let source = "
        uint[32] a;
        duration(a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-41]:
                        annotations: <empty>
                        kind: ExprStmt [29-41]:
                            expr: Expr [29-40]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type
              | Duration(false)
               ,-[test:3:9]
             2 |         uint[32] a;
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
fn uint_to_int() {
    let source = "
        uint a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-32]:
                expr: Expr [25-31]:
                    ty: Int(None, false)
                    kind: Cast [25-31]:
                        ty: Int(None, false)
                        expr: Expr [29-30]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn uint_to_sized_int() {
    let source = "
        uint a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-36]:
                expr: Expr [25-35]:
                    ty: Int(Some(32), false)
                    kind: Cast [25-35]:
                        ty: Int(Some(32), false)
                        expr: Expr [33-34]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_int() {
    let source = "
        uint[32] a;
        int(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-36]:
                expr: Expr [29-35]:
                    ty: Int(None, false)
                    kind: Cast [29-35]:
                        ty: Int(None, false)
                        expr: Expr [33-34]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_int() {
    let source = "
        uint[32] a;
        int[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-40]:
                expr: Expr [29-39]:
                    ty: Int(Some(32), false)
                    kind: Cast [29-39]:
                        ty: Int(Some(32), false)
                        expr: Expr [37-38]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_int_truncating() {
    let source = "
        uint[32] a;
        int[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-40]:
                expr: Expr [29-39]:
                    ty: Int(Some(16), false)
                    kind: Cast [29-39]:
                        ty: Int(Some(16), false)
                        expr: Expr [37-38]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_int_expanding() {
    let source = "
        uint[32] a;
        int[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-40]:
                expr: Expr [29-39]:
                    ty: Int(Some(64), false)
                    kind: Cast [29-39]:
                        ty: Int(Some(64), false)
                        expr: Expr [37-38]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//===========================
// Casts to uint and uint[n]
//===========================

#[test]
fn uint_to_uint() {
    let source = "
        uint a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-33]:
                expr: Expr [25-32]:
                    ty: UInt(None, false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn uint_to_sized_uint() {
    let source = "
        uint a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-37]:
                expr: Expr [25-36]:
                    ty: UInt(Some(32), false)
                    kind: Cast [25-36]:
                        ty: UInt(Some(32), false)
                        expr: Expr [34-35]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_uint() {
    let source = "
        uint[32] a;
        uint(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-37]:
                expr: Expr [29-36]:
                    ty: UInt(None, false)
                    kind: Cast [29-36]:
                        ty: UInt(None, false)
                        expr: Expr [34-35]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_uint() {
    let source = "
        uint[32] a;
        uint[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-41]:
                expr: Expr [29-40]:
                    ty: UInt(Some(32), false)
                    kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_uint_truncating() {
    let source = "
        uint[32] a;
        uint[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-41]:
                expr: Expr [29-40]:
                    ty: UInt(Some(16), false)
                    kind: Cast [29-40]:
                        ty: UInt(Some(16), false)
                        expr: Expr [38-39]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_uint_expanding() {
    let source = "
        uint[32] a;
        uint[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-41]:
                expr: Expr [29-40]:
                    ty: UInt(Some(64), false)
                    kind: Cast [29-40]:
                        ty: UInt(Some(64), false)
                        expr: Expr [38-39]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=============================
// Casts to float and float[n]
//=============================

#[test]
fn uint_to_float() {
    let source = "
        uint a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-34]:
                expr: Expr [25-33]:
                    ty: Float(None, false)
                    kind: Cast [25-33]:
                        ty: Float(None, false)
                        expr: Expr [31-32]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn uint_to_sized_float() {
    let source = "
        uint a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-38]:
                expr: Expr [25-37]:
                    ty: Float(Some(32), false)
                    kind: Cast [25-37]:
                        ty: Float(Some(32), false)
                        expr: Expr [35-36]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_float() {
    let source = "
        uint[32] a;
        float(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-38]:
                expr: Expr [29-37]:
                    ty: Float(None, false)
                    kind: Cast [29-37]:
                        ty: Float(None, false)
                        expr: Expr [35-36]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_float() {
    let source = "
        uint[32] a;
        float[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-42]:
                expr: Expr [29-41]:
                    ty: Float(Some(32), false)
                    kind: Cast [29-41]:
                        ty: Float(Some(32), false)
                        expr: Expr [39-40]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_float_truncating() {
    let source = "
        uint[32] a;
        float[16](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-42]:
                expr: Expr [29-41]:
                    ty: Float(Some(16), false)
                    kind: Cast [29-41]:
                        ty: Float(Some(16), false)
                        expr: Expr [39-40]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_float_expanding() {
    let source = "
        uint[32] a;
        float[64](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-42]:
                expr: Expr [29-41]:
                    ty: Float(Some(64), false)
                    kind: Cast [29-41]:
                        ty: Float(Some(64), false)
                        expr: Expr [39-40]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=============================
// Casts to angle and angle[n]
//=============================

#[test]
fn uint_to_angle_fails() {
    let source = "
        uint a;
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
                                ty: UInt(None, true)
                                kind: Lit: Int(0)
                    Stmt [25-34]:
                        annotations: <empty>
                        kind: ExprStmt [25-34]:
                            expr: Expr [25-33]:
                                ty: UInt(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(None, false) to type Angle(None,
              | false)
               ,-[test:3:9]
             2 |         uint a;
             3 |         angle(a);
               :         ^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn uint_to_sized_angle_fails() {
    let source = "
        uint a;
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
                                ty: UInt(None, true)
                                kind: Lit: Int(0)
                    Stmt [25-38]:
                        annotations: <empty>
                        kind: ExprStmt [25-38]:
                            expr: Expr [25-37]:
                                ty: UInt(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(None, false) to type Angle(Some(32),
              | false)
               ,-[test:3:9]
             2 |         uint a;
             3 |         angle[32](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_angle_fails() {
    let source = "
        uint[32] a;
        angle(a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-38]:
                        annotations: <empty>
                        kind: ExprStmt [29-38]:
                            expr: Expr [29-37]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type Angle(None,
              | false)
               ,-[test:3:9]
             2 |         uint[32] a;
             3 |         angle(a);
               :         ^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_sized_angle_fails() {
    let source = "
        uint[32] a;
        angle[32](a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-42]:
                        annotations: <empty>
                        kind: ExprStmt [29-42]:
                            expr: Expr [29-41]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type
              | Angle(Some(32), false)
               ,-[test:3:9]
             2 |         uint[32] a;
             3 |         angle[32](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_sized_angle_truncating_fails() {
    let source = "
        uint[32] a;
        angle[16](a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-42]:
                        annotations: <empty>
                        kind: ExprStmt [29-42]:
                            expr: Expr [29-41]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type
              | Angle(Some(16), false)
               ,-[test:3:9]
             2 |         uint[32] a;
             3 |         angle[16](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_sized_angle_expanding_fails() {
    let source = "
        uint[32] a;
        angle[64](a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-42]:
                        annotations: <empty>
                        kind: ExprStmt [29-42]:
                            expr: Expr [29-41]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type
              | Angle(Some(64), false)
               ,-[test:3:9]
             2 |         uint[32] a;
             3 |         angle[64](a);
               :         ^^^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

//=================================
// Casts to complex and complex[n]
//=================================

#[test]
fn uint_to_complex() {
    let source = "
        uint a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-36]:
                expr: Expr [25-35]:
                    ty: Complex(None, false)
                    kind: Cast [25-35]:
                        ty: Complex(None, false)
                        expr: Expr [33-34]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn uint_to_sized_complex() {
    let source = "
        uint a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-47]:
                expr: Expr [25-46]:
                    ty: Complex(Some(32), false)
                    kind: Cast [25-46]:
                        ty: Complex(Some(32), false)
                        expr: Expr [44-45]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_complex() {
    let source = "
        uint[32] a;
        complex(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-40]:
                expr: Expr [29-39]:
                    ty: Complex(None, false)
                    kind: Cast [29-39]:
                        ty: Complex(None, false)
                        expr: Expr [37-38]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_complex() {
    let source = "
        uint[32] a;
        complex[float[32]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-51]:
                expr: Expr [29-50]:
                    ty: Complex(Some(32), false)
                    kind: Cast [29-50]:
                        ty: Complex(Some(32), false)
                        expr: Expr [48-49]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_complex_truncating() {
    let source = "
        uint[32] a;
        complex[float[16]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-51]:
                expr: Expr [29-50]:
                    ty: Complex(Some(16), false)
                    kind: Cast [29-50]:
                        ty: Complex(Some(16), false)
                        expr: Expr [48-49]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_sized_complex_expanding() {
    let source = "
        uint[32] a;
        complex[float[64]](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-51]:
                expr: Expr [29-50]:
                    ty: Complex(Some(64), false)
                    kind: Cast [29-50]:
                        ty: Complex(Some(64), false)
                        expr: Expr [48-49]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

//=================================
// Casts to bit and bit[n]
//=================================

#[test]
fn uint_to_bit() {
    let source = "
        uint a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-16]:
                symbol_id: 8
                ty_span: [9-13]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            ExprStmt [25-32]:
                expr: Expr [25-31]:
                    ty: Bit(false)
                    kind: Cast [25-31]:
                        ty: Bit(false)
                        expr: Expr [29-30]:
                            ty: UInt(None, false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn uint_to_bitarray_fails() {
    let source = "
        uint a;
        bit[32](a);
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
                                ty: UInt(None, true)
                                kind: Lit: Int(0)
                    Stmt [25-36]:
                        annotations: <empty>
                        kind: ExprStmt [25-36]:
                            expr: Expr [25-35]:
                                ty: UInt(None, false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(None, false) to type BitArray(32,
              | false)
               ,-[test:3:9]
             2 |         uint a;
             3 |         bit[32](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_bit() {
    let source = "
        uint[32] a;
        bit(a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-36]:
                expr: Expr [29-35]:
                    ty: Bit(false)
                    kind: Cast [29-35]:
                        ty: Bit(false)
                        expr: Expr [33-34]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_bitarray() {
    let source = "
        uint[32] a;
        bit[32](a);
    ";
    check(
        source,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-17]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(32), true)
                    kind: Lit: Int(0)
            ExprStmt [29-40]:
                expr: Expr [29-39]:
                    ty: BitArray(32, false)
                    kind: Cast [29-39]:
                        ty: BitArray(32, false)
                        expr: Expr [37-38]:
                            ty: UInt(Some(32), false)
                            kind: SymbolId(8)
        "#]],
    );
}

#[test]
fn sized_uint_to_bitarray_truncating_fails() {
    let source = "
        uint[32] a;
        bit[16](a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-40]:
                        annotations: <empty>
                        kind: ExprStmt [29-40]:
                            expr: Expr [29-39]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type BitArray(16,
              | false)
               ,-[test:3:9]
             2 |         uint[32] a;
             3 |         bit[16](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn sized_uint_to_bitarray_expanding_fails() {
    let source = "
        uint[32] a;
        bit[64](a);
    ";
    check(
        source,
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
                                ty: UInt(Some(32), true)
                                kind: Lit: Int(0)
                    Stmt [29-40]:
                        annotations: <empty>
                        kind: ExprStmt [29-40]:
                            expr: Expr [29-39]:
                                ty: UInt(Some(32), false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type UInt(Some(32), false) to type BitArray(64,
              | false)
               ,-[test:3:9]
             2 |         uint[32] a;
             3 |         bit[64](a);
               :         ^^^^^^^^^^
             4 |     
               `----
            ]"#]],
    );
}
