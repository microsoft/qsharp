// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    estimates::ErrorCorrection,
    system::modeling::{PhysicalQubit, surface_code_gate_based},
};

#[test]
fn compute_code_distance() -> Result<(), String> {
    let qubit = PhysicalQubit::default();
    let ftp = surface_code_gate_based();
    assert!(
        (ftp.logical_error_rate(&qubit, &5)? - 3.000_000_000_000_000_8e-5).abs() <= f64::EPSILON
    );
    assert_eq!(ftp.logical_cycle_time(&qubit, &1)?, 400);

    Ok(())
}

#[test]
fn compute_distance_is_inverse() -> Result<(), String> {
    let qubit = PhysicalQubit::default();
    let mut ftp = surface_code_gate_based();

    for distance_coefficient_power in [0, 1, 2] {
        ftp.set_distance_coefficient_power(distance_coefficient_power);

        for code_distance in (1..=49).step_by(2) {
            let error_rate = ftp.logical_error_rate(&qubit, &code_distance)?;
            let computed_distance = ftp.compute_code_parameter(&qubit, error_rate)?;
            assert_eq!(computed_distance, code_distance);
        }
    }

    Ok(())
}
