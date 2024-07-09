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
    // We perform the test 100 times because of the probabilistic nature of the MZ measurement.
    for _ in 0..100 {
        tests::bell_pair_sampling::<StateVectorSimulator>();
    }
}

#[test]
fn bell_pair_projection_mz0() {
    tests::bell_pair_projection_mz0::<StateVectorSimulator>().expect("test should pass");
}

#[test]
fn bell_pair_projection_mz1() {
    tests::bell_pair_projection_mz1::<StateVectorSimulator>().expect("test should pass");
}

#[test]
#[should_panic(expected = "test should fail: ProbabilityZeroEvent")]
fn bell_pair_projection_oposite_directions() {
    tests::bell_pair_projection_oposite_directions::<StateVectorSimulator>()
        .expect("test should fail");
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
    // We perform the test 100 times because of the probabilistic nature of the MZ measurement.
    for _ in 0..100 {
        tests::repeated_mz::<StateVectorSimulator>();
    }
}

#[test]
fn alternating_mz_and_mx() {
    tests::alternating_mz_and_mx::<StateVectorSimulator>();
}
