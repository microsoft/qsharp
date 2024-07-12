// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Instrument;
use crate::{
    operation::{operation, Operation},
    tests::assert_approx_eq,
    SquareMatrix,
};
use nalgebra::ComplexField;
use rand::{rngs::StdRng, Rng, SeedableRng};

/// Seed for the random number generators.
const SEED: u64 = 42;

/// Check that the inner matrices of the instrument are constructed correctly.
#[test]
fn check_non_selective_operation_matrix_is_computed_correctly() {
    let mut rng = StdRng::seed_from_u64(SEED);
    let mut rng = || rng.gen::<f64>();

    let op0 = operation!([rng(), rng(); rng(), rng();]).expect("operation should be valid");
    let op1 = operation!([rng(), rng(); rng(), rng();]).expect("operation should be valid");
    let instrument = Instrument::new(vec![op0, op1]).expect("instrument should be valid");
    let sum = instrument.non_selective_operation_matrix();
    let op0 = instrument.operation(0);
    let op1 = instrument.operation(1);

    for row in 0..4 {
        for col in 0..4 {
            assert_approx_eq(
                0.,
                (sum[(row, col)] - (op0.matrix()[(row, col)] + op1.matrix()[(row, col)])).abs(),
            );
        }
    }
}

#[test]
fn check_non_selective_evolution_operator_is_computed_correctly() {
    let dim = 8;
    let mut rng = StdRng::seed_from_u64(SEED);
    let mut rng = || rng.gen::<f64>();

    // Create dim^2 random kraus operators.
    let kraus_operators: Vec<SquareMatrix> = (0..dim * dim)
        .map(|_| SquareMatrix::from_fn(dim, dim, |_, _| (0.5 - rng()).into()))
        .collect();
    let op0 = Operation::new(kraus_operators).expect("operation should be valid");
    let instrument_0 = Instrument::new(vec![op0]).expect("instrument should be valid");
    let kraus_operators: Vec<SquareMatrix> = instrument_0.non_selective_kraus_operators().to_vec();
    let op1 = Operation::new(kraus_operators).expect("operation should be valid");
    let instrument_1 = Instrument::new(vec![op1]).expect("instrument should be valid");
    let m0 = instrument_0.non_selective_operation_matrix();
    let m1 = instrument_1.non_selective_operation_matrix();

    for (x0, x1) in m0.iter().zip(m1.iter()) {
        assert_approx_eq(x0.re, x1.re);
        assert_approx_eq(x0.im, x1.im);
    }
}
