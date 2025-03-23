// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn shadowing_loop_variable_in_single_stmt_body_fails() {
    check_stmt_kinds(
        "
    for int x in {}
        int x = 2;
    ",
        &expect![[r#"
            Program:
                version: <none>
                statements:
                    Stmt [5-39]:
                        annotations: <empty>
                        kind: ForStmt [5-39]:
                            loop_variable: 8
                            iterable: DiscreteSet [18-20]:
                                values: <empty>
                            body: Stmt [29-39]:
                                annotations: <empty>
                                kind: ClassicalDeclarationStmt [29-39]:
                                    symbol_id: 8
                                    ty_span: [29-32]
                                    init_expr: Expr [37-38]:
                                        ty: Int(None, false)
                                        kind: Lit: Int(2)

            [Qsc.Qasm3.Compile.RedefinedSymbol

              x Redefined symbol: x.
               ,-[test:3:13]
             2 |     for int x in {}
             3 |         int x = 2;
               :             ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn shadowing_loop_variable_in_block_body_succeeds() {
    check_stmt_kinds(
        "
    for int x in {} {
        int x = 2;
    }
    ",
        &expect![[r#"
            ForStmt [5-47]:
                loop_variable: 8
                iterable: DiscreteSet [18-20]:
                    values: <empty>
                body: Stmt [21-47]:
                    annotations: <empty>
                    kind: Block [21-47]:
                        Stmt [31-41]:
                            annotations: <empty>
                            kind: ClassicalDeclarationStmt [31-41]:
                                symbol_id: 9
                                ty_span: [31-34]
                                init_expr: Expr [39-40]:
                                    ty: Int(None, false)
                                    kind: Lit: Int(2)
        "#]],
    );
}

#[test]
fn loop_creates_its_own_scope() {
    check_stmt_kinds(
        "
    int a = 0;
    for int x in {}
        // shadowing works because this
        // declaration is in a different
        // scope from `int a = 0;` scope.
        int a = 1;
    ",
        &expect![[r#"
            ClassicalDeclarationStmt [5-15]:
                symbol_id: 8
                ty_span: [5-8]
                init_expr: Expr [13-14]:
                    ty: Int(None, false)
                    kind: Lit: Int(0)
            ForStmt [20-177]:
                loop_variable: 9
                iterable: DiscreteSet [33-35]:
                    values: <empty>
                body: Stmt [167-177]:
                    annotations: <empty>
                    kind: ClassicalDeclarationStmt [167-177]:
                        symbol_id: 10
                        ty_span: [167-170]
                        init_expr: Expr [175-176]:
                            ty: Int(None, false)
                            kind: Lit: Int(1)
        "#]],
    );
}
