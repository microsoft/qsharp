// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
#![allow(clippy::too_many_lines)]
use super::test_expression;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Convert namespace

#[test]
fn check_bool_array_as_int() {
    test_expression(
        "Microsoft.Quantum.Convert.BoolArrayAsInt([true, false, true, false])",
        &Value::Int(0b0101),
    );
}

#[test]
fn check_int_as_bool_array() {
    test_expression(
        "Microsoft.Quantum.Convert.IntAsBoolArray(5,4)",
        &Value::Array(
            vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
                Value::Bool(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_bigint_as_bool_array() {
    test_expression(
        "Microsoft.Quantum.Convert.BigIntAsBoolArray(18446744073709551616L, 128)", // note: 18446744073709551616L == 2^64
        &Value::Array(
            vec![
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
                Value::Bool(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_bool_array_as_big_int() {
    test_expression(
        "Microsoft.Quantum.Convert.BoolArrayAsBigInt([false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, true, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false])",
        &Value::BigInt(18_446_744_073_709_551_616_u128.into()),
    );
}

#[test]
fn check_result_array_as_int() {
    test_expression(
        "Microsoft.Quantum.Convert.ResultArrayAsInt([One, Zero, One, Zero])",
        &Value::Int(0b0101),
    );
}

#[test]
fn check_result_zero_as_bool() {
    test_expression(
        "Microsoft.Quantum.Convert.ResultAsBool(Zero)",
        &Value::Bool(false),
    );
}

#[test]
fn check_result_one_as_bool() {
    test_expression(
        "Microsoft.Quantum.Convert.ResultAsBool(One)",
        &Value::Bool(true),
    );
}

#[test]
fn check_result_array_as_bool_array() {
    test_expression(
        "Microsoft.Quantum.Convert.ResultArrayAsBoolArray([One, Zero, One, Zero])",
        &Value::Array(
            vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
                Value::Bool(false),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_bool_true_as_result() {
    test_expression(
        "Microsoft.Quantum.Convert.BoolAsResult(true)",
        &Value::RESULT_ONE,
    );
}

#[test]
fn check_bool_false_as_result() {
    test_expression(
        "Microsoft.Quantum.Convert.BoolAsResult(false)",
        &Value::RESULT_ZERO,
    );
}

#[test]
fn check_bool_array_as_result_array() {
    test_expression(
        "Microsoft.Quantum.Convert.BoolArrayAsResultArray([true, false, true, false])",
        &Value::Array(
            vec![
                Value::RESULT_ONE,
                Value::RESULT_ZERO,
                Value::RESULT_ONE,
                Value::RESULT_ZERO,
            ]
            .into(),
        ),
    );
}

#[test]
fn test_complex_as_complex_polar() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Math;
            let a = Complex(2.0*Cos(1.0), 2.0*Sin(1.0));
            Microsoft.Quantum.Convert.ComplexAsComplexPolar(a)
        }"
        },
        &Value::Tuple(vec![Value::Double(2.0), Value::Double(1.0)].into()),
    );
}

#[test]
fn test_complex_polar_as_complex() {
    test_expression(
        {
            "{
            open Microsoft.Quantum.Math;
            let a = ComplexPolar(Sqrt(5.0), ArcTan2(1.0, 2.0));
            Microsoft.Quantum.Convert.ComplexPolarAsComplex(a)
        }"
        },
        &Value::Tuple(vec![Value::Double(2.0), Value::Double(1.0)].into()),
    );
}
