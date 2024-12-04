// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::Serialize;

use crate::estimates::{
    ErrorBudget, ErrorCorrection, Factory, FactoryBuilder, LogicalPatch, Overhead,
    PhysicalResourceEstimation, RealizedOverhead,
};

/// Resource estimation result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalResourceEstimationResult<E: ErrorCorrection, F> {
    #[serde(bound = "E::Parameter: Serialize")]
    logical_patch: LogicalPatch<E>,
    num_cycles: u64,
    #[serde(bound = "F: Serialize")]
    factory_parts: Vec<Option<FactoryPart<F>>>,
    required_logical_error_rate: f64,
    physical_qubits_for_factories: u64,
    physical_qubits_for_algorithm: u64,
    physical_qubits: u64,
    runtime: u64,
    rqops: u64,
    layout_overhead: RealizedOverhead,
    error_budget: ErrorBudget,
}

impl<E: ErrorCorrection<Parameter = impl Clone>, F: Factory<Parameter = E::Parameter> + Clone>
    PhysicalResourceEstimationResult<E, F>
{
    pub fn new(
        estimation: &PhysicalResourceEstimation<
            E,
            impl FactoryBuilder<E, Factory = F>,
            impl Overhead,
        >,
        logical_patch: LogicalPatch<E>,
        num_cycles: u64,
        factory_parts: Vec<Option<FactoryPart<F>>>,
        required_logical_error_rate: f64,
    ) -> Self {
        let physical_qubits_for_factories = factory_parts
            .iter()
            .filter_map(|f| f.as_ref().map(FactoryPart::physical_qubits))
            .sum();
        let num_logical_patches = estimation
            .layout_overhead
            .logical_qubits()
            .div_ceil(logical_patch.logical_qubits());
        let physical_qubits_for_algorithm = num_logical_patches * logical_patch.physical_qubits();

        let physical_qubits = physical_qubits_for_algorithm + physical_qubits_for_factories;

        let runtime = (logical_patch.logical_cycle_time()) * num_cycles;

        let rqops = (estimation.layout_overhead().logical_qubits() as f64
            * logical_patch.logical_cycles_per_second())
        .ceil() as u64;

        Self {
            logical_patch,
            num_cycles,
            factory_parts,
            required_logical_error_rate,
            physical_qubits_for_factories,
            physical_qubits_for_algorithm,
            physical_qubits,
            runtime,
            rqops,
            layout_overhead: RealizedOverhead::from_overhead(
                estimation.layout_overhead(),
                estimation.error_budget(),
                estimation.factory_builder().num_magic_state_types(),
            ),
            error_budget: estimation.error_budget().clone(),
        }
    }

    pub fn without_factories(
        estimation: &PhysicalResourceEstimation<
            E,
            impl FactoryBuilder<E, Factory = F>,
            impl Overhead,
        >,
        logical_patch: LogicalPatch<E>,
        num_cycles: u64,
        required_logical_patch_error_rate: f64,
    ) -> Self {
        Self::new(
            estimation,
            logical_patch,
            num_cycles,
            std::iter::repeat(())
                .map(|()| None)
                .take(estimation.factory_builder.num_magic_state_types())
                .collect(),
            required_logical_patch_error_rate,
        )
    }

    pub fn logical_patch(&self) -> &LogicalPatch<E> {
        &self.logical_patch
    }

    pub fn take(self) -> (LogicalPatch<E>, Vec<Option<FactoryPart<F>>>, ErrorBudget) {
        (self.logical_patch, self.factory_parts, self.error_budget)
    }

    pub fn num_cycles(&self) -> u64 {
        self.num_cycles
    }

    pub fn factory_parts(&self) -> &[Option<FactoryPart<F>>] {
        &self.factory_parts
    }

    /// The required logical error rate for one logical operation on one logical
    /// qubit
    pub fn required_logical_error_rate(&self) -> f64 {
        self.required_logical_error_rate
    }

    pub fn physical_qubits_for_factories(&self) -> u64 {
        self.physical_qubits_for_factories
    }

    pub fn physical_qubits_for_algorithm(&self) -> u64 {
        self.physical_qubits_for_algorithm
    }

    pub fn physical_qubits(&self) -> u64 {
        self.physical_qubits
    }

    pub fn runtime(&self) -> u64 {
        self.runtime
    }

    pub fn rqops(&self) -> u64 {
        self.rqops
    }

    pub fn layout_overhead(&self) -> &RealizedOverhead {
        &self.layout_overhead
    }

    pub fn error_budget(&self) -> &ErrorBudget {
        &self.error_budget
    }

    pub fn algorithmic_logical_depth(&self) -> u64 {
        self.layout_overhead.logical_depth()
    }

    /// The argument index indicates for which type of magic state (starting
    /// from 0) the number is requested for.
    pub fn num_magic_states(&self, index: usize) -> u64 {
        self.layout_overhead.num_magic_states()[index]
    }
}

/// Results for a factory part in the overall quantum algorithm
#[derive(Serialize)]
#[serde(rename_all = "camelCase", bound = "F: Serialize")]
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
