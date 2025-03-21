// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod angle;
mod bit;
mod bool;
mod complex;
mod creg;
mod duration;
mod float;
mod int;
mod qreg;
mod stretch;
mod uint;

use expect_test::expect;

use super::check;

#[test]
#[ignore = "Not yet implemented"]
fn duration_and_stretch_types_without_init_exprs() {
    check(
        r#"
        duration i;
        stretch n;
        "#,
        &expect![[r#"


            [Qsc.Qasm3.Compile.NotSupported

              x Duration type values are not supported.
               ,-[test:2:9]
             1 |
             2 |         duration i;
               :         ^^^^^^^^
             3 |         stretch n;
               `----
            , Qsc.Qasm3.Compile.NotSupported

              x Stretch type values are not supported.
               ,-[test:3:9]
             2 |         duration i;
             3 |         stretch n;
               :         ^^^^^^^
             4 |
               `----
            ]"#]],
    );
}

#[test]
fn scalar_ty_designator_must_be_positive() {
    check(
        "int[-5] i;",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-10]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [0-10]:
                            symbol_id: 6
                            ty_span: [0-7]
                            init_expr: Expr [0-0]:
                                ty: Err
                                kind: Err

            [Qsc.Qasm3.Compile.DesignatorMustBePositiveIntLiteral

              x Designator must be a positive literal integer.
               ,-[test:1:5]
             1 | int[-5] i;
               :     ^^
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: Converting Err
              | to Q# type
               ,-[test:1:1]
             1 | int[-5] i;
               : ^^^^^^^
               `----
            , Qsc.Qasm3.Compile.NotSupported

              x Default values for Err are unsupported. are not supported.
               ,-[test:1:1]
             1 | int[-5] i;
               : ^^^^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn scalar_ty_designator_must_be_int_literal() {
    check(
        r#"int[size] i; float[0.0] j;"#,
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-12]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [0-12]:
                            symbol_id: 6
                            ty_span: [0-9]
                            init_expr: Expr [0-0]:
                                ty: Err
                                kind: Err
                    Stmt [13-26]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [13-26]:
                            symbol_id: 7
                            ty_span: [13-23]
                            init_expr: Expr [0-0]:
                                ty: Err
                                kind: Err

            [Qsc.Qasm3.Compile.DesignatorMustBePositiveIntLiteral

              x Designator must be a positive literal integer.
               ,-[test:1:5]
             1 | int[size] i; float[0.0] j;
               :     ^^^^
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: Converting Err
              | to Q# type
               ,-[test:1:1]
             1 | int[size] i; float[0.0] j;
               : ^^^^^^^^^
               `----
            , Qsc.Qasm3.Compile.NotSupported

              x Default values for Err are unsupported. are not supported.
               ,-[test:1:1]
             1 | int[size] i; float[0.0] j;
               : ^^^^^^^^^^^^
               `----
            , Qsc.Qasm3.Compile.DesignatorMustBePositiveIntLiteral

              x Designator must be a positive literal integer.
               ,-[test:1:20]
             1 | int[size] i; float[0.0] j;
               :                    ^^^
               `----
            , Qsc.Qasm3.Compile.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: Converting Err
              | to Q# type
               ,-[test:1:14]
             1 | int[size] i; float[0.0] j;
               :              ^^^^^^^^^^
               `----
            , Qsc.Qasm3.Compile.NotSupported

              x Default values for Err are unsupported. are not supported.
               ,-[test:1:14]
             1 | int[size] i; float[0.0] j;
               :              ^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}
