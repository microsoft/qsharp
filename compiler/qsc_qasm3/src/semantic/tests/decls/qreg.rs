// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_stmt_kind;

#[test]
#[ignore = "unimplemented"]
fn with_no_init_expr_has_generated_lit_expr() {
    check_stmt_kind(
        "qreg a;",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: qubit decl
              | stmt
               ,-[test:1:1]
             1 | qreg a;
               : ^^^^^^^
               `----
            ]"#]],
    );
}
