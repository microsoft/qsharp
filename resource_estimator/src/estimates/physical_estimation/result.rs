// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::Factory;

/// Results for a factory part in the overall quantum algorithm
pub struct FactoryPart<F> {
    /// The factory used in this part
    factory: F,

    /// The number of factory copies
    copies: u64,

    /// The number of factory runs
    runs: u64,

    /// The required logical magic state error rate used to find this factory
    required_output_error_rate: f64,
}

impl<F: Factory> FactoryPart<F> {
    pub fn new(
        factory: F,
        copies: u64,
        num_magic_states: u64,
        required_output_error_rate: f64,
    ) -> Self {
        let magic_states_per_run = copies * factory.num_output_states();
        let runs = num_magic_states.div_ceil(magic_states_per_run);

        Self {
            factory,
            copies,
            runs,
            required_output_error_rate,
        }
    }

    pub fn factory(&self) -> &F {
        &self.factory
    }

    pub fn copies(&self) -> u64 {
        self.copies
    }

    pub fn runs(&self) -> u64 {
        self.runs
    }

    pub fn required_output_error_rate(&self) -> f64 {
        self.required_output_error_rate
    }

    pub fn physical_qubits(&self) -> u64 {
        self.factory.physical_qubits() * self.copies
    }

    pub fn into_factory(self) -> F {
        self.factory
    }
}
