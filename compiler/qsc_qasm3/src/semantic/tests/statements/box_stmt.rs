// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn with_invalid_instruction_fails() {
    check_stmt_kinds(
        "box {
        2 + 4;
    }",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-26]:
                        annotations: <empty>
                        kind: Err

            [Qsc.Qasm3.Lowerer.ClassicalStmtInBox

              x invalid classical statement in box
               ,-[test:2:9]
             1 | box {
             2 |         2 + 4;
               :         ^^^^^^
             3 |     }
               `----
            , Qsc.Qasm3.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: box stmt
               ,-[test:1:1]
             1 | ,-> box {
             2 | |           2 + 4;
             3 | `->     }
               `----
            ]"#]],
    );
}

#[test]
fn with_duration_fails() {
    check_stmt_kinds(
        "box [4us] { }",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [0-13]:
                        annotations: <empty>
                        kind: Err

            [Qsc.Qasm3.Lowerer.NotSupported

              x Box with duration are not supported.
               ,-[test:1:6]
             1 | box [4us] { }
               :      ^^^
               `----
            , Qsc.Qasm3.Lowerer.Unimplemented

              x this statement is not yet handled during OpenQASM 3 import: box stmt
               ,-[test:1:1]
             1 | box [4us] { }
               : ^^^^^^^^^^^^^
               `----
            ]"#]],
    );
}
