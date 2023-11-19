// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    super::{
        modeling::{ErrorBudget, LogicalQubit},
        stages::{
            layout::Overhead, physical_estimation::PhysicalResourceEstimationResult,
            tfactory::TFactory,
        },
        Error,
    },
    FormattedPhysicalResourceCounts, JobParams, LogicalResourceCounts, PhysicalResourceCounts,
    PhysicalResourceCountsBreakdown, Report,
};
use miette::Diagnostic;
use serde::{ser::SerializeMap, Serialize};

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Success {
    status: &'static str,
    job_params: JobParams,
    physical_counts: PhysicalResourceCounts,
    physical_counts_formatted: FormattedPhysicalResourceCounts,
    logical_qubit: LogicalQubit,
    tfactory: Option<TFactory>,
    error_budget: ErrorBudget,
    logical_counts: LogicalResourceCounts,
    report_data: Report,
}

impl Success {
    pub fn new<L: Overhead + Clone>(
        logical_resources: LogicalResourceCounts,
        job_params: JobParams,
        result: PhysicalResourceEstimationResult<L>,
    ) -> Self {
        let num_ts_per_rotation = result
            .layout_overhead()
            .num_ts_per_rotation(result.error_budget().rotations());

        let breakdown = PhysicalResourceCountsBreakdown {
            algorithmic_logical_qubits: result.layout_overhead().logical_qubits(),
            algorithmic_logical_depth: result
                .layout_overhead()
                .logical_depth(num_ts_per_rotation.unwrap_or_default()),
            logical_depth: result.num_cycles(),
            clock_frequency: result.logical_qubit().logical_cycles_per_second(),
            num_tstates: result
                .layout_overhead()
                .num_tstates(num_ts_per_rotation.unwrap_or_default()),
            num_tfactories: result.num_tfactories(),
            num_tfactory_runs: result.num_tfactory_runs(),
            physical_qubits_for_tfactories: result.physical_qubits_for_tfactories(),
            physical_qubits_for_algorithm: result.physical_qubits_for_algorithm(),
            required_logical_qubit_error_rate: result.required_logical_qubit_error_rate(),
            required_logical_tstate_error_rate: result.required_logical_tstate_error_rate(),
            num_ts_per_rotation,
            clifford_error_rate: result
                .logical_qubit()
                .physical_qubit()
                .clifford_error_rate(),
        };

        let counts = PhysicalResourceCounts {
            physical_qubits: result.physical_qubits(),
            runtime: result.runtime(),
            rqops: result.rqops(),
            breakdown,
        };

        let formatted_counts: FormattedPhysicalResourceCounts =
            FormattedPhysicalResourceCounts::new(&result, &logical_resources, &job_params);
        let report_data = Report::new(&logical_resources, &job_params, &result, &formatted_counts);

        let (logical_qubit, tfactory, error_budget) = result.take();

        Self {
            status: "success",
            job_params,
            physical_counts: counts,
            physical_counts_formatted: formatted_counts,
            logical_qubit,
            tfactory,
            error_budget,
            logical_counts: logical_resources,
            report_data,
        }
    }
}

pub struct Failure {
    error: Error,
    batch_index: Option<usize>,
}

impl Failure {
    #[must_use]
    pub fn new(error: Error) -> Self {
        Self {
            error,
            batch_index: None,
        }
    }
}

impl Serialize for Failure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;

        map.serialize_entry(
            "code",
            &self
                .error
                .code()
                .expect("error should have code")
                .to_string(),
        )?;
        if let Some(batch_index) = self.batch_index {
            map.serialize_entry(
                "message",
                &format!("[batch index {}] {:?}", batch_index, self.error),
            )?;
        } else {
            map.serialize_entry("message", &format!("{:?}", self.error))?;
        }

        map.end()
    }
}
