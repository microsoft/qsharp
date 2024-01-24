// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::test_expression_with_lib;
use qsc::interpret::Value;

// Tests for Microsoft.Quantum.StatePreparation namespace

const STATE_PREPARATION_TEST_LIB: &str = include_str!("resources/state_preparation.qs");

#[test]
fn check_plus_state_preparation() {
    test_expression_with_lib(
        "Test.TestPlusState()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_minus_state_preparation() {
    test_expression_with_lib(
        "Test.TestMinusState()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_bell_state_preparation() {
    test_expression_with_lib(
        "Test.TestBellState()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_cat_state_preparation() {
    test_expression_with_lib(
        "Test.TestCat3State()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_complex_preparation() {
    test_expression_with_lib(
        "Test.TestPrepareComplex()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}

#[test]
fn check_preparation_completion() {
    test_expression_with_lib(
        "Test.TestPreparationCompletion()",
        STATE_PREPARATION_TEST_LIB,
        &Value::Tuple(vec![].into()),
    );
}
