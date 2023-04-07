// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{block, stmt};
use crate::parse::tests::check;
use expect_test::expect;

#[test]
fn let_stmt() {
    check(
        stmt,
        "let x = 2;",
        &expect![[r#"
            Stmt 4294967295 [0-10]: Local (Immutable):
                Pat 4294967295 [4-5]: Bind:
                    Ident 4294967295 [4-5] "x"
                Expr 4294967295 [8-9]: Lit: Int(2)"#]],
    );
}

#[test]
fn let_pat_match() {
    check(
        stmt,
        "let (x, (y, z)) = foo;",
        &expect![[r#"
            Stmt 4294967295 [0-22]: Local (Immutable):
                Pat 4294967295 [4-15]: Tuple:
                    Pat 4294967295 [5-6]: Bind:
                        Ident 4294967295 [5-6] "x"
                    Pat 4294967295 [8-14]: Tuple:
                        Pat 4294967295 [9-10]: Bind:
                            Ident 4294967295 [9-10] "y"
                        Pat 4294967295 [12-13]: Bind:
                            Ident 4294967295 [12-13] "z"
                Expr 4294967295 [18-21]: Path: Path 4294967295 [18-21] (Ident 4294967295 [18-21] "foo")"#]],
    );
}

#[test]
fn mutable_stmt() {
    check(
        stmt,
        "mutable x = 2;",
        &expect![[r#"
            Stmt 4294967295 [0-14]: Local (Mutable):
                Pat 4294967295 [8-9]: Bind:
                    Ident 4294967295 [8-9] "x"
                Expr 4294967295 [12-13]: Lit: Int(2)"#]],
    );
}

#[test]
fn use_stmt() {
    check(
        stmt,
        "use q = Qubit();",
        &expect![[r#"
            Stmt 4294967295 [0-16]: Qubit (Fresh)
                Pat 4294967295 [4-5]: Bind:
                    Ident 4294967295 [4-5] "q"
                QubitInit 4294967295 [8-15] Single"#]],
    );
}

#[test]
fn use_qubit_array() {
    check(
        stmt,
        "use qs = Qubit[5];",
        &expect![[r#"
            Stmt 4294967295 [0-18]: Qubit (Fresh)
                Pat 4294967295 [4-6]: Bind:
                    Ident 4294967295 [4-6] "qs"
                QubitInit 4294967295 [9-17] Array:
                    Expr 4294967295 [15-16]: Lit: Int(5)"#]],
    );
}

#[test]
fn use_pat_match() {
    check(
        stmt,
        "use (q1, q2) = (Qubit(), Qubit());",
        &expect![[r#"
            Stmt 4294967295 [0-34]: Qubit (Fresh)
                Pat 4294967295 [4-12]: Tuple:
                    Pat 4294967295 [5-7]: Bind:
                        Ident 4294967295 [5-7] "q1"
                    Pat 4294967295 [9-11]: Bind:
                        Ident 4294967295 [9-11] "q2"
                QubitInit 4294967295 [15-33] Tuple:
                    QubitInit 4294967295 [16-23] Single
                    QubitInit 4294967295 [25-32] Single"#]],
    );
}

#[test]
fn use_paren() {
    check(
        stmt,
        "use q = (Qubit());",
        &expect![[r#"
            Stmt 4294967295 [0-18]: Qubit (Fresh)
                Pat 4294967295 [4-5]: Bind:
                    Ident 4294967295 [4-5] "q"
                QubitInit 4294967295 [8-17] Parens:
                    QubitInit 4294967295 [9-16] Single"#]],
    );
}

#[test]
fn use_single_tuple() {
    check(
        stmt,
        "use (q,) = (Qubit(),);",
        &expect![[r#"
            Stmt 4294967295 [0-22]: Qubit (Fresh)
                Pat 4294967295 [4-8]: Tuple:
                    Pat 4294967295 [5-6]: Bind:
                        Ident 4294967295 [5-6] "q"
                QubitInit 4294967295 [11-21] Tuple:
                    QubitInit 4294967295 [12-19] Single"#]],
    );
}

#[test]
fn borrow_stmt() {
    check(
        stmt,
        "borrow q = Qubit();",
        &expect![[r#"
            Stmt 4294967295 [0-19]: Qubit (Dirty)
                Pat 4294967295 [7-8]: Bind:
                    Ident 4294967295 [7-8] "q"
                QubitInit 4294967295 [11-18] Single"#]],
    );
}

#[test]
fn let_in_block() {
    check(
        block,
        "{ let x = 2; x }",
        &expect![[r#"
            Block 4294967295 [0-16]:
                Stmt 4294967295 [2-12]: Local (Immutable):
                    Pat 4294967295 [6-7]: Bind:
                        Ident 4294967295 [6-7] "x"
                    Expr 4294967295 [10-11]: Lit: Int(2)
                Stmt 4294967295 [13-14]: Expr: Expr 4294967295 [13-14]: Path: Path 4294967295 [13-14] (Ident 4294967295 [13-14] "x")"#]],
    );
}

#[test]
fn exprs_in_block() {
    check(
        block,
        "{ x; y; z }",
        &expect![[r#"
            Block 4294967295 [0-11]:
                Stmt 4294967295 [2-4]: Semi: Expr 4294967295 [2-3]: Path: Path 4294967295 [2-3] (Ident 4294967295 [2-3] "x")
                Stmt 4294967295 [5-7]: Semi: Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")
                Stmt 4294967295 [8-9]: Expr: Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "z")"#]],
    );
}

#[test]
fn trailing_semi_expr() {
    check(
        block,
        "{ x; y; z; }",
        &expect![[r#"
            Block 4294967295 [0-12]:
                Stmt 4294967295 [2-4]: Semi: Expr 4294967295 [2-3]: Path: Path 4294967295 [2-3] (Ident 4294967295 [2-3] "x")
                Stmt 4294967295 [5-7]: Semi: Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "y")
                Stmt 4294967295 [8-10]: Semi: Expr 4294967295 [8-9]: Path: Path 4294967295 [8-9] (Ident 4294967295 [8-9] "z")"#]],
    );
}

#[test]
fn stmt_missing_semi() {
    check(
        stmt,
        "let x = 2",
        &expect![[r#"
            Err(
                Token(
                    Semi,
                    Eof,
                    Span {
                        lo: 9,
                        hi: 9,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn if_followed_by() {
    check(
        block,
        "{ if c { x } return x; }",
        &expect![[r#"
            Block 4294967295 [0-24]:
                Stmt 4294967295 [2-12]: Expr: Expr 4294967295 [2-12]: If:
                    Expr 4294967295 [5-6]: Path: Path 4294967295 [5-6] (Ident 4294967295 [5-6] "c")
                    Block 4294967295 [7-12]:
                        Stmt 4294967295 [9-10]: Expr: Expr 4294967295 [9-10]: Path: Path 4294967295 [9-10] (Ident 4294967295 [9-10] "x")
                Stmt 4294967295 [13-22]: Semi: Expr 4294967295 [13-21]: Return: Expr 4294967295 [20-21]: Path: Path 4294967295 [20-21] (Ident 4294967295 [20-21] "x")"#]],
    );
}

#[test]
fn let_if() {
    check(
        block,
        "{ let x = if c { true } else { false }; x }",
        &expect![[r#"
            Block 4294967295 [0-43]:
                Stmt 4294967295 [2-39]: Local (Immutable):
                    Pat 4294967295 [6-7]: Bind:
                        Ident 4294967295 [6-7] "x"
                    Expr 4294967295 [10-38]: If:
                        Expr 4294967295 [13-14]: Path: Path 4294967295 [13-14] (Ident 4294967295 [13-14] "c")
                        Block 4294967295 [15-23]:
                            Stmt 4294967295 [17-21]: Expr: Expr 4294967295 [17-21]: Lit: Bool(true)
                        Expr 4294967295 [24-38]: Expr Block: Block 4294967295 [29-38]:
                            Stmt 4294967295 [31-36]: Expr: Expr 4294967295 [31-36]: Lit: Bool(false)
                Stmt 4294967295 [40-41]: Expr: Expr 4294967295 [40-41]: Path: Path 4294967295 [40-41] (Ident 4294967295 [40-41] "x")"#]],
    );
}
