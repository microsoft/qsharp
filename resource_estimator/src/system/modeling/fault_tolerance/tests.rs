// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    estimates::ErrorCorrection,
    system::modeling::{PhysicalQubit, Protocol},
};

#[test]
fn compute_code_distance() -> Result<(), String> {
    let qubit = PhysicalQubit::default();
    let ftp = Protocol::surface_code_gate_based();
    assert!(
        (ftp.logical_failure_probability(&qubit, 5)? - 3.000_000_000_000_000_8e-5).abs()
            <= f64::EPSILON
    );
    assert_eq!(ftp.logical_cycle_time(&qubit, 1)?, 400);

    Ok(())
}
