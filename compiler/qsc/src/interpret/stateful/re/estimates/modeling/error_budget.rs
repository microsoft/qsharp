// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ErrorBudget {
    /// Probability of at least one logical error
    logical: f64,
    /// Probability of at least one faulty T distillation
    tstates: f64,
    /// Probability of at least one failed rotation synthesis
    rotations: f64,
}

impl ErrorBudget {
    pub fn new(logical: f64, tstates: f64, rotations: f64) -> Self {
        Self {
            logical,
            tstates,
            rotations,
        }
    }

    /// Get the error budget's plogical.
    pub fn logical(&self) -> f64 {
        self.logical
    }

    /// Get the error budget's tstates.
    pub fn tstates(&self) -> f64 {
        self.tstates
    }

    /// Get the error budget's rotations.
    pub fn rotations(&self) -> f64 {
        self.rotations
    }
}
