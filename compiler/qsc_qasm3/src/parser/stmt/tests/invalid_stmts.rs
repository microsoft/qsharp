use crate::parser::{stmt::parse, tests::check};
use expect_test::expect;

#[test]
fn assignment_in_if_condition() {
    check(
        parse,
        "if (x = 2) { 3; }",
        &expect![[r#"
            Stmt [0-17]:
                annotations: <empty>
                kind: IfStmt [0-17]:
                    condition: Expr [4-5]: Ident [4-5] "x"
                    if_block:
                        Stmt [13-15]:
                            annotations: <empty>
                            kind: ExprStmt [13-15]:
                                expr: Expr [13-14]: Lit: Int(3)
                    else_block: <none>

            [
                Error(
                    Token(
                        Close(
                            Paren,
                        ),
                        Eq,
                        Span {
                            lo: 6,
                            hi: 7,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn binary_op_assignment_in_if_condition() {
    check(
        parse,
        "if (x += 2) { 3; }",
        &expect![[r#"
            Stmt [0-18]:
                annotations: <empty>
                kind: IfStmt [0-18]:
                    condition: Expr [4-5]: Ident [4-5] "x"
                    if_block:
                        Stmt [14-16]:
                            annotations: <empty>
                            kind: ExprStmt [14-16]:
                                expr: Expr [14-15]: Lit: Int(3)
                    else_block: <none>

            [
                Error(
                    Token(
                        Close(
                            Paren,
                        ),
                        BinOpEq(
                            Plus,
                        ),
                        Span {
                            lo: 6,
                            hi: 8,
                        },
                    ),
                ),
            ]"#]],
    );
}
