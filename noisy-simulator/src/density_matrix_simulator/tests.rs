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
    // We perform the test 100 times because of the probabilistic nature of the MZ measurement.
    for seed in 0..100 {
        tests::bell_pair_sampling::<DensityMatrixSimulator>(seed);
    }
}

#[test]
fn bell_pair_projection_mz0() {
    tests::bell_pair_projection_mz0::<DensityMatrixSimulator>();
}

#[test]
fn bell_pair_projection_mz1() {
    tests::bell_pair_projection_mz1::<DensityMatrixSimulator>();
}

#[test]
#[should_panic(expected = "operation should fail: ProbabilityZeroEvent")]
fn bell_pair_projection_oposite_directions() {
    tests::bell_pair_projection_oposite_directions::<DensityMatrixSimulator>();
}

#[test]
fn crx_gate_projection_mz0() {
    tests::crx_gate_projection_mz0::<DensityMatrixSimulator>();
}

#[test]
fn crx_gate_projection_mz1() {
    tests::crx_gate_projection_mz1::<DensityMatrixSimulator>();
}

#[test]
fn repeated_mz() {
    // We perform the test 100 times because of the probabilistic nature of the MZ measurement.
    for seed in 0..100 {
        tests::repeated_mz::<DensityMatrixSimulator>(seed);
    }
}

#[test]
fn alternating_mz_and_mx() {
    tests::alternating_mz_and_mx::<DensityMatrixSimulator>();
}
