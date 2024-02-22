// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::estimates::{Error, ErrorCorrection};

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

/// Logical qubit model.
///
/// A logical qubit is derived from a physical qubit and a fault-tolerance
/// protocol.  Construction methods are provided that take as additional input
/// the code distance, or alternatively the target error rate from which the
/// code distance is computed.
pub struct LogicalQubit<E: ErrorCorrection> {
    physical_qubit: Rc<E::Qubit>,
    code_parameter: E::Parameter,
    physical_qubits: u64,
    logical_cycle_time: u64,
    logical_error_rate: f64,
}

impl<E: ErrorCorrection> LogicalQubit<E> {
    pub fn new(ftp: &E, code_parameter: E::Parameter, qubit: Rc<E::Qubit>) -> Result<Self, Error> {
        // safe to convert here because we check for negative values before
        let physical_qubits = ftp
            .physical_qubits_per_logical_qubit(&code_parameter)
            .map_err(Error::PhysicalQubitComputationFailed)?;
        let logical_cycle_time = ftp
            .logical_cycle_time(&qubit, &code_parameter)
            .map_err(Error::LogicalCycleTimeComputationFailed)?;
        let logical_error_rate = ftp
            .logical_error_rate(&qubit, &code_parameter)
            .map_err(Error::LogicalFailureProbabilityFailed)?;

        Ok(Self {
            physical_qubit: qubit,
            code_parameter,
            physical_qubits,
            logical_cycle_time,
            logical_error_rate,
        })
    }

    /// Returns a reference to the logical qubit's underlying physical qubit model.
    pub fn physical_qubit(&self) -> &E::Qubit {
        &self.physical_qubit
    }

    /// Returns the code distance.
    pub fn code_parameter(&self) -> &E::Parameter {
        &self.code_parameter
    }

    /// Returns the number of physical qubits to encode the logical qubit.
    pub fn physical_qubits(&self) -> u64 {
        self.physical_qubits
    }

    /// Returns the logical cycle time.
    pub fn logical_cycle_time(&self) -> u64 {
        self.logical_cycle_time
    }

    /// Returns the qubit's logical error rate
    pub fn logical_error_rate(&self) -> f64 {
        self.logical_error_rate
    }

    /// Returns the number of logical cycles per second
    pub fn logical_cycles_per_second(&self) -> f64 {
        1e9 / (self.logical_cycle_time as f64)
    }
}

impl<E: ErrorCorrection> Debug for LogicalQubit<E>
where
    E::Parameter: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LQubit(d={})", self.code_parameter())
    }
}
