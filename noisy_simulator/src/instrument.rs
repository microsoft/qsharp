// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the `Instrument` struct, used to make measurments
//! in a quantum system.

#[cfg(test)]
mod tests;

use crate::{operation::Operation, Error, SquareMatrix, TOLERANCE};

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
    pub fn new(operations: Vec<Operation>) -> Result<Self, Error> {
        if operations.is_empty() {
            return Err(Error::FailedToConstructInstrument(
                "there should be at least one Operation".to_string(),
            ));
        }

        let number_of_qubits = operations
            .first()
            .expect("there should be at least an element")
            .number_of_qubits();

        if !operations
            .iter()
            .all(|op| number_of_qubits == op.number_of_qubits())
        {
            return Err(Error::FailedToConstructInstrument(
                "all Operations should target the same number of qubits".to_string(),
            ));
        }

        let summed_operation: SquareMatrix = operations.iter().map(Operation::matrix).sum();
        let summed_effect: SquareMatrix = operations.iter().map(Operation::effect_matrix).sum();
        let summed_effect_transpose = summed_effect.transpose();
        let summed_kraus_operators = summed_kraus_operators(&operations)?;

        Ok(Self {
            operations,
            summed_operation,
            summed_effect,
            summed_effect_transpose,
            summed_kraus_operators,
        })
    }

    /// Return operation corresponding to the i-th outcome.
    #[must_use]
    pub fn operation(&self, i: usize) -> &Operation {
        &self.operations[i]
    }

    /// Return the matrix corresponding to the sum over all
    /// operations in this instrument:
    /// Σᵢ Σₖ (Kᵢₖ ⊗ Kᵢₖ*)
    /// where ⊗ denotes the Kronecker product
    /// and * denotes the complex conjugate (entry-wise).
    #[must_use]
    pub fn non_selective_operation_matrix(&self) -> &SquareMatrix {
        &self.summed_operation
    }

    /// Return Kraus operators of the operation corresponding to non selective evolution.
    #[must_use]
    pub fn non_selective_kraus_operators(&self) -> &Vec<SquareMatrix> {
        &self.summed_kraus_operators
    }

    /// Return total effect Σᵢ Σₖ (Kᵢₖ† Kᵢₖ), where † denotes the adjoint.
    #[must_use]
    pub fn total_effect(&self) -> &SquareMatrix {
        &self.summed_effect
    }

    /// Return transposed total effect Σᵢ Σₖ (Kᵢₖ† Kᵢₖ)^T, where † denotes the adjoint.
    #[must_use]
    pub fn total_effect_transposed(&self) -> &SquareMatrix {
        &self.summed_effect_transpose
    }

    /// Return number of operations/outcomes in this instrument.
    #[must_use]
    pub fn num_operations(&self) -> usize {
        self.operations.len()
    }
}

fn summed_kraus_operators(operations: &[Operation]) -> Result<Vec<SquareMatrix>, Error> {
    // Since all the Kraus operators are square matrices of the same dimension
    // we can cache the `vector_to_matrix_index` computation beforehand.
    let vector_to_matrix_index = cache_vector_to_matrix_index(operations);

    let choi_matrix: SquareMatrix = operations
        .iter()
        .map(|op| {
            op.kraus_operators()
                .iter()
                .map(|k| {
                    // This code is doing the equivalent to:
                    // choi_matrix += vec(K) * vec(K*)
                    // where * denotes the complex conjugate
                    let dim = k.shape().0.pow(2);
                    let mut choi = SquareMatrix::zeros(dim, dim);
                    for row in 0..dim {
                        for col in 0..dim {
                            choi[(row, col)] += k[vector_to_matrix_index[col]]
                                * k[vector_to_matrix_index[row]].conj();
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
            return Err(Error::FailedToConstructInstrument(format!(
                "failed to decompose Choi matrix, lambda_i = {eigenvalue}"
            )));
        }
    }

    Ok(summed_kraus_operators)
}

/// Caches `vector_to_matrix_index` into a vector.
fn cache_vector_to_matrix_index(operations: &[Operation]) -> Vec<(usize, usize)> {
    operations
        .first()
        .map(|op| {
            if let Some(k) = op.kraus_operators().first() {
                let num_elements = k.shape().0.pow(2);
                (0..num_elements)
                    .map(|idx| k.vector_to_matrix_index(idx))
                    .collect()
            } else {
                Vec::default()
            }
        })
        .unwrap_or_default()
}
