// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod error;
pub use error::Error;
mod error_budget;
pub use error_budget::ErrorBudget;
mod factory;
pub use factory::{DistillationRound, DistillationUnit, FactoryBuildError, RoundBasedFactory};
mod physical_estimation;
pub use physical_estimation::{
    ErrorCorrection, Factory, FactoryBuilder, PhysicalResourceEstimation,
    PhysicalResourceEstimationResult,
};
mod layout;
mod logical_qubit;
pub use layout::Overhead;
pub use logical_qubit::LogicalPatch;
pub mod optimization;
