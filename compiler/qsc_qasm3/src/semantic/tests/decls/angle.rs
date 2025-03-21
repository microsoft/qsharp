// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn implicit_bitness_default() {
    check_classical_decl(
        "angle x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-8]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [0-0]:
                    ty: Angle(None, true)
                    kind: Lit: Float(0.0)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit() {
    check_classical_decl(
        "angle x = 42.1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-14]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit() {
    check_classical_decl(
        "const angle x = 42.1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-20]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_explicit_width() {
    check_classical_decl(
        "angle[64] x = 42.1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-19]:
                symbol_id: 6
                ty_span: [0-9]
                init_expr: Expr [14-18]:
                    ty: Angle(Some(64), true)
                    kind: Lit: Float(42.1)
            [6] Symbol [10-11]:
                name: x
                type: Angle(Some(64), false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_explicit_width_lit() {
    check_classical_decl(
        "const angle[64] x = 42.1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-25]:
                symbol_id: 6
                ty_span: [6-15]
                init_expr: Expr [20-24]:
                    ty: Angle(Some(64), true)
                    kind: Lit: Float(42.1)
            [6] Symbol [16-17]:
                name: x
                type: Angle(Some(64), true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_decl_leading_dot() {
    check_classical_decl(
        "angle x = .421;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-14]:
                    ty: Angle(None, true)
                    kind: Lit: Float(0.421)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_leading_dot() {
    check_classical_decl(
        "const angle x = .421;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-20]:
                    ty: Angle(None, true)
                    kind: Lit: Float(0.421)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_leading_dot_scientific() {
    check_classical_decl(
        "const angle x = .421e2;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-23]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-22]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_decl_trailing_dot() {
    check_classical_decl(
        "angle x = 421.;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-14]:
                    ty: Angle(None, true)
                    kind: Lit: Float(421.0)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_trailing_dot() {
    check_classical_decl(
        "const angle x = 421.;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-20]:
                    ty: Angle(None, true)
                    kind: Lit: Float(421.0)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_decl_scientific() {
    check_classical_decl(
        "angle x = 4.21e1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-17]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-16]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_scientific() {
    check_classical_decl(
        "const angle x = 4.21e1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-23]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-22]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_decl_scientific_signed_pos() {
    check_classical_decl(
        "angle x = 4.21e+1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-18]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-17]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_scientific_signed_pos() {
    check_classical_decl(
        "const angle x = 4.21e+1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-24]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-23]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_decl_scientific_cap_e() {
    check_classical_decl(
        "angle x = 4.21E1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-17]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-16]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_scientific_cap_e() {
    check_classical_decl(
        "const angle x = 4.21E1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-23]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-22]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn lit_decl_scientific_signed_neg() {
    check_classical_decl(
        "angle x = 421.0e-1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-19]:
                symbol_id: 6
                ty_span: [0-5]
                init_expr: Expr [10-18]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [6-7]:
                name: x
                type: Angle(None, false)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_scientific_signed_neg() {
    check_classical_decl(
        "const angle x = 421.0e-1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-25]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [16-24]:
                    ty: Angle(None, true)
                    kind: Lit: Float(42.1)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_signed_float_lit_cast_neg() {
    check_classical_decl(
        "const angle x = -7.;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-20]:
                symbol_id: 6
                ty_span: [6-11]
                init_expr: Expr [17-19]:
                    ty: Angle(None, true)
                    kind: Cast [0-0]:
                        ty: Angle(None, true)
                        expr: Expr [17-19]:
                            ty: Float(None, true)
                            kind: UnaryOpExpr [17-19]:
                                op: Neg
                                expr: Expr [17-19]:
                                    ty: Float(None, true)
                                    kind: Lit: Float(7.0)
            [6] Symbol [12-13]:
                name: x
                type: Angle(None, true)
                qsharp_type: Double
                io_kind: Default"#]],
    );
}

#[test]
fn const_lit_decl_signed_int_lit_cast_neg_fails() {
    check_classical_decl(
        "const angle x = -7;",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-19]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [0-19]:
                            symbol_id: 6
                            ty_span: [6-11]
                            init_expr: Expr [17-18]:
                                ty: Int(None, true)
                                kind: UnaryOpExpr [17-18]:
                                    op: Neg
                                    expr: Expr [17-18]:
                                        ty: Int(None, true)
                                        kind: Lit: Int(7)

            [Qsc.Qasm3.Compile.CannotCast

              x Cannot cast expression of type Int(None, true) to type Angle(None, true)
               ,-[test:1:18]
             1 | const angle x = -7;
               :                  ^
               `----
            ]"#]],
    );
}
