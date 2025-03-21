// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kinds;
use expect_test::expect;

#[test]
fn if_branch_doesnt_create_its_own_scope() {
    check_stmt_kinds(
        "
    int a = 2;
    if (true) int a = 1;
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
                                ty: Int(None, false)
                                kind: Lit: Int(2)
                    Stmt [20-40]:
                        annotations: <empty>
                        kind: IfStmt [20-40]:
                            condition: Expr [24-28]:
                                ty: Bool(true)
                                kind: Lit: Bool(true)
                            if_body: Stmt [30-40]:
                                annotations: <empty>
                                kind: ClassicalDeclarationStmt [30-40]:
                                    symbol_id: 6
                                    ty_span: [30-33]
                                    init_expr: Expr [38-39]:
                                        ty: Int(None, false)
                                        kind: Lit: Int(1)
                            else_body: <none>

            [Qsc.Qasm3.Compile.RedefinedSymbol

              x Redefined symbol: a.
               ,-[test:3:19]
             2 |     int a = 2;
             3 |     if (true) int a = 1;
               :                   ^
             4 |     
               `----
            ]"#]],
    );
}

#[test]
fn else_branch_doesnt_create_its_own_scope() {
    check_stmt_kinds(
        "
    int a = 2;
    if (true) {}
    else int a = 1;
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
                                ty: Int(None, false)
                                kind: Lit: Int(2)
                    Stmt [20-52]:
                        annotations: <empty>
                        kind: IfStmt [20-52]:
                            condition: Expr [24-28]:
                                ty: Bool(true)
                                kind: Lit: Bool(true)
                            if_body: Stmt [30-32]:
                                annotations: <empty>
                                kind: Block [30-32]: <empty>
                            else_body: Stmt [42-52]:
                                annotations: <empty>
                                kind: ClassicalDeclarationStmt [42-52]:
                                    symbol_id: 6
                                    ty_span: [42-45]
                                    init_expr: Expr [50-51]:
                                        ty: Int(None, false)
                                        kind: Lit: Int(1)

            [Qsc.Qasm3.Compile.RedefinedSymbol

              x Redefined symbol: a.
               ,-[test:4:14]
             3 |     if (true) {}
             4 |     else int a = 1;
               :              ^
             5 |     
               `----
            ]"#]],
    );
}

#[test]
fn branch_block_creates_a_new_scope() {
    check_stmt_kinds(
        "
    int a = 2;
    if (true) { int a = 1; }
    ",
        &expect![[r#"
            ClassicalDeclarationStmt [5-15]:
                symbol_id: 6
                ty_span: [5-8]
                init_expr: Expr [13-14]:
                    ty: Int(None, false)
                    kind: Lit: Int(2)
            IfStmt [20-44]:
                condition: Expr [24-28]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
                if_body: Stmt [30-44]:
                    annotations: <empty>
                    kind: Block [30-44]:
                        Stmt [32-42]:
                            annotations: <empty>
                            kind: ClassicalDeclarationStmt [32-42]:
                                symbol_id: 7
                                ty_span: [32-35]
                                init_expr: Expr [40-41]:
                                    ty: Int(None, false)
                                    kind: Lit: Int(1)
                else_body: <none>
        "#]],
    );
}

#[test]
fn if_scope_and_else_scope_are_different() {
    check_stmt_kinds(
        "
    int a = 2;
    if (true) { int a = 1; }
    else { int a = 2; }
    ",
        &expect![[r#"
            ClassicalDeclarationStmt [5-15]:
                symbol_id: 6
                ty_span: [5-8]
                init_expr: Expr [13-14]:
                    ty: Int(None, false)
                    kind: Lit: Int(2)
            IfStmt [20-68]:
                condition: Expr [24-28]:
                    ty: Bool(true)
                    kind: Lit: Bool(true)
                if_body: Stmt [30-44]:
                    annotations: <empty>
                    kind: Block [30-44]:
                        Stmt [32-42]:
                            annotations: <empty>
                            kind: ClassicalDeclarationStmt [32-42]:
                                symbol_id: 7
                                ty_span: [32-35]
                                init_expr: Expr [40-41]:
                                    ty: Int(None, false)
                                    kind: Lit: Int(1)
                else_body: Stmt [54-68]:
                    annotations: <empty>
                    kind: Block [54-68]:
                        Stmt [56-66]:
                            annotations: <empty>
                            kind: ClassicalDeclarationStmt [56-66]:
                                symbol_id: 8
                                ty_span: [56-59]
                                init_expr: Expr [64-65]:
                                    ty: Int(None, false)
                                    kind: Lit: Int(2)
        "#]],
    );
}

#[test]
fn condition_cast() {
    check_stmt_kinds(
        "if (1) true;",
        &expect![[r#"
            IfStmt [0-12]:
                condition: Expr [4-5]:
                    ty: Bool(true)
                    kind: Cast [0-0]:
                        ty: Bool(true)
                        expr: Expr [4-5]:
                            ty: Int(None, true)
                            kind: Lit: Int(1)
                if_body: Stmt [7-12]:
                    annotations: <empty>
                    kind: ExprStmt [7-12]:
                        expr: Expr [7-11]:
                            ty: Bool(true)
                            kind: Lit: Bool(true)
                else_body: <none>
        "#]],
    );
}
