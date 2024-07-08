// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::StateVectorSimulator;
use crate::tests;

#[test]
fn measure_0() {
    tests::measure_0::<StateVectorSimulator>();
}

#[test]
fn measure_1() {
    tests::measure_1::<StateVectorSimulator>();
}

#[test]
fn bell_pair_sampling() {
    tests::bell_pair_sampling::<StateVectorSimulator>();
}

#[test]
fn bell_pair_projection_outcome_0() {
    tests::bell_pair_projection::<StateVectorSimulator>(0).expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn bell_pair_projection_outcome_1() {
    tests::bell_pair_projection::<StateVectorSimulator>(1).expect("test should fail");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn bell_pair_projection_outcome_2() {
    tests::bell_pair_projection::<StateVectorSimulator>(1).expect("test should fail");
}

#[test]
fn bell_pair_projection_outcome_3() {
    tests::bell_pair_projection::<StateVectorSimulator>(3).expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn two_qubit_gate_outcome_0() {
    tests::two_qubit_gate::<StateVectorSimulator>(0).expect("test should fail");
}

#[test]
fn two_qubit_gate_outcome_1() {
    tests::two_qubit_gate::<StateVectorSimulator>(1).expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn two_qubit_gate_outcome_2() {
    tests::two_qubit_gate::<StateVectorSimulator>(2).expect("test should fail");
}

#[test]
fn two_qubit_gate_outcome_3() {
    tests::two_qubit_gate::<StateVectorSimulator>(3).expect("test should pass");
}

#[test]
fn repeated_mz() {
    tests::repeated_mz::<StateVectorSimulator>();
}

#[test]
fn alternating_mz_and_mx() {
    tests::alternating_mz_and_mx::<StateVectorSimulator>();
}
