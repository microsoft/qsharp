// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::run_stdlib_test;
use num_bigint::BigInt;
use qsc::interpret::Value;

// Testing functionality of Microsoft.Quantum.Convert namespace

#[test]
fn int_as_double() {
    run_stdlib_test(
        "Microsoft.Quantum.Convert.IntAsDouble(2)",
        &Value::Double(2.0),
    );
}

#[test]
fn int_as_double_precision_loss() {
    run_stdlib_test(
        "Microsoft.Quantum.Convert.IntAsDouble(9_223_372_036_854_775_807)",
        &Value::Double(9_223_372_036_854_775_808.0),
    );
}

#[test]
fn int_as_bigint() {
    run_stdlib_test(
        "Microsoft.Quantum.Convert.IntAsBigInt(0)",
        &Value::BigInt(BigInt::from(0)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Convert.IntAsBigInt(-10000)",
        &Value::BigInt(BigInt::from(-10000)),
    );
    run_stdlib_test(
        "Microsoft.Quantum.Convert.IntAsBigInt(9_223_372_036_854_775_807)",
        &Value::BigInt(BigInt::from(9_223_372_036_854_775_807i64)),
    );
}

#[test]
fn check_bool_array_as_int() {
    run_stdlib_test(
        {
            "{
            let b = [true, false, true, false];
            return Microsoft.Quantum.Convert.BoolArrayAsInt(b);
        }"
        },
        &Value::Int(0b0101),
    );
}

#[test]
fn check_int_as_bool_array() {
    run_stdlib_test(
        {
            "{
            return Microsoft.Quantum.Convert.IntAsBoolArray(5,4);
        }"
        },
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
fn check_result_array_as_int() {
    run_stdlib_test(
        {
            "{
            let b = [One, Zero, One, Zero];
            return Microsoft.Quantum.Convert.ResultArrayAsInt(b);
        }"
        },
        &Value::Int(0b0101),
    );
}

#[test]
fn check_result_array_as_bool_array() {
    run_stdlib_test(
        {
            "{
            let r = [One, Zero, One, Zero];
            return Microsoft.Quantum.Convert.ResultArrayAsBoolArray(r);
        }"
        },
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
fn check_bool_array_as_result_array() {
    run_stdlib_test(
        {
            "{
            let b = [true, false, true, false];
            return Microsoft.Quantum.Convert.BoolArrayAsResultArray(b);
        }"
        },
        &Value::Array(
            vec![
                Value::Result(true),
                Value::Result(false),
                Value::Result(true),
                Value::Result(false),
            ]
            .into(),
        ),
    );
}
