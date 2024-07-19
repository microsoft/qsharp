// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains the `apply_kernel` function used by the `DensityMatrixSimualtor`
//! and the `TrajectorySimulator`.

use crate::{ComplexVector, Error, SquareMatrix};
use nalgebra::Complex;

/// This function extracts the relevant entries from the `state_vector` into its own vector.
/// Then it applies the `operation_matrix` to this extracted entries.
/// Finally it stores the results back into the state vector.
///
/// Errors: If the `operation_matrix` doesn't have the right dimension for the number of target `qubits`,
/// this function will return `Error::MatrixVecDimensionMismatch`.
pub fn apply_kernel(
    state: &mut ComplexVector,
    operation_matrix: &SquareMatrix,
    qubits: &[usize],
) -> Result<(), Error> {
    // Construct a mask that has 1s at locations given by the target `qubits` ids.
    let mask = make_mask(state, qubits);

    // Number of elements in small matrix-vector multiplications (dimension of gate matrix).
    let num_elements: usize = 1 << qubits.len();
    let (nrows, ncols) = operation_matrix.shape();

    if num_elements != ncols {
        return Err(Error::MatrixVecDimensionMismatch {
            nrows,
            ncols,
            vec_dim: num_elements,
        });
    }

    // Compute all index offsets of the entries to load.
    // E.g., for a 2-qubit gate acting on qubits [k1,k2],
    // index offsets are [0, 2^k1, 2^k2, 2^(k1+k2)]
    let mut index_offsets: Vec<usize> = std::vec::Vec::with_capacity(num_elements);
    for k in 0..num_elements {
        let mut j = 0;
        let mut k_ = k;
        let mut idx = 0;
        while k_ != 0 {
            idx |= (k_ & 1) << qubits[j];
            k_ >>= 1;
            j += 1;
        }
        index_offsets.push(idx);
    }

    // Main loop.
    let mut extracted_entries = ComplexVector::zeros(num_elements);
    let mut new_entries = ComplexVector::zeros(num_elements);
    for s in 0..state.len() {
        if (s & mask) == 0 {
            // Extract relevant entries into a vector to make the gate application easier.
            for k in 0..num_elements {
                // SAFETY: index_offsets and extracted_entries have size `num_elements`,
                // so those get_unchecked are safe. state.get_unchecked(idx) is safe because:
                //  1. s has total_qubits_in_system bits
                //  2. index_offset has total_qubits_in_system bits
                //  3. Therefore, idx = s | index_offset also has total_qubits_in_system bits.
                //  4. idx < (1 << total_qubits_in_system) = state.len() because
                //     it has (total_qubits_in_system + 1) bits.
                //  5. Therefore state.get_unchecked(idx) is safe.
                let idx = s | unsafe { index_offsets.get_unchecked(k) };
                unsafe {
                    *extracted_entries.get_unchecked_mut(k) = *state.get_unchecked(idx);
                }
            }

            // Apply the gate.
            new_entries.gemv(
                Complex::ONE,
                operation_matrix,
                &extracted_entries,
                Complex::ZERO,
            );

            // Store accumulated result back into the state vector.
            for k in 0..num_elements {
                // SAFETY: index_offsets and new_entries have size `num_elements`,
                // so those get_unchecked are safe. state.get_unchecked(idx) is safe because:
                //  1. s has total_qubits_in_system bits
                //  2. index_offset has total_qubits_in_system bits
                //  3. Therefore, idx = s | index_offset also has total_qubits_in_system bits.
                //  4. idx < (1 << total_qubits_in_system) = state.len() because
                //     it has (total_qubits_in_system + 1) bits.
                //  5. Therefore state.get_unchecked(idx) is safe.
                let idx = s | unsafe { index_offsets.get_unchecked(k) };
                unsafe {
                    *state.get_unchecked_mut(idx) = *new_entries.get_unchecked(k);
                }
            }
        }
    }

    Ok(())
}

/// Construct a mask that has 1s at locations given by the target `qubits` ids.
fn make_mask(state: &ComplexVector, qubits: &[usize]) -> usize {
    // Number of elements in the density matrix.
    let num_elements = state.len();
    let mut mask: usize = 0;
    for id in qubits {
        let id_mask: usize = 1 << id;
        assert!(id_mask < num_elements, "invalid qubit id: {id}");
        mask |= id_mask;
    }
    mask
}
