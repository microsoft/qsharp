// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression_with_lib;
use qsc::interpret::Value;

const DURR_HOYER_LIB: &str = include_str!("resources/src/durrhoyerlibrary.qs");

#[test]
fn check_durr_hoyer_minimum_test_case() {
    // Call the test expression for running the Durr-Hoyer Minimum Unit Test
    test_expression_with_lib(
        "Test.RunDurrHoyerMinimumUnitTestWithShots(1000)",
        DURR_HOYER_LIB,
        &Value::unit(),
    );
}

#[test]
fn check_durr_hoyer_maximum_test_case() {
    // Call the test expression for running the Durr-Hoyer Maximum Unit Test
    test_expression_with_lib(
        "Test.RunDurrHoyerMaximumUnitTestWithShots(1000)",
        DURR_HOYER_LIB,
        &Value::unit(),
    );
}

#[test]
fn check_durr_hoyer_zero_test_case() {
    // Call the test expression for running the Durr-Hoyer Maximum Unit Test
    test_expression_with_lib(
        "Test.RunDurrHoyerZeroValuesUnitTestWithShots(1000)",
        DURR_HOYER_LIB,
        &Value::unit(),
    );
}
