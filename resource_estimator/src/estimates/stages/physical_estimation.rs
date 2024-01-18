// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use super::{
    super::{
        error::InvalidInput::{
            self, BothDurationAndPhysicalQubitsProvided, InvalidCodeDistance, MaxDurationTooSmall,
            MaxPhysicalQubitsTooSmall, NoSolutionFoundForMaxTFactories, NoTFactoriesFound,
        },
        modeling::{ErrorBudget, LogicalQubit, PhysicalQubit, Protocol},
        optimization::{find_nondominated_tfactories, Point2D, Population},
        Result,
    },
    layout::Overhead,
    tfactory::{TFactory, TFactoryDistillationUnitTemplate},
};
use std::{cmp::Ordering, rc::Rc};

pub struct PhysicalResourceEstimationResult<L: Overhead + Clone> {
    logical_qubit: LogicalQubit,
    num_cycles: u64,
    tfactory: Option<TFactory>,
    num_tfactories: u64,
    required_logical_qubit_error_rate: f64,
    required_logical_tstate_error_rate: Option<f64>,
    num_tfactory_runs: u64,
    physical_qubits_for_tfactories: u64,
    physical_qubits_for_algorithm: u64,
    physical_qubits: u64,
    runtime: u64,
    rqops: u64,
    layout_overhead: L,
    error_budget: ErrorBudget,
}

impl<L: Overhead + Clone> PhysicalResourceEstimationResult<L> {
    pub fn new(
        estimation: &PhysicalResourceEstimation<L>,
        logical_qubit: LogicalQubit,
        num_cycles: u64,
        tfactory: Option<TFactory>,
        num_tfactories: u64,
        required_logical_qubit_error_rate: f64,
        required_logical_tstate_error_rate: Option<f64>,
    ) -> Self {
        // Compute statistics for single T-factory
        let t_states_per_run = tfactory
            .as_ref()
            .map_or(0, |tfactory| num_tfactories * tfactory.output_t_count());

        let num_ts_per_rotation = estimation
            .layout_overhead()
            .num_ts_per_rotation(estimation.error_budget().rotations());

        let num_tfactory_runs = if t_states_per_run == 0 {
            0
        } else {
            ((estimation
                .layout_overhead
                .num_tstates(num_ts_per_rotation.unwrap_or_default()) as f64)
                / t_states_per_run as f64)
                .ceil() as u64
        };
        let physical_qubits_for_single_tfactory =
            tfactory.as_ref().map_or(0, TFactory::physical_qubits);

        // Compute statistics for all T-factories and total overhead
        let physical_qubits_for_tfactories = num_tfactories * physical_qubits_for_single_tfactory;
        let physical_qubits_for_algorithm =
            estimation.layout_overhead.logical_qubits() * logical_qubit.physical_qubits();

        let physical_qubits = physical_qubits_for_algorithm + physical_qubits_for_tfactories;

        let runtime = (logical_qubit.logical_cycle_time()) * num_cycles;

        let rqops = (estimation.layout_overhead().logical_qubits() as f64
            * logical_qubit.logical_cycles_per_second())
        .ceil() as u64;

        Self {
            logical_qubit,
            num_cycles,
            tfactory,
            num_tfactories,
            required_logical_qubit_error_rate,
            required_logical_tstate_error_rate,
            num_tfactory_runs,
            physical_qubits_for_tfactories,
            physical_qubits_for_algorithm,
            physical_qubits,
            runtime,
            rqops,
            layout_overhead: estimation.layout_overhead().clone(),
            error_budget: estimation.error_budget().clone(),
        }
    }

    pub fn logical_qubit(&self) -> &LogicalQubit {
        &self.logical_qubit
    }

    pub fn take(self) -> (LogicalQubit, Option<TFactory>, ErrorBudget) {
        (self.logical_qubit, self.tfactory, self.error_budget)
    }

    pub fn num_cycles(&self) -> u64 {
        self.num_cycles
    }

    pub fn tfactory(&self) -> Option<&TFactory> {
        self.tfactory.as_ref()
    }

    pub fn num_tfactories(&self) -> u64 {
        self.num_tfactories
    }

    pub fn required_logical_qubit_error_rate(&self) -> f64 {
        self.required_logical_qubit_error_rate
    }

    pub fn required_logical_tstate_error_rate(&self) -> Option<f64> {
        self.required_logical_tstate_error_rate
    }

    pub fn num_tfactory_runs(&self) -> u64 {
        self.num_tfactory_runs
    }

