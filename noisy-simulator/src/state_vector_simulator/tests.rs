// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::StateVectorSimulator;
use crate::tests;

#[test]
fn check_measuring_plus_state_yields_zero_with_50_percent_probability() {
    tests::check_measuring_plus_state_yields_zero_with_50_percent_probability::<StateVectorSimulator>(
    );
}

#[test]
fn check_measuring_plus_state_yields_one_with_50_percent_probability() {
    tests::check_measuring_plus_state_yields_one_with_50_percent_probability::<StateVectorSimulator>(
    );
}

#[test]
fn check_bell_pair_sampling_yields_same_outcome_for_both_qubits() {
    // We perform the test 100 times because of the probabilistic nature of the MZ measurement.
    for seed in 0..100 {
        tests::check_bell_pair_sampling_yields_same_outcome_for_both_qubits::<StateVectorSimulator>(
            seed,
        );
    }
}

#[test]
fn check_bell_pair_projection_on_mz0_yields_50_percent_probability_trace() {
    tests::check_bell_pair_projection_on_mz0_yields_50_percent_probability_trace::<
        StateVectorSimulator,
    >();
}

#[test]
fn check_bell_pair_projection_on_mz1_yields_50_percent_probability_trace() {
    tests::check_bell_pair_projection_on_mz1_yields_50_percent_probability_trace::<
        StateVectorSimulator,
    >();
}

#[test]
#[should_panic(expected = "operation should fail: ProbabilityZeroEvent")]
fn check_bell_pair_projection_on_oposite_directions_yields_an_error() {
    tests::check_bell_pair_projection_on_oposite_directions_yields_an_error::<StateVectorSimulator>(
    );
}

#[test]
fn check_crx_gate_projection_on_mz0_yields_right_probabilities() {
    tests::check_crx_gate_projection_on_mz0_yields_right_probabilities::<StateVectorSimulator>();
}

#[test]
fn check_crx_gate_projection_on_mz1_yields_right_probabilities() {
    tests::check_crx_gate_projection_on_mz1_yields_right_probabilities::<StateVectorSimulator>();
}

#[test]
fn check_two_consecutive_mz_yield_same_outcome() {
    // We perform the test 100 times because of the probabilistic nature of the MZ measurement.
    for seed in 0..100 {
        tests::check_two_consecutive_mz_yield_same_outcome::<StateVectorSimulator>(seed);
    }
}

#[test]
fn check_alternating_mz_and_mx_yield_right_probabilities() {
    tests::check_alternating_mz_and_mx_yield_right_probabilities::<StateVectorSimulator>();
}
