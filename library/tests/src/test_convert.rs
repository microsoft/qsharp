// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::test_expression;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Convert namespace

#[test]
fn check_bool_array_as_int() {
    test_expression(
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
    test_expression(
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
    test_expression(
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
    test_expression(
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
    test_expression(
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
