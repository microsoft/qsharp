// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use super::super::{
    constants::{MAX_DISTILLATION_ROUNDS, MAX_EXTRA_DISTILLATION_ROUNDS},
    modeling::{LogicalQubit, PhysicalQubit, Protocol},
    stages::tfactory::{
        TFactory, TFactoryBuildStatus, TFactoryDistillationUnit, TFactoryDistillationUnitTemplate,
    },
};
use super::population;

use super::code_distance_iterators::{iterate_for_code_distances, search_for_code_distances};
use super::distillation_units_map::DistillationUnitsMap;
use super::population::{Point, Point2D, Point4D, Population};

#[derive(Default)]
struct TFactoryExhaustiveSearch<P>
where
    P: Point,
    P: Ord,
{
    /// Target output T-error rate
    output_t_error_rate: f64,
    /// Number of combinations for which a TFactory was tried to build
    num_combinations: usize,
    /// Number of valid TFactory instances that were successfully build
    num_valid: usize,
    /// Number of sufficient TFactory instances that do not succeed the user
    /// specified output T-error rate
    num_candidates: usize,
    /// Pareto frontier of currently best TFactories.
    /// We optimize them by two duration and normalized qubits.
    frontier_factories: Population<P>,
}

trait TFactoryExhaustiveSearchOptions {
    const ALLOW_GO_RIGHT_IF_DOMINATED: bool;
    const ALWAYS_GO_RIGHT_IN_FULL_ITERATION: bool;
    const ITERATE_MAX_NUM_ROUNDS: bool;
}

impl TFactoryExhaustiveSearchOptions for Point2D<TFactory> {
    const ALLOW_GO_RIGHT_IF_DOMINATED: bool = false;
    const ALWAYS_GO_RIGHT_IN_FULL_ITERATION: bool = false;
    const ITERATE_MAX_NUM_ROUNDS: bool = false;
}

// For search in 4D space, we need to itrerate over all combinations of code distances.
// Increasing code distances would increase costs (qubits and time) but decrease the error rate.
impl TFactoryExhaustiveSearchOptions for Point4D<TFactory> {
    const ALLOW_GO_RIGHT_IF_DOMINATED: bool = true;
    const ALWAYS_GO_RIGHT_IN_FULL_ITERATION: bool = true;
    const ITERATE_MAX_NUM_ROUNDS: bool = true;
}

