use std::{borrow::Cow, ops::Deref};

use crate::estimates::{Error, ErrorCorrection, Factory, FactoryBuilder, LogicalPatch, Overhead};

use super::{
    FactoryForCycles, FactoryPart, PhysicalResourceEstimation, PhysicalResourceEstimationResult,
};

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
                match self.compute_factory_part_for_index(
                    &logical_patch,
                    num_cycles,
                    max_allowed_num_cycles_for_code_parameter,
                    index,
                )? {
                    FactoryPartsResult::NoMagicStates => {
                        factory_parts.push(None);
                    }
                    FactoryPartsResult::NoFactories | FactoryPartsResult::NoSuitableFactory => {
                        break
                    }
                    FactoryPartsResult::Success {
                        factory_part,
                        num_cycles: num_required_cycles,
                    } => {
                        num_cycles = num_required_cycles;
                        factory_parts.push(Some(factory_part));
                    }
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

    fn compute_factory_part_for_index(
        &self,
        logical_patch: &LogicalPatch<E>,
        min_cycles: u64,
        max_cycles: u64,
        index: usize,
    ) -> Result<FactoryPartsResult<B::Factory>, Error> {
        let num_magic_states = self
            .layout_overhead
            .num_magic_states(&self.error_budget, index);

        if num_magic_states == 0 {
            return Ok(FactoryPartsResult::NoMagicStates);
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
            return Ok(FactoryPartsResult::NoFactories);
        }

        if let Some(FactoryForCycles {
            factory,
            num_cycles: num_cycles_required,
        }) = self.find_factory(index, &factories, logical_patch, min_cycles, max_cycles)
        {
            let num_factories =
                self.num_factories(logical_patch, index, &factory, num_cycles_required);
            Ok(FactoryPartsResult::Success {
                factory_part: FactoryPart::new(
                    factory.into_owned(),
                    num_factories,
                    num_magic_states,
                    required_logical_magic_state_error_rate,
                ),
                num_cycles: num_cycles_required,
            })
        } else {
            Ok(FactoryPartsResult::NoSuitableFactory)
        }
    }

    fn find_factory<'b>(
        &self,
        magic_state_index: usize,
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        min_cycles: u64,
        max_cycles: u64,
    ) -> Option<FactoryForCycles<'b, B::Factory>> {
        // First, try to find a factory that can be applied within min_cycles;
        // return it, if successful
        let algorithm_duration = min_cycles * logical_patch.logical_cycle_time();
        if let Some(factory) = factories
            .iter()
            .filter(|&factory| {
                factory.duration() <= algorithm_duration
                    && self.is_max_factories_constraint_satisfied(
                        logical_patch,
                        factory,
                        min_cycles,
                    )
            })
            .min_by(|&p, &q| p.normalized_volume().total_cmp(&q.normalized_volume()))
            .cloned()
        {
            return Some(FactoryForCycles::new(factory, min_cycles));
        }

        // If no factory was found, try to find a factory up to max_cycles
        if let Some(factory) = self.find_factory_within_max_cycles(
            magic_state_index,
            factories,
            logical_patch,
            max_cycles,
        ) {
            return Some(factory);
        }

        None
    }

    fn find_factory_within_max_cycles<'b>(
        &self,
        magic_state_index: usize,
        factories: &[Cow<'b, B::Factory>],
        logical_patch: &LogicalPatch<E>,
        max_cycles: u64,
    ) -> Option<FactoryForCycles<'b, B::Factory>> {
        self.max_factories.map_or_else(
            // if there is no max_factories constraint, pick whatever is best
            // for given max cycles
            || {
                PhysicalResourceEstimation::<E, B, L>::pick_factories_with_num_cycles(
                    factories,
                    logical_patch,
                    max_cycles,
                )
                .min()
            },
            // if there is a max_factories constraint, compute the maximum
            // duration based on the factory constraint
            |max_factories| {
                factories
                    .iter()
                    .filter_map(|factory| {
                        let magic_states_per_run = max_factories * factory.num_output_states();
                        let required_runs = self
                            .layout_overhead
                            .num_magic_states(&self.error_budget, magic_state_index)
                            .div_ceil(magic_states_per_run);
                        let required_duration = required_runs * factory.duration();
                        let num = required_duration.div_ceil(logical_patch.logical_cycle_time());

                        (num <= max_cycles).then_some(FactoryForCycles::new(factory.clone(), num))
                    })
                    .min()
            },
        )
    }

    // checks whether the provided parameters suffice to satisfy the
    // max_factories constraint.  If the max_factories constraint is not set,
    // this function returns true.
    fn is_max_factories_constraint_satisfied(
        &self,
        logical_patch: &LogicalPatch<E>,
        factory: &B::Factory,
        num_cycles: u64,
    ) -> bool {
        self.max_factories.map_or(true, |max_factories| {
            max_factories >= self.num_factories(logical_patch, 0, factory, num_cycles)
        })
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

enum FactoryPartsResult<F> {
    NoMagicStates,
    NoFactories,
    NoSuitableFactory,
    Success {
        factory_part: FactoryPart<F>,
        num_cycles: u64,
    },
}
