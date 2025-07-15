// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "duration a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-11]:
                symbol_id: 8
                ty_span: [0-8]
                init_expr: Expr [0-11]:
                    ty: const duration
                    kind: Lit: Duration(0.0, Ns)
            [8] Symbol [9-10]:
                name: a
                type: duration
                ty_span: [0-8]
                io_kind: Default"#]],
    );
}
