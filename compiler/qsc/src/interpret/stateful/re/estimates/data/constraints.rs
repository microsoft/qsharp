// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::super::serialization::time;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(
    rename_all(serialize = "camelCase", deserialize = "camelCase"),
    deny_unknown_fields
)]
pub struct Constraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logical_depth_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_t_factories: Option<u64>,
    #[serde(default, with = "time", skip_serializing_if = "Option::is_none")]
    pub max_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_physical_qubits: Option<u64>,
}

impl Constraints {
    pub fn is_default(&self) -> bool {
        self.logical_depth_factor.is_none()
            && self.max_t_factories.is_none()
            && self.max_duration.is_none()
            && self.max_physical_qubits.is_none()
    }
}
