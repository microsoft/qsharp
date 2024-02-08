// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::estimates::{Error, ErrorCorrection};

use serde::Serialize;
use std::{fmt::Debug, rc::Rc};

/// Logical qubit model.
///
/// A logical qubit is derived from a physical qubit and a fault-tolerance
/// protocol.  Construction methods are provided that take as additional input
/// the code distance, or alternatively the target error rate from which the
/// code distance is computed.
#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct LogicalQubit<P> {
    #[serde(skip_serializing)]
    physical_qubit: Rc<P>,
    code_distance: u64,
    physical_qubits: u64,
    logical_cycle_time: u64,
    logical_error_rate: f64,
}

impl<P> LogicalQubit<P> {
    pub fn new(
        ftp: &impl ErrorCorrection<Qubit = P>,
        code_distance: u64,
        qubit: Rc<P>,
    ) -> Result<Self, Error> {
        // safe to convert here because we check for negative values before
        let physical_qubits = ftp
            .physical_qubits_per_logical_qubit(code_distance)
            .map_err(Error::PhysicalQubitComputationFailed)?;
        let logical_cycle_time = ftp
            .logical_cycle_time(&qubit, code_distance)
            .map_err(Error::LogicalCycleTimeComputationFailed)?;
        let logical_error_rate = ftp
            .logical_failure_probability(&qubit, code_distance)
            .map_err(Error::LogicalFailureProbabilityFailed)?;

        Ok(Self {
            physical_qubit: qubit,
            code_distance,
            physical_qubits,
            logical_cycle_time,
            logical_error_rate,
        })
    }

    /// Returns a reference to the logical qubit's underlying physical qubit model.
    pub fn physical_qubit(&self) -> &P {
        &self.physical_qubit
    }

    /// Returns the code distance.
    pub fn code_distance(&self) -> u64 {
        self.code_distance
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

impl<P> Debug for LogicalQubit<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LQubit(d={})", self.code_distance())
    }
}
