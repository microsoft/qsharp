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
    /// Computed code distance is too high
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The computed code distance {0} is too high; maximum allowed code distance is {1}; try increasing the total logical error budget")]
    #[diagnostic(code("Qsc.Estimates.InvalidCodeDistance"))]
    InvalidCodeDistance(u64, u64),
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
    /// No solution found for the provided maximum number of magic state factories.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("No solution found for the provided maximum number of magic state factories.")]
    #[diagnostic(code("Qsc.Estimates.NoSolutionFoundForMaxFactories"))]
    NoSolutionFoundForMaxFactories,
    /// No T factories could be built for the provided range of code distances,
    /// the provided error budget and provided distillation units.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("No factories could be built for the provided range of code distances, the provided error budget and provided distillation units.")]
    #[diagnostic(code("Qsc.Estimates.NoFactoriesFound"))]
    NoFactoriesFound,
    /// The number of physical qubits per logical qubit cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The number of physical qubits per logical qubit cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.PhysicalQubitComputationFailed"))]
    PhysicalQubitComputationFailed(String),
    /// The logical cycle time cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The logical cycle time cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.LogicalCycleTimeComputationFailed"))]
    LogicalCycleTimeComputationFailed(String),
    /// The logical failure probability cannot be computed.
    ///
    /// ✅ This does not contain user data and can be logged
    /// 🧑‍💻 This indicates a user error
    #[error("The logical failure probability cannot be computed: {0}")]
    #[diagnostic(code("Qsc.Estimates.LogicalFailureProbabilityFailed"))]
    LogicalFailureProbabilityFailed(String),
}
