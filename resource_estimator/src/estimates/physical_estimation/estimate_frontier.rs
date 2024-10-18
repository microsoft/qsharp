use std::{borrow::Cow, ops::Deref};

use crate::estimates::{
    optimization::{Point2D, Population},
    Error, ErrorBudget, ErrorCorrection, Factory, FactoryBuilder, LogicalPatch, Overhead,
};

use super::{
    FactoryForCycles, FactoryPart, PhysicalResourceEstimation, PhysicalResourceEstimationResult,
};

pub struct EstimateFrontier<'a, E: ErrorCorrection, B: FactoryBuilder<E>, L> {
    estimator: &'a PhysicalResourceEstimation<E, B, L>,

    error_budget: ErrorBudget,
    min_cycles: u64,
    required_logical_error_rate: f64,
    required_logical_magic_state_error_rate: f64,
    num_magic_states: u64,
}

impl<
        'a,
        E: ErrorCorrection<Parameter = impl Clone>,
        B: FactoryBuilder<E, Factory = impl Factory<Parameter = E::Parameter> + Clone>,
        L: Overhead,
    > EstimateFrontier<'a, E, B, L>
{
    pub fn new(
        estimator: &'a PhysicalResourceEstimation<E, B, L>,
        error_budget: &ErrorBudget,
    ) -> Result<Self, Error> {
        if estimator.factory_builder.num_magic_state_types() == 1 {
            let min_cycles = estimator.compute_num_cycles(error_budget)?;

            let required_logical_error_rate =
                estimator.required_logical_error_rate(error_budget.logical(), min_cycles);

            // The required magic state error rate is computed by dividing the total
            // error budget for magic states by the number of magic states required
            // for the algorithm.
            let required_logical_magic_state_error_rate = error_budget.magic_states()
                / estimator.layout_overhead.num_magic_states(error_budget, 0) as f64;

            let num_magic_states = estimator.layout_overhead.num_magic_states(error_budget, 0);

            Ok(Self {
                estimator,
                error_budget: error_budget.clone(),
                min_cycles,
                required_logical_error_rate,
                required_logical_magic_state_error_rate,
                num_magic_states,
            })
        } else {
            Err(Error::MultipleMagicStatesNotSupported)
        }
    }

    pub fn estimate(&self) -> Result<Vec<PhysicalResourceEstimationResult<E, B::Factory>>, Error> {
        let min_code_parameter = self.compute_code_parameter(self.required_logical_error_rate)?;

        if self.num_magic_states == 0 {
            let logical_patch =
                LogicalPatch::new(&self.ftp, min_code_parameter, self.qubit.clone())?;

            return Ok(vec![PhysicalResourceEstimationResult::without_factories(
                self,
                logical_patch,
                &self.error_budget,
                self.min_cycles,
                self.required_logical_error_rate,
            )]);
        }

        let mut best_estimation_results = Population::new();

        let mut last_factories = Vec::new();
        let mut last_code_parameter = None;

        for code_parameter in self
            .ftp
            .code_parameter_range(Some(&min_code_parameter))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            // The initial value for the last code parameter is `None`. This
            // ensures that the first code parameter is always tried. After
            // that, the last code parameter governs the reuse of the magic
            // state factory.
            if last_code_parameter.as_ref().map_or(true, |d| {
                self.ftp
                    .code_parameter_cmp(self.qubit.as_ref(), d, &code_parameter)
                    .is_gt()
            }) {
                last_factories = self
                    .factory_builder
                    .find_factories(
                        &self.ftp,
                        &self.qubit,
                        0,
                        self.required_logical_magic_state_error_rate,
                        &code_parameter,
                    )
                    .ok_or(Error::CannotComputeMagicStates(
                        self.required_logical_magic_state_error_rate,
                    ))?;

                last_code_parameter = self.find_highest_code_parameter(&last_factories);
            }

            self.estimate_for_parameter(
                &code_parameter,
                &last_factories,
                &mut best_estimation_results,
            )?;
        }

        best_estimation_results.filter_out_dominated();
        best_estimation_results.sort_items();

        Ok(best_estimation_results
            .extract()
            .into_iter()
            .map(|p| p.item)
            .collect())
    }

    fn estimate_for_parameter<'b>(
        &self,
        code_parameter: &E::Parameter,
        factories: &[Cow<'b, B::Factory>],
        best_estimation_results: &mut Population<
            Point2D<PhysicalResourceEstimationResult<E, B::Factory>>,
        >,
    ) -> Result<(), Error>
    where
        'a: 'b,
    {
        let logical_patch =
            LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?;

        let max_num_cycles_allowed_by_error_rate =
            self.logical_cycles_for_code_parameter(self.error_budget.logical(), code_parameter)?;

        if max_num_cycles_allowed_by_error_rate < self.min_cycles {
            return Ok(());
        }

        let max_num_cycles_allowed = max_num_cycles_allowed_by_error_rate;

        for FactoryForCycles { factory, .. } in
            PhysicalResourceEstimation::<E, B, L>::pick_factories_with_num_cycles(
                factories,
                &logical_patch,
                max_num_cycles_allowed,
            )
        {
            // Here we compute the number of factories required limited by the
            // maximum number of cycles allowed by the duration constraint (and
            // the error rate).
            let min_num_factories = self.num_factories(
                &logical_patch,
                0,
                &factory,
                &self.error_budget,
                max_num_cycles_allowed,
            );

            for num_factories in min_num_factories.. {
                let num_cycles_required_for_magic_states = self
                    .compute_num_cycles_required_for_magic_states(
                        0,
                        num_factories,
                        factory.as_ref(),
                        &logical_patch,
                        &self.error_budget,
                    );

                // This num_cycles could be larger than min_cycles but must
                // still not exceed the maximum number of cycles allowed by the
                // duration constraint (and the error rate).
                let num_cycles = num_cycles_required_for_magic_states.max(self.min_cycles);

                let factory_part = FactoryPart::new(
                    factory.clone().into_owned(),
                    num_factories,
                    self.num_magic_states,
                    self.required_logical_magic_state_error_rate,
                );
                let num_factory_runs = factory_part.runs();

                let result = PhysicalResourceEstimationResult::new(
                    self,
                    LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?,
                    &self.error_budget,
                    num_cycles,
                    vec![Some(factory_part)],
                    self.required_logical_error_rate,
                );

                let physical_qubits = result.physical_qubits() as f64;
                let runtime = result.runtime();
                best_estimation_results.push(Point2D::new(result, physical_qubits, runtime));

                if num_cycles_required_for_magic_states <= self.min_cycles || num_factory_runs <= 1
                {
                    break;
                }
            }
        }

        Ok(())
    }
}

impl<
        'a,
        E: ErrorCorrection<Parameter = impl Clone>,
        B: FactoryBuilder<E, Factory = impl Factory<Parameter = E::Parameter> + Clone>,
        L: Overhead,
    > Deref for EstimateFrontier<'a, E, B, L>
{
    type Target = PhysicalResourceEstimation<E, B, L>;

    fn deref(&self) -> &Self::Target {
        self.estimator
    }
}
