// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// Input algorithm has no resources
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("Algorithm requires at least one magic state or measurement to estimate resources")]
    #[diagnostic(code("Qsc.Estimates.AlgorithmHasNoResources"))]
    AlgorithmHasNoResources,
    /// Both constraints for maximal time and
    /// maximal number of qubits are provided
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error(
        "Both duration and number of physical qubits constraints are provided, but only one is allowed"
    )]
    #[diagnostic(code("Qsc.Estimates.BothDurationAndPhysicalQubitsProvided"))]
    BothDurationAndPhysicalQubitsProvided,
    /// No solution found for the provided maximum duration.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("No solution found for the provided maximum duration.")]
    #[diagnostic(code("Qsc.Estimates.MaxDurationTooSmall"))]
    MaxDurationTooSmall,
    /// No solution found for the provided maximum number of physical qubits
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("No solution found for the provided maximum number of physical qubits.")]
    #[diagnostic(code("Qsc.Estimates.MaxPhysicalQubitsTooSmall"))]
    MaxPhysicalQubitsTooSmall,
    /// The number of physical qubits required for a code cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The number of physical qubits required for a code cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.PhysicalQubitComputationFailed"))]
    PhysicalQubitComputationFailed(String),
    /// The number of logical qubits provided by a code cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The number of logical qubits provided by a code cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.LogicalQubitComputationFailed"))]
    LogicalQubitComputationFailed(String),
    /// The logical cycle time cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The logical cycle time cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.LogicalCycleTimeComputationFailed"))]
    LogicalCycleTimeComputationFailed(String),
    /// The logical error rate cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The logical error rate cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.LogicalErrorRateComputationFailed"))]
    LogicalErrorRateComputationFailed(String),
    /// The code parameter cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The code parameter cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.CodeParameterComputationFailed"))]
    CodeParameterComputationFailed(String),
}
