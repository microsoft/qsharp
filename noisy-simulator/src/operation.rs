// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the `Operation` struct and the `operation!` macro
//! to conveniently construct operations from Kraus matrices.

#[cfg(test)]
mod tests;
use crate::SquareMatrix;

/// A helper macro to write operations more conveniently.
///
/// Example usage:
/// ```
/// // Create operation from two 2x2 Kraus matrices.
/// use noisy_simulator::{Operation, operation};
///
/// let op = operation!(
///     [1., 0.;
///      0., 0.;],
///     [0., 0.;
///      0., 0.;]
/// );
/// ```
#[macro_export]
macro_rules! operation {
    ($([$($($v:expr),* );*]),*) => {
        Operation::new(vec![
            $(nalgebra::dmatrix![
                $($(num_complex::Complex::<f64>::from($v)),* );*
            ]),*
        ])
    };
}

/// A quantum operation is a linear transformation that maps a valid density
/// matrix to another valid density matrices.
#[derive(Clone)]
pub struct Operation {
    number_of_qubits: usize,
    kraus_operators: Vec<SquareMatrix>,
    operation_matrix: SquareMatrix,
    effect_matrix: SquareMatrix,
    effect_matrix_transpose: SquareMatrix,
}

impl Operation {
    /// Construct an operation from a list of Kraus operators.
    /// Matrices must be of dimension 2^k x 2^k, where k is an integer.
    /// Returns `None` if the
    pub fn new(kraus_operators: Vec<SquareMatrix>) -> Self {
        let (dim, _) = kraus_operators
            .first()
            .expect("there should be at least one Kraus Operator")
            .shape();

        let number_of_qubits = (dim as f64).log2() as usize;
        assert!(
            1 << number_of_qubits == dim,
            "kraus operators should have dimensions 2^k x 2^k"
        );

        for kraus_operator in kraus_operators.iter() {
            let (rows, cols) = kraus_operator.shape();
            assert!(
                rows == dim && cols == dim,
                "kraus operators should be square matrices and have the same dimensions"
            );
        }

        let effect_matrix: SquareMatrix = kraus_operators.iter().map(|k| k.adjoint() * k).sum();

        let operation_matrix: SquareMatrix = kraus_operators
            .iter()
            .map(|k| k.kronecker(&k.conjugate()))
            .sum();

        let effect_matrix_transpose = effect_matrix.transpose();

        Self {
            number_of_qubits,
            kraus_operators,
            operation_matrix,
            effect_matrix,
            effect_matrix_transpose,
        }
    }

    /// Return matrix representation:
    /// $$ \sum_i K_i \otimes K_{i}* $$
    /// where $K_i$ are Kraus operators.
    pub fn matrix(&self) -> &SquareMatrix {
        &self.operation_matrix
    }

    /// Return effect matrix:
    /// $$ (\sum_i K_i^{\dagger} K_i) $$
    pub fn effect_matrix(&self) -> &SquareMatrix {
        &self.effect_matrix
    }

    /// Return transpose of effect matrix:
    /// $$ (\sum_i K_i^{\dagger} K_i)^T $$
    pub fn effect_matrix_transpose(&self) -> &SquareMatrix {
        &self.effect_matrix_transpose
    }

    /// Return list of Kraus operators.
    pub fn kraus_operators(&self) -> &Vec<SquareMatrix> {
        &self.kraus_operators
    }

    /// Return the number of qubits that the operation acts on.
    pub fn number_of_qubits(&self) -> usize {
        self.number_of_qubits
    }
}

#[cfg(test)]
pub(crate) use operation;
