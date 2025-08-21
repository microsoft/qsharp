// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::semantic::tests::check_stmt_kind;
use expect_test::expect;

#[test]
fn void_no_args() {
    check_stmt_kind(
        "extern f();",
        &expect![[r#"
            ExternDecl [0-11]:
                symbol_id: 8
                default_value_expr: <none>
                param_spans: <empty>
                return_ty_span: [0-0]"#]],
    );
}

#[test]
fn void_one_arg() {
    check_stmt_kind(
        "extern f(int);",
        &expect![[r#"
            ExternDecl [0-14]:
                symbol_id: 8
                default_value_expr: <none>
                param_spans:
                    [9-12]
                return_ty_span: [0-0]"#]],
    );
}

#[test]
fn void_multiple_args() {
    check_stmt_kind(
        "extern f(uint, int, float, bit, bool);",
        &expect![[r#"
            ExternDecl [0-38]:
                symbol_id: 8
                default_value_expr: <none>
                param_spans:
                    [9-13]
                    [15-18]
                    [20-25]
                    [27-30]
                    [32-36]
                return_ty_span: [0-0]"#]],
    );
}

#[test]
fn return_type() {
    check_stmt_kind(
        "extern f() -> int;",
        &expect![[r#"
            ExternDecl [0-18]:
                symbol_id: 8
                default_value_expr: Expr [14-17]:
                    ty: const int
                    kind: Lit: Int(0)
                param_spans: <empty>
                return_ty_span: [14-17]"#]],
    );
}

#[test]
fn return_type_can_be_duration() {
    check_stmt_kind(
        "extern f() -> duration;",
        &expect![[r#"
            ExternDecl [0-23]:
                symbol_id: 8
                default_value_expr: Expr [14-22]:
                    ty: duration
                    kind: Lit: Duration(0.0 s)
                param_spans: <empty>
                return_ty_span: [14-22]"#]],
    );
}

#[test]
fn return_type_cannot_be_stretch() {
    check_stmt_kind(
        "extern f() -> stretch;",
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [0-22]:
                        annotations: <empty>
                        kind: ExternDecl [0-22]:
                            symbol_id: 8
                            default_value_expr: Expr [14-21]:
                                ty: stretch
                                kind: Lit: Duration(0.0 s)
                            param_spans: <empty>
                            return_ty_span: [14-21]

            [Qasm.Lowerer.ExternDeclarationCannotReturnStretch

              x extern declarations cannot return stretches
               ,-[test:1:15]
             1 | extern f() -> stretch;
               :               ^^^^^^^
               `----
            ]"#]],
    );
}

#[test]
fn not_allowed_in_non_global_scope() {
    check_stmt_kind(
        "{ extern f(); }",
        &expect![[r#"
            Program:
                version: <none>
                pragmas: <empty>
                statements:
                    Stmt [0-15]:
                        annotations: <empty>
                        kind: Block [0-15]:
                            Stmt [2-13]:
                                annotations: <empty>
                                kind: ExternDecl [2-13]:
                                    symbol_id: 8
                                    default_value_expr: <none>
                                    param_spans: <empty>
                                    return_ty_span: [0-0]

            [Qasm.Lowerer.ExternDeclarationInNonGlobalScope

              x extern declarations must be done in global scope
               ,-[test:1:3]
             1 | { extern f(); }
               :   ^^^^^^^^^^^
               `----
            ]"#]],
    );
}
