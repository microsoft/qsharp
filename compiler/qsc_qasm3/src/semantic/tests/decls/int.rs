// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn implicit_bitness_int_negative() {
    check_classical_decl(
        "int x = -42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-12]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [9-11]:
                    ty: Int(None, true)
                    kind: Lit: Int(-42)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_const_negative() {
    check_classical_decl(
        "const int x = -42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-18]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [15-17]:
                    ty: Int(None, true)
                    kind: Lit: Int(-42)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_default() {
    check_classical_decl(
        "int x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-6]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [0-0]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_default() {
    check_classical_decl(
        "const int x;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qasm3.Parse.Token

              x expected `=`, found `;`
               ,-[test:1:12]
             1 | const int x;
               :            ^
               `----
            , Qsc.Qasm3.Compile.UnexpectedParserError

              x Unexpected parser error: Unexpected error.
               ,-[test:1:1]
             1 | const int x;
               : ^^^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn const_implicit_bitness_int_lit() {
    check_classical_decl(
        "const int x = 42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-17]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-16]:
                    ty: Int(None, true)
                    kind: Lit: Int(42)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_hex_cap() {
    check_classical_decl(
        "int x = 0XFa_1F;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-16]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [8-15]:
                    ty: Int(None, true)
                    kind: Lit: Int(64031)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_hex_cap() {
    check_classical_decl(
        "const int y = 0XFa_1F;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-22]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-21]:
                    ty: Int(None, true)
                    kind: Lit: Int(64031)
            [6] Symbol [10-11]:
                name: y
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_octal() {
    check_classical_decl(
        "int x = 0o42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-13]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [8-12]:
                    ty: Int(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_octal() {
    check_classical_decl(
        "const int x = 0o42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-19]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-18]:
                    ty: Int(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_octal_cap() {
    check_classical_decl(
        "const int x = 0O42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-19]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-18]:
                    ty: Int(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_binary_low() {
    check_classical_decl(
        "int x = 0b1001_1001;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-20]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [8-19]:
                    ty: Int(None, true)
                    kind: Lit: Int(153)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_binary_cap() {
    check_classical_decl(
        "int x = 0B1010;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [8-14]:
                    ty: Int(None, true)
                    kind: Lit: Int(10)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_binary_low() {
    check_classical_decl(
        "const int x = 0b1001_1001;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-26]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-25]:
                    ty: Int(None, true)
                    kind: Lit: Int(153)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_binary_cap() {
    check_classical_decl(
        "const int x = 0B1010;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-20]:
                    ty: Int(None, true)
                    kind: Lit: Int(10)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_formatted() {
    check_classical_decl(
        "int x = 2_0_00;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 6
                ty_span: [0-3]
                init_expr: Expr [8-14]:
                    ty: Int(None, true)
                    kind: Lit: Int(2000)
            [6] Symbol [4-5]:
                name: x
                type: Int(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_formatted() {
    check_classical_decl(
        "const int x = 2_0_00;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [6-9]
                init_expr: Expr [14-20]:
                    ty: Int(None, true)
                    kind: Lit: Int(2000)
            [6] Symbol [10-11]:
                name: x
                type: Int(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn explicit_bitness_int_default() {
    check_classical_decl(
        "int[10] x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-10]:
                symbol_id: 6
                ty_span: [0-7]
                init_expr: Expr [0-0]:
                    ty: Int(Some(10), true)
                    kind: Lit: Int(0)
            [6] Symbol [8-9]:
                name: x
                type: Int(Some(10), false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_explicit_bitness_int_default() {
    check_classical_decl(
        "const int[10] x;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qasm3.Parse.Token

              x expected `=`, found `;`
               ,-[test:1:16]
             1 | const int[10] x;
               :                ^
               `----
            , Qsc.Qasm3.Compile.UnexpectedParserError

              x Unexpected parser error: Unexpected error.
               ,-[test:1:1]
             1 | const int[10] x;
               : ^^^^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn explicit_bitness_int() {
    check_classical_decl(
        "int[10] x = 42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 6
                ty_span: [0-7]
                init_expr: Expr [12-14]:
                    ty: Int(Some(10), true)
                    kind: Lit: Int(42)
            [6] Symbol [8-9]:
                name: x
                type: Int(Some(10), false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_explicit_bitness_int() {
    check_classical_decl(
        "const int[10] x = 42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [6-13]
                init_expr: Expr [18-20]:
                    ty: Int(Some(10), true)
                    kind: Lit: Int(42)
            [6] Symbol [14-15]:
                name: x
                type: Int(Some(10), true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_negative_float_decl_causes_semantic_error() {
    check_classical_decl(
        "int x = -42.;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qsc.Qasm3.Compile.CannotAssignToType

              x Cannot assign a value of Float(None, true) type to a classical variable of
              | Int(None, false) type.
               ,-[test:1:1]
             1 | int x = -42.;
               : ^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}
