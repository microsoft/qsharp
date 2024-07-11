// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    operation::Operation, tests::assert_approx_eq_with_tolerance, NoisySimulator, SquareMatrix,
};
use nalgebra::dmatrix;
use num_complex::Complex;

use super::noiseless_tests;

const I: Complex<f64> = Complex::I;
const ZERO: Complex<f64> = Complex::ZERO;
const ONE: Complex<f64> = Complex::ONE;

fn pauli_identity() -> SquareMatrix {
    SquareMatrix::identity(2, 2)
}

fn pauli_x() -> SquareMatrix {
    dmatrix![ZERO, ONE;
             ONE,  ZERO;]
}

fn pauli_y() -> SquareMatrix {
    dmatrix![ZERO, -I;
             I,     ZERO;]
}

fn pauli_z() -> SquareMatrix {
    dmatrix![ONE,   ZERO;
             ZERO, -ONE;]
}

/// Returns a noisy identity gate.
fn depolarizing_channel(lambda: f64) -> Operation {
    // 0 <= 𝜆 <= 1 + 1 / (d^2 - 1)
    const LAMBDA_UPPER_BOUND: f64 = 1. + 1. / (4. - 1.);
    assert!((0. ..=LAMBDA_UPPER_BOUND).contains(&lambda));

    let lambda: Complex<f64> = lambda.into();

    // Define kraus matrices of the depolarizing channel.
    let k0 = pauli_identity() * (1. - 3. * lambda / 4.).sqrt();
    let k1 = pauli_x() * (lambda / 4.).sqrt();
    let k2 = pauli_y() * (lambda / 4.).sqrt();
    let k3 = pauli_z() * (lambda / 4.).sqrt();

    Operation::new(vec![k0, k1, k2, k3]).expect("operation should be valid")
}

pub fn check_noisy_identity_yields_same_qubit_with_right_probability<NS: NoisySimulator>() {
    let depolarizing_channel = depolarizing_channel(0.1);
    let mz = noiseless_tests::noiseless_mz();

    let mut total_outcome: f64 = 0.0;

    // Run 1000 simulations, on average we should measure the wrong outcome, i.e. 1,
    // 5% of the times (𝜆 / 2).
    const SHOTS: u64 = 500_000;

    for seed in 0..SHOTS {
        let mut sim = NS::new_with_seed(1, seed);
        sim.apply_operation(&depolarizing_channel, &[0])
            .expect("operation should succeed");
        total_outcome += sim
            .sample_instrument(&mz, &[0])
            .expect("operation should succeed") as f64;
    }

    total_outcome /= SHOTS as f64;
    assert_approx_eq_with_tolerance(0.05, total_outcome, 0.001);
}
