// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Calculations make frequent use of conversion back and forth between f64 and u64.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

#[cfg(test)]
mod tests;

mod compiled_expression;
mod constants;
mod data;
pub(crate) mod error;
mod modeling;
mod optimization;
mod serialization;

use crate::estimates::PhysicalResourceEstimation;

use self::{modeling::Protocol, optimization::TFactoryBuilder};
use super::LogicalResources;
use data::{EstimateType, JobParams, LogicalResourceCounts};
pub use error::Error;

pub(crate) type Result<T> = std::result::Result<T, error::Error>;

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
    let job_params_array = if params.is_empty() {
        vec![JobParams::default()]
    } else {
        serde_json::from_str(params).map_err(|e| error::Error::IO(error::IO::CannotParseJSON(e)))?
    };

    let mut results: Vec<String> = Vec::with_capacity(job_params_array.len());
    for job_params in job_params_array {
        let result = estimate_single(logical_resources, job_params);
        match result {
            Ok(result) => results.push(
                serde_json::to_string(&result).expect("serializing to json string should succeed"),
            ),
            Err(err) => {
                results.push(serialize_error(err));
            }
        }
    }

    Ok(format!("[{}]", results.join(",")))
}

fn estimate_single(
    logical_resources: LogicalResourceCounts,
    mut job_params: JobParams,
) -> Result<data::Success> {
    let qubit = job_params.qubit_params().clone();

    let ftp = Protocol::load_from_specification(job_params.qec_scheme_mut(), &qubit)?;
    let distillation_unit_templates = job_params
        .distillation_unit_specifications()
        .as_templates()?;
    // create error budget partitioning
    let partitioning = job_params.error_budget().partitioning(&logical_resources)?;

    let mut estimation = PhysicalResourceEstimation::new(
        ftp,
        qubit,
        TFactoryBuilder::default(),
        logical_resources,
        partitioning,
    );
    if let Some(logical_depth_factor) = job_params.constraints().logical_depth_factor {
        estimation.set_logical_depth_factor(logical_depth_factor);
    }
    if let Some(max_t_factories) = job_params.constraints().max_t_factories {
        estimation.set_max_factories(max_t_factories);
    }
    if let Some(max_duration) = job_params.constraints().max_duration {
        estimation.set_max_duration(max_duration);
    }
    if let Some(max_physical_qubits) = job_params.constraints().max_physical_qubits {
        estimation.set_max_physical_qubits(max_physical_qubits);
    }
    estimation
        .factory_builder_mut()
        .set_distillation_unit_templates(distillation_unit_templates);

    match job_params.estimate_type() {
        EstimateType::Frontier => {
            if job_params.constraints().max_duration.is_some()
                || job_params.constraints().max_physical_qubits.is_some()
                || job_params.constraints().max_t_factories.is_some()
            {
                // We can technically handle those scenarios but do not see a practial use case for it.
                return Err(error::Error::InvalidInput(
                    error::InvalidInput::ConstraintsProvidedForFrontierEstimation,
                ));
            }

            let estimation_result = estimation.build_frontier().map_err(|err| err.into());
            estimation_result.map(|result| {
                data::Success::new_from_multiple(logical_resources, job_params, result)
            })
        }
        EstimateType::SinglePoint => {
            let estimation_result = estimation.estimate().map_err(|err| err.into());
            estimation_result
                .map(|result| data::Success::new(logical_resources, job_params, result))
        }
    }
}

fn serialize_error(err: error::Error) -> String {
    serde_json::to_string(&data::Failure::new(err))
        .expect("serializing to json string should succeed")
}
