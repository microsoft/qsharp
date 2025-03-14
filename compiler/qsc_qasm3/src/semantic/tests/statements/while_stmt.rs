// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn single_stmt_body_doesnt_creates_its_own_scope() {
    check_stmt_kinds(
        "
    int a = 0;
    while(true) int a = 1;
    ",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [5-15]:
                        annotations: <empty>
                        kind: ClassicalDeclarationStmt [5-15]:
                            symbol_id: 6
                            ty_span: [5-8]
                            init_expr: Expr [13-14]:
                                ty: Int(None, true)
                                kind: Lit: Int(0)

            [Qsc.Qasm3.Compile.RedefinedSymbol

              x Redefined symbol: a.
               ,-[test:3:21]
             2 |     int a = 0;
             3 |     while(true) int a = 1;
               :                     ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn block_body_creates_its_own_scope() {
    check_stmt_kinds(
        "
    int a = 0;
    while(true) { int a = 1; }
    ",
        &expect![[r#"
            ClassicalDeclarationStmt [5-15]:
                symbol_id: 6
                ty_span: [5-8]
                init_expr: Expr [13-14]:
                    ty: Int(None, true)
                    kind: Lit: Int(0)
            WhileLoop [20-46]:
                condition: Expr [26-30]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
                body: Stmt [32-46]:
                    annotations: <empty>
                    kind: Block [32-46]:
                        Stmt [34-44]:
                            annotations: <empty>
                            kind: ClassicalDeclarationStmt [34-44]:
                                symbol_id: 7
                                ty_span: [34-37]
                                init_expr: Expr [42-43]:
                                    ty: Int(None, true)
                                    kind: Lit: Int(1)
        "#]],
    );
}
