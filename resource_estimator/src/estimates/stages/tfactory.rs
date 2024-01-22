// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use core::fmt;
use std::{collections::BTreeMap, vec};

use probability::{distribution::Inverse, prelude::Binomial};
use serde::{ser::SerializeMap, Serialize};

use crate::estimates::modeling::TPhysicalQubit;

use super::super::{
    compiled_expression::CompiledExpression,
    error::IO::{self, CannotParseJSON},
    modeling::LogicalQubit,
};

pub enum TFactoryQubit<'a, P: TPhysicalQubit> {
    Logical(&'a LogicalQubit<P>),
    Physical(&'a P),
}

impl<'a, P: TPhysicalQubit> TFactoryQubit<'a, P> {
    pub fn physical_qubits(&self) -> u64 {
        match self {
            Self::Logical(qubit) => qubit.physical_qubits(),
            Self::Physical(_) => 1,
        }
    }

    pub fn cycle_time(&self) -> u64 {
        match self {
            Self::Logical(qubit) => qubit.logical_cycle_time(),
            Self::Physical(qubit) => qubit.one_qubit_measurement_time(),
        }
    }

    pub fn clifford_error_rate(&self) -> f64 {
        match self {
            Self::Logical(qubit) => qubit.logical_error_rate(),
            Self::Physical(qubit) => qubit.clifford_error_rate(),
        }
    }

    pub fn readout_error_rate(&self) -> f64 {
        match self {
            // We did not push for a readout error rate for logical qubits,
            // since destructive measurement on surface or Floquet codes has orders of magnitude better fidelity
            // than the logical Clifford operations.
            // Hence for modeling purposes, the logical readout error rate is always 0 and
            // hence we do not even treat it as a parameter.
            // This is not a great model, and the logical readout error rate is a function of the distance.
            // But we would only see an effect on the final results in the cases of distance three (maybe distance five).
            // This may be worthwhile for t-factory DUs where the lowest level can be at distance three or five.
            // But we'd need to do some examples to see if it's worth the effort in changing
            // how logical modeling works for QEC.
            Self::Logical(_) => 0.0,
            Self::Physical(qubit) => qubit.readout_error_rate(),
        }
    }

    pub fn t_error_rate(&self) -> f64 {
        match self {
            Self::Logical(qubit) => qubit.physical_qubit().t_gate_error_rate(),
            Self::Physical(qubit) => qubit.t_gate_error_rate(),
        }
    }

    pub fn code_distance(&self) -> u64 {
        match self {
            Self::Logical(qubit) => qubit.code_distance(),
            Self::Physical(_) => 1,
        }
    }
}

impl ToString for TFactoryDistillationUnitType {
    fn to_string(&self) -> String {
        match self {
            TFactoryDistillationUnitType::Logical => String::from("Logical"),
            TFactoryDistillationUnitType::Physical => String::from("Physical"),
            TFactoryDistillationUnitType::Combined => String::from("Combined"),
        }
    }
}

/// A formula to represent the evaluation of failure probabilities and error
/// rates.  The first argument is the `inputErrorRate`, the second argument the
/// `cliffordErrorRate`, and the last argument the `readoutErrorRate`
pub type TFactoryFormula = Box<dyn Fn(f64, f64, f64) -> f64>;

impl From<CompiledExpression> for TFactoryFormula {
    fn from(compiled_expression: CompiledExpression) -> Self {
        Box::new(
            move |input_error_rate: f64, clifford_error_rate: f64, readout_error_rate| -> f64 {
                let mut context = BTreeMap::new();
                context.insert("inputErrorRate".to_string(), input_error_rate);
                context.insert("cliffordErrorRate".to_string(), clifford_error_rate);
                context.insert("readoutErrorRate".to_string(), readout_error_rate);
                context.insert("z".to_string(), input_error_rate);
                context.insert("c".to_string(), clifford_error_rate);
                context.insert("r".to_string(), readout_error_rate);

                compiled_expression
                    .evaluate(&mut context)
                    .expect("expression evaluation should succeed")
            },
        )
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TFactoryDistillationUnitType {
    Logical,
    Physical,
    Combined,
}

pub struct TFactoryDistillationUnitResources {
    /// The number of unit qubits utilized in distillation.
    pub(crate) num_unit_qubits: u64,
    /// The duration of distillation measured in qubit cycles.
    pub(crate) duration_in_qubit_cycle_time: u64,
}

pub struct TFactoryDistillationUnitTemplate {
    /// The distillation unit output name.
    pub(crate) name: String,
    /// The number of input t states accepted by the distillation unit.
    pub(crate) num_input_ts: u64,
    /// The number of output t states generated by the distillation unit.
    pub(crate) num_output_ts: u64,
    /// The failure probability formula expression.
    pub(crate) failure_probability_function: TFactoryFormula,
    /// The output error rate formula expression.
    pub(crate) output_error_rate_function: TFactoryFormula,
    /// The type of distillation unit defines the scope of qubit types: physical, logical, or both.
    pub(crate) unit_type: TFactoryDistillationUnitType,
    /// Specification for the physical qubit protocol.
    pub(crate) physical_qubit_specification: Option<TFactoryDistillationUnitResources>,
    /// Specification for the logical qubit protocol.
    pub(crate) logical_qubit_specification: Option<TFactoryDistillationUnitResources>,
    /// Specification for the logical qubit protocol if necessary to override for the first round of distillation.
    pub(crate) logical_qubit_specification_first_round_override:
        Option<TFactoryDistillationUnitResources>,
}

impl TFactoryDistillationUnitTemplate {
    pub fn from_name(name: &str) -> core::result::Result<Self, IO> {
        match name {
            "15-1 RM" | "15-1 RM prep" | "15-to-1 RM" | "15-to-1 RM prep" => {
                Ok(Self::create_distillation_unit_15_to_1_rm_prep_template())
            }
            "15-1 space-efficient"
            | "15-1 space efficient"
            | "15-to-1 space-efficient"
            | "15-to-1 space efficient" => {
                Ok(Self::create_distillation_unit_15_to_1_rm_space_efficient_template())
            }
            _ => Err(CannotParseJSON(serde::de::Error::custom(format!(
                "Invalid distillation unit specification name: {name}."
            )))),
        }
    }

    pub fn create_distillation_unit_15_to_1_rm_prep_template() -> Self {
        Self {
            name: String::from("15-to-1 RM prep"),
            num_input_ts: 15,
            num_output_ts: 1,
            failure_probability_function: Box::new(Self::failure_probability),
            output_error_rate_function: Box::new(Self::output_error_rate),
            unit_type: TFactoryDistillationUnitType::Combined,
            physical_qubit_specification: Some(TFactoryDistillationUnitResources {
                num_unit_qubits: 31,
                duration_in_qubit_cycle_time: 24,
            }),
            logical_qubit_specification: Some(TFactoryDistillationUnitResources {
                num_unit_qubits: 31,
                duration_in_qubit_cycle_time: 11,
            }),

            logical_qubit_specification_first_round_override: None,
        }
    }

    pub fn create_distillation_unit_15_to_1_rm_space_efficient_template() -> Self {
        Self {
            name: String::from("15-to-1 space efficient"),
            num_input_ts: 15,
            num_output_ts: 1,
            failure_probability_function: Box::new(Self::failure_probability),
            output_error_rate_function: Box::new(Self::output_error_rate),
            unit_type: TFactoryDistillationUnitType::Combined,
            physical_qubit_specification: Some(TFactoryDistillationUnitResources {
                num_unit_qubits: 12,
                duration_in_qubit_cycle_time: 45,
            }),
            logical_qubit_specification: Some(TFactoryDistillationUnitResources {
                num_unit_qubits: 20,
                duration_in_qubit_cycle_time: 13,
            }),
            logical_qubit_specification_first_round_override: None,
        }
    }

    pub fn create_trivial_distillation_unit_1_to_1() -> Self {
        Self {
            name: String::from("trivial 1-to-1"),
            num_input_ts: 1,
            num_output_ts: 1,
            failure_probability_function: Box::new(Self::trivial_failutre_probability),
            output_error_rate_function: Box::new(Self::trivial_error_rate),
            unit_type: TFactoryDistillationUnitType::Logical,
            physical_qubit_specification: None,
            logical_qubit_specification: Some(TFactoryDistillationUnitResources {
                num_unit_qubits: 1,
                duration_in_qubit_cycle_time: 1,
            }),
            logical_qubit_specification_first_round_override: None,
        }
    }

    fn failure_probability(
        input_error_rate: f64,
        clifford_error_rate: f64,
        #[allow(unused_variables)] readout_error_rate: f64,
    ) -> f64 {
        15.0 * input_error_rate + 356.0 * clifford_error_rate
    }

    fn output_error_rate(
        input_error_rate: f64,
        clifford_error_rate: f64,
        #[allow(unused_variables)] readout_error_rate: f64,
    ) -> f64 {
        35.0 * input_error_rate.powi(3) + 7.1 * clifford_error_rate
    }

    fn trivial_failutre_probability(
        #[allow(unused_variables)] input_error_rate: f64,
        #[allow(unused_variables)] clifford_error_rate: f64,
        #[allow(unused_variables)] readout_error_rate: f64,
    ) -> f64 {
        0.0
    }

    fn trivial_error_rate(
        input_error_rate: f64,
        #[allow(unused_variables)] clifford_error_rate: f64,
        #[allow(unused_variables)] readout_error_rate: f64,
    ) -> f64 {
        input_error_rate
    }

    pub fn default_distillation_unit_templates() -> Vec<Self> {
        vec![
            Self::create_distillation_unit_15_to_1_rm_prep_template(),
            Self::create_distillation_unit_15_to_1_rm_space_efficient_template(),
        ]
    }
}

pub struct TFactoryDistillationUnit<'a> {
    unit_type: TFactoryDistillationUnitType,
    num_input_ts: u64,
    num_output_ts: u64,
    physical_qubits_at_first_round: u64,
    physical_qubits_at_subsequent_rounds: u64,
    duration_at_first_round: u64,
    duration_at_subsequent_rounds: u64,
    code_distance: u64,
    failure_probability_formula: &'a dyn Fn(f64, f64, f64) -> f64,
    output_error_rate_formula: &'a dyn Fn(f64, f64, f64) -> f64,
    pub(crate) name: String,
    clifford_error_rate: f64,
    readout_error_rate: f64,
    /// This is the qubit's T error rate that we need only to decide the input T
    /// error rate for the first unit
    qubit_t_error_rate: f64,
}

impl<'a> fmt::Debug for TFactoryDistillationUnit<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TFactoryDistillationUnit")
            .field("unit_type", &self.unit_type.to_string())
            .field("num_input_ts", &self.num_input_ts)
            .field("num_output_ts", &self.num_output_ts)
            .field(
                "physical_qubits_at_first_round",
                &self.physical_qubits_at_first_round,
            )
            .field(
                "physical_qubits_at_subsequent_rounds",
                &self.physical_qubits_at_subsequent_rounds,
            )
            .field("duration_at_first_round", &self.duration_at_first_round)
            .field(
                "duration_at_subsequent_rounds",
                &self.duration_at_subsequent_rounds,
            )
            .field("code_distance", &self.code_distance)
            .field("name", &self.name)
            .field("clifford_error_rate", &self.clifford_error_rate)
            .field("qubit_t_error_rate", &self.qubit_t_error_rate)
            .finish()
    }
}

