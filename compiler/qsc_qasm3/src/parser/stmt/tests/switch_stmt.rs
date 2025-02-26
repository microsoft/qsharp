use crate::parser::{stmt::parse_switch_stmt, tests::check};
use expect_test::expect;

#[test]
fn simple_switch() {
    check(
        parse_switch_stmt,
        "
        switch (x) {
          case 1 {}
          default {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-72]:
                Target: Expr [9-19]: Paren:
                    Expr [17-18]: Ident [17-18] "x"
                Cases:
                    Labels:
                        Expr [37-38]: Lit: Int(1)
                    Block [39-41]: <empty>
                    Default Case:
                    Block [60-62]: <empty>"#]],
    );
}

#[test]
fn no_cases_no_default() {
    check(
        parse_switch_stmt,
        "
        switch (x) {}
    ",
        &expect![[r#"
            SwitchStmt [9-22]:
                Target: Expr [9-19]: Paren:
                    Expr [17-18]: Ident [17-18] "x"
                <no cases>
                <no default>

            [
                Error(
                    MissingSwitchCases(
                        Span {
                            lo: 21,
                            hi: 21,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn no_cases() {
    check(
        parse_switch_stmt,
        "
        switch (x) {
          default {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-52]:
                Target: Expr [9-19]: Paren:
                    Expr [17-18]: Ident [17-18] "x"
                <no cases>
                Default Case:
                    Block [40-42]: <empty>

            [
                Error(
                    MissingSwitchCases(
                        Span {
                            lo: 32,
                            hi: 21,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn no_default() {
    check(
        parse_switch_stmt,
        "
        switch (x) {
          case 0, 1 {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-54]:
                Target: Expr [9-19]: Paren:
                    Expr [17-18]: Ident [17-18] "x"
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
        parse_switch_stmt,
        "
        switch (x) {
          case {}
        }
    ",
        &expect![[r#"
            SwitchStmt [9-49]:
                Target: Expr [9-19]: Paren:
                    Expr [17-18]: Ident [17-18] "x"
                Cases:
                    <no labels>
                    Block [37-39]: <empty>
                    <no default>

            [
                Error(
                    MissingSwitchCaseLabels(
                        Span {
                            lo: 32,
                            hi: 36,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn multiple_cases() {
    check(
        parse_switch_stmt,
        "
        switch (x) {
          case 0 { int x = 0; }
          case 1 { int y = 1; }
        }
    ",
        &expect![[r#"
            SwitchStmt [9-95]:
                Target: Expr [9-19]: Paren:
                    Expr [17-18]: Ident [17-18] "x"
                Cases:
                    Labels:
                        Expr [37-38]: Lit: Int(0)
                    Block [39-53]:
                        Stmt [41-51]
                            StmtKind: ClassicalDeclarationStmt [41-51]: ClassicalType [41-44]: IntType [41-44], Ident [45-46] "x", ValueExpression ExprStmt [49-50]: Expr [49-50]: Lit: Int(0)
                    Labels:
                        Expr [69-70]: Lit: Int(1)
                    Block [71-85]:
                        Stmt [73-83]
                            StmtKind: ClassicalDeclarationStmt [73-83]: ClassicalType [73-76]: IntType [73-76], Ident [77-78] "y", ValueExpression ExprStmt [81-82]: Expr [81-82]: Lit: Int(1)
                    <no default>"#]],
    );
}
