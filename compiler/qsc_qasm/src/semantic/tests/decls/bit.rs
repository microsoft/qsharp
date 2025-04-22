// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "bit a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-6]:
                symbol_id: 8
                ty_span: [0-3]
                init_expr: Expr [0-0]:
                    ty: Bit(true)
                    kind: Lit: Bit(0)
            [8] Symbol [4-5]:
                name: a
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default"#]],
    );
}

#[test]
fn array_with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "bit[4] a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-9]:
                symbol_id: 8
                ty_span: [0-6]
                init_expr: Expr [0-0]:
                    ty: BitArray(4, true)
                    kind: Lit: Bitstring("0000")
            [8] Symbol [7-8]:
                name: a
                type: BitArray(4, false)
                qsharp_type: Result[]
                io_kind: Default"#]],
    );
}

#[test]
fn decl_with_lit_0_init_expr() {
    check_classical_decl(
        "bit a = 0;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-10]:
                symbol_id: 8
                ty_span: [0-3]
                init_expr: Expr [8-9]:
                    ty: Bit(true)
                    kind: Lit: Bit(0)
            [8] Symbol [4-5]:
                name: a
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default"#]],
    );
}

#[test]
fn decl_with_lit_1_init_expr() {
    check_classical_decl(
        "bit a = 1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-10]:
                symbol_id: 8
                ty_span: [0-3]
                init_expr: Expr [8-9]:
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [4-5]:
                name: a
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default"#]],
    );
}

#[test]
fn const_decl_with_lit_0_init_expr() {
    check_classical_decl(
        "const bit a = 0;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-16]:
                symbol_id: 8
                ty_span: [6-9]
                init_expr: Expr [14-15]:
                    ty: Bit(true)
                    kind: Lit: Bit(0)
            [8] Symbol [10-11]:
                name: a
                type: Bit(true)
                qsharp_type: Result
                io_kind: Default"#]],
    );
}

#[test]
fn const_decl_with_lit_1_init_expr() {
    check_classical_decl(
        "const bit a = 1;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-16]:
                symbol_id: 8
                ty_span: [6-9]
                init_expr: Expr [14-15]:
                    ty: Bit(true)
                    kind: Lit: Bit(1)
            [8] Symbol [10-11]:
                name: a
                type: Bit(true)
                qsharp_type: Result
                io_kind: Default"#]],
    );
}