impl<'a> TFactoryDistillationUnit<'a> {
    pub fn by_template<P: TPhysicalQubit>(
        template: &'a TFactoryDistillationUnitTemplate,
        qubit: &TFactoryQubit<P>,
    ) -> Self {
        let code_distance = qubit.code_distance();

        let specification_for_first_round = match (code_distance, template.unit_type) {
            (
                1,
                TFactoryDistillationUnitType::Combined | TFactoryDistillationUnitType::Physical,
            ) => &template.physical_qubit_specification,
            (1, TFactoryDistillationUnitType::Logical)
            | (_, TFactoryDistillationUnitType::Physical) => &None,
            (_, TFactoryDistillationUnitType::Combined | TFactoryDistillationUnitType::Logical) => {
                if template
                    .logical_qubit_specification_first_round_override
                    .is_some()
                {
                    &template.logical_qubit_specification_first_round_override
                } else {
                    &template.logical_qubit_specification
                }
            }
        };

        let (physical_qubits_at_first_round, duration_at_first_round) =
            specification_for_first_round
                .as_ref()
                .map(|spec| {
                    (
                        spec.num_unit_qubits * qubit.physical_qubits(),
                        spec.duration_in_qubit_cycle_time * qubit.cycle_time(),
                    )
                })
                .unwrap_or_default();

        let specification_for_subsequent_rounds = match template.unit_type {
            TFactoryDistillationUnitType::Physical => &None,
            TFactoryDistillationUnitType::Combined | TFactoryDistillationUnitType::Logical => {
                &template.logical_qubit_specification
            }
        };

        let (physical_qubits_at_subsequent_rounds, duration_at_subsequent_rounds) =
            specification_for_subsequent_rounds
                .as_ref()
                .map(|spec| {
                    (
                        spec.num_unit_qubits * qubit.physical_qubits(),
                        spec.duration_in_qubit_cycle_time * qubit.cycle_time(),
                    )
                })
                .unwrap_or_default();

        let num_input_ts = template.num_input_ts;
        let num_output_ts = template.num_output_ts;

        let failure_probability_formula = &template.failure_probability_function;
        let output_error_rate_formula = &template.output_error_rate_function;

        let name = template.name.clone();

        let clifford_error_rate = qubit.clifford_error_rate();
        let qubit_t_error_rate = qubit.t_error_rate();
        let readout_error_rate = qubit.readout_error_rate();

        Self {
            unit_type: template.unit_type,
            num_input_ts,
            num_output_ts,
            physical_qubits_at_first_round,
            physical_qubits_at_subsequent_rounds,
            duration_at_first_round,
            duration_at_subsequent_rounds,
            code_distance,
            name,
            clifford_error_rate,
            qubit_t_error_rate,
            failure_probability_formula,
            output_error_rate_formula,
            readout_error_rate,
        }
    }

