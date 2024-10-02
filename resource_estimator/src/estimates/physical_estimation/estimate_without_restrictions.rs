use std::{borrow::Cow, ops::Deref};

use crate::estimates::{Error, ErrorCorrection, Factory, FactoryBuilder, LogicalPatch, Overhead};

use super::{FactoryPart, PhysicalResourceEstimation, PhysicalResourceEstimationResult};

pub struct EstimateWithoutRestrictions<'a, E: ErrorCorrection, B, L> {
    estimator: &'a PhysicalResourceEstimation<E, B, L>,
}

impl<
        'a,
        E: ErrorCorrection<Parameter = impl Clone>,
        B: FactoryBuilder<E, Factory = impl Factory<Parameter = E::Parameter> + Clone>,
        L: Overhead,
    > EstimateWithoutRestrictions<'a, E, B, L>
{
    pub fn new(estimator: &'a PhysicalResourceEstimation<E, B, L>) -> Self {
        Self { estimator }
    }

    pub fn estimate(&self) -> Result<PhysicalResourceEstimationResult<E, B::Factory>, Error> {
        let mut num_cycles = self.compute_num_cycles()?;

        loop {
            let required_logical_error_rate = self.required_logical_error_rate(num_cycles);
            let code_parameter = self.compute_code_parameter(required_logical_error_rate)?;
            let max_allowed_num_cycles_for_code_parameter =
                self.logical_cycles_for_code_parameter(&code_parameter)?;

            let logical_patch =
                LogicalPatch::new(&self.ftp, code_parameter.clone(), self.qubit.clone())?;

            let mut factory_parts = vec![];

            for index in 0..self.factory_builder.num_magic_state_types() {
                let num_magic_states = self
                    .layout_overhead
                    .num_magic_states(&self.error_budget, index);

                if num_magic_states == 0 {
                    factory_parts.push(None);
                    continue;
                }

                let required_logical_magic_state_error_rate = (self.error_budget.magic_states()
                    / self.factory_builder.num_magic_state_types() as f64)
                    / (num_magic_states as f64);

                let factories = self
                    .factory_builder
                    .find_factories(
                        &self.ftp,
                        &self.qubit,
                        index,
                        required_logical_magic_state_error_rate,
                        logical_patch.code_parameter(),
                    )
                    .ok_or(Error::CannotComputeMagicStates(
                        required_logical_magic_state_error_rate,
                    ))?;

                if factories.is_empty() {
                    break;
                }

                if let Some((factory, num_cycles_required, num_factories)) = self
                    .try_pick_factory_for_code_parameter_and_max_factories(
                        index,
                        &factories,
                        &logical_patch,
                        num_cycles,
                        max_allowed_num_cycles_for_code_parameter,
                    )
                {
                    num_cycles = num_cycles_required;
                    factory_parts.push(Some(FactoryPart::new(
                        factory.into_owned(),
                        num_factories,
                        num_magic_states,
                        required_logical_magic_state_error_rate,
                    )));
                } else {
                    break;
                }
            }

            if factory_parts.len() == self.factory_builder.num_magic_state_types() {
                return Ok(PhysicalResourceEstimationResult::new(
                    self,
                    logical_patch,
                    num_cycles,
                    factory_parts,
                    required_logical_error_rate,
                ));
            }

            num_cycles = max_allowed_num_cycles_for_code_parameter + 1;
        }
    }

    fn try_pick_factory_for_code_parameter_and_max_factories<'b>(
        &self,
        magic_state_index: usize,
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        num_cycles: u64,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Option<(Cow<'b, B::Factory>, u64, u64)> {
        if let Some(factory) = self
            .try_pick_factory_below_or_equal_max_duration_under_max_factories(
                factories,
                logical_patch,
                num_cycles,
            )
        {
            let num_factories =
                self.num_factories(logical_patch, magic_state_index, &factory, num_cycles);
            return Some((factory, num_cycles, num_factories));
        }
        if let Some((factory, num_cycles_required)) = self
            .try_find_factory_for_code_parameter_duration_and_max_factories(
                magic_state_index,
                factories,
                logical_patch,
                max_allowed_num_cycles_for_code_parameter,
            )
        {
            if num_cycles_required <= max_allowed_num_cycles_for_code_parameter {
                let num_factories = self.num_factories(
                    logical_patch,
                    magic_state_index,
                    &factory,
                    num_cycles_required,
                );
                return Some((factory, num_cycles_required, num_factories));
            }
        }

        None
    }

    fn try_pick_factory_below_or_equal_max_duration_under_max_factories<'b>(
        &self,
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        num_cycles: u64,
    ) -> Option<Cow<'b, B::Factory>> {
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

    fn try_find_factory_for_code_parameter_duration_and_max_factories<'b>(
        &self,
        magic_state_index: usize,
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Option<(Cow<'b, B::Factory>, u64)> {
        if let Some(max_factories) = self.max_factories {
            return self.try_pick_factory_with_num_cycles_and_max_factories(
                magic_state_index,
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

    fn try_pick_factory_with_num_cycles_and_max_factories<'b>(
        &self,
        magic_state_index: usize,
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
        max_factories: u64,
    ) -> Option<(Cow<'b, B::Factory>, u64)> {
        factories
            .iter()
            .map(|factory| {
                let magic_states_per_run = max_factories * factory.num_output_states();
                let required_runs = self
                    .layout_overhead
                    .num_magic_states(&self.error_budget, magic_state_index)
                    .div_ceil(magic_states_per_run);
                let required_duration = required_runs * factory.duration();
                let num = required_duration.div_ceil(logical_patch.logical_cycle_time());
                (factory.clone(), num)
            })
            .filter(|(_, num_cycles)| *num_cycles <= max_allowed_num_cycles_for_code_parameter)
            .min_by(|(p, num_p), (q, num_q)| {
                let comp1 = p
                    .normalized_volume()
                    .partial_cmp(&q.normalized_volume())
                    .expect("Could not compare factories normalized volume");

                comp1.then_with(|| {
                    num_p
                        .partial_cmp(num_q)
                        .expect("Could not compare factories num cycles")
                })
            })
    }

    fn try_pick_factory_with_num_cycles<'b>(
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        max_allowed_num_cycles_for_code_parameter: u64,
    ) -> Option<(Cow<'b, B::Factory>, u64)> {
        PhysicalResourceEstimation::<E, B, L>::pick_factories_with_num_cycles(
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

    fn is_max_factories_constraint_satisfied(
        &self,
        logical_patch: &LogicalPatch<E>,
        factory: &B::Factory,
        num_cycles: u64,
    ) -> bool {
        let num_factories = self.num_factories(logical_patch, 0, factory, num_cycles);

        if let Some(max_factories) = self.max_factories {
            if max_factories < num_factories {
                return false;
            }
        }
        true
    }
}

impl<
        'a,
        E: ErrorCorrection<Parameter = impl Clone>,
        B: FactoryBuilder<E, Factory = impl Factory<Parameter = E::Parameter> + Clone>,
        L: Overhead,
    > Deref for EstimateWithoutRestrictions<'a, E, B, L>
{
    type Target = PhysicalResourceEstimation<E, B, L>;

    fn deref(&self) -> &Self::Target {
        self.estimator
    }
}
