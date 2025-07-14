// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[derive(Default, Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PhysicalResourceCounts {
    /// These are the total number of physical qubits
    pub(crate) physical_qubits: u64,
    /// This is the total runtime to execute the algorithm in nanoseconds.
    pub(crate) runtime: u64,
    /// QOPS: number of logical qubits × instructions per cycle per qubit × clock frequency
    pub(crate) rqops: u64,
    /// Breakdown of estimates
    pub(crate) breakdown: PhysicalResourceCountsBreakdown,
}

#[derive(Default, Debug, serde::Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct PhysicalResourceCountsBreakdown {
    /// These are logical qubits required for running the algorithm and do not
    /// include resources for T factories.
    pub(crate) algorithmic_logical_qubits: u64,
    /// These are the logical cycles required for running the algorithm and do
    /// not include resources for T factories.
    pub(crate) algorithmic_logical_depth: u64,
    /// The possibly adjusted number of cycles that is computed whenever the
    /// T-factory execution time is faster then algorithm execution.
    pub(crate) logical_depth: u64,
    /// The number of T-states consumed by the algorithm
    pub(crate) num_tstates: u64,
    /// The number of logical cycles per second
    pub(crate) clock_frequency: f64,
    /// The number of T-factories (we assume uniform T-factory design)
    pub(crate) num_tfactories: u64,
    /// The number of how often all parallel T-factories should run
    pub(crate) num_tfactory_runs: u64,
    /// The number of physical qubits for all T-factories
    pub(crate) physical_qubits_for_tfactories: u64,
    /// The number of physical qubits for algorithm layout
    pub(crate) physical_qubits_for_algorithm: u64,
    /// The required logical error rate
    pub(crate) required_logical_qubit_error_rate: f64,
    /// The required logical T-state error rate
    pub(crate) required_logical_tstate_error_rate: Option<f64>,
    /// The number of T gates per rotation
    pub(crate) num_ts_per_rotation: Option<u64>,
    /// The Clifford error rate based on the qubit parameters
    pub(crate) clifford_error_rate: f64,
}