    pub fn physical_qubits(&self, position: usize) -> u64 {
        if position == 0 {
            self.physical_qubits_at_first_round
        } else {
            self.physical_qubits_at_subsequent_rounds
        }
    }

    pub fn duration(&self, position: usize) -> u64 {
        if position == 0 {
            self.duration_at_first_round
        } else {
            self.duration_at_subsequent_rounds
        }
    }

    pub fn code_distance(&self) -> u64 {
        self.code_distance
    }

    pub fn num_input_ts(&self) -> u64 {
        self.num_input_ts
    }

    pub fn num_output_ts(&self) -> u64 {
        self.num_output_ts
    }

    pub fn clifford_error_rate(&self) -> f64 {
        self.clifford_error_rate
    }

    pub fn failure_probability(&self, input_error_rate: f64) -> f64 {
        (self.failure_probability_formula)(
            input_error_rate,
            self.clifford_error_rate,
            self.readout_error_rate,
        )
    }

    pub fn output_error_rate(&self, input_error_rate: f64) -> f64 {
        (self.output_error_rate_formula)(
            input_error_rate,
            self.clifford_error_rate,
            self.readout_error_rate,
        )
    }

    pub fn qubit_t_error_rate(&self) -> f64 {
        self.qubit_t_error_rate
    }

