// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

mod arithmetic;
mod arrays;
mod canon;
mod convert;
mod core;
mod diagnostics;
mod math;
mod measurement;
mod state_preparation;
mod table_lookup;

use indoc::indoc;
use qsc::{
    interpret::{GenericReceiver, Interpreter, Value},
    target::Profile,
    PackageType, SourceMap,
};

/// # Panics
///
/// Will panic if compilation fails or the result is not the same as expected.
/// NOTE: Floating point numbers in tuples are compared taking precision into
/// account so that results of calculations can also be compared.
pub fn test_expression(expr: &str, expected: &Value) -> String {
    test_expression_with_lib(expr, "", expected)
}

pub fn test_expression_with_lib(expr: &str, lib: &str, expected: &Value) -> String {
    test_expression_with_lib_and_profile(expr, lib, Profile::Unrestricted, expected)
}

pub fn test_expression_with_lib_and_profile(
    expr: &str,
    lib: &str,
    profile: Profile,
    expected: &Value,
) -> String {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);

    let sources = SourceMap::new([("test".into(), lib.into())], Some(expr.into()));

    let mut interpreter = Interpreter::new(true, sources, PackageType::Exe, profile.into())
        .expect("test should compile");
    let result = interpreter
        .eval_entry(&mut out)
        .expect("test should run successfully");

    match (&expected, result) {
        (&Value::Tuple(tup1), Value::Tuple(tup2)) if tup1.len() == tup2.len() => {
            // If both values are tuples of the same length, we crack them open and compare elements
            for (value1, value2) in tup1.iter().zip(tup2.iter()) {
                if let (Value::Double(double1), Value::Double(double2)) = (value1, value2) {
                    // If both elements are doubles, we use approximate comparison
                    assert_doubles_almost_equal(*double1, *double2);
                } else {
                    assert_eq!(value1, value2);
                }
            }
        }
        (&Value::Double(double1), Value::Double(double2)) => {
            assert_doubles_almost_equal(*double1, double2);
        }
        (&expected, result) => assert_eq!(expected, &result),
    }

    String::from_utf8(stdout).expect("stdout should be valid utf8")
}

/// # Panics
///
/// Will panic if f64 values are significantly different.
fn assert_doubles_almost_equal(val1: f64, val2: f64) {
    let val1_abs = val1.abs();
    let val2_abs = val2.abs();
    if val1_abs < f64::MIN_POSITIVE && val2_abs < f64::MIN_POSITIVE {
        // Note, that f64::MIN_POSITIVE is not the smallest representable positive number.
        return;
    }
    assert!(
        ((val1 - val2).abs() / (val1_abs + val2_abs)) < 1e-15,
        "Significant difference between expected and actual values: val1={val1}, val2={val2}."
    );
}

//
// Core namespace
//
#[test]
fn check_repeated() {
    test_expression("Repeated(Zero, 0)", &Value::Array(vec![].into()));
    test_expression(
        "Repeated(One, 1)",
        &Value::Array(vec![Value::RESULT_ONE].into()),
    );
    test_expression(
        "Repeated(1, 2)",
        &Value::Array(vec![Value::Int(1), Value::Int(1)].into()),
    );
    test_expression(
        "Repeated(true, 3)",
        &Value::Array(vec![Value::Bool(true), Value::Bool(true), Value::Bool(true)].into()),
    );
}

#[test]
fn check_exp_with_cnot() {
    // This decomposition only holds if the magnitude of the angle used in Exp is correct and if the
    // sign convention between Rx, Rz, and Exp is consistent.
    test_expression(
        indoc! {r#"{
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Math;

            use (aux, control, target) = (Qubit(), Qubit(), Qubit());
            within {
                H(aux);
                CNOT(aux, control);
                CNOT(aux, target);
            }
            apply {
                let theta  = PI() / 4.0;
                Rx(-2.0 * theta, target);
                Rz(-2.0 * theta, control);
                Adjoint Exp([PauliZ, PauliX], theta, [control, target]);

                Adjoint CNOT(control, target);
            }

            CheckAllZero([aux, control, target])
        }"#},
        &Value::Bool(true),
    );
}

#[test]
fn check_exp_with_swap() {
    // This decomposition only holds if the magnitude of the angle used in Exp is correct.
    test_expression(
        indoc! {r#"{
            open Microsoft.Quantum.Diagnostics;
            open Microsoft.Quantum.Math;

            use (aux, qs) = (Qubit(), Qubit[2]);
            within {
                H(aux);
                CNOT(aux, qs[0]);
                CNOT(aux, qs[1]);
            }
            apply {
                let theta  = PI() / 4.0;
                Exp([PauliX, PauliX], theta, qs);
                Exp([PauliY, PauliY], theta, qs);
                Exp([PauliZ, PauliZ], theta, qs);

                Adjoint SWAP(qs[0], qs[1]);
            }

            CheckAllZero([aux] + qs)
        }"#},
        &Value::Bool(true),
    );
}

#[test]
fn check_base_profile_measure_resets_aux_qubits() {
    test_expression_with_lib_and_profile(
        indoc! {"{
            use q = Qubit();
            X(q);
            let result = M(q);
            Reset(q);
            result
        }"},
        "",
        Profile::Base,
        &Value::RESULT_ONE,
    );
}
