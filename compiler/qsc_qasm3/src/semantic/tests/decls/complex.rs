// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn implicit_bitness_default() {
    check_classical_decl(
        "complex[float] x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-17]:
                symbol_id: 6
                ty_span: [0-14]
                init_expr: Expr [0-0]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(0.0, 0.0)
            [6] Symbol [15-16]:
                name: x
                type: Complex(None, false)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn explicit_bitness_default() {
    check_classical_decl(
        "complex[float[42]] x;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [0-18]
                init_expr: Expr [0-0]:
                    ty: Complex(Some(42), true)
                    kind: Lit: Complex(0.0, 0.0)
            [6] Symbol [19-20]:
                name: x
                type: Complex(Some(42), false)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_double_img_only() {
    check_classical_decl(
        "const complex[float] x = 1.01im;",
        &expect![[r#"
        ClassicalDeclarationStmt [0-32]:
            symbol_id: 6
            ty_span: [6-20]
            init_expr: Expr [25-31]:
                ty: Complex(None, true)
                kind: Lit: Complex(0.0, 1.01)
        [6] Symbol [21-22]:
            name: x
            type: Complex(None, true)
            qsharp_type: Complex
            io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_img_only() {
    check_classical_decl(
        "const complex[float] x = 1im;",
        &expect![[r#"
        ClassicalDeclarationStmt [0-29]:
            symbol_id: 6
            ty_span: [6-20]
            init_expr: Expr [25-28]:
                ty: Complex(None, true)
                kind: Lit: Complex(0.0, 1.0)
        [6] Symbol [21-22]:
            name: x
            type: Complex(None, true)
            qsharp_type: Complex
            io_kind: Default"#]],
    );
}

#[test]
fn const_explicit_bitness_double_img_only() {
    check_classical_decl(
        "const complex[float[42]] x = 1.01im;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-36]:
                symbol_id: 6
                ty_span: [6-24]
                init_expr: Expr [29-35]:
                    ty: Complex(Some(42), true)
                    kind: Lit: Complex(0.0, 1.01)
            [6] Symbol [25-26]:
                name: x
                type: Complex(Some(42), true)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn const_explicit_bitness_int_img_only() {
    check_classical_decl(
        "const complex[float[42]] x = 1im;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-33]:
                symbol_id: 6
                ty_span: [6-24]
                init_expr: Expr [29-32]:
                    ty: Complex(Some(42), true)
                    kind: Lit: Complex(0.0, 1.0)
            [6] Symbol [25-26]:
                name: x
                type: Complex(Some(42), true)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_double_img_only() {
    check_classical_decl(
        "complex[float] x = 1.01im;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-26]:
                symbol_id: 6
                ty_span: [0-14]
                init_expr: Expr [19-25]:
                    ty: Complex(None, false)
                    kind: Lit: Complex(0.0, 1.01)
            [6] Symbol [15-16]:
                name: x
                type: Complex(None, false)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_img_only() {
    check_classical_decl(
        "complex[float] x = 1im;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-23]:
                symbol_id: 6
                ty_span: [0-14]
                init_expr: Expr [19-22]:
                    ty: Complex(None, false)
                    kind: Lit: Complex(0.0, 1.0)
            [6] Symbol [15-16]:
                name: x
                type: Complex(None, false)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_double_real_only() {
    check_classical_decl(
        "const complex[float] x = 1.01;",
        &expect![[r#"
        ClassicalDeclarationStmt [0-30]:
            symbol_id: 6
            ty_span: [6-20]
            init_expr: Expr [25-29]:
                ty: Complex(None, true)
                kind: Lit: Complex(1.01, 0.0)
        [6] Symbol [21-22]:
            name: x
            type: Complex(None, true)
            qsharp_type: Complex
            io_kind: Default"#]],
    );
}

#[test]
fn const_implicit_bitness_int_real_only() {
    check_classical_decl(
        "const complex[float] x = 1;",
        &expect![[r#"
        ClassicalDeclarationStmt [0-27]:
            symbol_id: 6
            ty_span: [6-20]
            init_expr: Expr [25-26]:
                ty: Complex(None, true)
                kind: Lit: Complex(1.0, 0.0)
        [6] Symbol [21-22]:
            name: x
            type: Complex(None, true)
            qsharp_type: Complex
            io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_double_real_only() {
    check_classical_decl(
        "complex[float] x = 1.01;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-24]:
                symbol_id: 6
                ty_span: [0-14]
                init_expr: Expr [19-23]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(1.01, 0.0)
            [6] Symbol [15-16]:
                name: x
                type: Complex(None, false)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
fn implicit_bitness_int_real_only() {
    check_classical_decl(
        "complex[float] x = 1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 6
                ty_span: [0-14]
                init_expr: Expr [19-20]:
                    ty: Complex(None, true)
                    kind: Lit: Complex(1.0, 0.0)
            [6] Symbol [15-16]:
                name: x
                type: Complex(None, false)
                qsharp_type: Complex
                io_kind: Default"#]],
    );
}

#[test]
#[ignore = "Requires support for binary operators"]
fn implicit_bitness_simple_double_pos_im() {
    check_classical_decl(
        "complex[float] x = 1.1 + 2.2im;",
        &expect![[r#"
        Program:
            version: <none>
            statements: <empty>

        [Qsc.Qasm3.Compile.Unimplemented

          x this statement is not yet handled during OpenQASM 3 import: binary op expr
           ,-[test:1:20]
         1 | complex[float] x = 1.1 + 2.2im;
           :                    ^^^^^^^^^^^
           `----
        ]"#]],
    );
}

#[test]
#[ignore = "Requires support for binary operators"]
fn implicit_bitness_simple_double_neg_im() {
    check_classical_decl(
        "complex[float] x = 1.1 - 2.2im;",
        &expect![[r#"
        Program:
            version: <none>
            statements: <empty>

        [Qsc.Qasm3.Compile.Unimplemented

          x this statement is not yet handled during OpenQASM 3 import: binary op expr
           ,-[test:1:20]
         1 | complex[float] x = 1.1 - 2.2im;
           :                    ^^^^^^^^^^^
           `----
        ]"#]],
    );
}

#[test]
#[ignore = "Requires support for binary operators"]
fn const_implicit_bitness_simple_double_neg_im() {
    check_classical_decl(
        "const complex[float] x = 1.1 - 2.2im;",
        &expect![[r#"
        Program:
            version: <none>
            statements: <empty>

        [Qsc.Qasm3.Compile.Unimplemented

          x this statement is not yet handled during OpenQASM 3 import: binary op expr
           ,-[test:1:26]
         1 | const complex[float] x = 1.1 - 2.2im;
           :                          ^^^^^^^^^^^
           `----
        ]"#]],
    );
}

#[test]
#[ignore = "Requires support for binary operators"]
fn implicit_bitness_simple_double_neg_real() {
    check_classical_decl(
        "complex[float] x = -1.1 + 2.2im;",
        &expect![[r#"
        Program:
            version: <none>
            statements: <empty>

        [Qsc.Qasm3.Compile.Unimplemented

          x this statement is not yet handled during OpenQASM 3 import: binary op expr
           ,-[test:1:20]
         1 | complex[float] x = -1.1 + 2.2im;
           :                    ^^^^^^^^^^^^
           `----
        ]"#]],
    );
}

#[test]
#[ignore = "Requires support for binary operators"]
fn const_implicit_bitness_simple_double_neg_real() {
    check_classical_decl(
        "const complex[float] x = -1.1 + 2.2im;",
        &expect![[r#"
        Program:
            version: <none>
            statements: <empty>

        [Qsc.Qasm3.Compile.Unimplemented

          x this statement is not yet handled during OpenQASM 3 import: binary op expr
           ,-[test:1:26]
         1 | const complex[float] x = -1.1 + 2.2im;
           :                          ^^^^^^^^^^^^
           `----
        ]"#]],
    );
}