    pub fn is_valid(&self) -> bool {
        self.clifford_error_rate() <= 0.1 * self.qubit_t_error_rate
    }
}

/// One round of distillation in a T-factory
///
/// All units per round are the same.  The number is initialized to 1 and can be
/// iteratively adjusted to match some external constraints.
#[derive(Debug, Clone)]
struct TFactoryDistillationRound {
    num_units: u64,
    failure_probability_requirement: f64,
    num_output_ts: u64,
    num_input_ts: u64,
    duration: u64,
    physical_qubits: u64,
    name: String,
    code_distance: u64,
}

impl TFactoryDistillationRound {
    pub fn new(
        unit: &TFactoryDistillationUnit,
        failure_probability_requirement: f64,
        position: usize,
    ) -> Self {
        Self {
            num_units: 1,
            failure_probability_requirement,
            num_output_ts: unit.num_output_ts(),
            num_input_ts: unit.num_input_ts(),
            duration: unit.duration(position),
            physical_qubits: unit.physical_qubits(position),
            name: unit.name.clone(),
            code_distance: unit.code_distance(),
        }
    }

    fn adjust_num_units_to(
        &mut self,
        t_needed_next: u64,
        failure_probability: f64,
    ) -> TFactoryBuildStatus {
        // initial value
        self.num_units = ((t_needed_next as f64) / (self.max_num_output_ts() as f64)).ceil() as u64;

        loop {
            let num_output_ts = self.compute_num_output_ts(failure_probability);
            if num_output_ts < t_needed_next {
                self.num_units *= 2;

                // TFactory distillation round requires unreasonably high number of units?
                if self.num_units >= 1_000_000_000_000_000 {
                    return TFactoryBuildStatus::FailedDueToUnreasonableHighNumberOfUnitsRequired;
                }
            } else {
                break;
            }
        }

        let mut upper = self.num_units;
        let mut lower = self.num_units / 2;
        while lower < upper {
            self.num_units = (lower + upper) / 2;
            let num_output_ts = self.compute_num_output_ts(failure_probability);
            if num_output_ts >= t_needed_next {
                upper = self.num_units;
            } else {
                lower = self.num_units + 1;
            }
        }
        self.num_units = upper;

        TFactoryBuildStatus::Success
    }

