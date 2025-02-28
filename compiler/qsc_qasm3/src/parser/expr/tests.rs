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

#[test]
fn pratt_parsing_mul_add() {
    check(
        expr,
        "1 + 2 * 3",
        &expect![[r#"
            Expr [0-9]: BinOp (Add):
                Expr [0-1]: Lit: Int(1)
                Expr [4-9]: BinOp (Mul):
                    Expr [4-5]: Lit: Int(2)
                    Expr [8-9]: Lit: Int(3)"#]],
    );
}

#[test]
fn pratt_parsing_parens() {
    check(
        expr,
        "(1 + 2) * 3",
        &expect![[r#"
            Expr [0-11]: BinOp (Mul):
                Expr [0-7]: Paren:
                    Expr [1-6]: BinOp (Add):
                        Expr [1-2]: Lit: Int(1)
                        Expr [5-6]: Lit: Int(2)
                Expr [10-11]: Lit: Int(3)"#]],
    );
}

#[test]
fn prat_parsing_mul_unary() {
    check(
        expr,
        "2 * -3",
        &expect![[r#"
            Expr [0-6]: BinOp (Mul):
                Expr [0-1]: Lit: Int(2)
                Expr [4-6]: UnOp (Neg):
                    Expr [5-6]: Lit: Int(3)"#]],
    );
}

#[test]
fn prat_parsing_unary_mul() {
    check(
        expr,
        "-2 * 3",
        &expect![[r#"
            Expr [0-6]: BinOp (Mul):
                Expr [0-2]: UnOp (Neg):
                    Expr [1-2]: Lit: Int(2)
                Expr [5-6]: Lit: Int(3)"#]],
    );
}

#[test]
fn prat_parsing_exp_funcall() {
    check(
        expr,
        "2 ** square(3)",
        &expect![[r#"
            Expr [0-14]: BinOp (Exp):
                Expr [0-1]: Lit: Int(2)
                Expr [5-14]: FunctionCall [5-14]: Ident [5-11] "square"
                    Expr [12-13]: Lit: Int(3)"#]],
    );
}

#[test]
fn prat_parsing_funcall_exp() {
    check(
        expr,
        "square(2) ** 3",
        &expect![[r#"
            Expr [0-14]: BinOp (Exp):
                Expr [0-9]: FunctionCall [0-9]: Ident [0-6] "square"
                    Expr [7-8]: Lit: Int(2)
                Expr [13-14]: Lit: Int(3)"#]],
    );
}

#[test]
fn prat_parsing_funcall_exp_arg() {
    check(
        expr,
        "square(2 ** 3)",
        &expect![[r#"
            Expr [0-14]: FunctionCall [0-14]: Ident [0-6] "square"
                Expr [7-13]: BinOp (Exp):
                    Expr [7-8]: Lit: Int(2)
                    Expr [12-13]: Lit: Int(3)"#]],
    );
}

#[test]
fn funcall() {
    check(
        expr,
        "square(2)",
        &expect![[r#"
            Expr [0-9]: FunctionCall [0-9]: Ident [0-6] "square"
                Expr [7-8]: Lit: Int(2)"#]],
    );
}

#[test]
fn funcall_multiple_args() {
    check(
        expr,
        "square(2, 3)",
        &expect![[r#"
            Expr [0-12]: FunctionCall [0-12]: Ident [0-6] "square"
                Expr [7-8]: Lit: Int(2)
                Expr [10-11]: Lit: Int(3)"#]],
    );
}

#[test]
fn funcall_multiple_args_trailing_comma() {
    check(
        expr,
        "square(2, 3,)",
        &expect![[r#"
            Expr [0-13]: FunctionCall [0-13]: Ident [0-6] "square"
                Expr [7-8]: Lit: Int(2)
                Expr [10-11]: Lit: Int(3)"#]],
    );
}

#[test]
fn cast_to_bit() {
    check(
        expr,
        "bit(0)",
        &expect![[r#"
            Expr [0-3]: Cast [0-6]:
                ClassicalType [0-3]: BitType
                Expr [4-5]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_bit_with_designator() {
    check(
        expr,
        "bit[4](0)",
        &expect![[r#"
            Expr [0-6]: Cast [0-9]:
                ClassicalType [0-6]: BitType [0-6]: Expr [4-5]: Lit: Int(4)
                Expr [7-8]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_int() {
    check(
        expr,
        "int(0)",
        &expect![[r#"
            Expr [0-3]: Cast [0-6]:
                ClassicalType [0-3]: IntType [0-3]
                Expr [4-5]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_int_with_designator() {
    check(
        expr,
        "int[64](0)",
        &expect![[r#"
            Expr [0-7]: Cast [0-10]:
                ClassicalType [0-7]: IntType[Expr [4-6]: Lit: Int(64)]: [0-7]
                Expr [8-9]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_uint() {
    check(
        expr,
        "uint(0)",
        &expect![[r#"
            Expr [0-4]: Cast [0-7]:
                ClassicalType [0-4]: UIntType [0-4]
                Expr [5-6]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_uint_with_designator() {
    check(
        expr,
        "uint[64](0)",
        &expect![[r#"
            Expr [0-8]: Cast [0-11]:
                ClassicalType [0-8]: UIntType[Expr [5-7]: Lit: Int(64)]: [0-8]
                Expr [9-10]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_float() {
    check(
        expr,
        "float(0)",
        &expect![[r#"
            Expr [0-5]: Cast [0-8]:
                ClassicalType [0-5]: FloatType [0-5]
                Expr [6-7]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_float_with_designator() {
    check(
        expr,
        "float[64](0)",
        &expect![[r#"
            Expr [0-9]: Cast [0-12]:
                ClassicalType [0-9]: FloatType[Expr [6-8]: Lit: Int(64)]: [0-9]
                Expr [10-11]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_complex() {
    check(
        expr,
        "complex[float](0)",
        &expect![[r#"
            Expr [0-14]: Cast [0-17]:
                ClassicalType [0-14]: ComplexType[float[FloatType [8-13]]]: [0-14]
                Expr [15-16]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_complex_with_designator() {
    check(
        expr,
        "complex[float[64]](0)",
        &expect![[r#"
            Expr [0-18]: Cast [0-21]:
                ClassicalType [0-18]: ComplexType[float[FloatType[Expr [14-16]: Lit: Int(64)]: [8-17]]]: [0-18]
                Expr [19-20]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_bool() {
    check(
        expr,
        "bool(0)",
        &expect![[r#"
            Expr [0-4]: Cast [0-7]:
                ClassicalType [0-4]: BoolType
                Expr [5-6]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_duration() {
    check(
        expr,
        "duration(0)",
        &expect![[r#"
            Expr [0-8]: Cast [0-11]:
                ClassicalType [0-8]: Duration
                Expr [9-10]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_stretch() {
    check(
        expr,
        "stretch(0)",
        &expect![[r#"
            Expr [0-7]: Cast [0-10]:
                ClassicalType [0-7]: Stretch
                Expr [8-9]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_int_array() {
    check(
        expr,
        "array[int[64], 4](0)",
        &expect![[r#"
            Expr [0-17]: Cast [0-20]:
                ArrayType [0-17]: ArrayBaseTypeKind IntType[Expr [10-12]: Lit: Int(64)]: [6-13]
                Expr [15-16]: Lit: Int(4)
                Expr [18-19]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_uint_array() {
    check(
        expr,
        "array[uint[64], 4](0)",
        &expect![[r#"
            Expr [0-18]: Cast [0-21]:
                ArrayType [0-18]: ArrayBaseTypeKind UIntType[Expr [11-13]: Lit: Int(64)]: [6-14]
                Expr [16-17]: Lit: Int(4)
                Expr [19-20]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_float_array() {
    check(
        expr,
        "array[float[64], 4](0)",
        &expect![[r#"
            Expr [0-19]: Cast [0-22]:
                ArrayType [0-19]: ArrayBaseTypeKind FloatType[Expr [12-14]: Lit: Int(64)]: [6-15]
                Expr [17-18]: Lit: Int(4)
                Expr [20-21]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_angle_array() {
    check(
        expr,
        "array[angle[64], 4](0)",
        &expect![[r#"
            Expr [0-19]: Cast [0-22]:
                ArrayType [0-19]: ArrayBaseTypeKind AngleType [6-15]: Expr [12-14]: Lit: Int(64)
                Expr [17-18]: Lit: Int(4)
                Expr [20-21]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_bool_array() {
    check(
        expr,
        "array[bool, 4](0)",
        &expect![[r#"
            Expr [0-14]: Cast [0-17]:
                ArrayType [0-14]: ArrayBaseTypeKind BoolType
                Expr [12-13]: Lit: Int(4)
                Expr [15-16]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_duration_array() {
    check(
        expr,
        "array[duration, 4](0)",
        &expect![[r#"
            Expr [0-18]: Cast [0-21]:
                ArrayType [0-18]: ArrayBaseTypeKind DurationType
                Expr [16-17]: Lit: Int(4)
                Expr [19-20]: Lit: Int(0)"#]],
    );
}

#[test]
fn cast_to_complex_array() {
    check(
        expr,
        "array[complex[float[32]], 4](0)",
        &expect![[r#"
            Expr [0-28]: Cast [0-31]:
                ArrayType [0-28]: ArrayBaseTypeKind ComplexType[float[FloatType[Expr [20-22]: Lit: Int(32)]: [14-23]]]: [6-24]
                Expr [26-27]: Lit: Int(4)
                Expr [29-30]: Lit: Int(0)"#]],
    );
}

#[test]
fn index_expr() {
    check(
        expr,
        "foo[1]",
        &expect![[r#"
            Expr [0-6]: IndexExpr [3-6]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                IndexSetItem Expr [4-5]: Lit: Int(1)"#]],
    );
}

#[test]
fn index_set() {
    check(
        expr,
        "foo[{1, 4, 5}]",
        &expect![[r#"
            Expr [0-14]: IndexExpr [3-14]: Expr [0-3]: Ident [0-3] "foo", IndexElement DiscreteSet [4-13]:
                Expr [5-6]: Lit: Int(1)
                Expr [8-9]: Lit: Int(4)
                Expr [11-12]: Lit: Int(5)"#]],
    );
}

#[test]
fn index_multiple_ranges() {
    check(
        expr,
        "foo[1:5, 3:7, 4:8]",
        &expect![[r#"
            Expr [0-18]: IndexExpr [3-18]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                Range: [4-7]
                    start: Expr [4-5]: Lit: Int(1)
                    <no step>
                    end: Expr [6-7]: Lit: Int(5)
                Range: [9-12]
                    start: Expr [9-10]: Lit: Int(3)
                    <no step>
                    end: Expr [11-12]: Lit: Int(7)
                Range: [14-17]
                    start: Expr [14-15]: Lit: Int(4)
                    <no step>
                    end: Expr [16-17]: Lit: Int(8)"#]],
    );
}

#[test]
fn index_range() {
    check(
        expr,
        "foo[1:5:2]",
        &expect![[r#"
            Expr [0-10]: IndexExpr [3-10]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                Range: [4-9]
                    start: Expr [4-5]: Lit: Int(1)
                    step: Expr [6-7]: Lit: Int(5)
                    end: Expr [8-9]: Lit: Int(2)"#]],
    );
}

#[test]
fn index_full_range() {
    check(
        expr,
        "foo[:]",
        &expect![[r#"
            Expr [0-6]: IndexExpr [3-6]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                Range: [4-5]
                    <no start>
                    <no step>
                    <no end>"#]],
    );
}

#[test]
fn index_range_start() {
    check(
        expr,
        "foo[1:]",
        &expect![[r#"
            Expr [0-7]: IndexExpr [3-7]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                Range: [4-6]
                    start: Expr [4-5]: Lit: Int(1)
                    <no step>
                    <no end>"#]],
    );
}

#[test]
fn index_range_end() {
    check(
        expr,
        "foo[:5]",
        &expect![[r#"
            Expr [0-7]: IndexExpr [3-7]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                Range: [4-6]
                    <no start>
                    <no step>
                    end: Expr [5-6]: Lit: Int(5)"#]],
    );
}

#[test]
fn index_range_step() {
    check(
        expr,
        "foo[:2:]",
        &expect![[r#"
            Expr [0-8]: IndexExpr [3-8]: Expr [0-3]: Ident [0-3] "foo", IndexElement:
                Range: [4-7]
                    <no start>
                    step: Expr [5-6]: Lit: Int(2)
                    <no end>"#]],
    );
}

#[test]
fn set_expr() {
    check(
        super::set_expr,
        "{2, 3, 4}",
        &expect![[r#"
        DiscreteSet [0-9]:
            Expr [1-2]: Lit: Int(2)
            Expr [4-5]: Lit: Int(3)
            Expr [7-8]: Lit: Int(4)"#]],
    );
}

#[test]
fn lit_array() {
    check(
        super::lit_array,
        "{{2, {5}}, 1 + z}",
        &expect![[r#"
            Expr [0-17]: Lit: Array:
                Expr { span: Span { lo: 1, hi: 9 }, kind: Lit(Lit { span: Span { lo: 1, hi: 9 }, kind: Array([Expr { span: Span { lo: 2, hi: 3 }, kind: Lit(Lit { span: Span { lo: 2, hi: 3 }, kind: Int(2) }) }, Expr { span: Span { lo: 5, hi: 8 }, kind: Lit(Lit { span: Span { lo: 5, hi: 8 }, kind: Array([Expr { span: Span { lo: 6, hi: 7 }, kind: Lit(Lit { span: Span { lo: 6, hi: 7 }, kind: Int(5) }) }]) }) }]) }) }
                Expr { span: Span { lo: 11, hi: 16 }, kind: BinaryOp(BinaryOpExpr { op: Add, lhs: Expr { span: Span { lo: 11, hi: 12 }, kind: Lit(Lit { span: Span { lo: 11, hi: 12 }, kind: Int(1) }) }, rhs: Expr { span: Span { lo: 15, hi: 16 }, kind: Ident(Ident { span: Span { lo: 15, hi: 16 }, name: "z" }) } }) }"#]],
    );
}

#[test]
fn assignment_and_unop() {
    check(
        expr,
        "c = a && !b",
        &expect![[r#"
            Expr [0-11]: Assign:
                Expr [0-1]: Ident [0-1] "c"
                Expr [4-11]: BinOp (AndL):
                    Expr [4-5]: Ident [4-5] "a"
                    Expr [9-11]: UnOp (NotL):
                        Expr [10-11]: Ident [10-11] "b""#]],
    );
}

#[test]
fn assignment_unop_and() {
    check(
        expr,
        "d = !a && b",
        &expect![[r#"
            Expr [0-11]: Assign:
                Expr [0-1]: Ident [0-1] "d"
                Expr [4-11]: BinOp (AndL):
                    Expr [4-6]: UnOp (NotL):
                        Expr [5-6]: Ident [5-6] "a"
                    Expr [10-11]: Ident [10-11] "b""#]],
    );
}

#[test]
fn hardware_qubit() {
    check(
        super::hardware_qubit,
        "$12",
        &expect!["HardwareQubit [0-3]: 12"],
    );
}

#[test]
fn indexed_identifier() {
    check(
        super::indexed_identifier,
        "arr[1][2]",
        &expect![[r#"
        IndexedIdent [0-9]: Ident [0-3] "arr"[
        IndexElement:
            IndexSetItem Expr [4-5]: Lit: Int(1)
        IndexElement:
            IndexSetItem Expr [7-8]: Lit: Int(2)]"#]],
    );
}

#[test]
fn measure_hardware_qubit() {
    check(
        super::measure_expr,
        "measure $12",
        &expect!["MeasureExpr [0-7]: GateOperand HardwareQubit [8-11]: 12"],
    );
}

#[test]
fn measure_indexed_identifier() {
    check(
        super::measure_expr,
        "measure qubits[1][2]",
        &expect![[r#"
        MeasureExpr [0-7]: GateOperand IndexedIdent [8-20]: Ident [8-14] "qubits"[
        IndexElement:
            IndexSetItem Expr [15-16]: Lit: Int(1)
        IndexElement:
            IndexSetItem Expr [18-19]: Lit: Int(2)]"#]],
    );
}
