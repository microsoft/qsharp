use expect_test::expect;

use crate::parser::{stmt::switch_stmt, tests::check};

#[test]
fn simple_switch() {
    check(
        switch_stmt,
        "
        switch (x) {
          case 1 {}
          default {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-72]:
                Cases:
                    Labels:
                    Expr [37-38]: Lit: Int(1)Block [39-41]: <empty>
                Default Case:
                    Block [60-62]: <empty>"#]],
    );
}

#[test]
fn no_cases_no_default() {
    check(
        switch_stmt,
        "
        switch (x) {}
    ",
        &expect![[r#"
            SwitchStmt [9-22]:
                <no cases>
                <no default>"#]],
    );
}

#[test]
fn no_cases() {
    check(
        switch_stmt,
        "
        switch (x) {
          default {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-52]:
                <no cases>
                Default Case:
                    Block [40-42]: <empty>"#]],
    );
}

#[test]
fn no_default() {
    check(
        switch_stmt,
        "
        switch (x) {
          case 0, 1 {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-54]:
                Cases:
                    Labels:
                        Expr [37-38]: Lit: Int(0)
                        Expr [40-41]: Lit: Int(1)
                    Block [42-44]: <empty>
                    <no default>"#]],
    );
}

#[test]
fn case_with_no_labels() {
    check(
        switch_stmt,
        "
        switch (x) {
          case {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-49]:
                Cases:
                    <no labels>
                    Block [37-39]: <empty>
                    <no default>"#]],
    );
}

#[test]
fn multiple_cases() {
    check(
        switch_stmt,
        "
        switch (x) {
          case 0 { x = 0; }
          case 1 { y = 1; }
        }
    ",
        &expect![[r#"
            SwitchStmt [9-87]:
                Cases:
                    Labels:
                        Expr [37-38]: Lit: Int(0)
                    Block [39-49]: <empty>
                    Labels:
                        Expr [65-66]: Lit: Int(1)
                    Block [67-77]: <empty>
                    <no default>

            [
                Error(
                    Token(
                        Close(
                            Brace,
                        ),
                        Identifier,
                        Span {
                            lo: 41,
                            hi: 42,
                        },
                    ),
                ),
                Error(
                    Token(
                        Close(
                            Brace,
                        ),
                        Identifier,
                        Span {
                            lo: 69,
                            hi: 70,
                        },
                    ),
                ),
            ]"#]],
    );
}
