#![allow(unused_variables)]

use nalgebra::dmatrix;
use noisy_simulator_rs::{
    density_matrix_simulator::DensityMatrixSimulator, operation::Operation, SquareMatrix,
};

fn main() {
    let start = std::time::Instant::now();

    let x_gate = Operation::new(vec![
        dmatrix![
            0.97467943.into(), 0.0.into();
            0.0.into(), 0.97467943.into();
        ],
        dmatrix![
            0.0.into(), 0.2236068.into();
            2.23606798e-01.into(), (-4.96506831e-17).into();
        ],
    ]);

    let swap_gate = Operation::new(vec![
        dmatrix![
            (-9.48683298e-01).into(),( 1.49060684e-17).into(),( 0.00000000e+00).into(),( 9.67616954e-34).into();
            (-4.68050548e-16).into(),(-9.48683298e-01).into(),( 1.96632058e-35).into(),(-2.16922051e-33).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-9.00000000e-01).into(),( 1.41411381e-17).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-1.41411381e-17).into(),(-9.00000000e-01).into();
        ],
        dmatrix![
            ( 4.02463847e-16).into(),( 3.16227766e-01).into(),( 0.00000000e+00).into(),( 2.05277032e-17).into();
            (-3.16227766e-01).into(),(-1.88206545e-16).into(),( 4.17149009e-19).into(),(-4.60193618e-17).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-1.24045223e-16).into(),( 3.00000000e-01).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-3.00000000e-01).into(),(-1.24045223e-16).into();
        ],
        dmatrix![
            ( 0.00000000e+00).into(),( 2.76431139e-18).into(),( 3.00000000e-01).into(),(-1.07821193e-16).into();
            ( 9.54458026e-18).into(),( 2.57366538e-18).into(),( 7.72779882e-18).into(),( 3.00000000e-01).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-1.18229727e-17).into(),( 6.86496458e-18).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-8.36358131e-19).into(),(-1.18229727e-17).into();
        ],
        dmatrix![
            (-0.00000000e+00).into(),( 7.41230943e-17).into(),( 1.81541941e-17).into(),(-1.00000000e-01).into();
            ( 1.12487417e-17).into(),(-2.22560746e-17).into(),( 1.00000000e-01).into(),(-3.93749749e-17).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),( 3.66852908e-18).into(),(-9.04966992e-17).into();
            ( 0.00000000e+00).into(),( 0.00000000e+00).into(),(-1.83842117e-17).into(),( 3.66852908e-18).into();
        ],
    ]);

    let mut sim = DensityMatrixSimulator::new(12);
    sim.apply_operation(&x_gate, &[0]);
    sim.apply_operation(&swap_gate, &[0, 1]);
    sim.apply_operation(&swap_gate, &[1, 2]);
    sim.apply_operation(&swap_gate, &[2, 3]);
    sim.apply_operation(&swap_gate, &[3, 4]);
    sim.apply_operation(&swap_gate, &[4, 5]);
    sim.apply_operation(&swap_gate, &[5, 6]);
    sim.apply_operation(&swap_gate, &[6, 7]);
    sim.apply_operation(&swap_gate, &[7, 8]);
    sim.apply_operation(&swap_gate, &[8, 9]);
    sim.apply_operation(&swap_gate, &[9, 10]);
    sim.apply_operation(&swap_gate, &[10, 11]);

    println!("trace_change: {}", sim.trace_change());
    // print only first element, since this matrix is too big.
    println!("{}", sim.state().data[0]);
    println!("{:.1}s", (start.elapsed().as_millis() as f32) / 1000.0)
}

#[allow(dead_code)]
fn pp_matrix(matrix: &SquareMatrix) {
    let (nrows, ncols) = matrix.shape();
    for row in 0..nrows {
        for col in 0..ncols {
            let elt = matrix[(row, col)].re;
            print!("{:>10.3}", elt)
        }
        println!();
    }
}
