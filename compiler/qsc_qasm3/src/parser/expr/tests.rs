// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::parser::tests::check;

use super::expr;

use expect_test::expect;

#[test]
fn lit_int() {
    check(expr, "123", &expect!["Expr [0-3]: Lit: Int(123)"]);
}

#[test]
fn lit_int_underscore() {
    check(expr, "123_456", &expect!["Expr [0-7]: Lit: Int(123456)"]);
}

#[test]
fn lit_int_leading_zero() {
    check(expr, "0123", &expect!["Expr [0-4]: Lit: Int(123)"]);
}

#[test]
fn lit_int_max() {
    check(
        expr,
        "9_223_372_036_854_775_807",
        &expect!["Expr [0-25]: Lit: Int(9223372036854775807)"],
    );
}

// NOTE: Since we need to support literals of value i64::MIN while also parsing the negative sign
// as a unary operator, we need to allow one special case of overflow that is the absolute value
// of i64::MIN. This will wrap to a negative value. See the `lit_int_min` test below.
// To check for other issues with handling i64::MIN, hexadecimal and binary literals
// of i64::MIN also need to be tested.
#[test]
fn lit_int_overflow_min() {
    check(
        expr,
        "9_223_372_036_854_775_808",
        &expect!["Expr [0-25]: Lit: Int(-9223372036854775808)"],
    );
}

#[test]
fn lit_int_overflow_min_hexadecimal() {
    check(
        expr,
        "0x8000000000000000",
        &expect!["Expr [0-18]: Lit: Int(-9223372036854775808)"],
    );
}

#[test]
fn lit_int_overflow_min_binary() {
    check(
        expr,
        "0b1000000000000000000000000000000000000000000000000000000000000000",
        &expect!["Expr [0-66]: Lit: Int(-9223372036854775808)"],
    );
}

#[test]
fn lit_int_too_big_for_i64() {
    check(
        expr,
        "9_223_372_036_854_775_809",
        &expect!["Expr [0-25]: Lit: BigInt(9223372036854775809)"],
    );
}

#[test]
fn lit_int_too_big_hexadecimal_promotes_to_bigint() {
    check(
        expr,
        "0x8000000000000001",
        &expect!["Expr [0-18]: Lit: BigInt(9223372036854775809)"],
    );
}

#[test]
fn lit_int_too_big_binary_promotes_to_bigint() {
    check(
        expr,
        "0b1000000000000000000000000000000000000000000000000000000000000001",
        &expect!["Expr [0-66]: Lit: BigInt(9223372036854775809)"],
    );
}

