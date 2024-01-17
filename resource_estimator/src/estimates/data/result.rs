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
use serde::{ser::SerializeMap, Serialize, Serializer};

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct Success {
    status: &'static str,
    job_params: JobParams,
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_counts: Option<PhysicalResourceCounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_counts_formatted: Option<FormattedPhysicalResourceCounts>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logical_qubit: Option<LogicalQubit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tfactory: Option<TFactory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_budget: Option<ErrorBudget>,
    logical_counts: LogicalResourceCounts,
    report_data: Report,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    frontier_entries: Vec<FrontierEntry>,
}

impl Success {
    pub fn new<L: Overhead + Clone>(
        logical_resources: LogicalResourceCounts,
        job_params: JobParams,
        result: PhysicalResourceEstimationResult<L>,
    ) -> Self {
        let counts = create_physical_resource_counts(&result);

        let formatted_counts: FormattedPhysicalResourceCounts =
            FormattedPhysicalResourceCounts::new(&result, &logical_resources, &job_params);

        let report_data = Report::new(&logical_resources, &job_params, &result, &formatted_counts);

        let (logical_qubit, tfactory, error_budget) = result.take();

        Self {
            status: "success",
            job_params,
            physical_counts: Some(counts),
            physical_counts_formatted: Some(formatted_counts),
            logical_qubit: Some(logical_qubit),
            tfactory,
            error_budget: Some(error_budget),
            logical_counts: logical_resources,
            report_data,
            frontier_entries: Vec::new(),
        }
    }

    pub fn new_from_multiple<L: Overhead + Clone>(
        logical_resources: LogicalResourceCounts,
        job_params: JobParams,
        mut results: Vec<PhysicalResourceEstimationResult<L>>,
    ) -> Self {
        let mut report_data: Option<Report> = None;

        let mut frontier_entries: Vec<FrontierEntry> = Vec::new();

        // we will pick the shortest runtime result as the first result.
        results.sort_by_key(PhysicalResourceEstimationResult::runtime);
        for result in results {
            let (frontier_entry, report) = create_frontier_entry(
                &logical_resources,
                &job_params,
                result,
                report_data.is_none(),
            );

            if report_data.is_none() {
                report_data = Some(report.expect("error should have report"));
            }

            frontier_entries.push(frontier_entry);
        }

        Self {
            status: "success",
            job_params,
            physical_counts: None,
            physical_counts_formatted: None,
            logical_qubit: None,
            tfactory: None,
            error_budget: None,
            logical_counts: logical_resources,
            report_data: report_data.expect("error should have report"), // Here we assume that at least a single solution was found.
            frontier_entries,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FrontierEntry {
    pub logical_qubit: LogicalQubit,
    pub tfactory: Option<TFactory>,
    pub error_budget: ErrorBudget,
    pub physical_counts: PhysicalResourceCounts,
    pub physical_counts_formatted: FormattedPhysicalResourceCounts,
}

fn create_frontier_entry<L: Overhead + Clone>(
    logical_resources: &LogicalResourceCounts,
    job_params: &JobParams,
    result: PhysicalResourceEstimationResult<L>,
    create_report: bool,
) -> (FrontierEntry, Option<Report>) {
    let physical_counts = create_physical_resource_counts(&result);

    let physical_counts_formatted: FormattedPhysicalResourceCounts =
        FormattedPhysicalResourceCounts::new(&result, logical_resources, job_params);

    let report_data = if create_report {
        Some(Report::new(
            logical_resources,
            job_params,
            &result,
            &physical_counts_formatted,
        ))
    } else {
        None
    };

    let (logical_qubit, tfactory, error_budget) = result.take();

    (
        FrontierEntry {
            logical_qubit,
            tfactory,
            error_budget,
            physical_counts,
            physical_counts_formatted,
        },
        report_data,
    )
}

fn create_physical_resource_counts<L: Overhead + Clone>(
    result: &PhysicalResourceEstimationResult<L>,
) -> PhysicalResourceCounts {
    let breakdown = create_physical_resource_counts_breakdown(result);

    PhysicalResourceCounts {
        physical_qubits: result.physical_qubits(),
        runtime: result.runtime(),
        rqops: result.rqops(),
        breakdown,
    }
}

fn create_physical_resource_counts_breakdown<L: Overhead + Clone>(
    result: &PhysicalResourceEstimationResult<L>,
) -> PhysicalResourceCountsBreakdown {
    let num_ts_per_rotation = result
        .layout_overhead()
        .num_ts_per_rotation(result.error_budget().rotations());
    PhysicalResourceCountsBreakdown {
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
        S: Serializer,
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
            map.serialize_entry("message", &self.error.to_string())?;
        }

        map.end()
    }
}
