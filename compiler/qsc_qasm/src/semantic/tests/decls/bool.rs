// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "bool a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-7]:
                symbol_id: 8
                ty_span: [0-4]
                init_expr: Expr [0-0]:
                    ty: const bool
                    kind: Lit: Bool(false)
            [8] Symbol [5-6]:
                name: a
                type: bool
                qsharp_type: bool
                io_kind: Default"#]],
    );
}

#[test]
fn array_with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "array[bool, 4] a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-17]:
                symbol_id: 8
                ty_span: [0-14]
                init_expr: Expr [0-0]:
                    ty: BoolArray(One(4))
                    kind: Lit:     array:
                            Expr [0-0]:
                                ty: const bool
                                kind: Lit: Bool(false)
                            Expr [0-0]:
                                ty: const bool
                                kind: Lit: Bool(false)
                            Expr [0-0]:
                                ty: const bool
                                kind: Lit: Bool(false)
                            Expr [0-0]:
                                ty: const bool
                                kind: Lit: Bool(false)
            [8] Symbol [15-16]:
                name: a
                type: BoolArray(One(4))
                qsharp_type: bool[]
                io_kind: Default"#]],
    );
}

#[test]
fn decl_with_lit_false_init_expr() {
    check_classical_decl(
        "bool a = false;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-15]:
                symbol_id: 8
                ty_span: [0-4]
                init_expr: Expr [9-14]:
                    ty: bool
                    kind: Lit: Bool(false)
            [8] Symbol [5-6]:
                name: a
                type: bool
                qsharp_type: bool
                io_kind: Default"#]],
    );
}

#[test]
fn decl_with_lit_true_init_expr() {
    check_classical_decl(
        "bool a = true;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-14]:
                symbol_id: 8
                ty_span: [0-4]
                init_expr: Expr [9-13]:
                    ty: bool
                    kind: Lit: Bool(true)
            [8] Symbol [5-6]:
                name: a
                type: bool
                qsharp_type: bool
                io_kind: Default"#]],
    );
}

#[test]
fn const_decl_with_lit_false_init_expr() {
    check_classical_decl(
        "const bool a = false;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-21]:
                symbol_id: 8
                ty_span: [6-10]
                init_expr: Expr [15-20]:
                    ty: const bool
                    kind: Lit: Bool(false)
            [8] Symbol [11-12]:
                name: a
                type: const bool
                qsharp_type: bool
                io_kind: Default"#]],
    );
}

#[test]
fn const_decl_with_lit_true_init_expr() {
    check_classical_decl(
        "const bool a = true;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-20]:
                symbol_id: 8
                ty_span: [6-10]
                init_expr: Expr [15-19]:
                    ty: const bool
                    kind: Lit: Bool(true)
            [8] Symbol [11-12]:
                name: a
                type: const bool
                qsharp_type: bool
                io_kind: Default"#]],
    );
}
