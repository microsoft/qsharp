// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    estimates::Overhead,
    system::constants::{
        NUM_MEASUREMENTS_PER_R, NUM_MEASUREMENTS_PER_TOF, NUM_TS_PER_ROTATION_A_COEFFICIENT,
        NUM_TS_PER_ROTATION_B_COEFFICIENT,
    },
    LogicalResources,
};
use serde::{Deserialize, Serialize};
use std::convert::From;

/// Resource counts output from `qir_estimate_counts` program
#[derive(Default, Debug, Deserialize, Serialize, Clone, Copy)]
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
}

impl From<&LogicalResources> for LogicalResourceCounts {
    fn from(logical_resources: &LogicalResources) -> Self {
        Self {
            num_qubits: logical_resources.num_qubits as _,
            t_count: logical_resources.t_count as _,
            rotation_count: logical_resources.rotation_count as _,
            rotation_depth: logical_resources.rotation_depth as _,
            ccz_count: logical_resources.ccz_count as _,
            ccix_count: 0,
            measurement_count: logical_resources.measurement_count as _,
        }
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

    fn logical_qubits_without_padding(&self) -> u64 {
        2 * self.num_qubits
    }

    fn logical_depth(&self, num_ts_per_rotation: u64) -> u64 {
        (self.measurement_count + self.rotation_count + self.t_count) * NUM_MEASUREMENTS_PER_R
            + (self.ccz_count + self.ccix_count) * NUM_MEASUREMENTS_PER_TOF
            + num_ts_per_rotation * self.rotation_depth * NUM_MEASUREMENTS_PER_R
    }

    fn num_magic_states(&self, num_ts_per_rotation: u64) -> u64 {
        4 * (self.ccz_count + self.ccix_count)
            + self.t_count
            + num_ts_per_rotation * self.rotation_count
    }

    fn num_magic_states_per_rotation(&self, eps_synthesis: f64) -> Option<u64> {
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
