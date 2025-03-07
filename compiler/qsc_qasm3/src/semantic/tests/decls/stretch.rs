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
                statements: <empty>

            [Qsc.Qasm3.Compile.NotSupported

              x Stretch type values are not supported.
               ,-[test:1:1]
             1 | stretch a;
               : ^^^^^^^
               `----
            ]"#]],
    );
}
