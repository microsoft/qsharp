// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Public API for preparing Adaptive programs, driving shots through region
//! consumers, and describing backend-neutral operations and observables.

mod adaptive;
mod immediate;
mod operator_matrix;
mod pauli_sum;
mod protocol;
mod region;
pub mod tooling;
mod unitary;

pub use adaptive::{
    AdaptiveExecution, AdaptiveExecutionError, MeasuredQubit, MeasurementMetadataError,
    PreparedAdaptiveProgram, RegionPartitionError, RegionSite, partition_unitary_regions,
};
pub use immediate::{
    ImmediateExecutionReport, ImmediatePreparedRegion, ImmediateRegionReport,
    ImmediateSimulatorConsumer, ShotExecutionError, ShotExecutionOutput, ShotExecutionResult,
    drive_prepared_shot, run_prepared_shot,
};
pub use operator_matrix::{OperatorMatrix, basis_operator, unitary_matrix};
pub use pauli_sum::{Pauli, PauliSum, PauliSumError, PauliTerm};
pub use protocol::{
    AdaptiveCommand, AdaptiveResponse, MeasurementKind, MeasurementRequest, RegionId,
};
pub use region::{QuantumEvolutionRegion, RegionConsumer};
pub use unitary::UnitaryOperation;
pub(crate) use unitary::{
    OPID_MRESETZ, OPID_MZ, OPID_RESETZ, apply_unitary_immediately, resolve_unitary_operation,
};

#[cfg(test)]
use adaptive::OP_QUANTUM_GATE;

#[cfg(test)]
mod tests;
