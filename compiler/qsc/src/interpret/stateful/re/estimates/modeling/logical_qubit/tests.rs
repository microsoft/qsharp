// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use super::super::super::{
    constants::FLOAT_COMPARISON_EPSILON,
    modeling::{LogicalQubit, PhysicalQubit, Protocol},
    Result,
};

#[test]
fn logical_qubit_from_fast_gate_based_and_surface_code() -> Result<()> {
    let physical_qubit = Rc::new(PhysicalQubit::default());

    let ftp = Protocol::default();

    let logical_qubit = LogicalQubit::new(&ftp, 7, physical_qubit)?;

    assert_eq!(logical_qubit.physical_qubits(), 98);
    assert!(
        (logical_qubit.logical_error_rate() - 3.000_000_000_000_001_3e-6).abs()
            < FLOAT_COMPARISON_EPSILON
    );

    Ok(())
}
