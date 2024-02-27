// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[derive(Clone)]
pub struct ErrorBudget {
    /// Probability of at least one logical error
    logical: f64,
    /// Probability of at least one faulty magic state distillation
    magic_states: f64,
    /// Probability of at least one failed rotation synthesis
    rotations: f64,
}

impl ErrorBudget {
    pub fn new(logical: f64, magic_states: f64, rotations: f64) -> Self {
        Self {
            logical,
            magic_states,
            rotations,
        }
    }

    /// Get the error budget's plogical.
    pub fn logical(&self) -> f64 {
        self.logical
    }

    /// Get the error budget's tstates.
    pub fn magic_states(&self) -> f64 {
        self.magic_states
    }

    /// Get the error budget's rotations.
    pub fn rotations(&self) -> f64 {
        self.rotations
    }
}
