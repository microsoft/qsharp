// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{ident, opt, pat, path, seq};
use crate::{
    keyword::Keyword,
    lex::{ClosedBinOp, TokenKind},
    scan::Scanner,
    tests::{check, check_opt, check_seq},
    Error, ErrorKind,
};
use expect_test::expect;
use qsc_data_structures::span::Span;

#[test]
fn ident_basic() {
    check(ident, "foo", &expect![[r#"Ident _id_ [0-3] "foo""#]]);
}

#[test]
fn ident_num_suffix() {
    check(ident, "foo2", &expect![[r#"Ident _id_ [0-4] "foo2""#]]);
}

#[test]
fn ident_underscore_prefix() {
    check(ident, "_foo", &expect![[r#"Ident _id_ [0-4] "_foo""#]]);
}

#[test]
fn ident_num_prefix() {
    check(
        ident,
        "2foo",
        &expect![[r#"
            Error(
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
            hi: keyword
                .as_str()
                .len()
                .try_into()
                .expect("keyword length should fit into u32"),
        };

        let expected = Error(match keyword {
            Keyword::And => {
                ErrorKind::Rule("identifier", TokenKind::ClosedBinOp(ClosedBinOp::And), span)
            }
            Keyword::Or => {
                ErrorKind::Rule("identifier", TokenKind::ClosedBinOp(ClosedBinOp::Or), span)
            }
            _ => ErrorKind::Rule("identifier", TokenKind::Keyword(keyword), span),
        });

        assert_eq!(actual, Err(expected), "{keyword}");
    }
}

#[test]
fn path_single() {
    check(
        path,
        "Foo",
        &expect![[r#"Path _id_ [0-3] (Ident _id_ [0-3] "Foo")"#]],
    );
}

#[test]
fn path_double() {
    check(
        path,
        "Foo.Bar",
        &expect![[r#"Path _id_ [0-7] (Ident _id_ [0-3] "Foo") (Ident _id_ [4-7] "Bar")"#]],
    );
}

#[test]
fn path_triple() {
    check(
        path,
        "Foo.Bar.Baz",
        &expect![[r#"Path _id_ [0-11] (Ident _id_ [0-7] "Foo.Bar") (Ident _id_ [8-11] "Baz")"#]],
    );
}

#[test]
fn path_trailing_dot() {
    check(
        path,
        "Foo.Bar.",
        &expect![[r#"
            Error(
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
            Pat _id_ [0-3]: Bind:
                Ident _id_ [0-3] "foo""#]],
    );
}

#[test]
fn pat_bind_ty() {
    check(
        pat,
        "foo : Int",
        &expect![[r#"
            Pat _id_ [0-9]: Bind:
                Ident _id_ [0-3] "foo"
                Type _id_ [6-9]: Path: Path _id_ [6-9] (Ident _id_ [6-9] "Int")"#]],
    );
}

#[test]
fn pat_bind_discard() {
    check(pat, "_", &expect!["Pat _id_ [0-1]: Discard"]);
}

#[test]
fn pat_discard_ty() {
    check(
        pat,
        "_ : Int",
        &expect![[r#"
            Pat _id_ [0-7]: Discard:
                Type _id_ [4-7]: Path: Path _id_ [4-7] (Ident _id_ [4-7] "Int")"#]],
    );
}

#[test]
fn pat_paren() {
    check(
        pat,
        "(foo)",
        &expect![[r#"
            Pat _id_ [0-5]: Paren:
                Pat _id_ [1-4]: Bind:
                    Ident _id_ [1-4] "foo""#]],
    );
}

#[test]
fn pat_singleton_tuple() {
    check(
        pat,
        "(foo,)",
        &expect![[r#"
            Pat _id_ [0-6]: Tuple:
                Pat _id_ [1-4]: Bind:
                    Ident _id_ [1-4] "foo""#]],
    );
}

#[test]
fn pat_tuple() {
    check(
        pat,
        "(foo, bar)",
        &expect![[r#"
            Pat _id_ [0-10]: Tuple:
                Pat _id_ [1-4]: Bind:
                    Ident _id_ [1-4] "foo"
                Pat _id_ [6-9]: Bind:
                    Ident _id_ [6-9] "bar""#]],
    );
}

#[test]
fn pat_tuple_ty_discard() {
    check(
        pat,
        "(foo : Int, _)",
        &expect![[r#"
            Pat _id_ [0-14]: Tuple:
                Pat _id_ [1-10]: Bind:
                    Ident _id_ [1-4] "foo"
                    Type _id_ [7-10]: Path: Path _id_ [7-10] (Ident _id_ [7-10] "Int")
                Pat _id_ [12-13]: Discard"#]],
    );
}

#[test]
fn pat_invalid() {
    check(
        pat,
        "@",
        &expect![[r#"
            Error(
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
            Error(
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
        &expect![[r#"Path _id_ [0-7] (Ident _id_ [0-3] "Foo") (Ident _id_ [4-7] "Bar")"#]],
    );
}

#[test]
fn opt_fail_no_consume() {
    check_opt(|s| opt(s, path), "123", &expect!["None"]);
}

#[test]
fn opt_fail_consume() {
    check_opt(
        |s| opt(s, path),
        "Foo.#",
        &expect![[r#"
            Error(
                Rule(
                    "identifier",
                    Eof,
                    Span {
                        lo: 5,
                        hi: 5,
                    },
                ),
            )

            [
                Error(
                    Lex(
                        Unknown(
                            '#',
                            Span {
                                lo: 4,
                                hi: 5,
                            },
                        ),
                    ),
                ),
            ]"#]],
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
        &expect![[r#"(Ident _id_ [0-3] "foo", Missing)"#]],
    );
}

#[test]
fn seq_double() {
    check_seq(
        |s| seq(s, ident),
        "foo, bar",
        &expect![[r#"
            (Ident _id_ [0-3] "foo",
            Ident _id_ [5-8] "bar", Missing)"#]],
    );
}

#[test]
fn seq_trailing() {
    check_seq(
        |s| seq(s, ident),
        "foo, bar,",
        &expect![[r#"
            (Ident _id_ [0-3] "foo",
            Ident _id_ [5-8] "bar", Present)"#]],
    );
}

#[test]
fn seq_fail_no_consume() {
    check_seq(
        |s| seq(s, ident),
        "foo, 2",
        &expect![[r#"(Ident _id_ [0-3] "foo", Present)"#]],
    );
}

#[test]
fn seq_fail_consume() {
    check_seq(
        |s| seq(s, path),
        "foo, bar.",
        &expect![[r#"
            Error(
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
