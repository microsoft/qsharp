// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Calculations make frequent use of conversion back and forth between f64 and u64.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod compiled_expression;
mod constants;
mod data;
mod error;
mod modeling;
mod optimization;
mod serialization;
mod stages;

use self::{modeling::Protocol, stages::physical_estimation::PhysicalResourceEstimation};
use super::LogicalResources;
use data::{JobParams, LogicalResourceCounts};
pub use error::Error;

type Result<T> = std::result::Result<T, error::Error>;

pub fn estimate_physical_resources(
    logical_resources: &LogicalResources,
    params: &str,
) -> Result<String> {
    estimate(logical_resources.into(), params)
}

pub fn estimate_physical_resources_from_json(
    logical_resources: &str,
    params: &str,
) -> std::result::Result<String, Error> {
    let logical_resources = serde_json::from_str(logical_resources)
        .map_err(|e| error::Error::IO(error::IO::CannotParseJSON(e)))?;
    estimate(logical_resources, params)
}

fn estimate(logical_resources: LogicalResourceCounts, params: &str) -> Result<String> {
    let job_params = if params.is_empty() {
        vec![JobParams::default()]
    } else {
        serde_json::from_str(params).map_err(|e| error::Error::IO(error::IO::CannotParseJSON(e)))?
    };

    let mut results = Vec::with_capacity(job_params.len());
    for mut job_params in job_params {
        let qubit = job_params.qubit_params().clone();
        let ftp = Protocol::load_from_specification(job_params.qec_scheme_mut(), &qubit)?;
        let distillation_unit_templates = job_params
            .distillation_unit_specifications()
            .as_templates()?;
        // create error buget partitioning
        let partitioning = job_params.error_budget().partitioning(&logical_resources)?;

        let mut estimation =
            PhysicalResourceEstimation::new(ftp, qubit, logical_resources, partitioning);
        if let Some(logical_depth_factor) = job_params.constraints().logical_depth_factor {
            estimation.set_logical_depth_factor(logical_depth_factor);
        }
        if let Some(max_t_factories) = job_params.constraints().max_t_factories {
            estimation.set_max_t_factories(max_t_factories);
        }
        if let Some(max_duration) = job_params.constraints().max_duration {
            estimation.set_max_duration(max_duration);
        }
        if let Some(max_physical_qubits) = job_params.constraints().max_physical_qubits {
            estimation.set_max_physical_qubits(max_physical_qubits);
        }
        estimation.set_distillation_unit_templates(distillation_unit_templates);

        let estimation_result = estimation.estimate();
        match estimation_result {
            Ok(estimation_result) => results.push(
                serde_json::to_string(&data::Success::new(
                    logical_resources,
                    job_params,
                    estimation_result,
                ))
                .expect("serializing to json string should succeed"),
            ),
            Err(err) => results.push(
                serde_json::to_string(&data::Failure::new(err))
                    .expect("serializing to json string should succeed"),
            ),
        }
    }
    Ok(format!("[{}]", results.join(",")))
}
