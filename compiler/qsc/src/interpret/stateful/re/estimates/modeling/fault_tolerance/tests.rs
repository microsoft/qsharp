// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::*;

#[test]
fn compute_code_distance() -> Result<()> {
    let qubit = PhysicalQubit::default();
    let ftp = Protocol::surface_code_gate_based();
    assert!(
        (ftp.code_distance_lookup()
            .logical_failure_probability(&qubit, 5)?
            - 3.000_000_000_000_000_8e-5)
            .abs()
            <= f64::EPSILON
    );
    assert_eq!(ftp.logical_cycle_time(&qubit, 1)?, 400);

    Ok(())
}
