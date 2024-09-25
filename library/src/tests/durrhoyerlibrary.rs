// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression_with_lib;
use qsc::interpret::Value;

// Library that includes the necessary DurrHoyerAlgorithm implementation
const DURR_HOYER_LIB: &str = include_str!("resources/src/durrhoyerlibrary.qs");

#[test]
fn check_durr_hoyer_minimum_test_case_1() {
    test_expression_with_lib(
        "Test.RunDurrHoyerMinimumUnitTestWithShots(1000)",
        DURR_HOYER_LIB,
    );
}

#[test]
fn check_durr_hoyer_maximum_test_case_3() {
    test_expression_with_lib(
        "Test.RunDurrHoyerMaximumUnitTestWithShots(1000)",
        DURR_HOYER_LIB,
    );
}
