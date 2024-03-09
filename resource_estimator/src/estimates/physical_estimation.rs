// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::{
    optimization::{Point2D, Population},
    Error, ErrorBudget, LogicalPatch, Overhead,
};
use std::{borrow::Cow, cmp::Ordering, rc::Rc};

/// Trait to model quantum error correction.
///
/// This trait models one quantum error correction code that encodes k logical
/// qubits using n physical qubits.  The physical qubits are of type
/// `Self::Qubit`.  Each code is parameterized by assignments to parameters of
/// type `Self::Parameter`.  Implementors of this trait need to specify values
/// for k, n, the logical cycle time, and the logical error rate, given an
/// assignment to the code parameter.
///
/// In order to define the space of possible code parameters, implementers of
/// this trait need to provide a range of code parameters as well as a
/// comparison function that orders all possible code parameter assignments.
pub trait ErrorCorrection {
    /// The underlying physical qubit type for the code
    type Qubit;
    /// The type for the code parameter
    ///
    /// This could be a numeric type in case the code parameter is the code
    /// distance, or a tuple type, if the code is parameterized over multiple
    /// values.
    type Parameter;

    /// The total number of physical qubits required by the code
    fn physical_qubits(&self, code_parameter: &Self::Parameter) -> Result<u64, String>;

    /// The number of logical qubits provided by the code
    fn logical_qubits(&self, code_parameter: &Self::Parameter) -> Result<u64, String>;

    /// The logical cycle time in nano seconds
    fn logical_cycle_time(
        &self,
        qubit: &Self::Qubit,
        code_parameter: &Self::Parameter,
    ) -> Result<u64, String>;

    /// The logical error rate
    fn logical_error_rate(
        &self,
        qubit: &Self::Qubit,
        code_parameter: &Self::Parameter,
    ) -> Result<f64, String>;

    /// Computes a code parameter assignment for a provided required logical
    /// error rate
    ///
    /// The default implementation iterates through all code parameters using
    /// the `Self::code_parameter_range` method and returns the first parameter
    /// for which the logical error rate is less or equal the required logical
    /// error rate.
    fn compute_code_parameter(
        &self,
        qubit: &Self::Qubit,
        required_logical_error_rate: f64,
    ) -> Result<Self::Parameter, String> {
        for parameter in self.code_parameter_range(None) {
            if let Ok(probability) = self.logical_error_rate(qubit, &parameter) {
                if probability <= required_logical_error_rate {
                    return Ok(parameter);
                }
            }
        }

        Err("No code parameter achieves required logical error rate".into())
    }

    /// Returns an iterator of all possible code parameters
    ///
    /// Implementors of this method should sort the code parameters such that
    /// the least costly parameters appear first.  Least costly may be defined
    /// in terms of physical qubits, the logical cycle time, or a combination of
    /// both.
    fn code_parameter_range(
        &self,
        lower_bound: Option<&Self::Parameter>,
    ) -> impl Iterator<Item = Self::Parameter>;

    /// Compares to code parameters
    ///
    /// A code parameter is less than another code parameter, if it requires
    /// less cost in the implementation.  The cost may be defined in terms of
    /// physical qubits, the logical cycle time, or a combination of both.
    fn code_parameter_cmp(
        &self,
        qubit: &Self::Qubit,
        p1: &Self::Parameter,
        p2: &Self::Parameter,
    ) -> Ordering;
}

pub trait FactoryBuilder<E: ErrorCorrection> {
    type Factory;

    fn find_factories(
        &self,
        ftp: &E,
        qubit: &Rc<E::Qubit>,
        output_error_rate: f64,
        max_code_parameter: &E::Parameter,
    ) -> Vec<Self::Factory>;
}

pub trait Factory
where
    Self::Parameter: std::clone::Clone,
{
    type Parameter;

    fn physical_qubits(&self) -> u64;
    fn duration(&self) -> u64;
    /// The number of magic states produced by the factory
    fn num_output_states(&self) -> u64;
    fn normalized_volume(&self) -> f64 {
        ((self.physical_qubits() * self.duration()) as f64) / (self.num_output_states() as f64)
    }
    /// The maximum code parameter setting for a magic state factory. This is
    /// used to constrain the search space, when looking for magic state
    /// factories.
    fn max_code_parameter(&self) -> Option<Cow<Self::Parameter>>;
}

pub struct PhysicalResourceEstimationResult<E: ErrorCorrection, F, L> {
    logical_patch: LogicalPatch<E>,
    num_cycles: u64,
    factory: Option<F>,
    num_factories: u64,
    required_logical_patch_error_rate: f64,
    required_logical_magic_state_error_rate: Option<f64>,
    num_factory_runs: u64,
    physical_qubits_for_factories: u64,
    physical_qubits_for_algorithm: u64,
    physical_qubits: u64,
    runtime: u64,
    rqops: u64,
    layout_overhead: L,
    error_budget: ErrorBudget,
}