    pub fn physical_qubits(&self) -> u64 {
        self.num_units * self.physical_qubits
    }

    pub fn duration(&self) -> u64 {
        self.duration
    }

    #[allow(clippy::cast_possible_truncation)]
    fn compute_num_output_ts(&self, failure_probability: f64) -> u64 {
        // special case when not necessary to run actual distillation:
        // the physcial qubit error rate is already below the threshold
        if failure_probability == 0.0 && self.failure_probability_requirement == 0.0 {
            return self.num_units * self.num_output_ts;
        }
        let dist = Binomial::with_failure(self.num_units as usize, failure_probability);
        dist.inverse(self.failure_probability_requirement) as u64 * self.num_output_ts
    }

    fn max_num_output_ts(&self) -> u64 {
        self.num_units * self.num_output_ts
    }

    fn num_units(&self) -> u64 {
        self.num_units
    }
}

#[derive(Debug, Clone)]
pub struct TFactory {
    length: usize,
    failure_probability_requirement: f64,
    rounds: Vec<TFactoryDistillationRound>,
    input_t_error_rate_before_each_round: Vec<f64>,
    failure_probability_after_each_round: Vec<f64>,
}

impl TFactory {
    fn new(length: usize, initial_t_error_rate: f64, failure_probability_requirement: f64) -> Self {
        let rounds = Vec::with_capacity(length);
        let mut input_t_error_rate_before_each_round = Vec::with_capacity(length + 1);
        input_t_error_rate_before_each_round.push(initial_t_error_rate);
        let failure_probability_after_each_round: Vec<f64> = vec![1.0; length + 1];

        Self {
            length,
            failure_probability_requirement,
            rounds,
            input_t_error_rate_before_each_round,
            failure_probability_after_each_round,
        }
    }