    pub fn physical_qubits_for_tfactories(&self) -> u64 {
        self.physical_qubits_for_tfactories
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
                .num_ts_per_rotation(self.error_budget.rotations())
                .unwrap_or_default(),
        )
    }

    pub fn num_tstates(&self) -> u64 {
        self.layout_overhead.num_tstates(
            self.layout_overhead
                .num_ts_per_rotation(self.error_budget.rotations())
                .unwrap_or_default(),
        )
    }
}

pub struct PhysicalResourceEstimation<L: Overhead> {
    // required parameters
    ftp: Protocol,
    qubit: Rc<PhysicalQubit>,
    layout_overhead: L,
    error_budget: ErrorBudget,
    // optional constraint parameters
    logical_depth_factor: Option<f64>,
    max_t_factories: Option<u64>,
    max_duration: Option<u64>,
    max_physical_qubits: Option<u64>,
    // distillation unit parameters
    distillation_unit_templates: Vec<TFactoryDistillationUnitTemplate>,
}

impl<L: Overhead + Clone> PhysicalResourceEstimation<L> {
    pub fn new(
        ftp: Protocol,
        qubit: Rc<PhysicalQubit>,
        layout_overhead: L,
        error_budget: ErrorBudget,
    ) -> Self {
        Self {
            ftp,
            qubit,
            layout_overhead,
            error_budget,
            logical_depth_factor: None,
            max_t_factories: None,
            max_duration: None,
            max_physical_qubits: None,
            distillation_unit_templates:
                TFactoryDistillationUnitTemplate::default_distillation_unit_templates(),
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
    pub fn set_max_t_factories(&mut self, max_t_factories: u64) {
        self.max_t_factories = Some(max_t_factories);
    }
    pub fn set_max_duration(&mut self, max_duration: u64) {
        self.max_duration = Some(max_duration);
    }
    pub fn set_max_physical_qubits(&mut self, max_physical_qubits: u64) {
        self.max_physical_qubits = Some(max_physical_qubits);
    }

    pub fn set_distillation_unit_templates(
        &mut self,
        distillation_unit_templates: Vec<TFactoryDistillationUnitTemplate>,
    ) {
        self.distillation_unit_templates = distillation_unit_templates;
    }

    pub fn estimate(&self) -> Result<PhysicalResourceEstimationResult<L>> {
        match (self.max_duration, self.max_physical_qubits) {
            (None, None) => self.estimate_without_restrictions(),
            (None, Some(max_physical_qubits)) => {
                self.estimate_with_max_num_qubits(max_physical_qubits)
            }
            (Some(max_duration), None) => self.estimate_with_max_duration(max_duration),
            _ => Err(BothDurationAndPhysicalQubitsProvided.into()),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn build_frontier(&self) -> Result<Vec<PhysicalResourceEstimationResult<L>>> {
        let num_cycles_required_by_layout_overhead = self.compute_num_cycles()?;

        // The required T-state error rate is computed by dividing the total
        // error budget for T states by the number of T-states required for the
        // algorithm.
        let num_ts_per_rotation = self
            .layout_overhead
            .num_ts_per_rotation(self.error_budget.rotations());
        let required_logical_tstate_error_rate = self.error_budget.tstates()
            / self
                .layout_overhead
                .num_tstates(num_ts_per_rotation.unwrap_or_default()) as f64;

        // Required logical error rate (\eps_{\log} / (Q * C) in the paper)
        let required_logical_qubit_error_rate = self.error_budget.logical()
            / (self.layout_overhead.logical_qubits() * num_cycles_required_by_layout_overhead)
                as f64;

        let min_code_distance = self.compute_code_distance(required_logical_qubit_error_rate);
        let max_code_distance = self.ftp.max_code_distance();

        if min_code_distance > max_code_distance {
            return Err(InvalidCodeDistance(min_code_distance, max_code_distance).into());
        }

        if self
            .layout_overhead
            .num_tstates(num_ts_per_rotation.unwrap_or_default())
            == 0
        {
            let logical_qubit =
                LogicalQubit::new(&self.ftp, min_code_distance, self.qubit.clone())?;

            return Ok(vec![PhysicalResourceEstimationResult::new(
                self,
                logical_qubit,
                num_cycles_required_by_layout_overhead,
                None,
                0,
                required_logical_qubit_error_rate,
                None,
            )]);
        }

        let mut best_estimation_results =
            Population::<Point2D<PhysicalResourceEstimationResult<L>>>::new();

        let max_odd_code_distance = self.get_max_odd_code_distance();
        let mut last_tfactories: Vec<TFactory> = Vec::new();
        let mut last_code_distance = max_code_distance + 1;

        for code_distance in (min_code_distance..=max_odd_code_distance).rev().step_by(2) {
            let logical_qubit = LogicalQubit::new(&self.ftp, code_distance, self.qubit.clone())?;

            let allowed_logical_qubit_error_rate = self.compute_logical_error_rate(code_distance);

            let max_num_cycles_allowed_by_error_rate = (self.error_budget.logical()
                / (self.layout_overhead.logical_qubits() as f64 * allowed_logical_qubit_error_rate))
                .floor() as u64;

            if max_num_cycles_allowed_by_error_rate < num_cycles_required_by_layout_overhead {
                continue;
            }

            let max_num_cycles_allowed = max_num_cycles_allowed_by_error_rate;

            // The initial value for the last code distance
            // is max_code_distance + 1 which is larger than any code distance in the loop.
            // This ensures that the first code distance is always tried.
            // After that, the last code distance governs the reuse of T-factory.
            if last_code_distance > code_distance {
                last_tfactories = find_nondominated_tfactories(
                    &self.ftp,
                    &self.qubit,
                    &self.distillation_unit_templates,
                    required_logical_tstate_error_rate,
                    code_distance,
                );

                last_code_distance = Self::find_highest_code_distance(&last_tfactories);
            }

            if let Some((tfactory, _)) = Self::try_pick_tfactory_with_num_cycles(
                &last_tfactories,
                &logical_qubit,
                max_num_cycles_allowed,
            ) {
                // Here we compute the number of T-factories required limited by the
                // maximum number of cycles allowed by the duration constraint (and the error rate).
                let min_num_tfactories =
                    self.num_tfactories(&logical_qubit, &tfactory, max_num_cycles_allowed);

                let mut num_tfactories = min_num_tfactories;

                loop {
                    // Based on the num_tfactories we compute the number of cycles required
                    // which must be smaller than the maximum number of cycles allowed by the
                    // duration constraint (and the error rate).
                    let num_cycles_required_for_tstates = self
                        .compute_num_cycles_required_for_tstates(
                            num_tfactories,
                            &tfactory,
                            &logical_qubit,
                        );

                    // This num_cycles could be larger than num_cycles_required_by_layout_overhead
                    // but must still not exceed the maximum number of cycles allowed by the
                    // duration constraint (and the error rate).
                    let num_cycles =
                        num_cycles_required_for_tstates.max(num_cycles_required_by_layout_overhead);

                    let result = PhysicalResourceEstimationResult::new(
                        self,
                        LogicalQubit::new(&self.ftp, code_distance, self.qubit.clone())?,
                        num_cycles,
                        Some(tfactory.clone()),
                        num_tfactories,
                        required_logical_qubit_error_rate,
                        Some(required_logical_tstate_error_rate),
                    );

                    let value1 = result.runtime() as f64;
                    let value2 = result.physical_qubits();
                    let num_t_factory_runs = result.num_tfactory_runs();
                    let point = Point2D::new(result, value1, value2);
                    best_estimation_results.push(point);

                    if num_cycles_required_for_tstates <= num_cycles_required_by_layout_overhead
                        || num_t_factory_runs <= 1
                    {
                        break;
                    }

                    num_tfactories += 1;
                }
            }
        }

        best_estimation_results.filter_out_dominated();

        Ok(best_estimation_results
            .extract()
            .into_iter()
            .map(|p| p.item)
            .collect())
    }

    fn estimate_without_restrictions(&self) -> Result<PhysicalResourceEstimationResult<L>> {
        let mut num_cycles = self.compute_num_cycles()?;

        let mut loaded_tfactories_at_least_once = false;

        let (
            logical_qubit,
            tfactory,
            num_tfactories,
            required_logical_qubit_error_rate,
            required_logical_tstate_error_rate,
        ) = loop {
            // Required logical error rate (\eps_{\log} / (Q * C) in the paper)
            let required_logical_qubit_error_rate = self.error_budget.logical()
                / ((self.layout_overhead.logical_qubits()) * num_cycles) as f64;

            let code_distance = self.compute_code_distance(required_logical_qubit_error_rate);

            if code_distance > self.ftp.max_code_distance() {
                if !loaded_tfactories_at_least_once {
                    return Err(NoTFactoriesFound.into());
                }

                if self.max_t_factories.is_some() {
                    return Err(NoSolutionFoundForMaxTFactories.into());
                }

                return Err(
                    InvalidCodeDistance(code_distance, self.ftp.max_code_distance()).into(),
                );
            }

            let logical_qubit = LogicalQubit::new(&self.ftp, code_distance, self.qubit.clone())?;

            let num_ts_per_rotation = self
                .layout_overhead
                .num_ts_per_rotation(self.error_budget.rotations());
            if self
                .layout_overhead
                .num_tstates(num_ts_per_rotation.unwrap_or_default())
                == 0
            {
                break (
                    logical_qubit,
                    None,
                    0,
                    required_logical_qubit_error_rate,
                    None,
                );
            }
            // The required T-state error rate is computed by dividing the total
            // error budget for T states by the number of T-states required for the
            // algorithm.
            let required_logical_tstate_error_rate = self.error_budget.tstates()
                / (self
                    .layout_overhead
                    .num_tstates(num_ts_per_rotation.unwrap_or_default())
                    as f64);

            let factories = find_nondominated_tfactories(
                &self.ftp,
                &self.qubit,
                &self.distillation_unit_templates,
                required_logical_tstate_error_rate,
                logical_qubit.code_distance(),
            );

            let max_allowed_error_rate = self.compute_logical_error_rate(code_distance);
            let max_allowed_num_cycles_for_code_distance = (self.error_budget.logical()
                / (self.layout_overhead.logical_qubits() as f64 * max_allowed_error_rate))
                .floor() as u64;

            if !factories.is_empty() {
                loaded_tfactories_at_least_once = true;
                if let Some((tfactory, num_cycles_required, num_tfactories)) = self
                    .try_pick_tfactory_for_code_distance_and_max_tfactories(
                        &factories,
                        &logical_qubit,
                        num_cycles,
                        max_allowed_num_cycles_for_code_distance,
                    )
                {
                    num_cycles = num_cycles_required;
                    break (
                        logical_qubit,
                        Some(tfactory),
                        num_tfactories,
                        required_logical_qubit_error_rate,
                        Some(required_logical_tstate_error_rate),
                    );
                }
            }

            num_cycles = max_allowed_num_cycles_for_code_distance + 1;
        };

        Ok(PhysicalResourceEstimationResult::new(
            self,
            logical_qubit,
            num_cycles,
            tfactory,
            num_tfactories,
            required_logical_qubit_error_rate,
            required_logical_tstate_error_rate,
        ))
    }

    fn try_pick_tfactory_for_code_distance_and_max_tfactories(
        &self,
        factories: &[TFactory],
        logical_qubit: &LogicalQubit,
        num_cycles: u64,
        max_allowed_num_cycles_for_code_distance: u64,
    ) -> Option<(TFactory, u64, u64)> {
        if let Some(tfactory) = self
            .try_pick_tfactory_below_or_equal_max_duration_under_max_t_factories(
                factories,
                logical_qubit,
                num_cycles,
            )
        {
            let num_tfactories = self.num_tfactories(logical_qubit, &tfactory, num_cycles);
            return Some((tfactory, num_cycles, num_tfactories));
        }
        if let Some((tfactory, num_cycles_required)) = self
            .try_find_tfactory_for_code_distance_duration_and_max_t_factories(
                factories,
                logical_qubit,
                max_allowed_num_cycles_for_code_distance,
            )
        {
            if num_cycles_required <= max_allowed_num_cycles_for_code_distance {
                let num_tfactories =
                    self.num_tfactories(logical_qubit, &tfactory, num_cycles_required);
                return Some((tfactory, num_cycles_required, num_tfactories));
            }
        }

        None
    }

    #[allow(clippy::too_many_lines)]
    fn estimate_with_max_duration(
        &self,
        max_duration_in_nanoseconds: u64,
    ) -> Result<PhysicalResourceEstimationResult<L>> {
        let num_cycles_required_by_layout_overhead = self.compute_num_cycles()?;

        // The required T-state error rate is computed by dividing the total
        // error budget for T states by the number of T-states required for the
        // algorithm.
        let num_ts_per_rotation = self
            .layout_overhead
            .num_ts_per_rotation(self.error_budget.rotations());
        let required_logical_tstate_error_rate = self.error_budget.tstates()
            / (self
                .layout_overhead
                .num_tstates(num_ts_per_rotation.unwrap_or_default()) as f64);

        // Required logical error rate (\eps_{\log} / (Q * C) in the paper)
        let required_logical_qubit_error_rate = self.error_budget.logical()
            / ((self.layout_overhead.logical_qubits() * num_cycles_required_by_layout_overhead)
                as f64);

        let min_code_distance = self.compute_code_distance(required_logical_qubit_error_rate);
        let max_code_distance = self.ftp.max_code_distance();

        if min_code_distance > max_code_distance {
            return Err(InvalidCodeDistance(min_code_distance, max_code_distance).into());
        }

        if self
            .layout_overhead
            .num_tstates(num_ts_per_rotation.unwrap_or_default())
            == 0
        {
            let logical_qubit =
                LogicalQubit::new(&self.ftp, min_code_distance, self.qubit.clone())?;

            if num_cycles_required_by_layout_overhead * (logical_qubit.logical_cycle_time())
                <= (max_duration_in_nanoseconds)
            {
                return Ok(PhysicalResourceEstimationResult::new(
                    self,
                    logical_qubit,
                    num_cycles_required_by_layout_overhead,
                    None,
                    0,
                    required_logical_qubit_error_rate,
                    None,
                ));
            }
            return Err(MaxDurationTooSmall.into());
        }

        let mut best_estimation_result: Option<PhysicalResourceEstimationResult<L>> = None;

        let max_odd_code_distance = self.get_max_odd_code_distance();
        let mut last_tfactories: Vec<TFactory> = Vec::new();
        let mut last_code_distance = max_code_distance + 1;

        for code_distance in (min_code_distance..=max_odd_code_distance).rev().step_by(2) {
            let logical_qubit = LogicalQubit::new(&self.ftp, code_distance, self.qubit.clone())?;

            let max_num_cycles_allowed_by_duration = ((max_duration_in_nanoseconds as f64)
                / logical_qubit.logical_cycle_time() as f64)
                .floor() as u64;
            if max_num_cycles_allowed_by_duration < num_cycles_required_by_layout_overhead {
                continue;
            }

            let allowed_logical_qubit_error_rate = self.compute_logical_error_rate(code_distance);

            let max_num_cycles_allowed_by_error_rate = (self.error_budget.logical()
                / (self.layout_overhead.logical_qubits() as f64 * allowed_logical_qubit_error_rate))
                .floor() as u64;

            if max_num_cycles_allowed_by_error_rate < num_cycles_required_by_layout_overhead {
                continue;
            }

            let max_num_cycles_allowed =
                max_num_cycles_allowed_by_duration.min(max_num_cycles_allowed_by_error_rate);

            // The initial value for the last code distance
            // is max_code_distance + 1 which is larger than any code distance in the loop.
            // This ensures that the first code distance is always tried.
            // After that, the last code distance governs the reuse of T-factory.
            if last_code_distance > code_distance {
                last_tfactories = find_nondominated_tfactories(
                    &self.ftp,
                    &self.qubit,
                    &self.distillation_unit_templates,
                    required_logical_tstate_error_rate,
                    code_distance,
                );

                last_code_distance = Self::find_highest_code_distance(&last_tfactories);
            }

            if let Some((tfactory, _)) = Self::try_pick_tfactory_with_num_cycles(
                &last_tfactories,
                &logical_qubit,
                max_num_cycles_allowed,
            ) {
                // Here we compute the number of T-factories required limited by the
                // maximum number of cycles allowed by the duration constraint (and the error rate).
                let num_tfactories =
                    self.num_tfactories(&logical_qubit, &tfactory, max_num_cycles_allowed);

                // Based on the num_tfactories we compute the number of cycles required
                // which must be smaller than the maximum number of cycles allowed by the
                // duration constraint (and the error rate).
                let num_cycles_required_for_tstates = self.compute_num_cycles_required_for_tstates(
                    num_tfactories,
                    &tfactory,
                    &logical_qubit,
                );

                // This num_cycles could be larger than num_cycles_required_by_layout_overhead
                // but must still not exceed the maximum number of cycles allowed by the
                // duration constraint (and the error rate).
                let num_cycles =
                    num_cycles_required_for_tstates.max(num_cycles_required_by_layout_overhead);

                if let Some(max_tfactories) = self.max_t_factories {
                    if num_tfactories > max_tfactories {
                        continue;
                    }
                }

                let result = PhysicalResourceEstimationResult::new(
                    self,
                    logical_qubit,
                    num_cycles,
                    Some(tfactory),
                    num_tfactories,
                    required_logical_qubit_error_rate,
                    Some(required_logical_tstate_error_rate),
                );

                if best_estimation_result
                    .as_ref()
                    .map_or(true, |r| result.physical_qubits() < r.physical_qubits())
                {
                    best_estimation_result = Some(result);
                }
            }
        }

        best_estimation_result.ok_or_else(|| MaxDurationTooSmall.into())
    }

    #[allow(clippy::too_many_lines)]
    fn estimate_with_max_num_qubits(
        &self,
        max_num_qubits: u64,
    ) -> Result<PhysicalResourceEstimationResult<L>> {
        let min_num_cycles_required_by_layout_overhead = self.compute_num_cycles()?;

        // The required T-state error rate is computed by dividing the total
        // error budget for T states by the number of T-states required for the
        // algorithm.
        let num_ts_per_rotation = self
            .layout_overhead
            .num_ts_per_rotation(self.error_budget.rotations());
        let required_logical_tstate_error_rate = self.error_budget.tstates()
            / (self
                .layout_overhead
                .num_tstates(num_ts_per_rotation.unwrap_or_default()) as f64);

        // Required logical error rate (\eps_{\log} / (Q * C) in the paper)
        let required_logical_qubit_error_rate = self.error_budget.logical()
            / ((self.layout_overhead.logical_qubits()) * min_num_cycles_required_by_layout_overhead)
                as f64;

        let min_code_distance = self.compute_code_distance(required_logical_qubit_error_rate);
        let max_code_distance = self.ftp.max_code_distance();

        if min_code_distance > max_code_distance {
            return Err(InvalidCodeDistance(min_code_distance, max_code_distance).into());
        }

        if self
            .layout_overhead
            .num_tstates(num_ts_per_rotation.unwrap_or_default())
            == 0
        {
            let logical_qubit =
                LogicalQubit::new(&self.ftp, min_code_distance, self.qubit.clone())?;
            if self.layout_overhead.logical_qubits() * logical_qubit.physical_qubits()
                <= max_num_qubits
            {
                return Ok(PhysicalResourceEstimationResult::new(
                    self,
                    logical_qubit,
                    min_num_cycles_required_by_layout_overhead,
                    None,
                    0,
                    required_logical_qubit_error_rate,
                    None,
                ));
            }
            return Err(MaxPhysicalQubitsTooSmall.into());
        }

        let mut best_estimation_result: Option<PhysicalResourceEstimationResult<L>> = None;

        let max_odd_code_distance = self.get_max_odd_code_distance();
        let mut last_tfactories: Vec<TFactory> = Vec::new();
        let mut last_code_distance = max_code_distance + 1;

        for code_distance in (min_code_distance..=max_odd_code_distance).rev().step_by(2) {
            let logical_qubit = LogicalQubit::new(&self.ftp, code_distance, self.qubit.clone())?;

            let physical_qubits_for_algorithm =
                self.layout_overhead.logical_qubits() * logical_qubit.physical_qubits();
            if max_num_qubits <= physical_qubits_for_algorithm {
                continue;
            }
            let physical_qubits_allowed_for_t_states =
                max_num_qubits - physical_qubits_for_algorithm;

            let min_allowed_logical_qubit_error_rate =
                self.compute_logical_error_rate(code_distance);
            let max_num_cycles_allowed_by_error_rate = (self.error_budget.logical()
                / (self.layout_overhead.logical_qubits() as f64
                    * min_allowed_logical_qubit_error_rate))
                .floor() as u64;

            if max_num_cycles_allowed_by_error_rate < min_num_cycles_required_by_layout_overhead {
                continue;
            }

            // The initial value for the last code distance
            // is max_code_distance + 1 which is larger than any code distance in the loop.
            // This ensures that the first code distance is always tried.
            // After that, the last code distance governs the reuse of T-factory.
            if last_code_distance > code_distance {
                last_tfactories = find_nondominated_tfactories(
                    &self.ftp,
                    &self.qubit,
                    &self.distillation_unit_templates,
                    required_logical_tstate_error_rate,
                    code_distance,
                );

                last_code_distance = Self::find_highest_code_distance(&last_tfactories);
            }

            if let Some(tfactory) = Self::try_pick_tfactory_below_or_equal_num_qubits(
                &last_tfactories,
                physical_qubits_allowed_for_t_states,
            ) {
                // need only integer part of num_factories
                let num_tfactories =
                    physical_qubits_allowed_for_t_states / tfactory.physical_qubits();

                if num_tfactories == 0 {
                    continue;
                }

                let num_cycles_required_for_tstates = self.compute_num_cycles_required_for_tstates(
                    num_tfactories,
                    &tfactory,
                    &logical_qubit,
                );

                let num_cycles =
                    num_cycles_required_for_tstates.max(min_num_cycles_required_by_layout_overhead);

                if num_cycles > max_num_cycles_allowed_by_error_rate {
                    continue;
                }

                if let Some(max_tfactories) = self.max_t_factories {
                    if num_tfactories > max_tfactories {
                        continue;
                    }
                }

                let result = PhysicalResourceEstimationResult::new(
                    self,
                    logical_qubit,
                    num_cycles,
                    Some(tfactory),
                    num_tfactories,
                    required_logical_qubit_error_rate,
                    Some(required_logical_tstate_error_rate),
                );

                if best_estimation_result
                    .as_ref()
                    .map_or(true, |r| result.runtime() < r.runtime())
                {
                    best_estimation_result = Some(result);
                }
            }
        }

        best_estimation_result.ok_or_else(|| MaxPhysicalQubitsTooSmall.into())
    }

    fn compute_num_cycles_required_for_tstates(
        &self,
        num_tfactories: u64,
        tfactory: &TFactory,
        logical_qubit: &LogicalQubit,
    ) -> u64 {
        let tstates_per_run = num_tfactories * tfactory.output_t_count();

        let num_ts_per_rotation = self
            .layout_overhead
            .num_ts_per_rotation(self.error_budget.rotations());
        let required_runs = self
            .layout_overhead
            .num_tstates(num_ts_per_rotation.unwrap_or_default())
            .div_ceil(tstates_per_run);

        let required_duration = required_runs * tfactory.duration();
        required_duration.div_ceil(logical_qubit.logical_cycle_time())
    }

    fn try_pick_tfactory_below_or_equal_num_qubits(
        factories: &[TFactory],
        max_num_qubits: u64,
    ) -> Option<TFactory> {
        factories
            .iter()
            .filter(|p| p.physical_qubits() <= max_num_qubits)
            .min_by(|&p, &q| {
                p.normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare T-factories normalized volume")
            })
            .cloned()
    }

    fn is_max_t_factories_constraint_satisfied(
        &self,
        logical_qubit: &LogicalQubit,
        tfactory: &TFactory,
        num_cycles: u64,
    ) -> bool {
        let num_tfactories = self.num_tfactories(logical_qubit, tfactory, num_cycles);

        if let Some(max_tfactories) = self.max_t_factories {
            if max_tfactories < num_tfactories {
                return false;
            }
        }
        true
    }

    fn try_pick_tfactory_below_or_equal_max_duration_under_max_t_factories(
        &self,
        factories: &[TFactory],
        logical_qubit: &LogicalQubit,
        num_cycles: u64,
    ) -> Option<TFactory> {
        let algorithm_duration = num_cycles * (logical_qubit.logical_cycle_time());
        factories
            .iter()
            .filter(|&tfactory| {
                (tfactory.duration()) <= algorithm_duration
                    && self.is_max_t_factories_constraint_satisfied(
                        logical_qubit,
                        tfactory,
                        num_cycles,
                    )
            })
            .min_by(|&p, &q| {
                p.normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare T-factories normalized volume")
            })
            .cloned()
    }

    fn try_find_tfactory_for_code_distance_duration_and_max_t_factories(
        &self,
        factories: &[TFactory],
        logical_qubit: &LogicalQubit,
        max_allowed_num_cycles_for_code_distance: u64,
    ) -> Option<(TFactory, u64)> {
        if let Some(max_tfactories) = self.max_t_factories {
            return self.try_pick_tfactory_with_num_cycles_and_max_tfactories(
                factories,
                logical_qubit,
                max_allowed_num_cycles_for_code_distance,
                max_tfactories,
            );
        }

        Self::try_pick_tfactory_with_num_cycles(
            factories,
            logical_qubit,
            max_allowed_num_cycles_for_code_distance,
        )
    }

    fn try_pick_tfactory_with_num_cycles_and_max_tfactories(
        &self,
        factories: &[TFactory],
        logical_qubit: &LogicalQubit,
        max_allowed_num_cycles_for_code_distance: u64,
        max_tfactories: u64,
    ) -> Option<(TFactory, u64)> {
        factories
            .iter()
            .map(|factory| {
                let tstates_per_run = max_tfactories * factory.output_t_count();
                let num_ts_per_rotation = self
                    .layout_overhead
                    .num_ts_per_rotation(self.error_budget.rotations());
                let required_runs = ((self
                    .layout_overhead
                    .num_tstates(num_ts_per_rotation.unwrap_or_default())
                    as f64)
                    / tstates_per_run as f64)
                    .ceil() as u64;
                let required_duration = required_runs * factory.duration();
                let num = (required_duration as f64 / logical_qubit.logical_cycle_time() as f64)
                    .ceil() as u64;
                (factory.clone(), num)
            })
            .filter(|(_, num_cycles)| *num_cycles <= max_allowed_num_cycles_for_code_distance)
            .min_by(|(p, num_p), (q, num_q)| {
                let comp1 = p
                    .normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare T-factories normalized volume");
                if comp1 == Ordering::Equal {
                    num_p
                        .partial_cmp(num_q)
                        .expect("Could not compare T-factories num cycles")
                } else {
                    comp1
                }
            })
    }

    fn try_pick_tfactory_with_num_cycles(
        factories: &[TFactory],
        logical_qubit: &LogicalQubit,
        max_allowed_num_cycles_for_code_distance: u64,
    ) -> Option<(TFactory, u64)> {
        factories
            .iter()
            .map(|factory| {
                let num = (factory.duration() as f64 / logical_qubit.logical_cycle_time() as f64)
                    .ceil() as u64;
                (factory.clone(), num)
            })
            .filter(|(_, num_cycles)| *num_cycles <= max_allowed_num_cycles_for_code_distance)
            .min_by(|(p, _), (q, _)| {
                p.normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare T-factories normalized volume")
            })
    }

    fn find_highest_code_distance(factories: &[TFactory]) -> u64 {
        factories
            .iter()
            .filter_map(|p| p.code_distance_per_round().last().copied())
            .max()
            .unwrap_or(0)
    }

    fn get_max_odd_code_distance(&self) -> u64 {
        let max_code_distance = self.ftp.max_code_distance();
        if max_code_distance % 2 == 0 {
            max_code_distance - 1
        } else {
            max_code_distance
        }
    }

    // Compute code distance d (Equation (E2) in paper)
    fn compute_code_distance(&self, required_logical_qubit_error_rate: f64) -> u64 {
        let numerator =
            2.0 * (self.ftp.crossing_prefactor() / required_logical_qubit_error_rate).ln();
        let denominator =
            (self.ftp.error_correction_threshold() / self.qubit.clifford_error_rate()).ln();

        (((numerator / denominator) - 1.0).ceil() as u64) | 0x1
    }

    fn compute_logical_error_rate(&self, code_distance: u64) -> f64 {
        self.ftp.crossing_prefactor()
            / (self.ftp.error_correction_threshold() / self.qubit.clifford_error_rate())
                .powf(((code_distance + 1) / 2) as f64)
    }

    // Possibly adjusts number of cycles C from initial starting point C_min
    fn compute_num_cycles(&self) -> Result<u64> {
        // Start loop with C = C_min
        let num_ts_per_rotation = self
            .layout_overhead
            .num_ts_per_rotation(self.error_budget.rotations());
        let mut num_cycles = self
            .layout_overhead
            .logical_depth(num_ts_per_rotation.unwrap_or_default());

        // Perform logical depth scaling if given by constraint
        if let Some(logical_depth_scaling) = self.logical_depth_factor {
            // TODO: error handling if value is <= 1.0
            num_cycles = ((num_cycles as f64) * logical_depth_scaling).ceil() as u64;
        }

        // We cannot perform resource estimation when there are neither T-states nor cycles
        if self
            .layout_overhead
            .num_tstates(num_ts_per_rotation.unwrap_or_default())
            == 0
            && num_cycles == 0
        {
            return Err(InvalidInput::AlgorithmHasNoResources.into());
        }

        Ok(num_cycles)
    }

    // Choose number of T factories to use; we can safely use unwrap on
    // the t_count here because the algorithm only finds T-factories
    // that provide this number
    fn num_tfactories(
        &self,
        logical_qubit: &LogicalQubit,
        tfactory: &TFactory,
        num_cycles: u64,
    ) -> u64 {
        let num_ts_per_rotation = self
            .layout_overhead
            .num_ts_per_rotation(self.error_budget.rotations());
        let num_tstates_big = u128::from(
            self.layout_overhead
                .num_tstates(num_ts_per_rotation.unwrap_or_default()),
        );
        let duration_big = u128::from(tfactory.duration());
        let output_t_count_big = u128::from(tfactory.output_t_count());
        let logical_cycle_time_big = u128::from(logical_qubit.logical_cycle_time());
        let num_cycles_big = u128::from(num_cycles);

        let result = num_tstates_big * duration_big
            / (output_t_count_big * logical_cycle_time_big * num_cycles_big);

        let rem = num_tstates_big * duration_big
            % (output_t_count_big * logical_cycle_time_big * num_cycles_big);

        // We expect the result to be small enough to fit into a u64.
        let result_u64 = u64::try_from(result).expect("result should fit into u64");

        if rem == 0 {
            result_u64
        } else {
            result_u64 + 1
        }
    }
}