impl<
        E: ErrorCorrection<Parameter = impl Clone>,
        F: Factory<Parameter = E::Parameter> + Clone,
        L: Overhead + Clone,
    > PhysicalResourceEstimationResult<E, F, L>
{
    pub fn new(
        estimation: &PhysicalResourceEstimation<E, impl FactoryBuilder<E, Factory = F>, L>,
        logical_patch: LogicalPatch<E>,
        num_cycles: u64,
        factory: Option<F>,
        num_factories: u64,
        required_logical_patch_error_rate: f64,
        required_logical_magic_state_error_rate: Option<f64>,
    ) -> Self {
        // Compute statistics for single factory
        let magic_states_per_run = factory
            .as_ref()
            .map_or(0, |factory| num_factories * factory.num_output_states());

        let num_magic_states_per_rotation = estimation
            .layout_overhead()
            .num_magic_states_per_rotation(estimation.error_budget().rotations());

        let num_factory_runs = if magic_states_per_run == 0 {
            0
        } else {
            ((estimation
                .layout_overhead
                .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
                as f64)
                / magic_states_per_run as f64)
                .ceil() as u64
        };
        let physical_qubits_for_single_factory = factory.as_ref().map_or(0, F::physical_qubits);

        // Compute statistics for all factories and total overhead
        let physical_qubits_for_factories = num_factories * physical_qubits_for_single_factory;
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
            factory,
            num_factories,
            required_logical_patch_error_rate,
            required_logical_magic_state_error_rate,
            num_factory_runs,
            physical_qubits_for_factories,
            physical_qubits_for_algorithm,
            physical_qubits,
            runtime,
            rqops,
            layout_overhead: estimation.layout_overhead().clone(),
            error_budget: estimation.error_budget().clone(),
        }
    }

    pub fn without_factories(
        estimation: &PhysicalResourceEstimation<E, impl FactoryBuilder<E, Factory = F>, L>,
        logical_patch: LogicalPatch<E>,
        num_cycles: u64,
        required_logical_patch_error_rate: f64,
    ) -> Self {
        Self::new(
            estimation,
            logical_patch,
            num_cycles,
            None,
            0,
            required_logical_patch_error_rate,
            None,
        )
    }

    pub fn logical_patch(&self) -> &LogicalPatch<E> {
        &self.logical_patch
    }

    pub fn take(self) -> (LogicalPatch<E>, Option<F>, ErrorBudget) {
        (self.logical_patch, self.factory, self.error_budget)
    }

    pub fn num_cycles(&self) -> u64 {
        self.num_cycles
    }

    pub fn factory(&self) -> Option<&F> {
        self.factory.as_ref()
    }

    pub fn num_factories(&self) -> u64 {
        self.num_factories
    }

    pub fn required_logical_patch_error_rate(&self) -> f64 {
        self.required_logical_patch_error_rate
    }

    pub fn required_logical_magic_state_error_rate(&self) -> Option<f64> {
        self.required_logical_magic_state_error_rate
    }

    pub fn num_factory_runs(&self) -> u64 {
        self.num_factory_runs
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

    pub fn layout_overhead(&self) -> &L {
        &self.layout_overhead
    }

    pub fn error_budget(&self) -> &ErrorBudget {
        &self.error_budget
    }

    pub fn algorithmic_logical_depth(&self) -> u64 {
        self.layout_overhead.logical_depth(
            self.layout_overhead
                .num_magic_states_per_rotation(self.error_budget.rotations())
                .unwrap_or_default(),
        )
    }

    pub fn num_magic_states(&self) -> u64 {
        self.layout_overhead.num_magic_states(
            self.layout_overhead
                .num_magic_states_per_rotation(self.error_budget.rotations())
                .unwrap_or_default(),
        )
    }
}

pub struct PhysicalResourceEstimation<E: ErrorCorrection, Builder, L> {
    // required parameters
    ftp: E,
    qubit: Rc<E::Qubit>,
    factory_builder: Builder,
    layout_overhead: L,
    error_budget: ErrorBudget,
    // optional constraint parameters
    logical_depth_factor: Option<f64>,
    max_factories: Option<u64>,
    max_duration: Option<u64>,
    max_physical_qubits: Option<u64>,
}

