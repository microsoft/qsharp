// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use super::test_expression;
use indoc::indoc;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.Measurement namespace

#[test]
fn check_measure_all_z() {
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            let result = Microsoft.Quantum.Measurement.MeasureAllZ(register);
            ResetAll(register);
            result
        }"#},
        &Value::RESULT_ZERO,
    );
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            Microsoft.Quantum.Arrays.ForEach(X, register);
            let result = Microsoft.Quantum.Measurement.MeasureAllZ(register);
            ResetAll(register);
            result
        }"#},
        &Value::RESULT_ZERO,
    );
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[0]);
            let result = Microsoft.Quantum.Measurement.MeasureAllZ(register);
            ResetAll(register);
            result
        }"#},
        &Value::RESULT_ONE,
    );
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            let result = Microsoft.Quantum.Measurement.MeasureAllZ(register);
            ResetAll(register);
            result
        }"#},
        &Value::RESULT_ONE,
    );
}

#[test]
fn check_measure_each_z() {
    test_expression(
        indoc! {r#"{
            use register = Qubit[3];
            X(register[0]);
            X(register[2]);
            let results = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            ResetAll(register);
            results
        }"#},
        &Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ZERO, Value::RESULT_ONE].into()),
    );
}

#[test]
fn check_mreset_each_z() {
    test_expression(
        indoc! {r#"{
            use register = Qubit[3];
            X(register[0]);
            X(register[2]);
            let resultsA = Microsoft.Quantum.Measurement.MResetEachZ(register);
            let resultsB = Microsoft.Quantum.Measurement.MeasureEachZ(register);
            (resultsA, resultsB)
        }"#},
        &Value::Tuple(
            vec![
                Value::Array(vec![Value::RESULT_ONE, Value::RESULT_ZERO, Value::RESULT_ONE].into()),
                Value::Array(
                    vec![Value::RESULT_ZERO, Value::RESULT_ZERO, Value::RESULT_ZERO].into(),
                ),
            ]
            .into(),
        ),
    );
}

#[test]
fn check_mreset_x() {
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            Microsoft.Quantum.Canon.ApplyToEach(H, register);
            let r0 = Microsoft.Quantum.Measurement.MResetX(register[0]);
            let r1 = Microsoft.Quantum.Measurement.MResetX(register[1]);
            [r0, r1]
        }"#},
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ONE].into()),
    );
}

#[test]
fn check_mreset_y() {
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            Microsoft.Quantum.Canon.ApplyToEach(H, register);
            Microsoft.Quantum.Canon.ApplyToEach(S, register);
            let r0 = Microsoft.Quantum.Measurement.MResetY(register[0]);
            let r1 = Microsoft.Quantum.Measurement.MResetY(register[1]);
            [r0, r1]
        }"#},
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ONE].into()),
    );
}

#[test]
fn check_mreset_z() {
    test_expression(
        indoc! {r#"{
            use register = Qubit[2];
            X(register[1]);
            let r0 = Microsoft.Quantum.Measurement.MResetZ(register[0]);
            let r1 = Microsoft.Quantum.Measurement.MResetZ(register[1]);
            [r0, r1]
        }"#},
        &Value::Array(vec![Value::RESULT_ZERO, Value::RESULT_ONE].into()),
    );
}

#[test]
fn check_measure_integer() {
    test_expression(
        {
            "{
                open Microsoft.Quantum.Measurement;
                use q = Qubit[16];
                ApplyXorInPlace(45967, q);
                let result = MeasureInteger(q);
                ResetAll(q);
                return result;
            }"
        },
        &Value::Int(45967),
    );
}
