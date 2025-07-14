// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;

use crate::estimates::{Error, ErrorCorrection};

use std::rc::Rc;

/// A logical patch from an error correction code
///
/// A logical patch is an instantiation of an error correcting code for some
/// assignment to the code parameters.  It stores all computed information such
/// as the number of physical and logical qubits, cycle time, and logical error
/// rate.
#[derive(Serialize)]
#[serde(rename_all = "camelCase", bound = "E::Parameter: Serialize")]
pub struct LogicalPatch<E: ErrorCorrection> {
    #[serde(skip)]
    physical_qubit: Rc<E::Qubit>,
    code_parameter: E::Parameter,
    physical_qubits: u64,
    logical_qubits: u64,
    logical_cycle_time: u64,
    logical_error_rate: f64,
}

impl<E: ErrorCorrection> LogicalPatch<E> {
    pub fn new(ftp: &E, code_parameter: E::Parameter, qubit: Rc<E::Qubit>) -> Result<Self, Error> {
        // safe to convert here because we check for negative values before
        let physical_qubits = ftp
            .physical_qubits(&code_parameter)
            .map_err(Error::PhysicalQubitComputationFailed)?;
        let logical_qubits = ftp
            .logical_qubits(&code_parameter)
            .map_err(Error::LogicalQubitComputationFailed)?;
        let logical_cycle_time = ftp
            .logical_cycle_time(&qubit, &code_parameter)
            .map_err(Error::LogicalCycleTimeComputationFailed)?;
        let logical_error_rate = ftp
            .logical_error_rate(&qubit, &code_parameter)
            .map_err(Error::LogicalErrorRateComputationFailed)?;

        Ok(Self {
            physical_qubit: qubit,
            code_parameter,
            physical_qubits,
            logical_qubits,
            logical_cycle_time,
            logical_error_rate,
        })
    }

    /// Returns a reference to the logical qubit's underlying physical qubit model.
    pub fn physical_qubit(&self) -> &E::Qubit {
        &self.physical_qubit
    }

    /// Returns the code parameter.
    pub fn code_parameter(&self) -> &E::Parameter {
        &self.code_parameter
    }

    /// Returns the number of logical qubits in the patch.
    pub fn logical_qubits(&self) -> u64 {
        self.logical_qubits
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
