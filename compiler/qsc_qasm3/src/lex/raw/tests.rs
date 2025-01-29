// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Lexer;
use crate::lex::raw::{Single, Token, TokenKind};
use expect_test::{expect, Expect};

fn check(input: &str, expect: &Expect) {
    let actual: Vec<_> = Lexer::new(input).collect();
    expect.assert_debug_eq(&actual);
}

#[test]
fn singles() {
    for single in enum_iterator::all::<Single>() {
        let actual: Vec<_> = Lexer::new(&single.to_string()).collect();
        let kind = TokenKind::Single(single);
        assert_eq!(actual, vec![Token { kind, offset: 0 }]);
    }
}

#[test]
fn braces() {
    check(
        "{}",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Open(
                            Brace,
                        ),
                    ),
                    offset: 0,
                },
                Token {
                    kind: Single(
                        Close(
                            Brace,
                        ),
                    ),
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn negate() {
    check(
        "-x",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Minus,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Ident,
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn whitespace() {
    check(
        "-   x",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Minus,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Whitespace,
                    offset: 1,
                },
                Token {
                    kind: Ident,
                    offset: 4,
                },
            ]
        "#]],
    );
}

#[test]
fn comment() {
    check(
        "//comment\nx",
        &expect![[r#"
            [
                Token {
                    kind: Comment(
                        Normal,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Whitespace,
                    offset: 9,
                },
                Token {
                    kind: Ident,
                    offset: 10,
                },
            ]
        "#]],
    );
}

#[test]
fn block_comment() {
    check(
        "/* comment\n x */",
        &expect![[r#"
            [
                Token {
                    kind: Comment(
                        Block,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn comment_four_slashes() {
    check(
        "////comment\nx",
        &expect![[r#"
            [
                Token {
                    kind: Comment(
                        Normal,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Whitespace,
                    offset: 11,
                },
                Token {
                    kind: Ident,
                    offset: 12,
                },
            ]
        "#]],
    );
}

#[test]
fn string() {
    check(
        r#""string""#,
        &expect![[r#"
            [
                Token {
                    kind: String(
                        StringToken {
                            terminated: true,
                        },
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn string_escape_quote() {
    check(
        r#""str\"ing""#,
        &expect![[r#"
            [
                Token {
                    kind: String(
                        StringToken {
                            terminated: true,
                        },
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn string_missing_ending() {
    check(
        r#""string"#,
        &expect![[r#"
            [
                Token {
                    kind: String(
                        StringToken {
                            terminated: false,
                        },
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn binary() {
    check(
        "0b10110",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Binary,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
    check(
        "0B10110",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Binary,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn octal() {
    check(
        "0o70351",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Octal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
    check(
        "0O70351",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Octal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn decimal() {
    check(
        "123",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn number_seps() {
    check(
        "123_456",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn number_underscore_prefix() {
    check(
        "_123_456",
        &expect![[r#"
            [
                Token {
                    kind: Ident,
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn int_dot_dot() {
    check(
        "0..",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 0,
                },
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 1,
                },
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 2,
                },
            ]
        "#]],
    );
}

#[test]
fn dot_dot_int() {
    check(
        "..0",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 1,
                },
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 2,
                },
            ]
        "#]],
    );
}

#[test]
fn dot_dot_dot_int() {
    check(
        "...0",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 1,
                },
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 2,
                },
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 3,
                },
            ]
        "#]],
    );
}

#[test]
fn hexadecimal() {
    check(
        "0x123abc",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Hexadecimal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
    check(
        "0X123abc",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Hexadecimal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn negative() {
    check(
        "-4",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Minus,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn positive() {
    check(
        "+4",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Plus,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn float() {
    check(
        "1.23",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn leading_zero() {
    check(
        "0123",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn leading_point() {
    check(
        ".123",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn trailing_point() {
    check(
        "123.",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn exp() {
    check(
        "1e23",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
    check(
        "1E23",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn exp_plus() {
    check(
        "1e+23",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn exp_minus() {
    check(
        "1e-23",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn leading_point_exp() {
    check(
        ".25e2",
        &expect![[r#"
            [
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 0,
                },
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn leading_zero_point() {
    check(
        "0.25",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn leading_zero_zero_point() {
    check(
        "00.25",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn leading_zero_exp() {
    check(
        "0.25e2",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Float,
                    ),
                    offset: 0,
                },
            ]
        "#]],
    );
}

#[test]
fn unknown() {
    check(
        "##",
        &expect![[r#"
            [
                Token {
                    kind: Unknown,
                    offset: 0,
                },
                Token {
                    kind: Unknown,
                    offset: 1,
                },
            ]
        "#]],
    );
}

#[test]
fn float_hexadecimal() {
    check(
        "0x123.45",
        &expect![[r#"
            [
                Token {
                    kind: Number(
                        Int(
                            Hexadecimal,
                        ),
                    ),
                    offset: 0,
                },
                Token {
                    kind: Single(
                        Dot,
                    ),
                    offset: 5,
                },
                Token {
                    kind: Number(
                        Int(
                            Decimal,
                        ),
                    ),
                    offset: 6,
                },
            ]
        "#]],
    );
}
