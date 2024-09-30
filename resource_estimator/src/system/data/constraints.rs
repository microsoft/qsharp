// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::system::constants::MAX_DISTILLATION_ROUNDS;

use super::super::serialization::time;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
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
    #[serde(default = "Constraints::max_distillation_rounds_default")]
    pub max_distillation_rounds: u64,
}

impl Default for Constraints {
    fn default() -> Self {
        Self {
            logical_depth_factor: None,
            max_t_factories: None,
            max_duration: None,
            max_physical_qubits: None,
            max_distillation_rounds: Self::max_distillation_rounds_default(),
        }
    }
}

impl Constraints {
    fn max_distillation_rounds_default() -> u64 {
        MAX_DISTILLATION_ROUNDS
    }
}
