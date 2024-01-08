// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod constraints;
mod job_params;
mod logical_counts;
mod physical_counts;
mod report;
mod result;
mod tfactory;

pub use constraints::Constraints;
pub use job_params::{EstimateType, JobParams};
pub use logical_counts::LogicalResourceCounts;
pub use physical_counts::{PhysicalResourceCounts, PhysicalResourceCountsBreakdown};
pub use report::{FormattedPhysicalResourceCounts, Report};
pub use result::{Failure, Success};

#[cfg(test)]
pub use tfactory::{
    TFactoryDistillationUnitSpecification, TFactoryProtocolSpecificDistillationUnitSpecification,
};

#[cfg(test)]
pub use job_params::ErrorBudgetSpecification;
