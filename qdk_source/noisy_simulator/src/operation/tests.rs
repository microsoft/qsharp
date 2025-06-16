// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use nalgebra::dmatrix;
use num_complex::{Complex, ComplexFloat};

use crate::{
    operation::{operation, Operation},
    tests::assert_approx_eq,
    SquareMatrix,
};

/// Constructs an operation using dense kraus matrices, to allow an exhaustive
/// testing of the inner linear operations used to construct the `Operator`.
fn dense_operation() -> Operation {
    const I: Complex<f64> = Complex::I;

    // Construct an operation from two kraus matrices.
    operation!(
        [
            0., 1., 2., 4.;
            1., 2., 3., 4.;
            1., 1., 3., 5.;
            3., 1., 1., 2.;
        ],
        [
            0.1 * I,       1. + I, 2., 4.;
            1.,            2. - I, 3., 4.;
            1. + 0.50 * I, 1.,     3., 5.;
            3. + 0.25 * I, 1.,     1., 2.;
        ]
    )
    .expect("operation should be valid")
}

#[test]
fn check_operation_number_of_qubits_is_computed_correctly() {
    let op = operation!(
        [
            0., 0.;
            0., 0.;
        ]
    )
    .expect("operation should be valid");

    assert_eq!(1, op.number_of_qubits());
}

#[test]
fn check_non_square_kraus_operator_does_not_panic() {
    let op = operation!(
        [
            0., 0.;
        ]
    );

    assert!(matches!(
        op,
        Err(crate::Error::FailedToConstructOperation(_))
    ));
}

/// Check that the inner matrices of the instrument are constructed correctly.
#[test]
fn check_effect_matrix_is_computed_correctly() {
    const I: Complex<f64> = Complex::I;
    let op = dense_operation();

    // Check that the effect matrix is the sum of the individual effect matrices
    // of each kraus operator.
    let eff0 = dmatrix![
        11.,  6.,  9., 15.;
         6.,  7., 12., 19.;
         9., 12., 23., 37.;
        15., 19., 37., 61.;
    ]
    .map(std::convert::Into::into);

    let eff1 = dmatrix! [
        11.3225 + 0.   * I, 6.1 - 1.85 * I, 9.  - 1.95 * I, 15. - 3.4 * I;
        6.1     + 1.85 * I, 9.  + 0.   * I, 12. + 1.   * I, 19. + 0.  * I;
        9.      + 1.95 * I, 12. - 1.   * I, 23. + 0.   * I, 37. + 0.  * I;
        15.     + 3.4  * I, 19. + 0.   * I, 37. + 0.   * I, 61. + 0.  * I;
    ];

    let eff = eff0 + eff1;

    for (x0, x1) in eff.iter().zip(op.effect_matrix().iter()) {
        assert_approx_eq(0., (x0 - x1.conj()).abs());
    }
}