    pub fn build(
        units: &[&TFactoryDistillationUnit],
        failure_probability_requirement: f64,
    ) -> (TFactoryBuildStatus, TFactory) {
        let initial_input_error_rate = units[0].qubit_t_error_rate();
        let mut pipeline = TFactory::new(
            units.len(),
            initial_input_error_rate,
            failure_probability_requirement,
        );

        (pipeline.compute_units_per_round(units, 1), pipeline)
    }

    fn add_rounds(&mut self, units: &[&TFactoryDistillationUnit]) -> TFactoryBuildStatus {
        for unit in units {
            let failure_probability_requirement =
                self.failure_probability_requirement / (self.length as f64);
            let &input_t_error_rate = self
                .input_t_error_rate_before_each_round
                .last()
                .unwrap_or_else(|| unreachable!());
            let output_t_error_rate = unit.output_error_rate(input_t_error_rate);
            if output_t_error_rate > input_t_error_rate {
                return TFactoryBuildStatus::FailedDueToOutputErrorRateHigherThanInputErrorRate;
            }
            let round = TFactoryDistillationRound::new(
                unit,
                failure_probability_requirement,
                self.rounds.len(),
            );
            self.rounds.push(round);
            self.input_t_error_rate_before_each_round
                .push(output_t_error_rate);
        }

        TFactoryBuildStatus::Success
    }

    pub fn default<P: TPhysicalQubit>(logical_qubit: &LogicalQubit<P>) -> Self {
        let tfactory_qubit = TFactoryQubit::Logical(logical_qubit);
        let template = TFactoryDistillationUnitTemplate::create_trivial_distillation_unit_1_to_1();
        let unit = TFactoryDistillationUnit::by_template(&template, &tfactory_qubit);

        let length = 1;
        let t_error_rate = logical_qubit.logical_error_rate();
        let failure_probability_requirement = 0.0;

        let round = TFactoryDistillationRound::new(&unit, failure_probability_requirement, 0);

        let rounds = vec![round];
        let input_t_error_rate_before_each_round = vec![t_error_rate; length + 1];
        let failure_probability_after_each_round: Vec<f64> =
            vec![failure_probability_requirement; length + 1];

        Self {
            length,
            failure_probability_requirement,
            rounds,
            input_t_error_rate_before_each_round,
            failure_probability_after_each_round,
        }
    }

    /// Number of distillation rounds
    pub fn num_rounds(&self) -> u64 {
        self.length as u64
    }

    /// Number of units per distillation round
    pub fn num_units_per_round(&self) -> Vec<u64> {
        self.rounds.iter().map(|round| round.num_units).collect()
    }

    /// Code distances per round
    pub fn code_distance_per_round(&self) -> Vec<u64> {
        self.rounds
            .iter()
            .map(|round| round.code_distance)
            .collect()
    }

    /// Physical qubits per round
    pub fn physical_qubits_per_round(&self) -> Vec<u64> {
        self.rounds
            .iter()
            .map(TFactoryDistillationRound::physical_qubits)
            .collect()
    }

    /// Runtime in ns per round
    pub fn duration_per_round(&self) -> Vec<u64> {
        self.rounds
            .iter()
            .map(TFactoryDistillationRound::duration)
            .collect()
    }

    /// Names of distillation units per round
    pub fn unit_names(&self) -> Vec<String> {
        self.rounds.iter().map(|round| round.name.clone()).collect()
    }

