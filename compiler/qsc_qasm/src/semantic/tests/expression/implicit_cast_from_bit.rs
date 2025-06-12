// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_angle_implicitly_fails() {
    let input = r#"
         bit x = 1;
         angle y = x;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [10-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [10-20]:
                            symbol_id: 8
                            ty_span: [10-13]
                            init_expr: Expr [18-19]:
                                ty: const bit
                                kind: Lit: Bit(1)
                    Stmt [30-42]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [30-42]:
                            symbol_id: 9
                            ty_span: [30-35]
                            init_expr: Expr [40-41]:
                                ty: bit
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type bit to type angle
               ,-[test:3:20]
             2 |          bit x = 1;
             3 |          angle y = x;
               :                    ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_explicit_angle_implicitly_fails() {
    let input = r#"
         bit x = 1;
         angle[4] y = x;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [10-20]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [10-20]:
                            symbol_id: 8
                            ty_span: [10-13]
                            init_expr: Expr [18-19]:
                                ty: const bit
                                kind: Lit: Bit(1)
                    Stmt [30-45]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [30-45]:
                            symbol_id: 9
                            ty_span: [30-38]
                            init_expr: Expr [43-44]:
                                ty: bit
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type bit to type angle[4]
               ,-[test:3:23]
             2 |          bit x = 1;
             3 |          angle[4] y = x;
               :                       ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_bool_implicitly() {
    let input = r#"
         bit x = 1;
         bool y = x;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [10-20]:
                symbol_id: 8
                ty_span: [10-13]
                init_expr: Expr [18-19]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [14-15]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [30-41]:
                symbol_id: 9
                ty_span: [30-34]
                init_expr: Expr [39-40]:
                    ty: bool
                    kind: Cast [0-0]:
                        ty: bool
                        expr: Expr [39-40]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [35-36]:
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
        bit x = 1;
        int y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: int
                    kind: Cast [0-0]:
                        ty: int
                        expr: Expr [36-37]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [32-33]:
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
        bit x = 1;
        int[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-42]:
                symbol_id: 9
                ty_span: [28-35]
                init_expr: Expr [40-41]:
                    ty: int[32]
                    kind: Cast [0-0]:
                        ty: int[32]
                        expr: Expr [40-41]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
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
        bit x = 1;
        uint y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-39]:
                symbol_id: 9
                ty_span: [28-32]
                init_expr: Expr [37-38]:
                    ty: uint
                    kind: Cast [0-0]:
                        ty: uint
                        expr: Expr [37-38]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [33-34]:
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
        bit x = 1;
        uint[32] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-43]:
                symbol_id: 9
                ty_span: [28-36]
                init_expr: Expr [41-42]:
                    ty: uint[32]
                    kind: Cast [0-0]:
                        ty: uint[32]
                        expr: Expr [41-42]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
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
        bit x = 1;
        int[65] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-42]:
                symbol_id: 9
                ty_span: [28-35]
                init_expr: Expr [40-41]:
                    ty: int[65]
                    kind: Cast [0-0]:
                        ty: int[65]
                        expr: Expr [40-41]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
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
        bit x = 1;
        float y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-19]:
                symbol_id: 8
                ty_span: [9-12]
                init_expr: Expr [17-18]:
                    ty: const bit
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: bit
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-40]:
                symbol_id: 9
                ty_span: [28-33]
                init_expr: Expr [38-39]:
                    ty: float
                    kind: Cast [0-0]:
                        ty: float
                        expr: Expr [38-39]:
                            ty: bit
                            kind: SymbolId(8)
            [9] Symbol [34-35]:
                name: y
                type: float
                qsharp_type: Double
                io_kind: Default
        "#]],
    );
}
