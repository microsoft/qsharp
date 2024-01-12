// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::LogicalResources;
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
