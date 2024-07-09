// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    instrument::Instrument,
    operation::{operation, Operation},
    Error, NoisySimulator, TOLERANCE,
};
use num_complex::Complex;

/// Assert that two f64 are equal up to a `TOLERANCE`.
fn assert_approx_eq(left: f64, right: f64) {
    if (left - right).abs() > TOLERANCE {
        panic!("aprox_equal failed, left = {left}, right = {right}");
    }
}

/// Returns an H gate.
fn h() -> Operation {
    let f = 0.5_f64.sqrt();
    operation!([f,  f;
                f, -f;])
}

/// Returns a CNOT gate.
fn cnot() -> Operation {
    operation!([1., 0., 0., 0.;
                0., 1., 0., 0.;
                0., 0., 0., 1.;
                0., 0., 1., 0.;])
}

/// Returns an MZ measurement.
fn mz() -> Instrument {
    let m0 = operation!([1., 0.;
                         0., 0.;]);
    let m1 = operation!([0., 0.;
                         0., 1.;]);
    Instrument::new(vec![m0, m1])
}

pub fn measure_0<NS: NoisySimulator>() {
    let h = h();
    let mz = mz();
    let mut sim = NS::new(1);
    sim.apply_operation(&h, &[0])
        .expect("operation should succeed");
    let measurement = sim
        .sample_instrument_with_distribution(&mz, &[0], 0.3)
        .expect("measurement should succeed");
    assert_eq!(measurement, 0);
}

pub fn measure_1<NS: NoisySimulator>() {
    let h = h();
    let mz = mz();
    let mut sim = NS::new(1);
    sim.apply_operation(&h, &[0])
        .expect("operation should succeed");
    let measurement = sim
        .sample_instrument_with_distribution(&mz, &[0], 0.7)
        .expect("measurement should succeed");
    assert_eq!(measurement, 1);
}

/// Check that both measurements in a Bell Pair yield the same result.
pub fn bell_pair_sampling<NS: NoisySimulator>() {
    let (h, cnot, mz) = (h(), cnot(), mz());
    let mut sim = NS::new(2);

    // Make a Bell Pair.
    sim.apply_operation(&h, &[0])
        .expect("operation should succeed");
    sim.apply_operation(&cnot, &[1, 0])
        .expect("operation should succeed");

    // Measure both qubits.
    let m1 = sim
        .sample_instrument(&mz, &[0])
        .expect("measurement should succeed");
    let m2 = sim
        .sample_instrument(&mz, &[1])
        .expect("measurement should succeed");

    assert_eq!(m1, m2);
}

pub fn bell_pair_projection<NS: NoisySimulator>(outcome: usize) -> Result<(), Error> {
    assert!((0..4).contains(&outcome));
    let (h, cnot, mz) = (h(), cnot(), mz());
    let mut sim = NS::new(2);

    // Make a Bell Pair.
    sim.apply_operation(&h, &[0])
        .expect("operation should succeed");
    sim.apply_operation(&cnot, &[1, 0])
        .expect("operation should succeed");
    sim.apply_operation(mz.operation(outcome & 1), &[0])
        .expect("operation should succeed");

    if outcome == 0 || outcome == 3 {
        sim.apply_operation(mz.operation((outcome >> 1) & 1), &[1])?;
        assert_approx_eq(0.5, sim.trace_change()?);
        sim.apply_operation(mz.operation(outcome & 1), &[0])?;
        sim.apply_operation(mz.operation((outcome >> 1) & 1), &[1])?;
        assert_approx_eq(0.5, sim.trace_change()?);
    } else {
        sim.apply_operation(mz.operation((outcome >> 1) & 1), &[1])?;
    }

    Ok(())
}

pub fn two_qubit_gate<NS: NoisySimulator>(outcome: usize) -> Result<(), Error> {
    assert!((0..4).contains(&outcome));
    let h = h();
    let m0 = operation!([1., 0.;
                         0., 0.;]);
    let m1 = operation!([0., 0.;
                         0., 1.;]);
    let mz = mz();
    let probabilities: Vec<f64> = vec![0.05, 0.1, 0.3, 0.7, 0.8, 0.9, 0.99];
    let crx = |t: f64| {
        let c = t.cos();
        let s = t.sin() * Complex::I;
        operation!([1., 0., 0., 0.;
                    0., 1., 0., 0.;
                    0., 0., c,  s;
                    0., 0., s,  c;])
    };

    for p in &probabilities {
        let t = p.sqrt().acos();
        let b1 = (outcome & 1) != 0;
        let b2 = (outcome >> 1) != 0;

        let mut sim = NS::new(2);
        sim.apply_operation(&h, &[0])?;
        sim.apply_operation(&crx(0.3 * t), &[1, 0])?;
        sim.apply_operation(&crx(0.7 * t), &[1, 0])?;
        sim.apply_operation(if b1 { &m1 } else { &m0 }, &[0])?;

        if b1 {
            assert_approx_eq(0.5, sim.trace_change()?);
            sim.apply_operation(if b2 { &m1 } else { &m0 }, &[1])?;
            assert_approx_eq(0.5 * if b2 { 1. - p } else { *p }, sim.trace_change()?);
            sim.apply_operation(if b2 { &m1 } else { &m0 }, &[1])?;
            assert_approx_eq(0.5 * if b2 { 1. - p } else { *p }, sim.trace_change()?);
        } else {
            assert_eq!(0, sim.sample_instrument(&mz, &[1])?);
            assert_approx_eq(0.5, sim.trace_change()?);
            sim.apply_operation(&m1, &[1])?;
        }
    }

    Ok(())
}

/// Check that two consecutive MZ on the same qubit yields the same outcome.
pub fn repeated_mz<NS: NoisySimulator>() {
    let h = h();
    let mz = mz();
    let mut sim = NS::new(1);

    sim.apply_operation(&h, &[0])
        .expect("operation should succeed");
    let outcome_0 = sim
        .sample_instrument(&mz, &[0])
        .expect("measurement should succeed");
    let outcome_1 = sim
        .sample_instrument(&mz, &[0])
        .expect("measurement should succeed");
    assert_eq!(outcome_0, outcome_1);
}

pub fn alternating_mz_and_mx<NS: NoisySimulator>() {
    let h = h();
    let mz = mz();
    let mx = Instrument::new(vec![
        operation!([0.5, 0.5;
                    0.5, 0.5;]),
        operation!([ 0.5, -0.5;
                    -0.5,  0.5;]),
    ]);

    let mut sim = NS::new(1);
    sim.apply_operation(&h, &[0])
        .expect("operation should succeed");
    let mut prob = 1.0;

    // Alternate MZ and MX 5 times.
    for _ in 0..5 {
        sim.sample_instrument(&mz, &[0])
            .expect("measurement should succeed");
        prob *= 0.5;
        assert_approx_eq(prob, sim.trace_change().expect("state should be valid"));
        sim.sample_instrument(&mx, &[0])
            .expect("measurement should succeed");
        prob *= 0.5;
        assert_approx_eq(prob, sim.trace_change().expect("state should be valid"));
    }
}