#[test]
fn check_operation_matrix_is_computed_correctly() {
    const I: Complex<f64> = Complex::I;
    let op = dense_operation();

    // Check that the operation matrix is the sum of the individual operation matrices
    // of each kraus operator.
    let op0: SquareMatrix = dmatrix![
        0.,  0.,  0.,  0. ,  0.,  1.,  2.,  4. ,  0.,  2.,  4.,  8. ,  0. ,  4. ,  8. ,  16.;
        0.,  0.,  0.,  0. ,  1.,  2.,  3.,  4. ,  2.,  4.,  6.,  8. ,  4. ,  8. ,  12.,  16.;
        0.,  0.,  0.,  0. ,  1.,  1.,  3.,  5. ,  2.,  2.,  6.,  10.,  4. ,  4. ,  12.,  20.;
        0.,  0.,  0.,  0. ,  3.,  1.,  1.,  2. ,  6.,  2.,  2.,  4. ,  12.,  4. ,  4. ,  8. ;
        0.,  1.,  2.,  4. ,  0.,  2.,  4.,  8. ,  0.,  3.,  6.,  12.,  0. ,  4. ,  8. ,  16.;
        1.,  2.,  3.,  4. ,  2.,  4.,  6.,  8. ,  3.,  6.,  9.,  12.,  4. ,  8. ,  12.,  16.;
        1.,  1.,  3.,  5. ,  2.,  2.,  6.,  10.,  3.,  3.,  9.,  15.,  4. ,  4. ,  12.,  20.;
        3.,  1.,  1.,  2. ,  6.,  2.,  2.,  4. ,  9.,  3.,  3.,  6. ,  12.,  4. ,  4. ,  8. ;
        0.,  1.,  2.,  4. ,  0.,  1.,  2.,  4. ,  0.,  3.,  6.,  12.,  0. ,  5. ,  10.,  20.;
        1.,  2.,  3.,  4. ,  1.,  2.,  3.,  4. ,  3.,  6.,  9.,  12.,  5. ,  10.,  15.,  20.;
        1.,  1.,  3.,  5. ,  1.,  1.,  3.,  5. ,  3.,  3.,  9.,  15.,  5. ,  5. ,  15.,  25.;
        3.,  1.,  1.,  2. ,  3.,  1.,  1.,  2. ,  9.,  3.,  3.,  6. ,  15.,  5. ,  5. ,  10.;
        0.,  3.,  6.,  12.,  0.,  1.,  2.,  4. ,  0.,  1.,  2.,  4. ,  0. ,  2. ,  4. ,  8. ;
        3.,  6.,  9.,  12.,  1.,  2.,  3.,  4. ,  1.,  2.,  3.,  4. ,  2. ,  4. ,  6. ,  8. ;
        3.,  3.,  9.,  15.,  1.,  1.,  3.,  5. ,  1.,  1.,  3.,  5. ,  2. ,  2. ,  6. ,  10.;
        9.,  3.,  3.,  6. ,  3.,  1.,  1.,  2. ,  3.,  1.,  1.,  2. ,  6. ,  2. ,  2. ,  4. ;
    ]
    .map(std::convert::Into::into);

    let op1 = dmatrix![
        0.01   + 0.   * I, 0.1  + 0.1  * I, 0. + 0.2  * I, 0.  + 0.4  * I, 0.1  - 0.1  * I, 2. + 0. * I, 2. + 2. * I, 4.  + 4. * I, 0. - 0.2  * I, 2. - 2. * I, 4. + 0. * I, 8.  + 0. * I, 0.  - 0.4  * I, 4.  - 4. * I, 8.  + 0. * I, 16. + 0. * I;
        0.     + 0.1  * I, -0.1 + 0.2  * I, 0. + 0.3  * I, 0.  + 0.4  * I, 1.   + 1.   * I, 1. + 3. * I, 3. + 3. * I, 4.  + 4. * I, 2. + 0.   * I, 4. + 2. * I, 6. + 0. * I, 8.  + 0. * I, 4.  + 0.   * I, 8.  + 4. * I, 12. + 0. * I, 16. + 0. * I;
        0.05   + 0.1  * I, 0.   + 0.1  * I, 0. + 0.3  * I, 0.  + 0.5  * I, 1.5  + 0.5  * I, 1. + 1. * I, 3. + 3. * I, 5.  + 5. * I, 2. - 1.   * I, 2. + 0. * I, 6. + 0. * I, 10. + 0. * I, 4.  - 2.   * I, 4.  + 0. * I, 12. + 0. * I, 20. + 0. * I;
        0.025  + 0.3  * I, 0.   + 0.1  * I, 0. + 0.1  * I, 0.  + 0.2  * I, 3.25 + 2.75 * I, 1. + 1. * I, 1. + 1. * I, 2.  + 2. * I, 6. - 0.5  * I, 2. + 0. * I, 2. + 0. * I, 4.  + 0. * I, 12. - 1.   * I, 4.  + 0. * I, 4.  + 0. * I, 8.  + 0. * I;
        0.     - 0.1  * I, 1.   - 1.   * I, 2. + 0.   * I, 4.  + 0.   * I, -0.1 - 0.2  * I, 1. - 3. * I, 4. - 2. * I, 8.  - 4. * I, 0. - 0.3  * I, 3. - 3. * I, 6. + 0. * I, 12. + 0. * I, 0.  - 0.4  * I, 4.  - 4. * I, 8.  + 0. * I, 16. + 0. * I;
        1.     + 0.   * I, 2.   + 1.   * I, 3. + 0.   * I, 4.  + 0.   * I, 2.   - 1.   * I, 5. + 0. * I, 6. - 3. * I, 8.  - 4. * I, 3. + 0.   * I, 6. + 3. * I, 9. + 0. * I, 12. + 0. * I, 4.  + 0.   * I, 8.  + 4. * I, 12. + 0. * I, 16. + 0. * I;
        1.     - 0.5  * I, 1.   + 0.   * I, 3. + 0.   * I, 5.  + 0.   * I, 1.5  - 2.   * I, 2. - 1. * I, 6. - 3. * I, 10. - 5. * I, 3. - 1.5  * I, 3. + 0. * I, 9. + 0. * I, 15. + 0. * I, 4.  - 2.   * I, 4.  + 0. * I, 12. + 0. * I, 20. + 0. * I;
        3.     - 0.25 * I, 1.   + 0.   * I, 1. + 0.   * I, 2.  + 0.   * I, 5.75 - 3.5  * I, 2. - 1. * I, 2. - 1. * I, 4.  - 2. * I, 9. - 0.75 * I, 3. + 0. * I, 3. + 0. * I, 6.  + 0. * I, 12. - 1.   * I, 4.  + 0. * I, 4.  + 0. * I, 8.  + 0. * I;
        0.05   - 0.1  * I, 1.5  - 0.5  * I, 2. + 1.   * I, 4.  + 2.   * I, 0.   - 0.1  * I, 1. - 1. * I, 2. + 0. * I, 4.  + 0. * I, 0. - 0.3  * I, 3. - 3. * I, 6. + 0. * I, 12. + 0. * I, 0.  - 0.5  * I, 5.  - 5. * I, 10. + 0. * I, 20. + 0. * I;
        1.     + 0.5  * I, 1.5  + 2.   * I, 3. + 1.5  * I, 4.  + 2.   * I, 1.   + 0.   * I, 2. + 1. * I, 3. + 0. * I, 4.  + 0. * I, 3. + 0.   * I, 6. + 3. * I, 9. + 0. * I, 12. + 0. * I, 5.  + 0.   * I, 10. + 5. * I, 15. + 0. * I, 20. + 0. * I;
        1.25   + 0.   * I, 1.   + 0.5  * I, 3. + 1.5  * I, 5.  + 2.5  * I, 1.   - 0.5  * I, 1. + 0. * I, 3. + 0. * I, 5.  + 0. * I, 3. - 1.5  * I, 3. + 0. * I, 9. + 0. * I, 15. + 0. * I, 5.  - 2.5  * I, 5.  + 0. * I, 15. + 0. * I, 25. + 0. * I;
        3.125  + 1.25 * I, 1.   + 0.5  * I, 1. + 0.5  * I, 2.  + 1.   * I, 3.   - 0.25 * I, 1. + 0. * I, 1. + 0. * I, 2.  + 0. * I, 9. - 0.75 * I, 3. + 0. * I, 3. + 0. * I, 6.  + 0. * I, 15. - 1.25 * I, 5.  + 0. * I, 5.  + 0. * I, 10. + 0. * I;
        0.025  - 0.3  * I, 3.25 - 2.75 * I, 6. + 0.5  * I, 12. + 1.   * I, 0.   - 0.1  * I, 1. - 1. * I, 2. + 0. * I, 4.  + 0. * I, 0. - 0.1  * I, 1. - 1. * I, 2. + 0. * I, 4.  + 0. * I, 0.  - 0.2  * I, 2.  - 2. * I, 4.  + 0. * I, 8.  + 0. * I;
        3.     + 0.25 * I, 5.75 + 3.5  * I, 9. + 0.75 * I, 12. + 1.   * I, 1.   + 0.   * I, 2. + 1. * I, 3. + 0. * I, 4.  + 0. * I, 1. + 0.   * I, 2. + 1. * I, 3. + 0. * I, 4.  + 0. * I, 2.  + 0.   * I, 4.  + 2. * I, 6.  + 0. * I, 8.  + 0. * I;
        3.125  - 1.25 * I, 3.   + 0.25 * I, 9. + 0.75 * I, 15. + 1.25 * I, 1.   - 0.5  * I, 1. + 0. * I, 3. + 0. * I, 5.  + 0. * I, 1. - 0.5  * I, 1. + 0. * I, 3. + 0. * I, 5.  + 0. * I, 2.  - 1.   * I, 2.  + 0. * I, 6.  + 0. * I, 10. + 0. * I;
        9.0625 + 0.   * I, 3.   + 0.25 * I, 3. + 0.25 * I, 6.  + 0.5  * I, 3.   - 0.25 * I, 1. + 0. * I, 1. + 0. * I, 2.  + 0. * I, 3. - 0.25 * I, 1. + 0. * I, 1. + 0. * I, 2.  + 0. * I, 6.  - 0.5  * I, 2.  + 0. * I, 2.  + 0. * I, 4.  + 0. * I;
    ];

    let op_matrix = op0 + op1;

    for (x0, x1) in op.matrix().transpose().iter().zip(op_matrix.iter()) {
        assert_approx_eq(0., (x0 - x1).abs());
    }
}
