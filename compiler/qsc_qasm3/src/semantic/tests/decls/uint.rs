// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn implicit_bitness_int_default() {
    check_classical_decl(
        "uint x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-7]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [0-0]:
                    ty: UInt(None, true)
                    kind: Lit: Int(0)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_default() {
    check_classical_decl(
        "const uint x;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qasm3.Parse.Token

              x expected `=`, found `;`
               ,-[test:1:13]
             1 | const uint x;
               :             ^
               `----
            , Qsc.Qasm3.Compile.UnexpectedParserError

              x Unexpected parser error: Unexpected error.
               ,-[test:1:1]
             1 | const uint x;
               : ^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn const_implicit_bitness_int_lit() {
    check_classical_decl(
        "const uint x = 42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-18]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-17]:
                    ty: UInt(None, true)
                    kind: Lit: Int(42)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_hex_cap() {
    check_classical_decl(
        "uint x = 0XFa_1F;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-17]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [9-16]:
                    ty: UInt(None, true)
                    kind: Lit: Int(64031)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_hex_low() {
    check_classical_decl(
        "const uint x = 0xFa_1F;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-23]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-22]:
                    ty: UInt(None, true)
                    kind: Lit: Int(64031)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_hex_cap() {
    check_classical_decl(
        "const uint y = 0XFa_1F;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-23]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-22]:
                    ty: UInt(None, true)
                    kind: Lit: Int(64031)
            [6] Symbol [11-12]:
                name: y
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_octal_low() {
    check_classical_decl(
        "uint x = 0o42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-14]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [9-13]:
                    ty: UInt(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_octal_cap() {
    check_classical_decl(
        "uint x = 0O42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-14]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [9-13]:
                    ty: UInt(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_octal_low() {
    check_classical_decl(
        "const uint x = 0o42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-20]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_octal_cap() {
    check_classical_decl(
        "const uint x = 0O42;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-20]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-19]:
                    ty: UInt(None, true)
                    kind: Lit: Int(34)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_binary_low() {
    check_classical_decl(
        "uint x = 0b1001_1001;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [9-20]:
                    ty: UInt(None, true)
                    kind: Lit: Int(153)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_binary_cap() {
    check_classical_decl(
        "uint x = 0B1010;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-16]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [9-15]:
                    ty: UInt(None, true)
                    kind: Lit: Int(10)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_binary_low() {
    check_classical_decl(
        "const uint x = 0b1001_1001;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-27]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-26]:
                    ty: UInt(None, true)
                    kind: Lit: Int(153)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_binary_cap() {
    check_classical_decl(
        "const uint x = 0B1010;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-22]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-21]:
                    ty: UInt(None, true)
                    kind: Lit: Int(10)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_formatted() {
    check_classical_decl(
        "uint x = 2_0_00;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-16]:
                symbol_id: 6
                ty_span: [0-4]
                init_expr: Expr [9-15]:
                    ty: UInt(None, true)
                    kind: Lit: Int(2000)
            [6] Symbol [5-6]:
                name: x
                type: UInt(None, false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_formatted() {
    check_classical_decl(
        "const uint x = 2_0_00;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-22]:
                symbol_id: 6
                ty_span: [6-10]
                init_expr: Expr [15-21]:
                    ty: UInt(None, true)
                    kind: Lit: Int(2000)
            [6] Symbol [11-12]:
                name: x
                type: UInt(None, true)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn const_explicit_bitness_int() {
    check_classical_decl(
        "uint[10] x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-11]:
                symbol_id: 6
                ty_span: [0-8]
                init_expr: Expr [0-0]:
                    ty: UInt(Some(10), true)
                    kind: Lit: Int(0)
            [6] Symbol [9-10]:
                name: x
                type: UInt(Some(10), false)
                qsharp_type: Int
                io_kind: Default"#]],
    );
}

#[test]
fn explicit_bitness_int() {
    check_classical_decl(
        "const uint[10] x;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qasm3.Parse.Token

              x expected `=`, found `;`
               ,-[test:1:17]
             1 | const uint[10] x;
               :                 ^
               `----
            , Qsc.Qasm3.Compile.UnexpectedParserError

              x Unexpected parser error: Unexpected error.
               ,-[test:1:1]
             1 | const uint[10] x;
               : ^^^^^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn assigning_uint_to_negative_lit_results_in_semantic_error() {
    check_classical_decl(
        "const uint[10] x = -42;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qsc.Qasm3.Compile.CannotAssignToType

              x Cannot assign a value of Int(None, true) type to a classical variable of
              | UInt(Some(10), true) type.
               ,-[test:1:1]
             1 | const uint[10] x = -42;
               : ^^^^^^^^^^^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn implicit_bitness_uint_const_negative_decl_raises_semantic_error() {
    check_classical_decl(
        "const uint x = -42;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qsc.Qasm3.Compile.CannotAssignToType

              x Cannot assign a value of Int(None, true) type to a classical variable of
              | UInt(None, true) type.
               ,-[test:1:1]
             1 | const uint x = -42;
               : ^^^^^^^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn explicit_bitness_uint_const_negative_decl_raises_semantic_error() {
    check_classical_decl(
        "const uint[32] x = -42;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qsc.Qasm3.Compile.CannotAssignToType

              x Cannot assign a value of Int(None, true) type to a classical variable of
              | UInt(Some(32), true) type.
               ,-[test:1:1]
             1 | const uint[32] x = -42;
               : ^^^^^^^^^^^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}
