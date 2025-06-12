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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-39]:
                symbol_id: 9
                ty_span: [29-32]
                init_expr: Expr [37-38]:
                    ty: bit
                    kind: Cast [0-0]:
                        ty: bit
                        expr: Expr [37-38]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [33-34]:
                name: y
                type: bit
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: bool
                    kind: Cast [0-0]:
                        ty: bool
                        expr: Expr [38-39]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [34-35]:
                name: y
                type: bool
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-39]:
                symbol_id: 9
                ty_span: [29-32]
                init_expr: Expr [37-38]:
                    ty: int
                    kind: SymbolId(8)
            [9] Symbol [33-34]:
                name: y
                type: int
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-43]:
                symbol_id: 9
                ty_span: [29-36]
                init_expr: Expr [41-42]:
                    ty: int[32]
                    kind: Cast [0-0]:
                        ty: int[32]
                        expr: Expr [41-42]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
                name: y
                type: int[32]
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-40]:
                symbol_id: 9
                ty_span: [29-33]
                init_expr: Expr [38-39]:
                    ty: uint
                    kind: Cast [0-0]:
                        ty: uint
                        expr: Expr [38-39]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [34-35]:
                name: y
                type: uint
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-44]:
                symbol_id: 9
                ty_span: [29-37]
                init_expr: Expr [42-43]:
                    ty: uint[32]
                    kind: Cast [0-0]:
                        ty: uint[32]
                        expr: Expr [42-43]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [38-39]:
                name: y
                type: uint[32]
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-43]:
                symbol_id: 9
                ty_span: [29-36]
                init_expr: Expr [41-42]:
                    ty: int[65]
                    kind: Cast [0-0]:
                        ty: int[65]
                        expr: Expr [41-42]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
                name: y
                type: int[65]
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-41]:
                symbol_id: 9
                ty_span: [29-34]
                init_expr: Expr [39-40]:
                    ty: float
                    kind: Cast [0-0]:
                        ty: float
                        expr: Expr [39-40]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [35-36]:
                name: y
                type: float
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-45]:
                symbol_id: 9
                ty_span: [29-38]
                init_expr: Expr [43-44]:
                    ty: float[32]
                    kind: Cast [0-0]:
                        ty: float[32]
                        expr: Expr [43-44]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [39-40]:
                name: y
                type: float[32]
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-50]:
                symbol_id: 9
                ty_span: [29-43]
                init_expr: Expr [48-49]:
                    ty: complex[float]
                    kind: Cast [0-0]:
                        ty: complex[float]
                        expr: Expr [48-49]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [44-45]:
                name: y
                type: complex[float]
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
                    ty: int
                    kind: Lit: Int(42)
            [8] Symbol [13-14]:
                name: x
                type: int
                qsharp_type: Int
                io_kind: Default
            ClassicalDeclarationStmt [29-54]:
                symbol_id: 9
                ty_span: [29-47]
                init_expr: Expr [52-53]:
                    ty: complex[float[32]]
                    kind: Cast [0-0]:
                        ty: complex[float[32]]
                        expr: Expr [52-53]:
                            ty: int
                            kind: SymbolId(8)
            [9] Symbol [48-49]:
                name: y
                type: complex[float[32]]
                qsharp_type: Complex
                io_kind: Default
        "#]],
    );
}
