// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::expect;

use crate::parser::tests::check;

use crate::parser::stmt::parse;

#[test]
fn pragma_decl() {
    check(
        parse,
        "pragma a.b.d 23",
        &expect![[r#"
            Stmt [0-15]
                StmtKind: Pragma [0-15]: (a.b.d, 23)"#]],
    );
}

#[test]
fn pragma_decl_ident_only() {
    check(
        parse,
        "pragma a.b.d",
        &expect![[r#"
            Stmt [0-12]
                StmtKind: Pragma [0-12]: (a.b.d)"#]],
    );
}

#[test]
fn pragma_decl_missing_ident() {
    check(
        parse,
        "pragma ",
        &expect![[r#"
            Stmt [0-7]
                StmtKind: Pragma [0-7]: ()

            [
                Error(
                    Rule(
                        "pragma missing identifier",
                        Pragma,
                        Span {
                            lo: 0,
                            hi: 7,
                        },
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn legacy_pragma_decl() {
    check(
        parse,
        "#pragma a.b.d 23",
        &expect![[r#"
            Stmt [0-16]
                StmtKind: Pragma [0-16]: (a, a.b.d 23)"#]],
    );
}

#[test]
fn legacy_pragma_decl_ident_only() {
    check(
        parse,
        "#pragma a.b.d",
        &expect![[r#"
            Stmt [0-13]
                StmtKind: Pragma [0-13]: (a, a.b.d)"#]],
    );
}

#[test]
fn legacy_pragma_ws_after_hash() {
    check(
        parse,
        "# pragma a.b.d",
        &expect![[r#"
            Stmt [2-14]
                StmtKind: Pragma [2-14]: (a.b.d)

            [
                Error(
                    Lex(
                        Incomplete(
                            Ident,
                            Identifier,
                            Whitespace,
                            Span {
                                lo: 1,
                                hi: 2,
                            },
                        ),
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn legacy_pragma_decl_missing_ident() {
    check(
        parse,
        "#pragma ",
        &expect![[r#"
            Stmt [0-8]
                StmtKind: Pragma [0-8]: (a, )"#]],
    );
}
