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