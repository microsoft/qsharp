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

pub mod density_matrix_simulator;
pub mod instrument;
pub mod kernel;
pub mod operation;
pub mod trajectory_simulator;

pub type SquareMatrix = DMatrix<Complex<f64>>;
pub type ComplexVector = DVector<Complex<f64>>;
pub const TOLERANCE: f64 = 1e-12;
