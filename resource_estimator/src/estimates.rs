// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod error;
pub use error::Error;
mod error_budget;
pub use error_budget::ErrorBudget;
mod physical_estimation;
pub use physical_estimation::{
    ErrorCorrection, Factory, FactoryBuilder, PhysicalResourceEstimation,
    PhysicalResourceEstimationResult,
};
mod layout;
mod logical_qubit;
pub use layout::Overhead;
pub use logical_qubit::LogicalQubit;
pub mod optimization;
