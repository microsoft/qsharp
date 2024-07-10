// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module is a thin `PyO3` wrapper around the rust `noisy_simulator` crate.

use noisy_simulator::{ComplexVector, NoisySimulator, SquareMatrix};
use num_complex::Complex;
use pyo3::{exceptions::PyException, prelude::*};
type PythonMatrix = Vec<Vec<Complex<f64>>>;

pub(crate) fn register_noisy_simulator_submodule(py: Python, m: &PyModule) -> PyResult<()> {
    // let m = PyModule::new(py, "noisy_simulator")?;
    m.add("NoisySimulatorError", py.get_type::<NoisySimulatorError>())?;
    m.add_class::<Operation>()?;
    m.add_class::<Instrument>()?;
    m.add_class::<DensityMatrixSimulator>()?;
    m.add_class::<StateVectorSimulator>()?;
    // parent_module.add_submodule(m)?;
    Ok(())
}

/// Performance Warning:
///  nalgebra stores its matrices in column major order, and we want to send it
///  to Python in row major order, this means that there will be lots of
///  cache-misses in the convertion from one format to another.
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

pyo3::create_exception!(qsharp.noisy_sim, NoisySimulatorError, PyException);

#[pyclass]
#[derive(Clone)]
pub(crate) struct Operation(noisy_simulator::Operation);

#[pymethods]
impl Operation {
    #[new]
    pub fn new(kraus_operators: Vec<PythonMatrix>) -> Self {
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

    pub fn get_number_of_qubits(&self) -> usize {
        self.0.number_of_qubits()
    }
}

#[pyclass]
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
#[derive(Clone)]
pub(crate) struct DensityMatrix {
    /// Dimension of the matrix. E.g.: If the matrix is 5 x 5, then dimension is 5.
    dimension: usize,
    /// Number of qubits in the system.
    number_of_qubits: usize,
    /// Theoretical change in trace due to operations that have been applied so far.
    trace_change: f64,
    /// Vector storing the entries of the density matrix.
    data: Vec<Complex<f64>>,
}

#[pymethods]
impl DensityMatrix {
    /// Returns a copy of the matrix data.
    fn data(&self) -> Vec<Complex<f64>> {
        self.data.clone()
    }

    /// Returns the dimension of the matrix.
    fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the number of qubits in the system.
    fn number_of_qubits(&self) -> usize {
        self.number_of_qubits
    }
}

impl From<&noisy_simulator::DensityMatrix> for DensityMatrix {
    fn from(dm: &noisy_simulator::DensityMatrix) -> Self {
        Self {
            dimension: dm.dimension(),
            number_of_qubits: dm.number_of_qubits(),
            trace_change: dm.trace_change(),
            data: dm.data().iter().copied().collect::<Vec<_>>(),
        }
    }
}

impl TryInto<noisy_simulator::DensityMatrix> for DensityMatrix {
    type Error = PyErr;

    fn try_into(self) -> PyResult<noisy_simulator::DensityMatrix> {
        noisy_simulator::DensityMatrix::try_from(
            self.dimension,
            self.number_of_qubits,
            self.trace_change,
            ComplexVector::from_vec(self.data),
        )
        .ok_or(NoisySimulatorError::new_err("invalid density matrix"))
    }
}

#[pyclass]
pub(crate) struct DensityMatrixSimulator(noisy_simulator::DensityMatrixSimulator);

#[pymethods]
impl DensityMatrixSimulator {
    #[new]
    pub fn new(number_of_qubits: usize, seed: Option<u64>) -> Self {
        let mut simulator = noisy_simulator::DensityMatrixSimulator::new(number_of_qubits);

        if let Some(seed) = seed {
            simulator.set_rng_seed(seed);
        }

        Self(simulator)
    }

