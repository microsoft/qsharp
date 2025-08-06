// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_bit_implicitly() {
    let input = "
        float x = 42.;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-42]:
                symbol_id: 9
                ty_span: [32-35]
                init_expr: Expr [40-41]:
                    ty: bit
                    kind: Cast [40-41]:
                        ty: bit
                        expr: Expr [40-41]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [36-37]:
                name: y
                type: bit
                ty_span: [32-35]
                io_kind: Default
        "#]],
    );
}

#[test]
fn explicit_width_to_bit_implicitly() {
    let input = "
        float[64] x = 42.;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-27]:
                symbol_id: 8
                ty_span: [9-18]
                init_expr: Expr [23-26]:
                    ty: const float[64]
                    kind: Lit: Float(42.0)
            [8] Symbol [19-20]:
                name: x
                type: float[64]
                ty_span: [9-18]
                io_kind: Default
            ClassicalDeclarationStmt [36-46]:
                symbol_id: 9
                ty_span: [36-39]
                init_expr: Expr [44-45]:
                    ty: bit
                    kind: Cast [44-45]:
                        ty: bit
                        expr: Expr [44-45]:
                            ty: float[64]
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [40-41]:
                name: y
                type: bit
                ty_span: [36-39]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_bool_implicitly() {
    let input = "
        float x = 42.;
        bool y = x;
    ";
    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-43]:
                symbol_id: 9
                ty_span: [32-36]
                init_expr: Expr [41-42]:
                    ty: bool
                    kind: Cast [41-42]:
                        ty: bool
                        expr: Expr [41-42]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [37-38]:
                name: y
                type: bool
                ty_span: [32-36]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_implicit_int_implicitly() {
    let input = "
        float x = 42.;
        int y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-42]:
                symbol_id: 9
                ty_span: [32-35]
                init_expr: Expr [40-41]:
                    ty: int
                    kind: Cast [40-41]:
                        ty: int
                        expr: Expr [40-41]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [36-37]:
                name: y
                type: int
                ty_span: [32-35]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_int_implicitly() {
    let input = "
        float x = 42.;
        int[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: int[32]
                    kind: Cast [44-45]:
                        ty: int[32]
                        expr: Expr [44-45]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [40-41]:
                name: y
                type: int[32]
                ty_span: [32-39]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_implicit_uint_implicitly() {
    let input = "
        float x = 42.;
        uint y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-43]:
                symbol_id: 9
                ty_span: [32-36]
                init_expr: Expr [41-42]:
                    ty: uint
                    kind: Cast [41-42]:
                        ty: uint
                        expr: Expr [41-42]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [37-38]:
                name: y
                type: uint
                ty_span: [32-36]
                io_kind: Default
        "#]],
    );
}

#[test]
fn negative_lit_to_implicit_uint_implicitly() {
    let input = "
        float x = -42.;
        uint y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-24]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [20-23]:
                    ty: float
                    kind: UnaryOpExpr [20-23]:
                        op: Neg
                        expr: Expr [20-23]:
                            ty: const float
                            kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [33-44]:
                symbol_id: 9
                ty_span: [33-37]
                init_expr: Expr [42-43]:
                    ty: uint
                    kind: Cast [42-43]:
                        ty: uint
                        expr: Expr [42-43]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [38-39]:
                name: y
                type: uint
                ty_span: [33-37]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_uint_implicitly() {
    let input = "
        float x = 42.;
        uint[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 9
                ty_span: [32-40]
                init_expr: Expr [45-46]:
                    ty: uint[32]
                    kind: Cast [45-46]:
                        ty: uint[32]
                        expr: Expr [45-46]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [41-42]:
                name: y
                type: uint[32]
                ty_span: [32-40]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_bigint_implicitly() {
    let input = "
        float x = 42.;
        int[65] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: int[65]
                    kind: Cast [44-45]:
                        ty: int[65]
                        expr: Expr [44-45]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [40-41]:
                name: y
                type: int[65]
                ty_span: [32-39]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_implicit_float_implicitly() {
    let input = "
        float x = 42.;
        float y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-44]:
                symbol_id: 9
                ty_span: [32-37]
                init_expr: Expr [42-43]:
                    ty: float
                    kind: SymbolId(8)
            [9] Symbol [38-39]:
                name: y
                type: float
                ty_span: [32-37]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_float_implicitly() {
    let input = "
        float x = 42.;
        float[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-48]:
                symbol_id: 9
                ty_span: [32-41]
                init_expr: Expr [46-47]:
                    ty: float[32]
                    kind: Cast [46-47]:
                        ty: float[32]
                        expr: Expr [46-47]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [42-43]:
                name: y
                type: float[32]
                ty_span: [32-41]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_implicit_complex_implicitly() {
    let input = "
        float x = 42.;
        complex[float] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-53]:
                symbol_id: 9
                ty_span: [32-46]
                init_expr: Expr [51-52]:
                    ty: complex[float]
                    kind: Cast [51-52]:
                        ty: complex[float]
                        expr: Expr [51-52]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [47-48]:
                name: y
                type: complex[float]
                ty_span: [32-46]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_complex_implicitly() {
    let input = "
        float x = 42.;
        complex[float[32]] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-57]:
                symbol_id: 9
                ty_span: [32-50]
                init_expr: Expr [55-56]:
                    ty: complex[float[32]]
                    kind: Cast [55-56]:
                        ty: complex[float[32]]
                        expr: Expr [55-56]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [51-52]:
                name: y
                type: complex[float[32]]
                ty_span: [32-50]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_angle_implicitly() {
    let input = "
        float x = 42.;
        angle y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-44]:
                symbol_id: 9
                ty_span: [32-37]
                init_expr: Expr [42-43]:
                    ty: angle
                    kind: Cast [42-43]:
                        ty: angle
                        expr: Expr [42-43]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [38-39]:
                name: y
                type: angle
                ty_span: [32-37]
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_angle_implicitly() {
    let input = "
        float x = 42.;
        angle[4] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 8
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: float
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: float
                ty_span: [9-14]
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 9
                ty_span: [32-40]
                init_expr: Expr [45-46]:
                    ty: angle[4]
                    kind: Cast [45-46]:
                        ty: angle[4]
                        expr: Expr [45-46]:
                            ty: float
                            kind: SymbolId(8)
                        kind: Implicit
            [9] Symbol [41-42]:
                name: y
                type: angle[4]
                ty_span: [32-40]
                io_kind: Default
        "#]],
    );
}
