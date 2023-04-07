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
            Stmt _id_ [0-10]: Local (Immutable):
                Pat _id_ [4-5]: Bind:
                    Ident _id_ [4-5] "x"
                Expr _id_ [8-9]: Lit: Int(2)"#]],
    );
}

#[test]
fn let_pat_match() {
    check(
        stmt,
        "let (x, (y, z)) = foo;",
        &expect![[r#"
            Stmt _id_ [0-22]: Local (Immutable):
                Pat _id_ [4-15]: Tuple:
                    Pat _id_ [5-6]: Bind:
                        Ident _id_ [5-6] "x"
                    Pat _id_ [8-14]: Tuple:
                        Pat _id_ [9-10]: Bind:
                            Ident _id_ [9-10] "y"
                        Pat _id_ [12-13]: Bind:
                            Ident _id_ [12-13] "z"
                Expr _id_ [18-21]: Path: Path _id_ [18-21] (Ident _id_ [18-21] "foo")"#]],
    );
}

#[test]
fn mutable_stmt() {
    check(
        stmt,
        "mutable x = 2;",
        &expect![[r#"
            Stmt _id_ [0-14]: Local (Mutable):
                Pat _id_ [8-9]: Bind:
                    Ident _id_ [8-9] "x"
                Expr _id_ [12-13]: Lit: Int(2)"#]],
    );
}

#[test]
fn use_stmt() {
    check(
        stmt,
        "use q = Qubit();",
        &expect![[r#"
            Stmt _id_ [0-16]: Qubit (Fresh)
                Pat _id_ [4-5]: Bind:
                    Ident _id_ [4-5] "q"
                QubitInit _id_ [8-15] Single"#]],
    );
}

#[test]
fn use_qubit_array() {
    check(
        stmt,
        "use qs = Qubit[5];",
        &expect![[r#"
            Stmt _id_ [0-18]: Qubit (Fresh)
                Pat _id_ [4-6]: Bind:
                    Ident _id_ [4-6] "qs"
                QubitInit _id_ [9-17] Array:
                    Expr _id_ [15-16]: Lit: Int(5)"#]],
    );
}

#[test]
fn use_pat_match() {
    check(
        stmt,
        "use (q1, q2) = (Qubit(), Qubit());",
        &expect![[r#"
            Stmt _id_ [0-34]: Qubit (Fresh)
                Pat _id_ [4-12]: Tuple:
                    Pat _id_ [5-7]: Bind:
                        Ident _id_ [5-7] "q1"
                    Pat _id_ [9-11]: Bind:
                        Ident _id_ [9-11] "q2"
                QubitInit _id_ [15-33] Tuple:
                    QubitInit _id_ [16-23] Single
                    QubitInit _id_ [25-32] Single"#]],
    );
}

#[test]
fn use_paren() {
    check(
        stmt,
        "use q = (Qubit());",
        &expect![[r#"
            Stmt _id_ [0-18]: Qubit (Fresh)
                Pat _id_ [4-5]: Bind:
                    Ident _id_ [4-5] "q"
                QubitInit _id_ [8-17] Parens:
                    QubitInit _id_ [9-16] Single"#]],
    );
}

#[test]
fn use_single_tuple() {
    check(
        stmt,
        "use (q,) = (Qubit(),);",
        &expect![[r#"
            Stmt _id_ [0-22]: Qubit (Fresh)
                Pat _id_ [4-8]: Tuple:
                    Pat _id_ [5-6]: Bind:
                        Ident _id_ [5-6] "q"
                QubitInit _id_ [11-21] Tuple:
                    QubitInit _id_ [12-19] Single"#]],
    );
}

#[test]
fn borrow_stmt() {
    check(
        stmt,
        "borrow q = Qubit();",
        &expect![[r#"
            Stmt _id_ [0-19]: Qubit (Dirty)
                Pat _id_ [7-8]: Bind:
                    Ident _id_ [7-8] "q"
                QubitInit _id_ [11-18] Single"#]],
    );
}

#[test]
fn let_in_block() {
    check(
        block,
        "{ let x = 2; x }",
        &expect![[r#"
            Block _id_ [0-16]:
                Stmt _id_ [2-12]: Local (Immutable):
                    Pat _id_ [6-7]: Bind:
                        Ident _id_ [6-7] "x"
                    Expr _id_ [10-11]: Lit: Int(2)
                Stmt _id_ [13-14]: Expr: Expr _id_ [13-14]: Path: Path _id_ [13-14] (Ident _id_ [13-14] "x")"#]],
    );
}

#[test]
fn exprs_in_block() {
    check(
        block,
        "{ x; y; z }",
        &expect![[r#"
            Block _id_ [0-11]:
                Stmt _id_ [2-4]: Semi: Expr _id_ [2-3]: Path: Path _id_ [2-3] (Ident _id_ [2-3] "x")
                Stmt _id_ [5-7]: Semi: Expr _id_ [5-6]: Path: Path _id_ [5-6] (Ident _id_ [5-6] "y")
                Stmt _id_ [8-9]: Expr: Expr _id_ [8-9]: Path: Path _id_ [8-9] (Ident _id_ [8-9] "z")"#]],
    );
}

#[test]
fn trailing_semi_expr() {
    check(
        block,
        "{ x; y; z; }",
        &expect![[r#"
            Block _id_ [0-12]:
                Stmt _id_ [2-4]: Semi: Expr _id_ [2-3]: Path: Path _id_ [2-3] (Ident _id_ [2-3] "x")
                Stmt _id_ [5-7]: Semi: Expr _id_ [5-6]: Path: Path _id_ [5-6] (Ident _id_ [5-6] "y")
                Stmt _id_ [8-10]: Semi: Expr _id_ [8-9]: Path: Path _id_ [8-9] (Ident _id_ [8-9] "z")"#]],
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
            Block _id_ [0-24]:
                Stmt _id_ [2-12]: Expr: Expr _id_ [2-12]: If:
                    Expr _id_ [5-6]: Path: Path _id_ [5-6] (Ident _id_ [5-6] "c")
                    Block _id_ [7-12]:
                        Stmt _id_ [9-10]: Expr: Expr _id_ [9-10]: Path: Path _id_ [9-10] (Ident _id_ [9-10] "x")
                Stmt _id_ [13-22]: Semi: Expr _id_ [13-21]: Return: Expr _id_ [20-21]: Path: Path _id_ [20-21] (Ident _id_ [20-21] "x")"#]],
    );
}

#[test]
fn let_if() {
    check(
        block,
        "{ let x = if c { true } else { false }; x }",
        &expect![[r#"
            Block _id_ [0-43]:
                Stmt _id_ [2-39]: Local (Immutable):
                    Pat _id_ [6-7]: Bind:
                        Ident _id_ [6-7] "x"
                    Expr _id_ [10-38]: If:
                        Expr _id_ [13-14]: Path: Path _id_ [13-14] (Ident _id_ [13-14] "c")
                        Block _id_ [15-23]:
                            Stmt _id_ [17-21]: Expr: Expr _id_ [17-21]: Lit: Bool(true)
                        Expr _id_ [24-38]: Expr Block: Block _id_ [29-38]:
                            Stmt _id_ [31-36]: Expr: Expr _id_ [31-36]: Lit: Bool(false)
                Stmt _id_ [40-41]: Expr: Expr _id_ [40-41]: Path: Path _id_ [40-41] (Ident _id_ [40-41] "x")"#]],
    );
}