impl<P> TFactoryExhaustiveSearch<P>
where
    P: Point,
    P: Ord,
    P: From<TFactory>,
    P: TFactoryExhaustiveSearchOptions,
{
    fn new(output_t_error_rate: f64) -> Self {
        Self {
            output_t_error_rate,
            frontier_factories: population::Population::<P>::new(),
            num_combinations: 0,
            num_valid: 0,
            num_candidates: 0,
        }
    }

    fn check_units_if_can_improve_with_increasing_code_distances(
        &mut self,
        units: &[&TFactoryDistillationUnit],
    ) -> bool {
        self.num_combinations += 1;

        self.check_units_if_can_improve_with_increasing_code_distances_internal(units)
    }

    fn check_units_if_can_improve_with_increasing_code_distances_internal(
        &mut self,
        units: &[&TFactoryDistillationUnit],
    ) -> bool {
        // This is the success probability of producing the expected number of T
        // states in sufficient quality (see Appendix C in paper)
        let (status, factory) = TFactory::build(units, 0.01);
        #[allow(clippy::match_same_arms)]
        match status {
            TFactoryBuildStatus::Success => {
                self.num_valid += 1;
                // This is the success probability of producing the expected number of T
                // states in sufficient quality (see Appendix C in paper)
                let is_below_or_equal_output_t_error_rate =
                    factory.output_t_error_rate() <= self.output_t_error_rate;
                if is_below_or_equal_output_t_error_rate {
                    self.num_candidates += 1;
                }

                // No need to check for frontier if above the required rate and allow to go right even if dominated.
                // Performance optimization.
                if !is_below_or_equal_output_t_error_rate && P::ALLOW_GO_RIGHT_IF_DOMINATED {
                    return true;
                }

                let is_not_dominated =
                    self.compare_with_frontier(factory, is_below_or_equal_output_t_error_rate);

                // The only option we should proceed with increasing the code distance are:
                // We still are above the required output T-error rate and the factory is not dominated (the cost is still low).
                (P::ALLOW_GO_RIGHT_IF_DOMINATED || is_not_dominated)
                    && !is_below_or_equal_output_t_error_rate
            }
            // The Clifford error rate is defined as a number < 1 to the power of (code_distance as i32 + 1) / 2.
            //Increasing the code distance decreases the Clifford error rate.
            TFactoryBuildStatus::FailedDueToLowFailureProbability => {
                // The failure probability is an increasing function of the Clifford error rate like:
                //  15.0 * input_error_rate + 356.0 * clifford_error_rate.
                // If increase the code distance, the Clifford error rate decreases, the failure probability decreases.
                // Should stop increasing the code distance.
                false
            }
            TFactoryBuildStatus::FailedDueToHighFailureProbability => {
                // This case happens when the failure probability is greater than 1.0.
                // The failure probability is a decreasing function of the Clifford error rate like:
                // 15.0 * input_error_rate + 356.0 * clifford_error_rate.
                // If increase the code distance, the Clifford error rate decreases, the failure probability decreases.
                // Should continue increasing the code distance.
                true
            }
            TFactoryBuildStatus::FailedDueToOutputErrorRateHigherThanInputErrorRate => {
                // TFactory distillation round returns a higher error rate than the input.
                // The output error rate is a increasing function of the Clifford error rate like:
                // 35.0 * input_error_rate.powi(3) + 7.1 * clifford_error_rate
                // If increase the code distance, the Clifford error rate decreases, the output error rate decreases.
                // Should continue increasing the code distance.
                true
            }
            TFactoryBuildStatus::FailedDueToUnreasonableHighNumberOfUnitsRequired => {
                // Building the TFactory involved too many qubits on an intermediate distillation round.
                // We assume that increasing the code distance could help because
                // the success probability should grow with increasing the code distance.
                true
            }
        }
    }

    fn compare_with_frontier(
        &mut self,
        factory: TFactory,
        is_below_output_t_error_rate: bool,
    ) -> bool {
        let point = P::from(factory);
        let is_not_dominated = !self.frontier_factories.dominates(&point);
        if is_not_dominated && is_below_output_t_error_rate {
            self.frontier_factories.push(point);
            self.frontier_factories.attempt_filter_out_dominated();
        }

        is_not_dominated
    }
}

impl From<TFactory> for Point2D<TFactory> {
    fn from(factory: TFactory) -> Self {
        let value1 = factory.normalized_qubits();
        let value2 = factory.duration();
        Point2D::new(factory, value1, value2)
    }
}

impl ToString for Point2D<TFactory> {
    fn to_string(&self) -> String {
        format!(
            "Pareto frontier point.   normalized qubits: {},    duration: {}",
            self.value1, self.value2,
        )
    }
}

impl From<TFactory> for Point4D<TFactory> {
    fn from(factory: TFactory) -> Self {
        let value1 = factory.normalized_qubits();
        let value2 = factory.duration();
        let value3 = factory.output_t_error_rate();
        let binding = factory.code_distance_per_round();
        let value4 = binding.last().expect("binding should not be empty");
        Point4D::new(factory, value1, value2, value3, *value4)
    }
}

impl ToString for Point4D<TFactory> {
    fn to_string(&self) -> String {
        format!(
            "Pareto frontier point.   normalized qubits: {},    duration: {},    output T-error rate: {},    code distance: {}",
            self.value1, self.value2, self.value3, self.value4,
        )
    }
}

pub(crate) fn find_nondominated_tfactories(
    ftp: &Protocol,
    qubit: &Rc<PhysicalQubit>,
    distillation_unit_templates: &[TFactoryDistillationUnitTemplate],
    output_t_error_rate: f64,
    max_code_distance: u64,
) -> Vec<TFactory> {
    let points = find_nondominated_population::<Point2D<TFactory>>(
        ftp,
        qubit,
        distillation_unit_templates,
        output_t_error_rate,
        max_code_distance,
    );

    points
        .items()
        .iter()
        .map(|point| point.item.clone())
        .collect()
}

