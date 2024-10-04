// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod error;
pub use error::Error;
mod error_budget;
pub use error_budget::ErrorBudget;
mod error_correction;
pub use error_correction::{
    CodeWithThresholdAndDistance, CodeWithThresholdAndDistanceEvaluator, ErrorCorrection,
};
mod factory;
pub use factory::{
    BuilderDispatch2, DistillationRound, DistillationUnit, Factory, FactoryBuildError,
    FactoryBuilder, FactoryDispatch2, NoFactories, PhysicalQubitCalculation, RoundBasedFactory,
};
mod physical_estimation;
pub use physical_estimation::{
    FactoryPart, PhysicalResourceEstimation, PhysicalResourceEstimationResult,
};
mod layout;
mod logical_qubit;
pub use layout::{Overhead, RealizedOverhead};
pub use logical_qubit::LogicalPatch;
pub mod optimization;
