// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::semantic::tests::check_classical_decl;

#[test]
fn with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "creg a;",
        &expect![[r#"
            ClassicalDeclarationStmt [0-7]:
                symbol_id: 6
                ty_span: [0-7]
                init_expr: Expr [0-0]:
                    ty: Bit(true)
                    kind: Lit: Bit(0)
            [6] Symbol [5-6]:
                name: a
                type: Bit(false)
                qsharp_type: Result
                io_kind: Default"#]],
    );
}

#[test]
#[ignore = "Unimplemented"]
fn array_with_no_init_expr_has_generated_lit_expr() {
    check_classical_decl(
        "creg a[4];",
        &expect![[r#"
            Program:
                version: <none>
                statements: <empty>

            [Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: bit array
              | default value
               ,-[test:1:1]
             1 | creg a[4];
               : ^^^^^^^^^^
               `----
            ]"#]],
    );
}
