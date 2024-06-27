// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module is a thin `PyO3` wrapper around the rust `noisy_simulator` crate.

use noisy_simulator::{ComplexVector, SquareMatrix};
use num_complex::Complex;
use pyo3::prelude::*;

type PythonMatrix = Vec<Vec<Complex<f64>>>;

fn python_to_nalgebra_matrix(matrix: PythonMatrix) -> SquareMatrix {
    let nrows = matrix.len();
    let ncols = matrix[0].len();
    // Check that matrix is well formed.
    for row in &matrix {
        assert!(
            ncols == row.len(),
            "ill formed matrix, all rows should be the same length"
        );
    }
    // Move matrix into a linear container.
    let mut data = Vec::with_capacity(nrows * ncols);
    for mut row in matrix {
        data.append(&mut row);
    }
    SquareMatrix::from_row_iterator(nrows, ncols, data)
}

/// Performance Warning:
///  nalgebra stores its matrices in column major order, and we want to send it
///  to Python in row major order, this means that there will be lots of
///  cache-misses in the convertion from one format to another.
fn nalgebra_matrix_to_python_list(matrix: &SquareMatrix) -> Vec<Complex<f64>> {
    let (nrows, ncols) = matrix.shape();
    let mut list = Vec::with_capacity(nrows * ncols);
    for row in 0..nrows {
        for col in 0..ncols {
            list.push(matrix[(row, col)]);
        }
    }
    list
}

#[pyclass(name = "operation")]
#[derive(Clone)]
pub(crate) struct Operation(noisy_simulator::Operation);

#[pymethods]
impl Operation {
    #[new]
    pub fn new(kraus_operators: Vec<PythonMatrix>) -> Self {
        // Transform Python matrix to nalgebra matrix.
        let kraus_operators: Vec<SquareMatrix> = kraus_operators
            .into_iter()
            .map(python_to_nalgebra_matrix)
            .collect();

        Self(noisy_simulator::Operation::new(kraus_operators))
    }

    pub fn get_effect_matrix(&self) -> Vec<Complex<f64>> {
        nalgebra_matrix_to_python_list(self.0.effect_matrix())
    }

    pub fn get_operation_matrix(&self) -> Vec<Complex<f64>> {
        nalgebra_matrix_to_python_list(self.0.matrix())
    }

    pub fn get_kraus_operators(&self) -> Vec<Vec<Complex<f64>>> {
        let mut kraus_operators = Vec::new();
        for kraus_operator in self.0.kraus_operators() {
            kraus_operators.push(nalgebra_matrix_to_python_list(kraus_operator));
        }
        kraus_operators
    }
}

#[pyclass(name = "instrument")]
pub(crate) struct Instrument(noisy_simulator::Instrument);

#[pymethods]
impl Instrument {
    #[new]
    pub fn new(operations: Vec<Operation>) -> Self {
        let operations = operations.into_iter().map(|op| op.0).collect();
        Self(noisy_simulator::Instrument::new(operations))
    }
}

#[pyclass]
pub(crate) struct DensityMatrixSimulator(noisy_simulator::DensityMatrixSimulator);

#[pymethods]
impl DensityMatrixSimulator {
    #[new]
    #[pyo3(signature = (number_of_qubits, seed=42))]
    #[allow(unused_variables)]
    pub fn new(number_of_qubits: usize, seed: usize) -> Self {
        Self(noisy_simulator::DensityMatrixSimulator::new(
            number_of_qubits,
        ))
    }

    /// Apply an arbitrary operation to given qubit ids.
    pub fn apply_operation(&mut self, operation: &Operation, qubits: Vec<usize>) {
        self.0.apply_operation(&operation.0, &qubits);
    }

    /// Apply non selective evolution.
    pub fn apply_instrument(&mut self, instrument: &Instrument, qubits: Vec<usize>) {
        self.0.apply_instrument(&instrument.0, &qubits);
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    ///
    /// Use this method to perform measurements on the quantum system.
    pub fn sample_instrument(&mut self, instrument: &Instrument, qubits: Vec<usize>) -> usize {
        self.0.sample_instrument(&instrument.0, &qubits)
    }

    /// For debugging and testing purposes.
    pub fn get_state(&self) -> Vec<Complex<f64>> {
        self.0.state().data().iter().copied().collect::<Vec<_>>()
    }

    /// For debugging and testing purposes.
    pub fn set_state(&mut self, state: Vec<Complex<f64>>) {
        let data = ComplexVector::from_vec(state);
        self.0.set_state_from_vec(data);
    }

    /// For debugging and testing purposes.
    pub fn set_trace(&mut self, trace: f64) {
        self.0.set_trace(trace);
    }
}

#[pyclass(name = "trajsimulator")]
pub(crate) struct TrajectorySimulator(noisy_simulator::TrajectorySimulator);

#[pymethods]
impl TrajectorySimulator {
    #[new]
    #[pyo3(signature = (number_of_qubits, seed=42))]
    #[allow(unused_variables)]
    pub fn new(number_of_qubits: usize, seed: usize) -> Self {
        Self(noisy_simulator::TrajectorySimulator::new(number_of_qubits))
    }

    /// Apply an arbitrary operation to given qubit ids.
    pub fn apply_operation(&mut self, operation: &Operation, qubits: Vec<usize>) {
        self.0.apply_operation(&operation.0, &qubits);
    }

    /// Apply non selective evolution.
    pub fn apply_instrument(&mut self, instrument: &Instrument, qubits: Vec<usize>) {
        self.0.apply_instrument(&instrument.0, &qubits);
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    ///
    /// Use this method to perform measurements on the quantum system.
    pub fn sample_instrument(&mut self, instrument: &Instrument, qubits: Vec<usize>) -> usize {
        self.0.sample_instrument(&instrument.0, &qubits)
    }

    /// For debugging and testing purposes.
    pub fn get_state(&self) -> Vec<Complex<f64>> {
        self.0.state().data().iter().copied().collect::<Vec<_>>()
    }

    /// For debugging and testing purposes.
    pub fn set_state(&mut self, state: Vec<Complex<f64>>) {
        let data = ComplexVector::from_vec(state);
        self.0.set_state_from_vec(data);
    }

    /// For debugging and testing purposes.
    pub fn set_trace(&mut self, trace: f64) {
        self.0.set_trace(trace);
    }
}
