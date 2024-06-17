// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::CursorAwareLexer;
use expect_test::expect;

#[test]
fn cursor_in_ident() {
    let actual: Vec<_> = CursorAwareLexer::new("hello", 1).collect();
    expect![[r"
        []
    "]]
    .assert_debug_eq(&actual);
}

#[test]
fn cursor_in_whitespace() {
    let actual: Vec<_> = CursorAwareLexer::new("hi     there", 5).collect();
    expect![[r"
        [
            Ok(
                Token {
                    kind: Ident,
                    span: Span {
                        lo: 0,
                        hi: 2,
                    },
                },
            ),
        ]
    "]]
    .assert_debug_eq(&actual);
}

#[test]
fn cursor_between_ops() {
    let actual: Vec<_> = CursorAwareLexer::new("foo()", 4).collect();
    expect![[r"
        [
            Ok(
                Token {
                    kind: Ident,
                    span: Span {
                        lo: 0,
                        hi: 3,
                    },
                },
            ),
            Ok(
                Token {
                    kind: Open(
                        Paren,
                    ),
                    span: Span {
                        lo: 3,
                        hi: 4,
                    },
                },
            ),
        ]
    "]]
    .assert_debug_eq(&actual);
}

#[test]
fn cursor_at_eof() {
    let actual: Vec<_> = CursorAwareLexer::new("(", 1).collect();
    expect![[r"
        [
            Ok(
                Token {
                    kind: Open(
                        Paren,
                    ),
                    span: Span {
                        lo: 0,
                        hi: 1,
                    },
                },
            ),
        ]
    "]]
    .assert_debug_eq(&actual);
}

#[test]
fn cursor_in_ident_eof() {
    let actual: Vec<_> = CursorAwareLexer::new("hello", 5).collect();
    expect![[r"
        []
    "]]
    .assert_debug_eq(&actual);
}
