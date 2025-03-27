// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_bit_implicitly() {
    let input = "
        int x = 42;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-39]:
                symbol_id: 9
                ty_span: [29-32]
                init_expr: Expr [37-38]:
                    ty: Bit(false)
                    kind: Cast [0-0]:
                        ty: Bit(false)
                        expr: Expr [37-38]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [33-34]:
                name: y
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_bool_implicitly() {
    let input = "
        int x = 42;
        bool y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: Bool(false)
                    kind: Cast [0-0]:
                        ty: Bool(false)
                        expr: Expr [38-39]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [34-35]:
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
        int x = 42;
        int y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-39]:
                symbol_id: 9
                ty_span: [29-32]
                init_expr: Expr [37-38]:
                    ty: Int(None, false)
                    kind: SymbolId(8)
            [9] Symbol [33-34]:
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
        int x = 42;
        int[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-43]:
                symbol_id: 9
                ty_span: [29-36]
                init_expr: Expr [41-42]:
                    ty: Int(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(32), false)
                        expr: Expr [41-42]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
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
        int x = 42;
        uint y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: UInt(None, false)
                    kind: Cast [0-0]:
                        ty: UInt(None, false)
                        expr: Expr [38-39]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [34-35]:
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
        int x = 42;
        uint[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-44]:
                symbol_id: 9
                ty_span: [29-37]
                init_expr: Expr [42-43]:
                    ty: UInt(Some(32), false)
                    kind: Cast [0-0]:
                        ty: UInt(Some(32), false)
                        expr: Expr [42-43]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [38-39]:
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
        int x = 42;
        int[65] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-43]:
                symbol_id: 9
                ty_span: [29-36]
                init_expr: Expr [41-42]:
                    ty: Int(Some(65), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(65), false)
                        expr: Expr [41-42]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
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
        int x = 42;
        float y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-41]:
                symbol_id: 9
                ty_span: [29-34]
                init_expr: Expr [39-40]:
                    ty: Float(None, false)
                    kind: Cast [0-0]:
                        ty: Float(None, false)
                        expr: Expr [39-40]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [35-36]:
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
        int x = 42;
        float[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-45]:
                symbol_id: 9
                ty_span: [29-38]
                init_expr: Expr [43-44]:
                    ty: Float(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Float(Some(32), false)
                        expr: Expr [43-44]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [39-40]:
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
        int x = 42;
        complex[float] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-50]:
                symbol_id: 9
                ty_span: [29-43]
                init_expr: Expr [48-49]:
                    ty: Complex(None, false)
                    kind: Cast [0-0]:
                        ty: Complex(None, false)
                        expr: Expr [48-49]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [44-45]:
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
        int x = 42;
        complex[float[32]] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-20]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-19]:
                    ty: Int(None, false)
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-54]:
                symbol_id: 9
                ty_span: [29-47]
                init_expr: Expr [52-53]:
                    ty: Complex(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Complex(Some(32), false)
                        expr: Expr [52-53]:
                            ty: Int(None, false)
                            kind: SymbolId(8)
            [9] Symbol [48-49]:
                name: y
                type: Complex(Some(32), false)
                qsharp_type: Complex
                io_kind: Default
        "#]],
    );
}
