// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn not_supported_before_version_3_1() {
    check_stmt_kinds(
        r#"
    OPENQASM 3.0;
    switch (1) { case 1 {} }
    "#,
        &expect![[r#"
            Program:
                version: 3.0
                statements: <empty>

            [Qsc.Qasm3.Compile.NotSupported

              x switch statements were introduced in version 3.1 are not supported.
               ,-[test:4:5]
             3 |
             4 | ,->     switch (1) {
             5 | |           case 1 {}
             6 | `->     }
             7 |
               `----
            ]"#]],
    );
}

#[test]
fn cases_introduce_their_own_scope() {
    check_stmt_kinds(
        r#"
    int a = 0;
    switch (1) {
        case 1 { int a = 1; }
        case 2, 3 { int a = 2; }
    }
    "#,
        &expect![[r#"
            ClassicalDeclarationStmt [5-15]:
                symbol_id: 6
                ty_span: [5-8]
                init_expr: Expr [13-14]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            SwitchStmt [20-101]:
                target: Expr [28-29]:
                    ty: Int(None, true)
                    kind: Lit: Int(1)
                cases:
                    SwitchCase [41-62]:
                        labels:
                            Expr [46-47]:
                                ty: Int(None, true)
                                kind: Lit: Int(1)
                        block: Block [48-62]:
                            Stmt [50-60]:
                                annotations: <empty>
                                kind: ClassicalDeclarationStmt [50-60]:
                                    symbol_id: 7
                                    ty_span: [50-53]
                                    init_expr: Expr [58-59]:
                                        ty: Int(None, true)
                                        kind: Lit: Int(1)
                    SwitchCase [71-95]:
                        labels:
                            Expr [76-77]:
                                ty: Int(None, true)
                                kind: Lit: Int(2)
                            Expr [79-80]:
                                ty: Int(None, true)
                                kind: Lit: Int(3)
                        block: Block [81-95]:
                            Stmt [83-93]:
                                annotations: <empty>
                                kind: ClassicalDeclarationStmt [83-93]:
                                    symbol_id: 8
                                    ty_span: [83-86]
                                    init_expr: Expr [91-92]:
                                        ty: Int(None, true)
                                        kind: Lit: Int(2)
                default_case: <none>
        "#]],
    );
}
