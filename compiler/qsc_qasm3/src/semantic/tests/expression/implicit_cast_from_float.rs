// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_bit_implicitly_fails() {
    let input = "
        float x = 42.;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-23]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-23]:
                            symbol_id: 8
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Float(None, false)
                                kind: Lit: Float(42.0)
                    Stmt [32-42]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-42]:
                            symbol_id: 9
                            ty_span: [32-35]
                            init_expr: Expr [40-41]:
                                ty: Float(None, false)
                                kind: SymbolId(8)

            [Qsc.Qasm3.Lowerer.CannotCast

              x cannot cast expression of type Float(None, false) to type Bit(false)
               ,-[test:3:17]
             2 |         float x = 42.;
             3 |         bit y = x;
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn explicit_width_to_bit_implicitly_fails() {
    let input = "
        float[64] x = 42.;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-27]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-27]:
                            symbol_id: 8
                            ty_span: [9-18]
                            init_expr: Expr [23-26]:
                                ty: Float(Some(64), true)
                                kind: Lit: Float(42.0)
                    Stmt [36-46]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [36-46]:
                            symbol_id: 9
                            ty_span: [36-39]
                            init_expr: Expr [44-45]:
                                ty: Float(Some(64), false)
                                kind: SymbolId(8)

            [Qsc.Qasm3.Lowerer.CannotCast

              x cannot cast expression of type Float(Some(64), false) to type Bit(false)
               ,-[test:3:17]
             2 |         float[64] x = 42.;
             3 |         bit y = x;
               :                 ^
             4 |     
               `----
            ]"#]],
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-43]:
                symbol_id: 9
                ty_span: [32-36]
                init_expr: Expr [41-42]:
                    ty: Bool(false)
                    kind: Cast [0-0]:
                        ty: Bool(false)
                        expr: Expr [41-42]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
                name: y
                type: Bool(false)
                qsharp_type: bool
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-42]:
                symbol_id: 9
                ty_span: [32-35]
                init_expr: Expr [40-41]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [40-41]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
                name: y
                type: Int(None, false)
                qsharp_type: Int
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(32), false)
                        expr: Expr [44-45]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [40-41]:
                name: y
                type: Int(Some(32), false)
                qsharp_type: Int
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-43]:
                symbol_id: 9
                ty_span: [32-36]
                init_expr: Expr [41-42]:
                    ty: UInt(None, false)
                    kind: Cast [0-0]:
                        ty: UInt(None, false)
                        expr: Expr [41-42]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
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
                    ty: Float(None, false)
                    kind: UnaryOpExpr [20-23]:
                        op: Neg
                        expr: Expr [20-23]:
                            ty: Float(None, true)
                            kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [33-44]:
                symbol_id: 9
                ty_span: [33-37]
                init_expr: Expr [42-43]:
                    ty: UInt(None, false)
                    kind: Cast [0-0]:
                        ty: UInt(None, false)
                        expr: Expr [42-43]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [38-39]:
                name: y
                type: UInt(None, false)
                qsharp_type: Int
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 9
                ty_span: [32-40]
                init_expr: Expr [45-46]:
                    ty: UInt(Some(32), false)
                    kind: Cast [0-0]:
                        ty: UInt(Some(32), false)
                        expr: Expr [45-46]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [41-42]:
                name: y
                type: UInt(Some(32), false)
                qsharp_type: Int
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 9
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(65), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(65), false)
                        expr: Expr [44-45]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [40-41]:
                name: y
                type: Int(Some(65), false)
                qsharp_type: BigInt
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-44]:
                symbol_id: 9
                ty_span: [32-37]
                init_expr: Expr [42-43]:
                    ty: Float(None, false)
                    kind: SymbolId(8)
            [9] Symbol [38-39]:
                name: y
                type: Float(None, false)
                qsharp_type: Double
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-48]:
                symbol_id: 9
                ty_span: [32-41]
                init_expr: Expr [46-47]:
                    ty: Float(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Float(Some(32), false)
                        expr: Expr [46-47]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [42-43]:
                name: y
                type: Float(Some(32), false)
                qsharp_type: Double
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-53]:
                symbol_id: 9
                ty_span: [32-46]
                init_expr: Expr [51-52]:
                    ty: Complex(None, false)
                    kind: Cast [0-0]:
                        ty: Complex(None, false)
                        expr: Expr [51-52]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [47-48]:
                name: y
                type: Complex(None, false)
                qsharp_type: Complex
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-57]:
                symbol_id: 9
                ty_span: [32-50]
                init_expr: Expr [55-56]:
                    ty: Complex(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Complex(Some(32), false)
                        expr: Expr [55-56]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [51-52]:
                name: y
                type: Complex(Some(32), false)
                qsharp_type: Complex
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-44]:
                symbol_id: 9
                ty_span: [32-37]
                init_expr: Expr [42-43]:
                    ty: Angle(None, false)
                    kind: Cast [0-0]:
                        ty: Angle(None, false)
                        expr: Expr [42-43]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [38-39]:
                name: y
                type: Angle(None, false)
                qsharp_type: Angle
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
                    ty: Float(None, false)
                    kind: Lit: Float(42.0)
            [8] Symbol [15-16]:
                name: x
                type: Float(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 9
                ty_span: [32-40]
                init_expr: Expr [45-46]:
                    ty: Angle(Some(4), false)
                    kind: Cast [0-0]:
                        ty: Angle(Some(4), false)
                        expr: Expr [45-46]:
                            ty: Float(None, false)
                            kind: SymbolId(8)
            [9] Symbol [41-42]:
                name: y
                type: Angle(Some(4), false)
                qsharp_type: Angle
                io_kind: Default
        "#]],
    );
}