fn find_nondominated_population<P>(
    ftp: &Protocol,
    qubit: &Rc<PhysicalQubit>,
    distillation_unit_templates: &[TFactoryDistillationUnitTemplate],
    output_t_error_rate: f64,
    max_code_distance: u64,
) -> Population<P>
where
    P: Point + Ord + ToString + From<TFactory> + TFactoryExhaustiveSearchOptions,
{
    let min_code_distance = 1;
    let distances: Vec<_> = (min_code_distance..=max_code_distance).step_by(2).collect();

    if output_t_error_rate > qubit.t_gate_error_rate() {
        let mut population = Population::<P>::new();

        if let Ok(logical_qubit) = LogicalQubit::new(ftp, max_code_distance, qubit.clone()) {
            let factory = TFactory::default(&logical_qubit);
            let point = P::from(factory);
            population.push(point);
        }

        return population;
    }

    let mut qubits = vec![None; max_code_distance as usize + 1];
    for &distance in &distances {
        qubits[distance as usize] = LogicalQubit::new(ftp, distance, qubit.clone())
            .ok()
            .map(Rc::new);
    }

    let distillation_units_map =
        DistillationUnitsMap::create(qubit, &qubits, distances, distillation_unit_templates);

    let mut searcher = TFactoryExhaustiveSearch::<P>::new(output_t_error_rate);

    for num_rounds in 1..=MAX_DISTILLATION_ROUNDS {
        process_for_num_rounds(&mut searcher, &distillation_units_map, num_rounds);
    }

    if searcher.frontier_factories.items().is_empty() || P::ITERATE_MAX_NUM_ROUNDS {
        for num_rounds in MAX_DISTILLATION_ROUNDS + 1..=MAX_EXTRA_DISTILLATION_ROUNDS {
            process_for_num_rounds(&mut searcher, &distillation_units_map, num_rounds);
        }
    }

    searcher.frontier_factories.filter_out_dominated();
    searcher.frontier_factories.sort_items();

    searcher.frontier_factories
}

fn process_for_num_rounds<P>(
    searcher: &mut TFactoryExhaustiveSearch<P>,
    distillation_units_map: &DistillationUnitsMap,
    num_rounds: usize,
) where
    P: Point + Ord + From<TFactory> + TFactoryExhaustiveSearchOptions,
{
    distillation_units_map.iterate_for_all_distillation_units(num_rounds, &mut |unit_indexes| {
        process_for_specifications_combination(searcher, distillation_units_map, unit_indexes);
    });
}

fn process_for_specifications_combination<P>(
    searcher: &mut TFactoryExhaustiveSearch<P>,
    distillation_units_map: &DistillationUnitsMap,
    unit_indexes: &[usize],
) where
    P: Point + Ord + From<TFactory> + TFactoryExhaustiveSearchOptions,
{
    let left_code_distance_indexes = distillation_units_map.get_min_distance_indexes(unit_indexes);
    let right_code_distance_indexes: Vec<usize> =
        distillation_units_map.get_max_distance_indexes(unit_indexes);

    let mut checker_for_search = |distance_indexes: &[usize]| -> bool {
        let units = distillation_units_map.get_many(distance_indexes, unit_indexes);
        searcher.check_units_if_can_improve_with_increasing_code_distances(&units)
    };

    if let Some(result) = search_for_code_distances(
        unit_indexes.len(),
        &left_code_distance_indexes,
        &right_code_distance_indexes,
        &mut checker_for_search,
    ) {
        let mut checker_for_full_iteration = |distance_indexes: &[usize]| -> bool {
            let units = distillation_units_map.get_many(distance_indexes, unit_indexes);
            let result = searcher.check_units_if_can_improve_with_increasing_code_distances(&units);
            P::ALWAYS_GO_RIGHT_IN_FULL_ITERATION || result
        };
        iterate_for_code_distances(
            unit_indexes.len(),
            &left_code_distance_indexes,
            &right_code_distance_indexes,
            &result,
            &mut checker_for_full_iteration,
        );
    }
}

#[cfg(test)]
mod tests;
