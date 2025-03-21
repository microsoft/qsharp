// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decls;

#[test]
fn to_bit_implicitly() {
    let input = "
        angle x = 42.;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-23]:
            symbol_id: 6
            ty_span: [9-14]
            init_expr: Expr [19-22]:
                ty: Angle(None, true)
                kind: Lit: Float(42.0)
        [6] Symbol [15-16]:
            name: x
            type: Angle(None, false)
            qsharp_type: Double
            io_kind: Default
        ClassicalDeclarationStmt [32-42]:
            symbol_id: 7
            ty_span: [32-35]
            init_expr: Expr [40-41]:
                ty: Bit(false)
                kind: Cast [0-0]:
                    ty: Bit(false)
                    expr: Expr [40-41]:
                        ty: Angle(None, false)
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
fn explicit_width_to_bit_implicitly_fails() {
    let input = "
        angle[64] x = 42.;
        bit y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-27]:
            symbol_id: 6
            ty_span: [9-18]
            init_expr: Expr [23-26]:
                ty: Angle(Some(64), true)
                kind: Lit: Float(42.0)
        [6] Symbol [19-20]:
            name: x
            type: Angle(Some(64), false)
            qsharp_type: Double
            io_kind: Default
        ClassicalDeclarationStmt [36-46]:
            symbol_id: 7
            ty_span: [36-39]
            init_expr: Expr [44-45]:
                ty: Bit(false)
                kind: Cast [0-0]:
                    ty: Bit(false)
                    expr: Expr [44-45]:
                        ty: Angle(Some(64), false)
                        kind: SymbolId(6)
        [7] Symbol [40-41]:
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
        angle x = 42.;
        bool y = x;
    ";
    check_classical_decls(
        input,
        &expect![[r#"
        ClassicalDeclarationStmt [9-23]:
            symbol_id: 6
            ty_span: [9-14]
            init_expr: Expr [19-22]:
                ty: Angle(None, true)
                kind: Lit: Float(42.0)
        [6] Symbol [15-16]:
            name: x
            type: Angle(None, false)
            qsharp_type: Double
            io_kind: Default
        ClassicalDeclarationStmt [32-43]:
            symbol_id: 7
            ty_span: [32-36]
            init_expr: Expr [41-42]:
                ty: Bool(false)
                kind: Cast [0-0]:
                    ty: Bool(false)
                    expr: Expr [41-42]:
                        ty: Angle(None, false)
                        kind: SymbolId(6)
        [7] Symbol [37-38]:
            name: y
            type: Bool(false)
            qsharp_type: bool
            io_kind: Default
    "#]],
    );
}

#[test]
fn to_implicit_int_implicitly_fails() {
    let input = "
        angle x = 42.;
        int y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-42]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-42]:
                            symbol_id: 7
                            ty_span: [32-35]
                            init_expr: Expr [40-41]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Int(None, false)
               ,-[test:3:17]
             2 |         angle x = 42.;
             3 |         int y = x;
               :                 ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_explicit_int_implicitly_fails() {
    let input = "
        angle x = 42.;
        int[32] y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-46]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-46]:
                            symbol_id: 7
                            ty_span: [32-39]
                            init_expr: Expr [44-45]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Int(Some(32),
              | false)
               ,-[test:3:21]
             2 |         angle x = 42.;
             3 |         int[32] y = x;
               :                     ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_implicit_uint_implicitly_fails() {
    let input = "
        angle x = 42.;
        uint y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-43]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-43]:
                            symbol_id: 7
                            ty_span: [32-36]
                            init_expr: Expr [41-42]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type UInt(None,
              | false)
               ,-[test:3:18]
             2 |         angle x = 42.;
             3 |         uint y = x;
               :                  ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn negative_lit_to_implicit_uint_implicitly_fails() {
    let input = "
        angle x = -42.;
        uint y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [9-24]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [9-24]:
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [20-23]:
                                ty: Angle(None, false)
                                kind: Cast [0-0]:
                                    ty: Angle(None, false)
                                    expr: Expr [20-23]:
                                        ty: Float(None, true)
                                        kind: UnaryOpExpr [20-23]:
                                            op: Neg
                                            expr: Expr [20-23]:
                                                ty: Float(None, true)
                                                kind: Lit: Float(42.0)
                    Stmt [33-44]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [33-44]:
                            symbol_id: 7
                            ty_span: [33-37]
                            init_expr: Expr [42-43]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type UInt(None,
              | false)
               ,-[test:3:18]
             2 |         angle x = -42.;
             3 |         uint y = x;
               :                  ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_explicit_uint_implicitly_fails() {
    let input = "
        angle x = 42.;
        uint[32] y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-47]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-47]:
                            symbol_id: 7
                            ty_span: [32-40]
                            init_expr: Expr [45-46]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type UInt(Some(32),
              | false)
               ,-[test:3:22]
             2 |         angle x = 42.;
             3 |         uint[32] y = x;
               :                      ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_explicit_bigint_implicitly_fails() {
    let input = "
        angle x = 42.;
        int[65] y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-46]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-46]:
                            symbol_id: 7
                            ty_span: [32-39]
                            init_expr: Expr [44-45]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Int(Some(65),
              | false)
               ,-[test:3:21]
             2 |         angle x = 42.;
             3 |         int[65] y = x;
               :                     ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_implicit_float_implicitly_fails() {
    let input = "
        angle x = 42.;
        float y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-44]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-44]:
                            symbol_id: 7
                            ty_span: [32-37]
                            init_expr: Expr [42-43]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Float(None,
              | false)
               ,-[test:3:19]
             2 |         angle x = 42.;
             3 |         float y = x;
               :                   ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_explicit_float_implicitly_fails() {
    let input = "
        angle x = 42.;
        float[32] y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-48]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-48]:
                            symbol_id: 7
                            ty_span: [32-41]
                            init_expr: Expr [46-47]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Float(Some(32),
              | false)
               ,-[test:3:23]
             2 |         angle x = 42.;
             3 |         float[32] y = x;
               :                       ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_implicit_complex_implicitly_fails() {
    let input = "
        angle x = 42.;
        complex[float] y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-53]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-53]:
                            symbol_id: 7
                            ty_span: [32-46]
                            init_expr: Expr [51-52]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type Complex(None,
              | false)
               ,-[test:3:28]
             2 |         angle x = 42.;
             3 |         complex[float] y = x;
               :                            ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_explicit_complex_implicitly_fails() {
    let input = "
        angle x = 42.;
        complex[float[32]] y = x;
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
                            symbol_id: 6
                            ty_span: [9-14]
                            init_expr: Expr [19-22]:
                                ty: Angle(None, true)
                                kind: Lit: Float(42.0)
                    Stmt [32-57]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [32-57]:
                            symbol_id: 7
                            ty_span: [32-50]
                            init_expr: Expr [55-56]:
                                ty: Angle(None, false)
                                kind: SymbolId(6)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Angle(None, false) to type
              | Complex(Some(32), false)
               ,-[test:3:32]
             2 |         angle x = 42.;
             3 |         complex[float[32]] y = x;
               :                                ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn to_angle_implicitly() {
    let input = "
        angle x = 42.;
        angle y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.0)
            [6] Symbol [15-16]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-44]:
                symbol_id: 7
                ty_span: [32-37]
                init_expr: Expr [42-43]:
                    ty: Angle(None, false)
                    kind: SymbolId(6)
            [7] Symbol [38-39]:
                name: y
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default
        "#]],
    );
}

#[test]
fn to_explicit_angle_implicitly() {
    let input = "
        angle x = 42.;
        angle[4] y = x;
    ";

    check_classical_decls(
        input,
        &expect![[r#"
            ClassicalDeclarationStmt [9-23]:
                symbol_id: 6
                ty_span: [9-14]
                init_expr: Expr [19-22]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.0)
            [6] Symbol [15-16]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default
            ClassicalDeclarationStmt [32-47]:
                symbol_id: 7
                ty_span: [32-40]
                init_expr: Expr [45-46]:
                    ty: Angle(Some(4), false)
                    kind: SymbolId(6)
            [7] Symbol [41-42]:
                name: y
                type: Angle(Some(4), false)
                qsharp_type: Double
                io_kind: Default
        "#]],
    );
}
