// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression;
use super::test_expression_with_lib;
use qsc::interpret::Value;

// Ensure the correct path to the DurrHoyerLibrary Q# implementation
// The path should point to the actual location of your Q# file.
const DURR_HOYER_LIB: &str = include_str!("/resources/src/durrhoyerlibrary.qs");

#[test]
fn check_durr_hoyer_minimum_test_case() {
    // Call the test expression for running the Durr-Hoyer Minimum Unit Test
    let result = test_expression_with_lib(
        "Test.RunDurrHoyerMinimumUnitTestWithShots(1000)",
        DURR_HOYER_LIB,
    );
}

#[test]
fn check_durr_hoyer_maximum_test_case() {
    // Call the test expression for running the Durr-Hoyer Maximum Unit Test
    let result = test_expression_with_lib(
        "Test.RunDurrHoyerMaximumUnitTestWithShots(1000)",
        DURR_HOYER_LIB,

}
