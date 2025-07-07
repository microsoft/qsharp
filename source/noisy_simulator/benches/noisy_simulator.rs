// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use criterion::{Criterion, criterion_group, criterion_main};
use nalgebra::dmatrix;
use noisy_simulator::{
    DensityMatrixSimulator, Error, NoisySimulator, Operation, StateVectorSimulator,
};
use std::time::Duration;

fn ten_qubits_ten_operations<NS: NoisySimulator>() -> Result<(), Error> {
    let x_gate = Operation::new(vec![
        dmatrix![
            0.974_679_43.into(), 0.0.into();
            0.0.into(), 0.974_679_43.into();
        ],
        dmatrix![
            0.0.into(), 0.223_606_8.into();
            2.236_067_98e-01.into(), (-4.965_068_31e-17).into();
        ],
    ])?;

    let swap_gate = Operation::new(vec![
        dmatrix![
            (-9.486_832_98e-01).into(),( 1.490_606_84e-17).into(),( 0.000_000_00e+00).into(),( 9.676_169_54e-34).into();
            (-4.680_505_48e-16).into(),(-9.486_832_98e-01).into(),( 1.966_320_58e-35).into(),(-2.169_220_51e-33).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-9.000_000_00e-01).into(),( 1.414_113_81e-17).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-1.414_113_81e-17).into(),(-9.000_000_00e-01).into();
        ],
        dmatrix![
            ( 4.024_638_47e-16).into(),( 3.162_277_66e-01).into(),( 0.000_000_00e+00).into(),( 2.052_770_32e-17).into();
            (-3.162_277_66e-01).into(),(-1.882_065_45e-16).into(),( 4.171_490_09e-19).into(),(-4.601_936_18e-17).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-1.240_452_23e-16).into(),( 3.000_000_00e-01).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-3.000_000_00e-01).into(),(-1.240_452_23e-16).into();
        ],
        dmatrix![
            ( 0.000_000_00e+00).into(),( 2.764_311_39e-18).into(),( 3.000_000_00e-01).into(),(-1.078_211_93e-16).into();
            ( 9.544_580_26e-18).into(),( 2.573_665_38e-18).into(),( 7.727_798_82e-18).into(),( 3.000_000_00e-01).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-1.182_297_27e-17).into(),( 6.864_964_58e-18).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-8.363_581_31e-19).into(),(-1.182_297_27e-17).into();
        ],
        dmatrix![
            (-0.000_000_00e+00).into(),( 7.412_309_43e-17).into(),( 1.815_419_41e-17).into(),(-1.000_000_00e-01).into();
            ( 1.124_874_17e-17).into(),(-2.225_607_46e-17).into(),( 1.000_000_00e-01).into(),(-3.937_497_49e-17).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),( 3.668_529_08e-18).into(),(-9.049_669_92e-17).into();
            ( 0.000_000_00e+00).into(),( 0.000_000_00e+00).into(),(-1.838_421_17e-17).into(),( 3.668_529_08e-18).into();
        ],
    ])?;

    let mut sim = NS::new_with_seed(10, 42);
    sim.apply_operation(&x_gate, &[0])?;
    sim.apply_operation(&swap_gate, &[0, 1])?;
    sim.apply_operation(&swap_gate, &[1, 2])?;
    sim.apply_operation(&swap_gate, &[2, 3])?;
    sim.apply_operation(&swap_gate, &[3, 4])?;
    sim.apply_operation(&swap_gate, &[4, 5])?;
    sim.apply_operation(&swap_gate, &[5, 6])?;
    sim.apply_operation(&swap_gate, &[6, 7])?;
    sim.apply_operation(&swap_gate, &[7, 8])?;
    sim.apply_operation(&swap_gate, &[8, 9])?;

    Ok(())
}

pub fn density_matrix_simulator(c: &mut Criterion) {
    c.benchmark_group("density_matrix_simulator")
        .measurement_time(Duration::from_secs(25))
        .bench_function("10 qubits and 10 operations circuit", |b| {
            b.iter(|| {
                ten_qubits_ten_operations::<DensityMatrixSimulator>()
                    .expect("bench should succeed");
            });
        });
}

pub fn state_vector_simulator(c: &mut Criterion) {
    c.benchmark_group("state_vector_simulator").bench_function(
        "10 qubits and 10 operations circuit",
        |b| {
            b.iter(|| {
                ten_qubits_ten_operations::<StateVectorSimulator>().expect("bench should succeed");
            });
        },
    );
}

criterion_group!(benches, density_matrix_simulator, state_vector_simulator);
criterion_main!(benches);
