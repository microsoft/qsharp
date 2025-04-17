// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_angle_implicitly() {
    let input = r#"
         bit x = 1;
         angle y = x;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [10-20]:
            symbol_id: 8
            ty_span: [10-13]
            init_expr: Expr [18-19]:
                ty: Bit(true)
                kind: Lit: Bit(1)
        [8] Symbol [14-15]:
            name: x
            type: Bit(false)
            qsharp_type: Result
            io_kind: Default
        ClassicalDeclarationStmt [30-42]:
            symbol_id: 9
            ty_span: [30-35]
            init_expr: Expr [40-41]:
                ty: Angle(None, false)
                kind: Cast [0-0]:
                    ty: Angle(None, false)
                    expr: Expr [40-41]:
                        ty: Bit(false)
                        kind: SymbolId(8)
        [9] Symbol [36-37]:
            name: y
            type: Angle(None, false)
            qsharp_type: Angle
            io_kind: Default
    "#]],
    );
}

#[test]
fn to_explicit_angle_implicitly() {
    let input = r#"
         bit x = 1;
         angle[4] y = x;
    "#;

    check_classical_decls(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [10-20]:
            symbol_id: 8
            ty_span: [10-13]
            init_expr: Expr [18-19]:
                ty: Bit(true)
                kind: Lit: Bit(1)
        [8] Symbol [14-15]:
            name: x
            type: Bit(false)
            qsharp_type: Result
            io_kind: Default
        ClassicalDeclarationStmt [30-45]:
            symbol_id: 9
            ty_span: [30-38]
            init_expr: Expr [43-44]:
                ty: Angle(Some(4), false)
                kind: Cast [0-0]:
                    ty: Angle(Some(4), false)
                    expr: Expr [43-44]:
                        ty: Bit(false)
                        kind: SymbolId(8)
        [9] Symbol [39-40]:
            name: y
            type: Angle(Some(4), false)
            qsharp_type: Angle
            io_kind: Default
    "#]],
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
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [14-15]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [30-41]:
                symbol_id: 9
                ty_span: [30-34]
                init_expr: Expr [39-40]:
                    ty: Bool(false)
                    kind: Cast [0-0]:
                        ty: Bool(false)
                        expr: Expr [39-40]:
                            ty: Bit(false)
                            kind: SymbolId(8)
            [9] Symbol [35-36]:
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
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-38]:
                symbol_id: 9
                ty_span: [28-31]
                init_expr: Expr [36-37]:
                    ty: Int(None, false)
                    kind: Cast [0-0]:
                        ty: Int(None, false)
                        expr: Expr [36-37]:
                            ty: Bit(false)
                            kind: SymbolId(8)
            [9] Symbol [32-33]:
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
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-42]:
                symbol_id: 9
                ty_span: [28-35]
                init_expr: Expr [40-41]:
                    ty: Int(Some(32), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(32), false)
                        expr: Expr [40-41]:
                            ty: Bit(false)
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
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
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-39]:
                symbol_id: 9
                ty_span: [28-32]
                init_expr: Expr [37-38]:
                    ty: UInt(None, false)
                    kind: Cast [0-0]:
                        ty: UInt(None, false)
                        expr: Expr [37-38]:
                            ty: Bit(false)
                            kind: SymbolId(8)
            [9] Symbol [33-34]:
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
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-43]:
                symbol_id: 9
                ty_span: [28-36]
                init_expr: Expr [41-42]:
                    ty: UInt(Some(32), false)
                    kind: Cast [0-0]:
                        ty: UInt(Some(32), false)
                        expr: Expr [41-42]:
                            ty: Bit(false)
                            kind: SymbolId(8)
            [9] Symbol [37-38]:
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
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [13-14]:
                name: x
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default
            ClassicalDeclarationStmt [28-42]:
                symbol_id: 9
                ty_span: [28-35]
                init_expr: Expr [40-41]:
                    ty: Int(Some(65), false)
                    kind: Cast [0-0]:
                        ty: Int(Some(65), false)
                        expr: Expr [40-41]:
                            ty: Bit(false)
                            kind: SymbolId(8)
            [9] Symbol [36-37]:
                name: y
                type: Int(Some(65), false)
                qsharp_type: BigInt
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_implicit_float_implicitly_fails() {
    let input = "
        bit x = 1;
        float y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-19]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-19]:
                            symbol_id: 8
                            ty_span: [9-12]
                            init_expr: Expr [17-18]:
                                ty: Bit(true)
                                kind: Lit: Bit(1)
                    Stmt [28-40]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [28-40]:
                            symbol_id: 9
                            ty_span: [28-33]
                            init_expr: Expr [38-39]:
                                ty: Bit(false)
                                kind: SymbolId(8)

            [Qasm.Lowerer.CannotCast

              x cannot cast expression of type Bit(false) to type Float(None, false)
               ,-[test:3:19]
             2 |         bit x = 1;
             3 |         float y = x;
               :                   ^
             4 |     
               `----
            ]"#]],
    );
}