    /// This computes the necessary number of units per round in order to
    /// achieve the required success probability
    /// Returning None means that the sequence of units does not provide a TFactory with the required output error rate.
    #[allow(clippy::doc_markdown)]
    pub fn compute_units_per_round(
        &mut self,
        units: &[&TFactoryDistillationUnit],
        multiplier: u64,
    ) -> TFactoryBuildStatus {
        let status = self.add_rounds(units);
        if !matches!(status, TFactoryBuildStatus::Success) {
            return status;
        }

        if self.length > 0 {
            let mut t_needed_next = self.rounds[self.length - 1].num_output_ts * multiplier;

            for idx in (0..self.length).rev() {
                let q =
                    units[idx].failure_probability(self.input_t_error_rate_before_each_round[idx]);
                if q <= 0.0 {
                    return TFactoryBuildStatus::FailedDueToLowFailureProbability;
                }

                if q >= 1.0 {
                    return TFactoryBuildStatus::FailedDueToHighFailureProbability;
                }

                self.failure_probability_after_each_round[idx] = q;
                let status = self.rounds[idx].adjust_num_units_to(t_needed_next, q);
                if !matches!(status, TFactoryBuildStatus::Success) {
                    return status;
                }

                t_needed_next = self.rounds[idx].num_input_ts * self.rounds[idx].num_units();
            }
        }

        TFactoryBuildStatus::Success
    }

    pub fn physical_qubits(&self) -> u64 {
        self.rounds
            .iter()
            .map(TFactoryDistillationRound::physical_qubits)
            .max()
            .unwrap_or(0)
    }

    pub fn duration(&self) -> u64 {
        self.rounds
            .iter()
            .map(TFactoryDistillationRound::duration)
            .sum()
    }

    #[allow(dead_code)]
    pub fn input_t_error_rate(&self) -> f64 {
        // Even when there are no units `input_t_error_rate_before_each_round`
        // has one element
        self.input_t_error_rate_before_each_round[0]
    }

    pub fn output_t_error_rate(&self) -> f64 {
        self.input_t_error_rate_before_each_round[self.length]
    }

    pub fn input_t_count(&self) -> u64 {
        self.rounds
            .first()
            .map_or(0, |round| round.num_input_ts * round.num_units())
    }

    pub fn output_t_count(&self) -> u64 {
        let last_round = self
            .rounds
            .last()
            .expect("at least one round should be present");
        let failure_probability = self.failure_probability_after_each_round[self.length - 1];
        // This should not fail, as we already evalauted this
        // failure_probability when building the TFactory
        last_round.compute_num_output_ts(failure_probability)
    }

    pub fn normalized_volume(&self) -> f64 {
        ((self.physical_qubits() * self.duration()) as f64) / (self.output_t_count() as f64)
    }

    pub fn normalized_qubits(&self) -> f64 {
        (self.physical_qubits() as f64) / (self.output_t_count() as f64)
    }
}

impl Serialize for TFactory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(8))?;

        map.serialize_entry("physicalQubits", &self.physical_qubits())?;
        map.serialize_entry("runtime", &self.duration())?;
        map.serialize_entry("numTstates", &self.output_t_count())?;
        map.serialize_entry("numInputTstates", &self.input_t_count())?;
        map.serialize_entry("numRounds", &self.num_rounds())?;
        map.serialize_entry("numUnitsPerRound", &self.num_units_per_round())?;
        map.serialize_entry("unitNamePerRound", &self.unit_names())?;
        map.serialize_entry("codeDistancePerRound", &self.code_distance_per_round())?;
        map.serialize_entry("physicalQubitsPerRound", &self.physical_qubits_per_round())?;
        map.serialize_entry("runtimePerRound", &self.duration_per_round())?;
        map.serialize_entry("logicalErrorRate", &self.output_t_error_rate())?;

        map.end()
    }
}

pub enum TFactoryBuildStatus {
    Success,
    FailedDueToLowFailureProbability,
    FailedDueToHighFailureProbability,
    FailedDueToOutputErrorRateHigherThanInputErrorRate,
    FailedDueToUnreasonableHighNumberOfUnitsRequired,
}
