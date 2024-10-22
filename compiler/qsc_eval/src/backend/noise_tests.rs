// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    backend::{Backend, SparseSim},
    noise::PauliNoise,
    state::{fmt_complex, format_state_id},
};
use expect_test::expect;
use num_bigint::BigUint;
use num_complex::Complex;
use std::fmt::Write;

#[test]
fn pauli_noise() {
    let noise = PauliNoise::from_probabilities(0.0, 0.0, 0.0);
    assert!(
        noise
            .expect("Expected construction of noiseless noise.")
            .is_noiseless(),
        "Expected noiseless noise."
    );
    let noise = PauliNoise::from_probabilities(1e-5, 0.0, 0.0);
    assert!(
        !noise
            .expect("Expected construction of 1e-5 bit flip noise.")
            .is_noiseless(),
        "Expected noise to be noisy."
    );
    let noise = PauliNoise::from_probabilities(1.0, 0.0, 0.0);
    assert!(
        !noise
            .expect("Expected construction of 1.0 bit flip noise.")
            .is_noiseless(),
        "Expected noise to be noisy."
    );
    let noise = PauliNoise::from_probabilities(0.01, 0.01, 0.01)
        .expect("Expected construction of 0.01 depolarizing noise.");
    assert!(!noise.is_noiseless(), "Expected noise to be noisy.");
    assert!(
        0.0 <= noise.distribution[0]
            && noise.distribution[0] <= noise.distribution[1]
            && noise.distribution[1] <= noise.distribution[2]
            && noise.distribution[2] <= 1.1,
        "Expected non-decreasing noise distribution."
    );
    let _ = PauliNoise::from_probabilities(-1e-10, 0.1, 0.1)
        .expect_err("Expected error for probabilities -1e-10, 0.1, 0.1.");
    let _ = PauliNoise::from_probabilities(1.0 + -1e-10, 0.1, 0.1)
        .expect_err("Expected error for probabilities 1.0+1e-10, 0.1, 0.1.");
    let _ = PauliNoise::from_probabilities(0.3, 0.4, 0.5)
        .expect_err("Expected error for probabilities 0.3, 0.4, 0.5.");
}

#[test]
fn noisy_simulator() {
    let sim = SparseSim::new();
    assert!(sim.is_noiseless(), "Expected noiseless simulator.");

    let noise = PauliNoise::from_probabilities(0.0, 0.0, 0.0).expect("Cannot construct noise.");
    let sim = SparseSim::new_with_noise(&noise);
    assert!(sim.is_noiseless(), "Expected noiseless simulator.");

    let noise = PauliNoise::from_probabilities(1e-10, 0.0, 0.0).expect("Cannot construct noise.");
    let sim = SparseSim::new_with_noise(&noise);
    assert!(!sim.is_noiseless(), "Expected noisy simulator.");

    let noise = PauliNoise::from_probabilities(0.0, 0.0, 1e-10).expect("Cannot construct noise.");
    let sim = SparseSim::new_with_noise(&noise);
    assert!(!sim.is_noiseless(), "Expected noisy simulator.");
}

#[test]
fn noiseless_gate() {
    let noise = PauliNoise::from_probabilities(0.0, 0.0, 0.0).expect("Cannot construct noise.");
    let mut sim = SparseSim::new_with_noise(&noise);
    let q = sim.qubit_allocate();
    for _ in 0..100 {
        sim.x(q);
        let res1 = sim.m(q);
        assert!(res1, "Unexpected value. True expected without noise.");
        sim.x(q);
        let res2 = sim.m(q);
        assert!(!res2, "Unexpected value. False expected without noise.");
    }
    assert!(sim.qubit_release(q), "Unexpected qubit state on release.");
}

#[test]
fn bitflip_measurement() {
    let noise = PauliNoise::from_probabilities(1.0, 0.0, 0.0).expect("Cannot construct noise.");
    let mut sim = SparseSim::new_with_noise(&noise);
    assert!(!sim.is_noiseless(), "Expected noisy simulator.");
    let q = sim.qubit_allocate();
    for _ in 0..100 {
        let res1 = sim.m(q); // Always applies X before measuring
        assert!(
            res1,
            "Unexpected value. True expected for 100% bitflip noise."
        );
        let res2 = sim.m(q); // Always applies X before measuring
        assert!(
            !res2,
            "Unexpected value. False expected for 100% bitflip noise."
        );
    }
    assert!(sim.qubit_release(q), "Unexpected qubit state on release.");
}

#[test]
fn bitflip_gate() {
    let noise = PauliNoise::from_probabilities(1.0, 0.0, 0.0).expect("Cannot construct noise.");
    let mut sim = SparseSim::new_with_noise(&noise);
    assert!(!sim.is_noiseless(), "Expected noisy simulator.");
    let q = sim.qubit_allocate();
    for _ in 0..100 {
        sim.x(q); // This is a no-op under 100% bitflip noise.
        let res1 = sim.m(q); // Always applies X before measuring
        assert!(
            res1,
            "Unexpected value. True expected for 100% bitflip noise."
        );
        sim.x(q); // This is a no-op under 100% bitflip noise.
        let res2 = sim.m(q);
        assert!(
            !res2,
            "Unexpected value. False expected for 100% bitflip noise."
        );
    }
    assert!(sim.qubit_release(q), "Unexpected qubit state on release.");
}

pub fn state_to_string(input: &(Vec<(BigUint, Complex<f64>)>, usize)) -> String {
    input
        .0
        .iter()
        .fold(String::new(), |mut output, (id, state)| {
            let _ = write!(
                output,
                "{}: {} ",
                format_state_id(id, input.1),
                fmt_complex(state)
            );
            output
        })
        .to_string()
}

#[test]
fn noisy_via_y() {
    let noise = PauliNoise::from_probabilities(0.0, 1.0, 0.0).expect("Cannot construct noise.");
    let mut sim = SparseSim::new_with_noise(&noise);
    assert!(!sim.is_noiseless(), "Expected noisy simulator.");
    let q = sim.qubit_allocate();
    sim.x(q); // Followed by Y.
    let state = sim.capture_quantum_state();
    expect!["|0⟩: 0.0000−1.0000𝑖 "].assert_eq(&state_to_string(&state));
    sim.y(q); // Followed by Y. So, no op.
    let state = sim.capture_quantum_state();
    expect!["|0⟩: 0.0000−1.0000𝑖 "].assert_eq(&state_to_string(&state));
    sim.z(q); // Followed by Y.
    let state = sim.capture_quantum_state();
    expect!["|1⟩: 1.0000+0.0000𝑖 "].assert_eq(&state_to_string(&state));
}

#[test]
fn noisy_via_z() {
    let noise = PauliNoise::from_probabilities(0.0, 0.0, 1.0).expect("Cannot construct noise.");
    let mut sim = SparseSim::new_with_noise(&noise);
    assert!(!sim.is_noiseless(), "Expected noisy simulator.");
    let q = sim.qubit_allocate();
    sim.x(q); // Followed by Z.
    let state = sim.capture_quantum_state();
    expect!["|1⟩: −1.0000+0.0000𝑖 "].assert_eq(&state_to_string(&state));
    sim.y(q); // Followed by Z.
    let state = sim.capture_quantum_state();
    expect!["|0⟩: 0.0000+1.0000𝑖 "].assert_eq(&state_to_string(&state));
    sim.z(q); // Followed by Z. So, no op.
    let state = sim.capture_quantum_state();
    expect!["|0⟩: 0.0000+1.0000𝑖 "].assert_eq(&state_to_string(&state));
}
