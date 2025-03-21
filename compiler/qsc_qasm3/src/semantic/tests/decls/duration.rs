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
                            symbol_id: 6
                            ty_span: [0-8]
                            init_expr: Expr [0-0]:
                                ty: Duration(true)
                                kind: Err

            [Qsc.Qasm3.Compile.NotSupported

              x Duration type values are not supported.
               ,-[test:1:1]
             1 | duration a;
               : ^^^^^^^^
               `----
            , Qsc.Qasm3.Compile.NotSupported

              x Default values for Duration(false) are unsupported. are not supported.
               ,-[test:1:1]
             1 | duration a;
               : ^^^^^^^^^^^
               `----
            ]"#]],
    );
}
