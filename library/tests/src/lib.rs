// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![warn(clippy::mod_module_files, clippy::pedantic, clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[cfg(test)]
mod test_arithmetic;
#[cfg(test)]
mod test_arrays;
#[cfg(test)]
mod test_canon;
#[cfg(test)]
mod test_convert;
#[cfg(test)]
mod test_diagnostics;
#[cfg(test)]
mod test_math;
#[cfg(test)]
mod test_measurement;
#[cfg(test)]
mod tests;

use qsc::{
    interpret::{stateful, GenericReceiver, Value},
    PackageType, SourceMap, TargetProfile,
};

/// # Panics
///
/// Will panic if compilation fails or the result is not the same as expected.
/// NOTE: Floating point numbers in tuples are compared taking precision into
/// account so that results of calculations can also be compared.
pub fn test_expression(expr: &str, expected: &Value) {
    test_expression_with_lib(expr, "", expected);
}

pub fn test_expression_with_lib(expr: &str, lib: &str, expected: &Value) {
    let mut stdout = vec![];
    let mut out = GenericReceiver::new(&mut stdout);

    let sources = SourceMap::new([("test".into(), lib.into())], Some(expr.into()));

    let mut interpreter =
        stateful::Interpreter::new(true, sources, PackageType::Exe, TargetProfile::Full)
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