    /// Apply an arbitrary operation to given qubit ids.
    #[allow(clippy::needless_pass_by_value)]
    pub fn apply_operation(&mut self, operation: &Operation, qubits: Vec<usize>) -> PyResult<()> {
        self.0
            .apply_operation(&operation.0, &qubits)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Apply non selective evolution.
    #[allow(clippy::needless_pass_by_value)]
    pub fn apply_instrument(
        &mut self,
        instrument: &Instrument,
        qubits: Vec<usize>,
    ) -> PyResult<()> {
        self.0
            .apply_instrument(&instrument.0, &qubits)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    ///
    /// Use this method to perform measurements on the quantum system.
    #[allow(clippy::needless_pass_by_value)]
    pub fn sample_instrument(
        &mut self,
        instrument: &Instrument,
        qubits: Vec<usize>,
    ) -> PyResult<usize> {
        self.0
            .sample_instrument(&instrument.0, &qubits)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Returns the `DensityMatrix` if the simulator is in a valid state.
    pub fn get_state(&self) -> Option<DensityMatrix> {
        match self.0.state() {
            Ok(dm) => Some(dm.into()),
            Err(_) => None,
        }
    }

    /// Set state of the quantum system.
    pub fn set_state(&mut self, state: DensityMatrix) -> PyResult<()> {
        self.0
            .set_state(state.try_into()?)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Set the trace of the quantum system.
    pub fn set_trace(&mut self, trace: f64) -> PyResult<()> {
        self.0
            .set_trace(trace)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }
}

#[pyclass]
#[derive(Clone)]
pub(crate) struct StateVector {
    /// Dimension of the matrix. E.g.: If the matrix is 5 x 5, then dimension is 5.
    dimension: usize,
    /// Number of qubits in the system.
    number_of_qubits: usize,
    /// Theoretical change in trace due to operations that have been applied so far.
    trace_change: f64,
    /// Vector storing the entries of the density matrix.
    data: Vec<Complex<f64>>,
}

#[pymethods]
impl StateVector {
    /// Returns a copy of the matrix data.
    fn data(&self) -> Vec<Complex<f64>> {
        self.data.clone()
    }

    /// Returns the dimension of the matrix.
    fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the number of qubits in the system.
    fn number_of_qubits(&self) -> usize {
        self.number_of_qubits
    }
}

impl From<&noisy_simulator::StateVector> for StateVector {
    fn from(dm: &noisy_simulator::StateVector) -> Self {
        Self {
            dimension: dm.dimension(),
            number_of_qubits: dm.number_of_qubits(),
            trace_change: dm.trace_change(),
            data: dm.data().iter().copied().collect::<Vec<_>>(),
        }
    }
}

impl TryInto<noisy_simulator::StateVector> for StateVector {
    type Error = PyErr;

    fn try_into(self) -> PyResult<noisy_simulator::StateVector> {
        noisy_simulator::StateVector::try_from(
            self.dimension,
            self.number_of_qubits,
            self.trace_change,
            ComplexVector::from_vec(self.data),
        )
        .ok_or(NoisySimulatorError::new_err("invalid density matrix"))
    }
}

#[pyclass]
pub(crate) struct StateVectorSimulator(noisy_simulator::StateVectorSimulator);

#[pymethods]
impl StateVectorSimulator {
    #[new]
    pub fn new(number_of_qubits: usize, seed: Option<u64>) -> Self {
        let mut simulator = noisy_simulator::StateVectorSimulator::new(number_of_qubits);

        if let Some(seed) = seed {
            simulator.set_rng_seed(seed);
        }

        Self(simulator)
    }

    /// Apply an arbitrary operation to given qubit ids.
    #[allow(clippy::needless_pass_by_value)]
    pub fn apply_operation(&mut self, operation: &Operation, qubits: Vec<usize>) -> PyResult<()> {
        self.0
            .apply_operation(&operation.0, &qubits)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Apply non selective evolution.
    #[allow(clippy::needless_pass_by_value)]
    pub fn apply_instrument(
        &mut self,
        instrument: &Instrument,
        qubits: Vec<usize>,
    ) -> PyResult<()> {
        self.0
            .apply_instrument(&instrument.0, &qubits)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Performs selective evolution under the given instrument.
    /// Returns the index of the observed outcome.
    ///
    /// Use this method to perform measurements on the quantum system.
    #[allow(clippy::needless_pass_by_value)]
    pub fn sample_instrument(
        &mut self,
        instrument: &Instrument,
        qubits: Vec<usize>,
    ) -> PyResult<usize> {
        self.0
            .sample_instrument(&instrument.0, &qubits)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Returns the `StateVector` if the simulator is in a valid state.
    pub fn get_state(&self) -> Option<StateVector> {
        match self.0.state() {
            Ok(dm) => Some(dm.into()),
            Err(_) => None,
        }
    }

    /// Set state of the quantum system.
    pub fn set_state(&mut self, state: StateVector) -> PyResult<()> {
        self.0
            .set_state(state.try_into()?)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }

    /// Set the trace of the quantum system.
    pub fn set_trace(&mut self, trace: f64) -> PyResult<()> {
        self.0
            .set_trace(trace)
            .map_err(|e| NoisySimulatorError::new_err(e.to_string()))
    }
}
