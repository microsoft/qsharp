use nalgebra::{DMatrix, DVector};
use num::Complex;

pub mod density_matrix_simulator;
pub mod instrument;
pub mod kernel;
pub mod operation;
pub mod trajectory_simulator;

pub type Float = f64;
pub type SquareMatrix = DMatrix<Complex<Float>>;
pub type ComplexVector = DVector<Complex<Float>>;
pub const TOLERANCE: Float = 1e-12;