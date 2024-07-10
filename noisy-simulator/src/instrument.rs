// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the `Instrument` struct, used to make measurments
//! in a quantum system.

#[cfg(test)]
mod tests;

use crate::{operation::Operation, SquareMatrix, TOLERANCE};

/// An instrument is the means by which we make measurements on a quantum system.
pub struct Instrument {
    operations: Vec<Operation>,
    summed_operation: SquareMatrix,
    summed_effect: SquareMatrix,
    summed_effect_transpose: SquareMatrix,
    summed_kraus_operators: Vec<SquareMatrix>,
}

impl Instrument {
    /// Creates a new instrument.
    pub fn new(operations: Vec<Operation>) -> Self {
        let summed_operation: SquareMatrix = operations.iter().map(|op| op.matrix()).sum();
        let summed_effect: SquareMatrix = operations.iter().map(|op| op.effect_matrix()).sum();
        let summed_effect_transpose = summed_effect.transpose();
        let summed_kraus_operators = summed_kraus_operators(&operations);

        Self {
            operations,
            summed_operation,
            summed_effect,
            summed_effect_transpose,
            summed_kraus_operators,
        }
    }

    /// Return operation corresponding to the i-th outcome.
    pub fn operation(&self, i: usize) -> &Operation {
        &self.operations[i]
    }

    /// Return the matrix corresponding to the sum over all
    /// operations in this instrument:
    /// $$ \sum_i \sum_k K_{ik} \otimes (K_{ik})* $$
    /// where \otimes denotes the Kronecker product
    /// and * denotes the complex conjugate (entry-wise).
    pub fn non_selective_operation_matrix(&self) -> &SquareMatrix {
        &self.summed_operation
    }

    /// Return Kraus operators of the operation corresponding to non selective evolution.
    pub fn non_selective_kraus_operators(&self) -> &Vec<SquareMatrix> {
        &self.summed_kraus_operators
    }

    /// Return total effect $(\sum_i \sum_k K_{ik}^\dagger K_{ik})$.
    pub fn total_effect(&self) -> &SquareMatrix {
        &self.summed_effect
    }

    /// Return transposed total effect $(\sum_i \sum_k K_{ik}^\dagger K_{ik})^T$.
    pub fn total_effect_transposed(&self) -> &SquareMatrix {
        &self.summed_effect_transpose
    }

    /// Return number of operations/outcomes in this instrument.
    pub fn num_operations(&self) -> usize {
        self.operations.len()
    }
}

fn summed_kraus_operators(operations: &[Operation]) -> Vec<SquareMatrix> {
    let choi_matrix: SquareMatrix = operations
        .iter()
        .map(|op| {
            op.kraus_operators()
                .iter()
                .map(|m| {
                    // choi_matrix += vec(K) * vec(K)^dagger
                    let dim = m.shape().0.pow(2);
                    let mut choi = SquareMatrix::zeros(dim, dim);
                    for row in 0..dim {
                        for col in 0..dim {
                            choi[(row, col)] += m[m.vector_to_matrix_index(col)]
                                * m[m.vector_to_matrix_index(row)].conj();
                        }
                    }
                    choi
                })
                .sum::<SquareMatrix>()
        })
        .sum();

    let (choi_dim, _) = choi_matrix.shape();
    let eigen_decomposition = choi_matrix.symmetric_eigen();
    let eigenvectors = eigen_decomposition.eigenvectors;
    let eigenvalues = eigen_decomposition.eigenvalues;
    let mut summed_kraus_operators = Vec::new();

    for (col, eigenvalue) in eigenvalues.iter().enumerate() {
        if *eigenvalue > 0. {
            let (krows, kcols) = operations[0].kraus_operators()[0].shape();
            let mut kraus_operator = SquareMatrix::zeros(krows, kcols);
            let sqrt_eigenvalue = eigenvalue.sqrt();
            for row in 0..choi_dim {
                let idx = kraus_operator.vector_to_matrix_index(row);
                kraus_operator[idx] = sqrt_eigenvalue * eigenvectors[(row, col)];
            }
            summed_kraus_operators.push(kraus_operator);
        } else if *eigenvalue < -TOLERANCE {
            panic!("failed to decompose Choi matrix, lambda_i = {eigenvalue}");
        }
    }

    summed_kraus_operators
}