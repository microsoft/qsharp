// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use crate::{
    instrument::Instrument, kernel::apply_kernel, operation::Operation, ComplexVector,
    SquareMatrix, TOLERANCE,
};
use num_complex::Complex;

/// A vectorized density matrix.
#[derive(Debug, Clone)]
pub struct DensityMatrix {
    /// Dimension of the matrix. E.g.: If the matrix is 5 x 5, then dim is 5.
    dim: usize,
    /// Number of qubits in the system.
    number_of_qubits: usize,
    /// Theoretical change in trace due to operations that have been applied so far.
    trace_change: f64,
    /// Vector storing the entries of the density matrix.
    // TODO [FIX]: Remove pub from this field.
    pub data: ComplexVector,
}

impl DensityMatrix {
    fn new(number_of_qubits: usize) -> Self {
        let dim = 1 << number_of_qubits;
        let mut data = ComplexVector::zeros(dim * dim);
        data[0].re = 1.0;
        Self {
            dim,
            number_of_qubits,
            trace_change: 1.0,
            data,
        }
    }

    /// Returns dimension of the matrix. E.g.: If the matrix is 5 x 5, then dim is 5.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Returns the number of qubits in the system.
    pub fn number_of_qubits(&self) -> usize {
        self.number_of_qubits
    }

    /// Returns `true` if the matrix is Hermitian.
    fn is_hermitian(&self) -> bool {
        for row in 0..self.dim {
            for col in 0..self.dim {
                let elt = self.data[self.dim * row + col];
                let mirror_elt = self.data[self.dim * col + row];
                if (elt.re - mirror_elt.re).abs() > TOLERANCE
                    || (elt.im + mirror_elt.im).abs() > TOLERANCE
                {
                    return false;
                }
            }
        }
        true
    }

    /// Returns `true` if the trace of the matrix is 1.
    fn is_normalized(&self) -> bool {
        (self.trace() - 1.0).abs() <= TOLERANCE
    }

    /// Returns the trace of the matrix. The trace is the sum of the diagonal entries of a matrix.
    fn trace(&self) -> f64 {
        let mut trace: Complex<f64> = Complex::ZERO;
        for idx in 0..self.dim {
            trace += self.data[(self.dim + 1) * idx];
        }
        assert!(
            trace.im <= TOLERANCE,
            "state trace is not real, imaginary part is {}",
            trace.im
        );
        trace.re
    }

    /// Return theoretical change in trace due to operations that have been applied so far.
    /// In reality, the density matrix is always renormalized after instruments / operations
    /// have been applied.
    fn trace_change(&self) -> f64 {
        self.trace_change
    }

    /// Renormalizes the matrix such that the trace is 1.
    fn renormalize(&mut self) {
        let trace = self.trace();
        assert!(trace >= TOLERANCE, "arrived at probability-0 event");
        self.trace_change *= trace;
        let renormalization_factor = 1.0 / trace;
        for entry in self.data.iter_mut() {
            *entry *= renormalization_factor;
        }
    }

    /// Renormalizes the matrix such that the trace is 1. Uses a precomputed `trace`.
    fn renormalize_with_trace(&mut self, trace: f64) {
        assert!(trace >= TOLERANCE, "arrived at probability-0 event");
        self.trace_change *= trace;
        let renormalization_factor = 1.0 / trace;
        for entry in self.data.iter_mut() {
            *entry *= renormalization_factor;
        }
    }

    /// TODO: write docstring
    fn apply_operation_matrix(&mut self, operation_matrix: &SquareMatrix, qubits: &[usize]) {
        // TODO [Research]: Figure out why they do this qubits_expanded thing.
        let mut qubits_expanded = Vec::with_capacity(2 * qubits.len());
        for id in qubits {
            qubits_expanded.push(*id);
        }
        for id in qubits {
            qubits_expanded.push(*id + self.number_of_qubits());
        }
        apply_kernel(&mut self.data, operation_matrix, &qubits_expanded);
    }
}

pub struct DensityMatrixSimulator {
    state: DensityMatrix,
}

impl DensityMatrixSimulator {
    pub fn new(number_of_qubits: usize) -> Self {
        Self {
            state: DensityMatrix::new(number_of_qubits),
        }
    }

    /// Apply an arbitrary operation to given qubit ids.
    pub fn apply_operation(&mut self, operation: &Operation, qubits: &[usize]) {
        self.state
            .apply_operation_matrix(operation.matrix(), qubits);
        self.state.renormalize();
    }

    /// Apply non selective evolution.
    pub fn apply_instrument(&mut self, instrument: &Instrument, qubits: &[usize]) {
        self.state
            .apply_operation_matrix(instrument.non_selective_operation_matrix(), qubits);
        self.state.renormalize();
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    pub fn sample_instrument(&mut self, instrument: &Instrument, qubits: &[usize]) -> usize {
        self.sample_instrument_with_distribution(instrument, qubits, rand::random())
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    pub fn sample_instrument_with_distribution(
        &mut self,
        instrument: &Instrument,
        qubits: &[usize],
        random_sample: f64,
    ) -> usize {
        let mut tmp_state = self.state.clone();
        apply_kernel(
            &mut tmp_state.data,
            instrument.total_effect_transposed(),
            qubits,
        );
        let total_effect_trace = tmp_state.trace();
        assert!(
            total_effect_trace >= TOLERANCE,
            "arrived at probability-0 event"
        );
        let mut last_non_zero_trace_outcome: usize = 0;
        let mut last_non_zero_trace: f64 = 0.0;
        let mut summed_probability: f64 = 0.0;

        for outcome in 0..instrument.num_operations() {
            if summed_probability > random_sample {
                break;
            }
            tmp_state = self.state.clone();
            apply_kernel(
                &mut tmp_state.data,
                instrument.operation(outcome).effect_matrix_transpose(),
                qubits,
            );
            let outcome_trace = tmp_state.trace();
            summed_probability += outcome_trace / total_effect_trace;
            if outcome_trace >= TOLERANCE {
                last_non_zero_trace_outcome = outcome;
                last_non_zero_trace = outcome_trace;
            }
        }

        assert!(
            summed_probability + TOLERANCE > random_sample && last_non_zero_trace >= TOLERANCE,
            "Numerical error? No outcome found when sampling instrument."
        );
        self.state.apply_operation_matrix(
            instrument.operation(last_non_zero_trace_outcome).matrix(),
            qubits,
        );
        self.state.renormalize_with_trace(last_non_zero_trace);
        last_non_zero_trace_outcome
    }

    /// For debugging and testing purposes.
    pub fn state(&self) -> &DensityMatrix {
        &self.state
    }

    /// For debugging and testing purposes.
    pub fn set_state(&mut self, state: DensityMatrix) {
        assert!(
            self.state.dim() == state.dim(),
            "`state` is of the wrong size {} != {}",
            self.state.dim(),
            state.dim(),
        );
        assert!(
            state.is_normalized(),
            "`state` is not normalized, trace is {}",
            state.trace()
        );
        assert!(state.is_hermitian(), "`state` is not Hermitian");

        // TODO [Fix]: Check if state is positive semidefinite? Might be too expensive.
        self.state = state;
    }

    /// Return theoretical change in trace due to operations that have been applied so far
    /// In reality, the density matrix is always renormalized after instruments/operations
    /// have been applied.
    pub fn trace_change(&self) -> f64 {
        self.state.trace_change()
    }
}
