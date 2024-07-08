// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::DensityMatrixSimulator;
use crate::tests;

#[test]
fn measure_0() {
    tests::measure_0::<DensityMatrixSimulator>();
}

#[test]
fn measure_1() {
    tests::measure_1::<DensityMatrixSimulator>();
}

#[test]
fn bell_pair_sampling() {
    tests::bell_pair_sampling::<DensityMatrixSimulator>();
}

#[test]
fn bell_pair_projection_outcome_0() {
    tests::bell_pair_projection::<DensityMatrixSimulator>(0).expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn bell_pair_projection_outcome_1() {
    tests::bell_pair_projection::<DensityMatrixSimulator>(1).expect("test should fail");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn bell_pair_projection_outcome_2() {
    tests::bell_pair_projection::<DensityMatrixSimulator>(1).expect("test should fail");
}

#[test]
fn bell_pair_projection_outcome_3() {
    tests::bell_pair_projection::<DensityMatrixSimulator>(3).expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn two_qubit_gate_outcome_0() {
    tests::two_qubit_gate::<DensityMatrixSimulator>(0).expect("test should fail");
}

#[test]
fn two_qubit_gate_outcome_1() {
    tests::two_qubit_gate::<DensityMatrixSimulator>(1).expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn two_qubit_gate_outcome_2() {
    tests::two_qubit_gate::<DensityMatrixSimulator>(2).expect("test should fail");
}

#[test]
fn two_qubit_gate_outcome_3() {
    tests::two_qubit_gate::<DensityMatrixSimulator>(3).expect("test should pass");
}

#[test]
fn repeated_mz() {
    tests::repeated_mz::<DensityMatrixSimulator>();
}

#[test]
fn alternating_mz_and_mx() {
    tests::alternating_mz_and_mx::<DensityMatrixSimulator>();
}
