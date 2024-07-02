// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This crate contains the noisy simulator backend for the Q# language.
//!
//! It includes two simulators:
//!  - A density matrix simulator.
//!  - A state vector simulator.
//!
//! # Density matrix simulator
//! The density matrix simulator is faster, since it evolves the entire probability space
//! of the system as a whole, so you only need to run the simulation once, and take all
//! the samples you need from the final density matrix. However, it is more memory intensive.
//!
//! A density matrix has 2 ^ (2 * number_of_qubits) complex numbers entries.
//! If each complex number is represented as two 64-bits floating point numbers,
//! the density matrix will be 2 ^ (2 * number_of_qubits) * 16 bytes. E.g., a density
//! matrix representing a 20 qubits system will be 17592186044416 bytes, or 16.4 TB.
//!
//! # State vector simulator
//! The state vector simulator allocates less memory, however if you want 1_000_000 shots
//! of the circuit, you need to run the simulation 1_000_000 times.
//! A state vector has 2 ^ (number_of_qubits) complex entries. So, a state vector of a 20
//! qubits system will be 16777216 bytes, or 16 MB.
//!
//! # Which one should I use?
//! If you are interested in running many shots of the circuit it is better to use the
//! density matrix simulator, as long as you have enough memory in your system (13 qubits or less).
//!
//! However if you are interested in a single or very few shots, you should use the state
//! vector simulator.

#![deny(missing_docs)]

use nalgebra::{DMatrix, DVector};
use num_complex::Complex;
use thiserror::Error;

pub(crate) mod density_matrix_simulator;
pub(crate) mod instrument;
pub(crate) mod kernel;
pub(crate) mod operation;
pub(crate) mod state_vector_simulator;

/// A square matrix of `Complex<f64>`.
pub type SquareMatrix = DMatrix<Complex<f64>>;
/// A complex vector.
pub type ComplexVector = DVector<Complex<f64>>;
/// Error tolerance used in the simulators.
pub(crate) const TOLERANCE: f64 = 1e-12;

/// A noisy simulation error.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum Error {
    /// Provided an invalid state when creating or setting the state of the simulator.
    #[error("provided an invalid state when creating or setting the state of the simulator: {0}")]
    InvalidState(String),
    /// `Matrix` ⋅ `Vector` multiplication mismatch.
    #[error("matrix ⋅ vector multiplication mismatch; matrix is of dimension ({nrows}, {ncols}) but vector has {vec_dim} entries")]
    MatrixVecDimensionMismatch {
        /// Number of rows in the matrix.
        nrows: usize,
        /// Number of columns in the matrix.
        ncols: usize,
        /// Number of elements in the vector.
        vec_dim: usize,
    },
    /// Failure when sampling instrument outcome.
    #[error("numerical error: no outcome found when sampling instrument")]
    FailedToSampleInstrumentOutcome,
    /// Failure when sampling Kraus operators.
    #[error("numerical error: no outcome found when sampling Kraus operators")]
    FailedToSampleKrausOperators,
    /// State is not normalized.
    #[error("numerical error: trace should be between 0 and 1, but it is {0}")]
    NotNormalized(f64),
    /// A numerical error, such as a probability-0 event.
    #[error("numerical error: probability-0 event")]
    ProbabilityZeroEvent,
}

impl Error {
    const fn is_unrecoverable(&self) -> bool {
        matches!(
            self,
            Error::ProbabilityZeroEvent
                | Error::FailedToSampleInstrumentOutcome
                | Error::FailedToSampleKrausOperators
        )
    }
}

impl From<&Error> for Error {
    fn from(value: &Error) -> Self {
        value.clone()
    }
}

impl From<&mut Error> for Error {
    fn from(value: &mut Error) -> Self {
        value.clone()
    }
}

/// Handles errors in the simulator.
/// If an error is unrecoverable, it will set the state of the simulator to that error,
/// invalidating any further evolution of the quantum system.
macro_rules! handle_error {
    ($self:expr, $err:expr) => {
        if $err.is_unrecoverable() {
            $self.state = Err($err.clone());
        }
        return Err($err)
    };
}

pub(crate) use handle_error;

pub use {
    density_matrix_simulator::{DensityMatrix, DensityMatrixSimulator},
    instrument::Instrument,
    operation::Operation,
    state_vector_simulator::{StateVector, StateVectorSimulator},
};