impl<
        E: ErrorCorrection<Parameter = impl Clone>,
        Builder: FactoryBuilder<E, Factory = impl Factory<Parameter = E::Parameter> + Clone>,
        L: Overhead + Clone,
    > PhysicalResourceEstimation<E, Builder, L>
{
    pub fn new(
        ftp: E,
        qubit: Rc<E::Qubit>,
        factory_builder: Builder,
        layout_overhead: L,
        error_budget: ErrorBudget,
    ) -> Self {
        Self {
            ftp,
            qubit,
            factory_builder,
            layout_overhead,
            error_budget,
            logical_depth_factor: None,
            max_factories: None,
            max_duration: None,
            max_physical_qubits: None,
        }
    }

    pub fn layout_overhead(&self) -> &L {
        &self.layout_overhead
    }

    pub fn error_budget(&self) -> &ErrorBudget {
        &self.error_budget
    }

    pub fn set_logical_depth_factor(&mut self, logical_depth_factor: f64) {
        self.logical_depth_factor = Some(logical_depth_factor);
    }
    pub fn set_max_factories(&mut self, max_factories: u64) {
        self.max_factories = Some(max_factories);
    }
    pub fn set_max_duration(&mut self, max_duration: u64) {
        self.max_duration = Some(max_duration);
    }
    pub fn set_max_physical_qubits(&mut self, max_physical_qubits: u64) {
        self.max_physical_qubits = Some(max_physical_qubits);
    }

    pub fn factory_builder_mut(&mut self) -> &mut Builder {
        &mut self.factory_builder
    }

    pub fn estimate(
        &self,
    ) -> Result<PhysicalResourceEstimationResult<E, Builder::Factory, L>, Error> {
        match (self.max_duration, self.max_physical_qubits) {
            (None, None) => self.estimate_without_restrictions(),
            (None, Some(max_physical_qubits)) => {
                self.estimate_with_max_num_qubits(max_physical_qubits)
            }
            (Some(max_duration), None) => self.estimate_with_max_duration(max_duration),
            _ => Err(Error::BothDurationAndPhysicalQubitsProvided),
        }
    }

    #[allow(clippy::too_many_lines, clippy::type_complexity)]
    pub fn build_frontier(
        &self,
    ) -> Result<Vec<PhysicalResourceEstimationResult<E, Builder::Factory, L>>, Error> {
        let num_cycles_required_by_layout_overhead = self.compute_num_cycles()?;

        // The required magic state error rate is computed by dividing the total
        // error budget for magic states by the number of magic states required
        // for the algorithm.
        let num_magic_states_per_rotation = self
            .layout_overhead
            .num_magic_states_per_rotation(self.error_budget.rotations());
        let required_logical_magic_state_error_rate = self.error_budget.magic_states()
            / self
                .layout_overhead
                .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
                as f64;

        let required_logical_error_rate =
            self.required_logical_error_rate(num_cycles_required_by_layout_overhead);

        let min_code_parameter = self
            .ftp
            .compute_code_parameter(&self.qubit, required_logical_error_rate)
            .map_err(Error::CodeParameterComputationFailed)?;

        if self
            .layout_overhead
            .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
            == 0
        {
            let logical_patch =
                LogicalPatch::new(&self.ftp, min_code_parameter, self.qubit.clone())?;

            return Ok(vec![PhysicalResourceEstimationResult::new(
                self,
                logical_patch,
                num_cycles_required_by_layout_overhead,
                None,
                0,
                required_logical_error_rate,
                None,
            )]);
        }

        let mut best_estimation_results =
            Population::<Point2D<PhysicalResourceEstimationResult<E, Builder::Factory, L>>>::new();

        let mut last_factories: Vec<Builder::Factory> = Vec::new();
        let mut last_code_parameter = None;

        for code_parameter in self
            .ftp
            .code_parameter_range(Some(&min_code_parameter))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let logical_patch =
                LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?;

            let allowed_logical_error_rate = self
                .ftp
                .logical_error_rate(&self.qubit, &code_parameter)
                .map_err(Error::LogicalErrorRateComputationFailed)?;

            let max_num_cycles_allowed_by_error_rate =
                self.logical_cycles_for_error_rate(allowed_logical_error_rate);

            if max_num_cycles_allowed_by_error_rate < num_cycles_required_by_layout_overhead {
                continue;
            }

            let max_num_cycles_allowed = max_num_cycles_allowed_by_error_rate;

            // The initial value for the last code parameter is `None`. This
            // ensures that the first code parameter is always tried. After
            // that, the last code parameter governs the reuse of the magic
            // state factory.
            if last_code_parameter.as_ref().map_or(true, |d| {
                self.ftp
                    .code_parameter_cmp(self.qubit.as_ref(), d, &code_parameter)
                    .is_gt()
            }) {
                last_factories = self.factory_builder.find_factories(
                    &self.ftp,
                    &self.qubit,
                    required_logical_magic_state_error_rate,
                    &code_parameter,
                );

                last_code_parameter = self.find_highest_code_parameter(&last_factories);
            }

            for (factory, _) in Self::pick_factories_with_num_cycles(
                &last_factories,
                &logical_patch,
                max_num_cycles_allowed,
            ) {
                // Here we compute the number of factories required limited by the
                // maximum number of cycles allowed by the duration constraint (and the error rate).
                let min_num_factories =
                    self.num_factories(&logical_patch, &factory, max_num_cycles_allowed);

                let mut num_factories = min_num_factories;

                loop {
                    let num_cycles_required_for_magic_states = self
                        .compute_num_cycles_required_for_magic_states(
                            num_factories,
                            &factory,
                            &logical_patch,
                        );

                    // This num_cycles could be larger than num_cycles_required_by_layout_overhead
                    // but must still not exceed the maximum number of cycles allowed by the
                    // duration constraint (and the error rate).
                    let num_cycles = num_cycles_required_for_magic_states
                        .max(num_cycles_required_by_layout_overhead);

                    let result = PhysicalResourceEstimationResult::new(
                        self,
                        LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?,
                        num_cycles,
                        Some(factory.clone()),
                        num_factories,
                        required_logical_error_rate,
                        Some(required_logical_magic_state_error_rate),
                    );

                    let value1 = result.physical_qubits() as f64;
                    let value2 = result.runtime();
                    let num_factory_runs = result.num_factory_runs();
                    let point = Point2D::new(result, value1, value2);
                    best_estimation_results.push(point);

                    if num_cycles_required_for_magic_states
                        <= num_cycles_required_by_layout_overhead
                        || num_factory_runs <= 1
                    {
                        break;
                    }

                    num_factories += 1;
                }
            }
        }

        best_estimation_results.filter_out_dominated();
        best_estimation_results.sort_items();

        Ok(best_estimation_results
            .extract()
            .into_iter()
            .map(|p| p.item)
            .collect())
    }

    pub fn estimate_without_restrictions(
        &self,
    ) -> Result<PhysicalResourceEstimationResult<E, Builder::Factory, L>, Error> {
        let mut num_cycles = self.compute_num_cycles()?;

        let (
            logical_patch,
            factory,
            num_factories,
            required_logical_patch_error_rate,
            required_logical_magic_state_error_rate,
        ) = loop {
            let required_logical_patch_error_rate = self.required_logical_error_rate(num_cycles);

            let code_parameter = self
                .ftp
                .compute_code_parameter(&self.qubit, required_logical_patch_error_rate)
                .map_err(Error::CodeParameterComputationFailed)?;

            let logical_patch =
                LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?;

            let num_magic_states_per_rotation = self
                .layout_overhead
                .num_magic_states_per_rotation(self.error_budget.rotations());
            if self
                .layout_overhead
                .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
                == 0
            {
                break (
                    logical_patch,
                    None,
                    0,
                    required_logical_patch_error_rate,
                    None,
                );
            }

            // The required magic state error rate is computed by dividing the total
            // error budget for magic states by the number of magic states required
            // for the algorithm.
            let required_logical_magic_state_error_rate = self.error_budget.magic_states()
                / (self
                    .layout_overhead
                    .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
                    as f64);

            let factories = self.factory_builder.find_factories(
                &self.ftp,
                &self.qubit,
                required_logical_magic_state_error_rate,
                logical_patch.code_parameter(),
            );

            let max_allowed_error_rate = self
                .ftp
                .logical_error_rate(&self.qubit, &code_parameter)
                .map_err(Error::LogicalErrorRateComputationFailed)?;
            let max_allowed_num_cycles_for_code_parameter =
                self.logical_cycles_for_error_rate(max_allowed_error_rate);

            if !factories.is_empty() {
                if let Some((factory, num_cycles_required, num_factories)) = self
                    .try_pick_factory_for_code_parameter_and_max_factories(
                        &factories,
                        &logical_patch,
                        num_cycles,
                        max_allowed_num_cycles_for_code_parameter,
                    )
                {
                    num_cycles = num_cycles_required;
                    break (
                        logical_patch,
                        Some(factory),
                        num_factories,
                        required_logical_patch_error_rate,
                        Some(required_logical_magic_state_error_rate),
                    );
                }
            }

            num_cycles = max_allowed_num_cycles_for_code_parameter + 1;
        };

        Ok(PhysicalResourceEstimationResult::new(
            self,
            logical_patch,
            num_cycles,
            factory,
            num_factories,
            required_logical_patch_error_rate,
            required_logical_magic_state_error_rate,
        ))
    }

    fn try_pick_factory_for_code_parameter_and_max_factories(
        &self,
        factories: &[Builder::Factory],
        logical_patch: &LogicalPatch<E>,
        num_cycles: u64,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Option<(Builder::Factory, u64, u64)> {
        if let Some(factory) = self
            .try_pick_factory_below_or_equal_max_duration_under_max_factories(
                factories,
                logical_patch,
                num_cycles,
            )
        {
            let num_factories = self.num_factories(logical_patch, &factory, num_cycles);
            return Some((factory, num_cycles, num_factories));
        }
        if let Some((factory, num_cycles_required)) = self
            .try_find_factory_for_code_parameter_duration_and_max_factories(
                factories,
                logical_patch,
                max_allowed_num_cycles_for_code_parameter,
            )
        {
            if num_cycles_required <= max_allowed_num_cycles_for_code_parameter {
                let num_factories =
                    self.num_factories(logical_patch, &factory, num_cycles_required);
                return Some((factory, num_cycles_required, num_factories));
            }
        }

        None
    }

    fn compute_initial_optimization_values(
        &self,
    ) -> Result<InitialOptimizationValues<E::Parameter>, Error> {
        let num_cycles_required_by_layout_overhead = self.compute_num_cycles()?;

        // The required magic state error rate is computed by dividing the total
        // error budget for magic states by the number of magic states required
        // for the algorithm.
        let num_magic_states_per_rotation = self
            .layout_overhead
            .num_magic_states_per_rotation(self.error_budget.rotations());
        let required_logical_magic_state_error_rate = self.error_budget.magic_states()
            / (self
                .layout_overhead
                .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
                as f64);

        let required_logical_error_rate =
            self.required_logical_error_rate(num_cycles_required_by_layout_overhead);

        let min_code_parameter = self
            .ftp
            .compute_code_parameter(&self.qubit, required_logical_error_rate)
            .map_err(Error::CodeParameterComputationFailed)?;

        Ok(InitialOptimizationValues {
            num_magic_states_per_rotation,
            min_code_parameter,
            num_cycles_required_by_layout_overhead,
            required_logical_error_rate,
            required_logical_magic_state_error_rate,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub fn estimate_with_max_duration(
        &self,
        max_duration_in_nanoseconds: u64,
    ) -> Result<PhysicalResourceEstimationResult<E, Builder::Factory, L>, Error> {
        let InitialOptimizationValues {
            num_magic_states_per_rotation,
            min_code_parameter,
            num_cycles_required_by_layout_overhead,
            required_logical_error_rate,
            required_logical_magic_state_error_rate,
        } = self.compute_initial_optimization_values()?;

        if self
            .layout_overhead
            .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
            == 0
        {
            let logical_patch =
                LogicalPatch::new(&self.ftp, min_code_parameter, self.qubit.clone())?;

            if num_cycles_required_by_layout_overhead * logical_patch.logical_cycle_time()
                <= max_duration_in_nanoseconds
            {
                return Ok(PhysicalResourceEstimationResult::without_factories(
                    self,
                    logical_patch,
                    num_cycles_required_by_layout_overhead,
                    required_logical_error_rate,
                ));
            }
            return Err(Error::MaxDurationTooSmall);
        }

        let mut best_estimation_result: Option<
            PhysicalResourceEstimationResult<E, Builder::Factory, L>,
        > = None;

        let mut last_factories: Vec<Builder::Factory> = Vec::new();
        let mut last_code_parameter = None;

        for code_parameter in self
            .ftp
            .code_parameter_range(Some(&min_code_parameter))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let logical_patch =
                LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?;

            let max_num_cycles_allowed_by_duration = ((max_duration_in_nanoseconds as f64)
                / logical_patch.logical_cycle_time() as f64)
                .floor() as u64;
            if max_num_cycles_allowed_by_duration < num_cycles_required_by_layout_overhead {
                continue;
            }

            let allowed_logical_error_rate = self
                .ftp
                .logical_error_rate(&self.qubit, &code_parameter)
                .map_err(Error::LogicalErrorRateComputationFailed)?;

            let max_num_cycles_allowed_by_error_rate =
                self.logical_cycles_for_error_rate(allowed_logical_error_rate);

            if max_num_cycles_allowed_by_error_rate < num_cycles_required_by_layout_overhead {
                continue;
            }

            let max_num_cycles_allowed =
                max_num_cycles_allowed_by_duration.min(max_num_cycles_allowed_by_error_rate);

            // The initial value for the last code parameter is `None`. This
            // ensures that the first code parameter is always tried. After
            // that, the last code parameter governs the reuse of the magic
            // state factory.
            if last_code_parameter.as_ref().map_or(true, |d| {
                self.ftp
                    .code_parameter_cmp(self.qubit.as_ref(), d, &code_parameter)
                    .is_gt()
            }) {
                last_factories = self.factory_builder.find_factories(
                    &self.ftp,
                    &self.qubit,
                    required_logical_magic_state_error_rate,
                    &code_parameter,
                );

                last_code_parameter = self.find_highest_code_parameter(&last_factories);
            }

            for (factory, _) in Self::pick_factories_with_num_cycles(
                &last_factories,
                &logical_patch,
                max_num_cycles_allowed,
            ) {
                let num_factories =
                    self.num_factories(&logical_patch, &factory, max_num_cycles_allowed);

                let num_cycles_required_for_magic_states = self
                    .compute_num_cycles_required_for_magic_states(
                        num_factories,
                        &factory,
                        &logical_patch,
                    );

                // This num_cycles could be larger than num_cycles_required_by_layout_overhead
                // but must still not exceed the maximum number of cycles allowed by the
                // duration constraint (and the error rate).
                let num_cycles = num_cycles_required_for_magic_states
                    .max(num_cycles_required_by_layout_overhead);

                if let Some(max_factories) = self.max_factories {
                    if num_factories > max_factories {
                        continue;
                    }
                }

                let result = PhysicalResourceEstimationResult::new(
                    self,
                    LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?,
                    num_cycles,
                    Some(factory),
                    num_factories,
                    required_logical_error_rate,
                    Some(required_logical_magic_state_error_rate),
                );

                if best_estimation_result
                    .as_ref()
                    .map_or(true, |r| result.physical_qubits() < r.physical_qubits())
                {
                    best_estimation_result = Some(result);
                }
            }
        }

        best_estimation_result.ok_or(Error::MaxDurationTooSmall)
    }

    #[allow(clippy::too_many_lines)]
    pub fn estimate_with_max_num_qubits(
        &self,
        max_num_qubits: u64,
    ) -> Result<PhysicalResourceEstimationResult<E, Builder::Factory, L>, Error> {
        let InitialOptimizationValues {
            num_magic_states_per_rotation,
            min_code_parameter,
            num_cycles_required_by_layout_overhead,
            required_logical_error_rate,
            required_logical_magic_state_error_rate,
        } = self.compute_initial_optimization_values()?;

        if self
            .layout_overhead
            .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
            == 0
        {
            let logical_patch =
                LogicalPatch::new(&self.ftp, min_code_parameter, self.qubit.clone())?;
            if self.num_algorithmic_physical_qubits(&logical_patch) <= max_num_qubits {
                return Ok(PhysicalResourceEstimationResult::without_factories(
                    self,
                    logical_patch,
                    num_cycles_required_by_layout_overhead,
                    required_logical_error_rate,
                ));
            }
            return Err(Error::MaxPhysicalQubitsTooSmall);
        }

        let mut best_estimation_result: Option<
            PhysicalResourceEstimationResult<E, Builder::Factory, L>,
        > = None;

        let mut last_factories: Vec<Builder::Factory> = Vec::new();
        let mut last_code_parameter = None;

        for code_parameter in self
            .ftp
            .code_parameter_range(Some(&min_code_parameter))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let logical_patch =
                LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?;

            let physical_qubits_for_algorithm =
                self.num_algorithmic_physical_qubits(&logical_patch);
            if max_num_qubits <= physical_qubits_for_algorithm {
                continue;
            }
            let physical_qubits_allowed_for_magic_states =
                max_num_qubits - physical_qubits_for_algorithm;

            let min_allowed_logical_error_rate = self
                .ftp
                .logical_error_rate(&self.qubit, &code_parameter)
                .map_err(Error::LogicalErrorRateComputationFailed)?;
            let max_num_cycles_allowed_by_error_rate =
                self.logical_cycles_for_error_rate(min_allowed_logical_error_rate);

            if max_num_cycles_allowed_by_error_rate < num_cycles_required_by_layout_overhead {
                continue;
            }

            // The initial value for the last code parameter is `None`. This
            // ensures that the first code parameter is always tried. After
            // that, the last code parameter governs the reuse of the magic
            // state factory.
            if last_code_parameter.as_ref().map_or(true, |d| {
                self.ftp
                    .code_parameter_cmp(self.qubit.as_ref(), d, &code_parameter)
                    .is_gt()
            }) {
                last_factories = self.factory_builder.find_factories(
                    &self.ftp,
                    &self.qubit,
                    required_logical_magic_state_error_rate,
                    &code_parameter,
                );

                last_code_parameter = self.find_highest_code_parameter(&last_factories);
            }

            if let Some(factory) = Self::try_pick_factory_below_or_equal_num_qubits(
                &last_factories,
                physical_qubits_allowed_for_magic_states,
            ) {
                // need only integer part of num_factories
                let num_factories =
                    physical_qubits_allowed_for_magic_states / factory.physical_qubits();

                if num_factories == 0 {
                    continue;
                }

                let num_cycles_required_for_magic_states = self
                    .compute_num_cycles_required_for_magic_states(
                        num_factories,
                        &factory,
                        &logical_patch,
                    );

                let num_cycles = num_cycles_required_for_magic_states
                    .max(num_cycles_required_by_layout_overhead);

                if num_cycles > max_num_cycles_allowed_by_error_rate {
                    continue;
                }

                if let Some(max_factories) = self.max_factories {
                    if num_factories > max_factories {
                        continue;
                    }
                }

                let result = PhysicalResourceEstimationResult::new(
                    self,
                    logical_patch,
                    num_cycles,
                    Some(factory),
                    num_factories,
                    required_logical_error_rate,
                    Some(required_logical_magic_state_error_rate),
                );

                if best_estimation_result
                    .as_ref()
                    .map_or(true, |r| result.runtime() < r.runtime())
                {
                    best_estimation_result = Some(result);
                }
            }
        }

        best_estimation_result.ok_or(Error::MaxPhysicalQubitsTooSmall)
    }

    /// Based on `num_factories`, we compute the number of cycles required which
    /// must be smaller than the maximum number of cycles allowed by the
    /// duration constraint (and the error rate).
    fn compute_num_cycles_required_for_magic_states(
        &self,
        num_factories: u64,
        factory: &Builder::Factory,
        logical_patch: &LogicalPatch<E>,
    ) -> u64 {
        let magic_states_per_run = num_factories * factory.num_output_states();

        let num_magic_states_per_rotation = self
            .layout_overhead
            .num_magic_states_per_rotation(self.error_budget.rotations());
        let required_runs = self
            .layout_overhead
            .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
            .div_ceil(magic_states_per_run);

        let required_duration = required_runs * factory.duration();
        required_duration.div_ceil(logical_patch.logical_cycle_time())
    }

    fn try_pick_factory_below_or_equal_num_qubits(
        factories: &[Builder::Factory],
        max_num_qubits: u64,
    ) -> Option<Builder::Factory> {
        factories
            .iter()
            .filter(|p| p.physical_qubits() <= max_num_qubits)
            .min_by(|&p, &q| {
                p.normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare factories normalized volume")
            })
            .cloned()
    }

    fn is_max_factories_constraint_satisfied(
        &self,
        logical_patch: &LogicalPatch<E>,
        factory: &Builder::Factory,
        num_cycles: u64,
    ) -> bool {
        let num_factories = self.num_factories(logical_patch, factory, num_cycles);

        if let Some(max_factories) = self.max_factories {
            if max_factories < num_factories {
                return false;
            }
        }
        true
    }

    fn try_pick_factory_below_or_equal_max_duration_under_max_factories(
        &self,
        factories: &[Builder::Factory],
        logical_patch: &LogicalPatch<E>,
        num_cycles: u64,
    ) -> Option<Builder::Factory> {
        let algorithm_duration = num_cycles * (logical_patch.logical_cycle_time());
        factories
            .iter()
            .filter(|&factory| {
                (factory.duration()) <= algorithm_duration
                    && self.is_max_factories_constraint_satisfied(
                        logical_patch,
                        factory,
                        num_cycles,
                    )
            })
            .min_by(|&p, &q| {
                p.normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare factories normalized volume")
            })
            .cloned()
    }

    fn try_find_factory_for_code_parameter_duration_and_max_factories(
        &self,
        factories: &[Builder::Factory],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Option<(Builder::Factory, u64)> {
        if let Some(max_factories) = self.max_factories {
            return self.try_pick_factory_with_num_cycles_and_max_factories(
                factories,
                logical_patch,
                max_allowed_num_cycles_for_code_parameter,
                max_factories,
            );
        }

        Self::try_pick_factory_with_num_cycles(
            factories,
            logical_patch,
            max_allowed_num_cycles_for_code_parameter,
        )
    }

    fn try_pick_factory_with_num_cycles_and_max_factories(
        &self,
        factories: &[Builder::Factory],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
        max_factories: u64,
    ) -> Option<(Builder::Factory, u64)> {
        factories
            .iter()
            .map(|factory| {
                let magic_states_per_run = max_factories * factory.num_output_states();
                let num_magic_states_per_rotation = self
                    .layout_overhead
                    .num_magic_states_per_rotation(self.error_budget.rotations());
                let required_runs = ((self
                    .layout_overhead
                    .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
                    as f64)
                    / magic_states_per_run as f64)
                    .ceil() as u64;
                let required_duration = required_runs * factory.duration();
                let num = (required_duration as f64 / logical_patch.logical_cycle_time() as f64)
                    .ceil() as u64;
                (factory.clone(), num)
            })
            .filter(|(_, num_cycles)| *num_cycles <= max_allowed_num_cycles_for_code_parameter)
            .min_by(|(p, num_p), (q, num_q)| {
                let comp1 = p
                    .normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare factories normalized volume");
                if comp1 == Ordering::Equal {
                    num_p
                        .partial_cmp(num_q)
                        .expect("Could not compare factories num cycles")
                } else {
                    comp1
                }
            })
    }

    fn try_pick_factory_with_num_cycles(
        factories: &[Builder::Factory],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Option<(Builder::Factory, u64)> {
        Self::pick_factories_with_num_cycles(
            factories,
            logical_patch,
            max_allowed_num_cycles_for_code_parameter,
        )
        .iter()
        .min_by(|(p, _), (q, _)| {
            p.normalized_volume()
                .partial_cmp(&q.normalized_volume())
                .expect("Could not compare factories normalized volume")
        })
        .cloned()
    }

    fn pick_factories_with_num_cycles(
        factories: &[Builder::Factory],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Vec<(Builder::Factory, u64)> {
        factories
            .iter()
            .map(|factory| {
                let num = (factory.duration() as f64 / logical_patch.logical_cycle_time() as f64)
                    .ceil() as u64;
                (factory.clone(), num)
            })
            .filter(|(_, num_cycles)| *num_cycles <= max_allowed_num_cycles_for_code_parameter)
            .collect()
    }

    fn find_highest_code_parameter(&self, factories: &[Builder::Factory]) -> Option<E::Parameter> {
        factories
            .iter()
            .filter_map(Factory::max_code_parameter)
            .max_by(|a, b| self.ftp.code_parameter_cmp(self.qubit.as_ref(), a, b))
            .map(Cow::into_owned)
    }

    /// Computes the number of logical patches required for the algorithm given
    /// a logical patch
    #[inline]
    fn num_logical_patches(&self, patch: &LogicalPatch<E>) -> u64 {
        self.layout_overhead
            .logical_qubits()
            .div_ceil(patch.logical_qubits())
    }

    /// Computes the number of algorithmic physical qubits given the layout
    /// overhead and a logical patch
    fn num_algorithmic_physical_qubits(&self, patch: &LogicalPatch<E>) -> u64 {
        self.num_logical_patches(patch) * patch.physical_qubits()
    }

    /// Computes required logical error rate
    ///
    /// The logical volume is the number of logical patches times the number of
    /// cycles.  We obtain the required logical error rate by dividing the error
    /// budget for logical operations by the volume.
    fn required_logical_error_rate(&self, num_cycles: u64) -> f64 {
        let volume = self.layout_overhead.logical_qubits() * num_cycles;

        self.error_budget.logical() / volume as f64
    }

    /// Computes the number of possible cycles given a logical error rate per
    /// operation
    fn logical_cycles_for_error_rate(&self, error_rate: f64) -> u64 {
        (self.error_budget.logical() / (self.layout_overhead.logical_qubits() as f64 * error_rate))
            .floor() as u64
    }

    // Possibly adjusts number of cycles C from initial starting point C_min
    fn compute_num_cycles(&self) -> Result<u64, Error> {
        // Start loop with C = C_min
        let num_magic_states_per_rotation = self
            .layout_overhead
            .num_magic_states_per_rotation(self.error_budget.rotations());
        let mut num_cycles = self
            .layout_overhead
            .logical_depth(num_magic_states_per_rotation.unwrap_or_default());

        // Perform logical depth scaling if given by constraint
        if let Some(logical_depth_scaling) = self.logical_depth_factor {
            // TODO: error handling if value is <= 1.0
            num_cycles = ((num_cycles as f64) * logical_depth_scaling).ceil() as u64;
        }

        // We cannot perform resource estimation when there are neither magic states nor cycles
        if self
            .layout_overhead
            .num_magic_states(num_magic_states_per_rotation.unwrap_or_default())
            == 0
            && num_cycles == 0
        {
            return Err(Error::AlgorithmHasNoResources);
        }

        Ok(num_cycles)
    }

    // Choose number of factories to use; we can safely use unwrap on the number
    // of magic states here, because the algorithm only finds factories that
    // provide this number
    fn num_factories(
        &self,
        logical_patch: &LogicalPatch<E>,
        factory: &Builder::Factory,
        num_cycles: u64,
    ) -> u64 {
        let num_magic_states_per_rotation = self
            .layout_overhead
            .num_magic_states_per_rotation(self.error_budget.rotations());
        let num_magic_states_big = u128::from(
            self.layout_overhead
                .num_magic_states(num_magic_states_per_rotation.unwrap_or_default()),
        );
        let duration_big = u128::from(factory.duration());
        let output_magic_count_big = u128::from(factory.num_output_states());
        let logical_cycle_time_big = u128::from(logical_patch.logical_cycle_time());
        let num_cycles_big = u128::from(num_cycles);

        let result = (num_magic_states_big * duration_big)
            .div_ceil(output_magic_count_big * logical_cycle_time_big * num_cycles_big);

        // We expect the result to be small enough to fit into a u64.
        u64::try_from(result).expect("result should fit into u64")
    }
}

struct InitialOptimizationValues<Parameter> {
    num_magic_states_per_rotation: Option<u64>,
    min_code_parameter: Parameter,
    num_cycles_required_by_layout_overhead: u64,
    required_logical_error_rate: f64,
    required_logical_magic_state_error_rate: f64,
}
