// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "stretch a;",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-10]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [0-10]:
                            symbol_id: 8
                            ty_span: [0-7]
                            init_expr: Expr [0-0]:
                                ty: Stretch(true)
                                kind: Err

            [Qsc.Qasm3.Lowerer.NotSupported

              x Stretch type values are not supported.
               ,-[test:1:1]
             1 | stretch a;
               : ^^^^^^^
               `----
            , Qsc.Qasm3.Lowerer.NotSupported

              x Stretch default values are not supported.
               ,-[test:1:1]
             1 | stretch a;
               : ^^^^^^^^^^
               `----
            ]"#]],
    );
}
