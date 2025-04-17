// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "duration a;",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-11]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [0-11]:
                            symbol_id: 8
                            ty_span: [0-8]
                            init_expr: Expr [0-0]:
                                ty: Duration(true)
                                kind: Lit: Duration(0.0, Ns)

            [Qsc.Qasm3.Lowerer.NotSupported

              x duration type values are not supported
               ,-[test:1:1]
             1 | duration a;
               : ^^^^^^^^
               `----
            ]"#]],
    );
}
