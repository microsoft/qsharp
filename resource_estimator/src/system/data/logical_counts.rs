// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    estimates::{ErrorBudget, Overhead},
    system::constants::{
        NUM_MEASUREMENTS_PER_R, NUM_MEASUREMENTS_PER_TOF, NUM_TS_PER_ROTATION_A_COEFFICIENT,
        NUM_TS_PER_ROTATION_B_COEFFICIENT,
    },
};
use serde::{Deserialize, Serialize};

use super::PartitioningOverhead;

pub trait LayoutReportData {
    fn num_qubits(&self) -> u64;
    fn t_count(&self) -> u64;
    fn rotation_count(&self) -> u64;
    fn rotation_depth(&self) -> u64;
    fn ccz_count(&self) -> u64;
    fn ccix_count(&self) -> u64;
    fn measurement_count(&self) -> u64;
    fn num_ts_per_rotation(&self, eps_synthesis: f64) -> Option<u64>;
}

#[derive(Default, Debug, Clone)]
pub struct VolumeEntry {
    pub(crate) m_count: u64,
    pub(crate) r_count: u64,
    pub(crate) r_depth: u64,
    pub(crate) t_count: u64,
    pub(crate) ccz_count: u64,
}

/// Resource counts output from `qir_estimate_counts` program
#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(
    rename_all(deserialize = "camelCase", serialize = "camelCase"),
    deny_unknown_fields
)]
pub struct LogicalResourceCounts {
    pub num_qubits: u64,
    #[serde(default)]
    pub t_count: u64,
    #[serde(default)]
    pub rotation_count: u64,
    #[serde(default)]
    pub rotation_depth: u64,
    #[serde(default)]
    pub ccz_count: u64,
    #[serde(default)]
    pub ccix_count: u64,
    #[serde(default)]
    pub measurement_count: u64,
    #[serde(default, skip)]
    pub precise_volume_data: Option<Vec<VolumeEntry>>,
}

impl LogicalResourceCounts {
    fn compute_logical_depth(
        &self,
        m_count: u64,
        r_count: u64,
        t_count: u64,
        ccz_count: u64,
        r_depth: u64,
        budget: &ErrorBudget,
    ) -> u64 {
        (m_count + r_count + t_count) * NUM_MEASUREMENTS_PER_R
            + ccz_count * NUM_MEASUREMENTS_PER_TOF
            + self
                .num_ts_per_rotation(budget.rotations())
                .unwrap_or_default()
                * r_depth
                * NUM_MEASUREMENTS_PER_R
    }
}

/// Models the logical resources after layout
///
/// The logical resources comprise the logical depth, the number of qubits, and
/// the number of T states.  If there are rotations, optionally the number of T
/// gates per rotation are specified.
impl Overhead for LogicalResourceCounts {
    // number of qubits per one logical qubit (part of Q in paper)
    fn logical_qubits(&self) -> u64 {
        // number of logical qubits for padding (part of Q in paper)
        let qubit_padding = ((8 * self.num_qubits) as f64).sqrt().ceil() as u64 + 1;

        2 * self.num_qubits + qubit_padding
    }

    fn logical_depth(&self, budget: &ErrorBudget) -> u64 {
        self.compute_logical_depth(
            self.measurement_count,
            self.rotation_count,
            self.t_count,
            self.ccz_count + self.ccix_count,
            self.rotation_depth,
            budget,
        )
    }

    fn num_magic_states(&self, budget: &ErrorBudget, _index: usize) -> u64 {
        4 * (self.ccz_count + self.ccix_count)
            + self.t_count
            + self
                .num_ts_per_rotation(budget.rotations())
                .unwrap_or_default()
                * self.rotation_count
    }

    fn logical_volume(&self, budget: &ErrorBudget, adjusted_logical_depth: u64) -> u64 {
        let depth = self.logical_depth(budget);

        if adjusted_logical_depth == depth {
            if let Some(data) = self.precise_volume_data.as_ref() {
                let qubit_padding = ((8 * self.num_qubits) as f64).sqrt().ceil() as u64 + 1;

                // initialize total with padding, which is active for the whole depth
                let mut total = qubit_padding * depth;

                for e in data {
                    // there are two mapped logical qubits for each unmapped logical qubit
                    total += 2 * self.compute_logical_depth(
                        e.m_count,
                        e.r_count,
                        e.t_count,
                        e.ccz_count,
                        e.r_depth,
                        budget,
                    );
                }

                return total;
            }
        }

        self.logical_qubits() * adjusted_logical_depth
    }
}

impl PartitioningOverhead for LogicalResourceCounts {
    fn has_tgates(&self) -> bool {
        self.t_count > 0 || self.ccz_count > 0 || self.ccix_count > 0 || self.rotation_count > 0
    }

    fn has_rotations(&self) -> bool {
        self.rotation_count > 0
    }
}

impl LayoutReportData for LogicalResourceCounts {
    fn num_qubits(&self) -> u64 {
        self.num_qubits
    }

    fn t_count(&self) -> u64 {
        self.t_count
    }

    fn rotation_count(&self) -> u64 {
        self.rotation_count
    }

    fn rotation_depth(&self) -> u64 {
        self.rotation_depth
    }

    fn ccz_count(&self) -> u64 {
        self.ccz_count
    }

    fn ccix_count(&self) -> u64 {
        self.ccix_count
    }

    fn measurement_count(&self) -> u64 {
        self.measurement_count
    }

    fn num_ts_per_rotation(&self, eps_synthesis: f64) -> Option<u64> {
        if self.rotation_count > 0 {
            Some(
                (NUM_TS_PER_ROTATION_A_COEFFICIENT
                    * ((self.rotation_count as f64) / eps_synthesis).log2()
                    + NUM_TS_PER_ROTATION_B_COEFFICIENT)
                    .ceil() as _,
            )
        } else {
            None
        }
    }
}