// NOTE: Since we need to support literals of value i64::MIN while also parsing the negative sign
// as a unary operator, we need to allow one special case of overflow that is the absolute value
// of i64::MIN. This will wrap to a negative value, and then negate of i64::MIN is i64::MIN, so
// the correct value is achieved at runtime.
#[test]
#[ignore = "Re-enable when we support unary ops"]
fn lit_int_min() {
    check(
        expr,
        "-9_223_372_036_854_775_808",
        &expect![[r#"
            Expr [0-26]: UnOp (Neg):
                Expr [1-26]: Lit: Int(-9223372036854775808)"#]],
    );
}

#[test]
fn lit_int_hexadecimal() {
    check(expr, "0x1a2b3c", &expect!["Expr [0-8]: Lit: Int(1715004)"]);
}

#[test]
fn lit_int_octal() {
    check(expr, "0o1234567", &expect!["Expr [0-9]: Lit: Int(342391)"]);
}

#[test]
fn lit_int_binary() {
    check(expr, "0b10110", &expect!["Expr [0-7]: Lit: Int(22)"]);
}

#[test]
fn lit_bigint_hexadecimal() {
    check(
        expr,
        "0x1a2b3c1a2b3c1a2b3c1a",
        &expect!["Expr [0-22]: Lit: BigInt(123579069371309093501978)"],
    );
}

#[test]
fn lit_bigint_hexadecimal_capital_x() {
    check(
        expr,
        "0X1a2b3c1a2b3c1a2b3c1a",
        &expect!["Expr [0-22]: Lit: BigInt(123579069371309093501978)"],
    );
}

#[test]
fn lit_bigint_octal() {
    check(
        expr,
        "0o1234567123456712345671234",
        &expect!["Expr [0-27]: Lit: BigInt(6167970861177743307420)"],
    );
}

#[test]
fn lit_bigint_octal_capital_o() {
    check(
        expr,
        "0O1234567123456712345671234",
        &expect!["Expr [0-27]: Lit: BigInt(6167970861177743307420)"],
    );
}

#[test]
fn lit_bigint_binary() {
    check(
        expr,
        "0b1011010110101101011010110101101011010110101101011010110101101011",
        &expect!["Expr [0-66]: Lit: BigInt(13091237729729359211)"],
    );
}

#[test]
fn lit_bigint_binary_capital_b() {
    check(
        expr,
        "0B1011010110101101011010110101101011010110101101011010110101101011",
        &expect!["Expr [0-66]: Lit: BigInt(13091237729729359211)"],
    );
}

#[test]
fn lit_float() {
    check(expr, "1.23", &expect!["Expr [0-4]: Lit: Float(1.23)"]);
}

#[test]
fn lit_float_leading_dot() {
    check(expr, ".23", &expect!["Expr [0-3]: Lit: Float(0.23)"]);
}

#[test]
fn lit_float_trailing_dot() {
    check(expr, "1.", &expect!["Expr [0-2]: Lit: Float(1.0)"]);
}

#[test]
fn lit_float_underscore() {
    check(
        expr,
        "123_456.78",
        &expect!["Expr [0-10]: Lit: Float(123456.78)"],
    );
}

#[test]
fn lit_float_leading_zero() {
    check(expr, "0.23", &expect!["Expr [0-4]: Lit: Float(0.23)"]);
}

#[test]
fn lit_float_trailing_exp_0() {
    check(
        expr,
        "0e",
        &expect![[r#"
            Error(
                Lit(
                    "floating-point",
                    Span {
                        lo: 0,
                        hi: 2,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lit_float_trailing_exp_1() {
    check(
        expr,
        "1e",
        &expect![[r#"
            Error(
                Lit(
                    "floating-point",
                    Span {
                        lo: 0,
                        hi: 2,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lit_float_trailing_dot_trailing_exp() {
    check(
        expr,
        "1.e",
        &expect![[r#"
            Error(
                Lit(
                    "floating-point",
                    Span {
                        lo: 0,
                        hi: 3,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lit_float_dot_trailing_exp() {
    check(
        expr,
        "1.2e",
        &expect![[r#"
            Error(
                Lit(
                    "floating-point",
                    Span {
                        lo: 0,
                        hi: 4,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lit_float_trailing_exp_dot() {
    check(
        expr,
        "1e.",
        &expect![[r#"
            Error(
                Lit(
                    "floating-point",
                    Span {
                        lo: 0,
                        hi: 2,
                    },
                ),
            )
        "#]],
    );
}

#[test]
#[ignore = "Re-enable when we support more than literals"]
fn lit_int_hexadecimal_dot() {
    check(
        expr,
        "0x123.45",
        &expect![[r#"
            Expr [0-6]: Field:
                Expr [0-5]: Lit: Int(291)
                Err

           [
                Error(
                    Rule(
                        "identifier",
                        Int(
                            Decimal,
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

#[test]
fn lit_string() {
    check(
        expr,
        r#""foo""#,
        &expect![[r#"Expr [0-5]: Lit: String("foo")"#]],
    );
}

#[test]
fn lit_string_single_quote() {
    check(
        expr,
        r#"'foo'"#,
        &expect![[r#"Expr [0-5]: Lit: String("foo")"#]],
    );
}

#[test]
fn lit_string_escape_quote() {
    check(
        expr,
        r#""foo\"bar""#,
        &expect![[r#"Expr [0-10]: Lit: String("foo\"bar")"#]],
    );
}

#[test]
fn lit_string_single_quote_escape_double_quote() {
    check(
        expr,
        r#"'foo\"bar'"#,
        &expect![[r#"Expr [0-10]: Lit: String("foo\"bar")"#]],
    );
}

#[test]
fn lit_string_escape_backslash() {
    check(
        expr,
        r#""\\""#,
        &expect![[r#"Expr [0-4]: Lit: String("\\")"#]],
    );
}

#[test]
fn lit_string_single_quote_escape_backslash() {
    check(
        expr,
        r#"'\\'"#,
        &expect![[r#"Expr [0-4]: Lit: String("\\")"#]],
    );
}

#[test]
fn lit_string_escape_newline() {
    check(
        expr,
        r#""\n""#,
        &expect![[r#"Expr [0-4]: Lit: String("\n")"#]],
    );
}

#[test]
fn lit_string_single_quote_escape_newline() {
    check(
        expr,
        r#"'\n'"#,
        &expect![[r#"Expr [0-4]: Lit: String("\n")"#]],
    );
}

#[test]
fn lit_string_escape_carriage_return() {
    check(
        expr,
        r#""\r""#,
        &expect![[r#"Expr [0-4]: Lit: String("\r")"#]],
    );
}

#[test]
fn lit_string_single_quote_escape_carriage_return() {
    check(
        expr,
        r#"'\r'"#,
        &expect![[r#"Expr [0-4]: Lit: String("\r")"#]],
    );
}

#[test]
fn lit_string_escape_tab() {
    check(
        expr,
        r#""\t""#,
        &expect![[r#"Expr [0-4]: Lit: String("\t")"#]],
    );
}

#[test]
fn lit_string_single_quote_escape_tab() {
    check(
        expr,
        r#"'\t'"#,
        &expect![[r#"Expr [0-4]: Lit: String("\t")"#]],
    );
}

#[test]
fn lit_string_unknown_escape() {
    check(
        expr,
        r#""\x""#,
        &expect![[r#"
            Error(
                Escape(
                    'x',
                    Span {
                        lo: 2,
                        hi: 3,
                    },
                ),
            )
        "#]],
    );
}

#[test]
fn lit_string_unmatched_quote() {
    check(
        expr,
        r#""Uh oh.."#,
        &expect![[r#"
            Error(
                Rule(
                    "expression",
                    Eof,
                    Span {
                        lo: 8,
                        hi: 8,
                    },
                ),
            )

            [
                Error(
                    Lex(
                        UnterminatedString(
                            Span {
                                lo: 0,
                                hi: 0,
                            },
                        ),
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn lit_string_empty() {
    check(expr, r#""""#, &expect![[r#"Expr [0-2]: Lit: String("")"#]]);
}

#[test]
fn lit_false() {
    check(expr, "false", &expect!["Expr [0-5]: Lit: Bool(false)"]);
}

#[test]
fn lit_true() {
    check(expr, "true", &expect!["Expr [0-4]: Lit: Bool(true)"]);
}

#[test]
fn lit_bitstring() {
    check(
        expr,
        r#""101010101""#,
        &expect![[r#"Expr [0-11]: Lit: Bitstring("101010101")"#]],
    );
}

#[test]
fn lit_bitstring_preserves_leading_zeroes() {
    check(
        expr,
        r#""00011000""#,
        &expect![[r#"Expr [0-10]: Lit: Bitstring("00011000")"#]],
    );
}

#[test]
fn lit_bitstring_separators() {
    check(
        expr,
        r#""10_10_10_101""#,
        &expect![[r#"Expr [0-14]: Lit: Bitstring("101010101")"#]],
    );
}

#[test]
fn lit_bitstring_unmatched_quote() {
    check(
        expr,
        r#""101010101"#,
        &expect![[r#"
            Error(
                Rule(
                    "expression",
                    Eof,
                    Span {
                        lo: 10,
                        hi: 10,
                    },
                ),
            )

            [
                Error(
                    Lex(
                        UnterminatedString(
                            Span {
                                lo: 0,
                                hi: 0,
                            },
                        ),
                    ),
                ),
            ]"#]],
    );
}

#[test]
fn lit_float_imag() {
    check(
        expr,
        r#"10.3im"#,
        &expect!["Expr [0-6]: Lit: Imaginary(10.3)"],
    );
}

#[test]
fn lit_float_imag_with_spacing() {
    check(
        expr,
        r#"10.3  im"#,
        &expect!["Expr [0-8]: Lit: Imaginary(10.3)"],
    );
}

#[test]
fn lit_int_imag() {
    check(expr, r#"10"#, &expect!["Expr [0-2]: Lit: Int(10)"]);
}

#[test]
fn lit_int_imag_with_spacing() {
    check(
        expr,
        r#"10  im"#,
        &expect!["Expr [0-6]: Lit: Imaginary(10.0)"],
    );
}

#[test]
fn lit_float_imag_leading_dot() {
    check(expr, ".23im", &expect!["Expr [0-5]: Lit: Imaginary(0.23)"]);
}

#[test]
fn lit_float_imag_trailing_dot() {
    check(expr, "1.im", &expect!["Expr [0-4]: Lit: Imaginary(1.0)"]);
}

#[test]
fn lit_float_imag_underscore() {
    check(
        expr,
        "123_456.78im",
        &expect!["Expr [0-12]: Lit: Imaginary(123456.78)"],
    );
}

#[test]
fn lit_float_imag_leading_zero() {
    check(expr, "0.23im", &expect!["Expr [0-6]: Lit: Imaginary(0.23)"]);
}
