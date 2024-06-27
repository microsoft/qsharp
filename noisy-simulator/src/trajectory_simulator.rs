// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module contains two structs: the `TrajectorySimulator` and its
//! internal `StateVector` state.

#[cfg(test)]
mod tests;

use crate::{
    instrument::Instrument, kernel::apply_kernel, operation::Operation, ComplexVector,
    SquareMatrix, TOLERANCE,
};

/// A vector representing the state of a quantum system.
pub struct StateVector {
    /// Dimension of the vector.
    dim: usize,
    /// Number of qubits in the system.
    number_of_qubits: usize,
    /// Theoretical change in trace due to operations that have been applied so far.
    trace_change: f64,
    /// Vector storing the entries of the density matrix.
    data: ComplexVector,
}

impl StateVector {
    fn new(number_of_qubits: usize) -> Self {
        let dim = 1 << number_of_qubits;
        let mut state_vector = ComplexVector::zeros(dim);
        state_vector[0].re = 1.0;
        Self {
            dim,
            number_of_qubits,
            trace_change: 1.0,
            data: state_vector,
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

    /// Returns `true` if the squared L2 norm of the matrix is 1.
    fn is_normalized(&self) -> bool {
        (self.norm_squared() - 1.0).abs() <= TOLERANCE
    }

    /// Returns the squared L2 norm of the matrix.
    fn norm_squared(&self) -> f64 {
        self.data.norm_squared()
    }

    /// Return theoretical change in trace due to operations that have been applied so far.
    /// In reality, the density matrix is always renormalized after instruments / operations
    /// have been applied.
    fn trace_change(&self) -> f64 {
        self.trace_change
    }

    /// Renormalizes the matrix such that the trace is 1.
    fn renormalize(&mut self) {
        let norm_squared = self.norm_squared();
        assert!(norm_squared >= TOLERANCE, "arrived at probability-0 event");
        let renormalization_factor = 1.0 / norm_squared.sqrt();
        for entry in self.data.iter_mut() {
            *entry *= renormalization_factor;
        }
    }

    /// Renormalizes the matrix such that the trace is 1. Uses a precomputed `norm_squared`.
    fn renormalize_with_norm(&mut self, norm_squared: f64) {
        assert!(norm_squared >= TOLERANCE, "arrived at probability-0 event");
        let renormalization_factor = 1.0 / norm_squared.sqrt();
        for entry in self.data.iter_mut() {
            *entry *= renormalization_factor;
        }
    }

    /// Return the probability of a given effect.
    fn effect_probability(&self, effect_matrix: &SquareMatrix, qubits: &[usize]) -> f64 {
        let mut state_copy = self.data.clone();
        apply_kernel(&mut state_copy, effect_matrix, qubits);
        state_copy.dot(&self.data.conjugate()).re
    }

    fn sample_kraus_operators(
        &mut self,
        kraus_operators: &[SquareMatrix],
        qubits: &[usize],
        renormalization_factor: f64,
        random_sample: f64,
    ) {
        let mut summed_probability = 0.0;
        let mut last_non_zero_probability = 0.0;
        let mut last_non_zero_probability_index = 0;

        for (i, kraus_operator) in kraus_operators.iter().enumerate() {
            let mut state_copy = self.data.clone();
            apply_kernel(&mut state_copy, kraus_operator, qubits);
            let norm_squared = state_copy.norm_squared();
            let p = norm_squared / renormalization_factor;
            summed_probability += p;
            if p >= TOLERANCE {
                last_non_zero_probability = p;
                last_non_zero_probability_index = i;
                if summed_probability > random_sample {
                    self.data = state_copy;
                    self.renormalize_with_norm(norm_squared);
                    return;
                }
            }
        }
        assert!(
            summed_probability + TOLERANCE > random_sample
                && last_non_zero_probability >= TOLERANCE,
            "numerical error; failed to sample Kraus operators"
        );
        apply_kernel(
            &mut self.data,
            &kraus_operators[last_non_zero_probability_index],
            qubits,
        );
        self.renormalize();
    }
}

/// A quantum circuit simulator using a state vector.
pub struct TrajectorySimulator {
    state: StateVector,
}

impl TrajectorySimulator {
    /// Creates a new `TrajectorySimulator`.
    pub fn new(number_of_qubits: usize) -> Self {
        Self {
            state: StateVector::new(number_of_qubits),
        }
    }

    /// Apply an operation to given qubit ids.
    pub fn apply_operation(&mut self, operation: &Operation, qubits: &[usize]) {
        let renormalization_factor = self
            .state
            .effect_probability(operation.effect_matrix(), qubits);
        self.state.trace_change *= renormalization_factor;
        self.state.sample_kraus_operators(
            operation.kraus_operators(),
            qubits,
            renormalization_factor,
            rand::random(),
        );
    }

    /// Apply non selective evolution.
    pub fn apply_instrument(&mut self, instrument: &Instrument, qubits: &[usize]) {
        let renormalization_factor = self
            .state
            .effect_probability(instrument.total_effect(), qubits);
        self.state.trace_change *= renormalization_factor;
        self.state.sample_kraus_operators(
            instrument.non_selective_kraus_operators(),
            qubits,
            renormalization_factor,
            rand::random(),
        );
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    ///
    /// Use this method to perform measurements on the quantum system.
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
        let renormalization_factor = self
            .state
            .effect_probability(instrument.total_effect(), qubits);
        let mut last_non_zero_norm_squared = 0.0;
        let mut summed_probability = 0.0;
        let mut last_non_zero_outcome = 0;
        for outcome in 0..instrument.num_operations() {
            let norm_squared = self
                .state
                .effect_probability(instrument.operation(outcome).effect_matrix(), qubits);
            let p = norm_squared / renormalization_factor;
            if p >= TOLERANCE {
                last_non_zero_outcome = outcome;
                last_non_zero_norm_squared = norm_squared;
            }
            summed_probability += p;
            if summed_probability > random_sample {
                break;
            }
        }

        assert!(
            summed_probability + TOLERANCE > random_sample
                && last_non_zero_norm_squared >= TOLERANCE,
            "Numerical error? No outcome found when sampling instrument."
        );
        self.state.trace_change *= last_non_zero_norm_squared;
        let rescaled_random_sample = ((summed_probability - random_sample)
            / last_non_zero_norm_squared
            * renormalization_factor)
            .max(0.0);
        self.state.sample_kraus_operators(
            instrument
                .operation(last_non_zero_outcome)
                .kraus_operators(),
            qubits,
            last_non_zero_norm_squared,
            rescaled_random_sample,
        );
        last_non_zero_outcome
    }

    /// For debugging and testing purposes.
    pub fn state(&self) -> &StateVector {
        &self.state
    }

    /// For debugging and testing purposes.
    pub fn set_state(&mut self, state: StateVector) {
        assert!(
            self.state.dim() == state.dim(),
            "`state` is of the wrong size {} != {}",
            self.state.dim(),
            state.dim(),
        );
        assert!(
            state.is_normalized(),
            "`state` is not normalized, norm_squared is {}",
            state.norm_squared()
        );

        self.state = state;
    }

    /// Return theoretical change in trace due to operations that have been applied so far
    /// In reality, the density matrix is always renormalized after instruments/operations
    /// have been applied.
    pub fn trace_change(&self) -> f64 {
        self.state.trace_change()
    }
}
