// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{ident, opt, pat, path, seq};
use crate::{
    lex::{ClosedBinOp, TokenKind},
    parse::{
        scan::Scanner,
        tests::{check, check_opt, check_seq},
        Error, Keyword,
    },
};
use expect_test::expect;
use qsc_ast::ast::Span;

#[test]
fn ident_basic() {
    check(ident, "foo", &expect![[r#"Ident 4294967295 [0-3] "foo""#]]);
}

#[test]
fn ident_num_suffix() {
    check(
        ident,
        "foo2",
        &expect![[r#"Ident 4294967295 [0-4] "foo2""#]],
    );
}

#[test]
fn ident_underscore_prefix() {
    check(
        ident,
        "_foo",
        &expect![[r#"Ident 4294967295 [0-4] "_foo""#]],
    );
}

#[test]
fn ident_num_prefix() {
    check(
        ident,
        "2foo",
        &expect![[r#"
            Err(
                Rule(
                    "identifier",
                    Int(
                        Decimal,
                    ),
                    Span {
                        lo: 0,
                        hi: 1,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn ident_keyword() {
    for keyword in enum_iterator::all::<Keyword>() {
        let mut scanner = Scanner::new(keyword.as_str());
        let actual = ident(&mut scanner);
        let span = Span {
            lo: 0,
            hi: keyword.as_str().len(),
        };

        let expected = match keyword {
            Keyword::And => {
                Error::Rule("identifier", TokenKind::ClosedBinOp(ClosedBinOp::And), span)
            }
            Keyword::Or => Error::Rule("identifier", TokenKind::ClosedBinOp(ClosedBinOp::Or), span),
            _ => Error::RuleKeyword("identifier", keyword, span),
        };

        assert_eq!(actual, Err(expected), "{keyword}");
    }
}

#[test]
fn path_single() {
    check(
        path,
        "Foo",
        &expect![[r#"Path 4294967295 [0-3] (Ident 4294967295 [0-3] "Foo")"#]],
    );
}

#[test]
fn path_double() {
    check(
        path,
        "Foo.Bar",
        &expect![[
            r#"Path 4294967295 [0-7] (Ident 4294967295 [0-3] "Foo") (Ident 4294967295 [4-7] "Bar")"#
        ]],
    );
}

#[test]
fn path_triple() {
    check(
        path,
        "Foo.Bar.Baz",
        &expect![[
            r#"Path 4294967295 [0-11] (Ident 4294967295 [0-7] "Foo.Bar") (Ident 4294967295 [8-11] "Baz")"#
        ]],
    );
}

#[test]
fn path_trailing_dot() {
    check(
        path,
        "Foo.Bar.",
        &expect![[r#"
            Err(
                Rule(
                    "identifier",
                    Eof,
                    Span {
                        lo: 8,
                        hi: 8,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn pat_bind() {
    check(
        pat,
        "foo",
        &expect![[r#"
            Pat 4294967295 [0-3]: Bind:
                Ident 4294967295 [0-3] "foo""#]],
    );
}

#[test]
fn pat_bind_ty() {
    check(
        pat,
        "foo : Int",
        &expect![[r#"
            Pat 4294967295 [0-9]: Bind:
                Ident 4294967295 [0-3] "foo"
                Type 4294967295 [6-9]: Prim (Int)"#]],
    );
}

#[test]
fn pat_bind_discard() {
    check(pat, "_", &expect!["Pat 4294967295 [0-1]: Discard"]);
}

#[test]
fn pat_discard_ty() {
    check(
        pat,
        "_ : Int",
        &expect![[r#"
            Pat 4294967295 [0-7]: Discard:
                Type 4294967295 [4-7]: Prim (Int)"#]],
    );
}

#[test]
fn pat_paren() {
    check(
        pat,
        "(foo)",
        &expect![[r#"
            Pat 4294967295 [0-5]: Paren:
                Pat 4294967295 [1-4]: Bind:
                    Ident 4294967295 [1-4] "foo""#]],
    );
}

#[test]
fn pat_singleton_tuple() {
    check(
        pat,
        "(foo,)",
        &expect![[r#"
            Pat 4294967295 [0-6]: Tuple:
                Pat 4294967295 [1-4]: Bind:
                    Ident 4294967295 [1-4] "foo""#]],
    );
}

#[test]
fn pat_tuple() {
    check(
        pat,
        "(foo, bar)",
        &expect![[r#"
            Pat 4294967295 [0-10]: Tuple:
                Pat 4294967295 [1-4]: Bind:
                    Ident 4294967295 [1-4] "foo"
                Pat 4294967295 [6-9]: Bind:
                    Ident 4294967295 [6-9] "bar""#]],
    );
}

#[test]
fn pat_tuple_ty_discard() {
    check(
        pat,
        "(foo : Int, _)",
        &expect![[r#"
            Pat 4294967295 [0-14]: Tuple:
                Pat 4294967295 [1-10]: Bind:
                    Ident 4294967295 [1-4] "foo"
                    Type 4294967295 [7-10]: Prim (Int)
                Pat 4294967295 [12-13]: Discard"#]],
    );
}

#[test]
fn pat_invalid() {
    check(
        pat,
        "@",
        &expect![[r#"
            Err(
                Rule(
                    "pattern",
                    At,
                    Span {
                        lo: 0,
                        hi: 1,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn pat_missing_ty() {
    check(
        pat,
        "foo :",
        &expect![[r#"
            Err(
                Rule(
                    "type",
                    Eof,
                    Span {
                        lo: 5,
                        hi: 5,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn opt_succeed() {
    check_opt(
        |s| opt(s, path),
        "Foo.Bar",
        &expect![[
            r#"Path 4294967295 [0-7] (Ident 4294967295 [0-3] "Foo") (Ident 4294967295 [4-7] "Bar")"#
        ]],
    );
}

#[test]
fn opt_fail_no_consume() {
    check_opt(
        |s| opt(s, path),
        "123",
        &expect![[r#"
            Ok(
                None,
            )
        "#]],
    );
}

#[test]
fn opt_fail_consume() {
    check_opt(
        |s| opt(s, path),
        "Foo.#",
        &expect![[r#"
            Err(
                Rule(
                    "identifier",
                    Eof,
                    Span {
                        lo: 5,
                        hi: 5,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn seq_empty() {
    check_seq(|s| seq(s, ident), "", &expect!["(, Missing)"]);
}

#[test]
fn seq_single() {
    check_seq(
        |s| seq(s, ident),
        "foo",
        &expect![[r#"(Ident 4294967295 [0-3] "foo", Missing)"#]],
    );
}

#[test]
fn seq_double() {
    check_seq(
        |s| seq(s, ident),
        "foo, bar",
        &expect![[r#"
            (Ident 4294967295 [0-3] "foo",
            Ident 4294967295 [5-8] "bar", Missing)"#]],
    );
}

#[test]
fn seq_trailing() {
    check_seq(
        |s| seq(s, ident),
        "foo, bar,",
        &expect![[r#"
            (Ident 4294967295 [0-3] "foo",
            Ident 4294967295 [5-8] "bar", Present)"#]],
    );
}

#[test]
fn seq_fail_no_consume() {
    check_seq(
        |s| seq(s, ident),
        "foo, 2",
        &expect![[r#"(Ident 4294967295 [0-3] "foo", Present)"#]],
    );
}

#[test]
fn seq_fail_consume() {
    check_seq(
        |s| seq(s, path),
        "foo, bar.",
        &expect![[r#"
            Err(
                Rule(
                    "identifier",
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
