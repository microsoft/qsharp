// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_bit_implicitly() {
    let input = "
        bool x = true;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-42]:
                symbol_id: 7
                ty_span: [32-35]
                init_expr: Expr [40-41]:
                    ty: Bit(false)
                    kind: Cast [0-0]:
                        ty: Bit(false)
                        expr: Expr [40-41]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [36-37]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_implicit_int_implicitly() {
    let input = "
        bool x = true;
        int y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-42]:
                symbol_id: 7
                ty_span: [32-35]
                init_expr: Expr [40-41]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [40-41]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [36-37]:
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
        bool x = true;
        int[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 7
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(32), false)
                        expr: Expr [44-45]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [40-41]:
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
        bool x = true;
        uint y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-43]:
                symbol_id: 7
                ty_span: [32-36]
                init_expr: Expr [41-42]:
                    ty: UInt(None, false)
                    kind: Cast [0-0]:
                        ty: UInt(None, false)
                        expr: Expr [41-42]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [37-38]:
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
        bool x = true;
        uint[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-40]
                init_expr: Expr [45-46]:
                    ty: UInt(Some(32), false)
                    kind: Cast [0-0]:
                        ty: UInt(Some(32), false)
                        expr: Expr [45-46]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [41-42]:
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
        bool x = true;
        int[65] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-46]:
                symbol_id: 7
                ty_span: [32-39]
                init_expr: Expr [44-45]:
                    ty: Int(Some(65), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(65), false)
                        expr: Expr [44-45]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [40-41]:
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
        bool x = true;
        float y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-44]:
                symbol_id: 7
                ty_span: [32-37]
                init_expr: Expr [42-43]:
                    ty: Float(None, false)
                    kind: Cast [0-0]:
                        ty: Float(None, false)
                        expr: Expr [42-43]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [38-39]:
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
        bool x = true;
        float[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-13]
                init_expr: Expr [18-22]:
                    ty: Bool(false)
                    kind: Lit: Bool(true)
            [6] Symbol [14-15]:
                name: x
                type: Bool(false)
                qsharp_type: bool
                io_kind: Default
            ClassicalDeclarationStmt [32-48]:
                symbol_id: 7
                ty_span: [32-41]
                init_expr: Expr [46-47]:
                    ty: Float(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Float(Some(32), false)
                        expr: Expr [46-47]:
                            ty: Bool(false)
                            kind: SymbolId(6)
            [7] Symbol [42-43]:
                name: y
                type: Float(Some(32), false)
                qsharp_type: Double
                io_kind: Default
        "#]],
    );
}
