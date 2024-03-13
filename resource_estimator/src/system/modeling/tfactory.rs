// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use core::fmt;
use std::{collections::BTreeMap, vec};

use serde::{ser::SerializeMap, Serialize};

use crate::estimates::{
    DistillationRound, DistillationUnit, Factory, LogicalPatch, RoundBasedFactory,
};

use super::{
    super::{
        compiled_expression::CompiledExpression,
        error::IO::{self, CannotParseJSON},
    },
    PhysicalQubit, Protocol,
};

pub enum TFactoryQubit<'a> {
    Logical(&'a LogicalPatch<Protocol>),
    Physical(&'a PhysicalQubit),
}

impl<'a> TFactoryQubit<'a> {
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
            Self::Logical(qubit) => *qubit.code_parameter(),
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
            failure_probability_function: Box::new(Self::trivial_failure_probability),
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
        _readout_error_rate: f64,
    ) -> f64 {
        15.0 * input_error_rate + 356.0 * clifford_error_rate
    }

    fn output_error_rate(
        input_error_rate: f64,
        clifford_error_rate: f64,
        _readout_error_rate: f64,
    ) -> f64 {
        35.0 * input_error_rate.powi(3) + 7.1 * clifford_error_rate
    }

    fn trivial_failure_probability(
        _input_error_rate: f64,
        _clifford_error_rate: f64,
        _readout_error_rate: f64,
    ) -> f64 {
        0.0
    }

    fn trivial_error_rate(
        input_error_rate: f64,
        _clifford_error_rate: f64,
        _readout_error_rate: f64,
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
    pub fn by_template(
        template: &'a TFactoryDistillationUnitTemplate,
        qubit: &TFactoryQubit,
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

    pub fn clifford_error_rate(&self) -> f64 {
        self.clifford_error_rate
    }

    pub fn qubit_t_error_rate(&self) -> f64 {
        self.qubit_t_error_rate
    }

    pub fn is_valid(&self) -> bool {
        self.clifford_error_rate() <= 0.1 * self.qubit_t_error_rate
    }
}

impl DistillationUnit<u64> for TFactoryDistillationUnit<'_> {
    fn num_output_states(&self) -> u64 {
        self.num_output_ts
    }

    fn num_input_states(&self) -> u64 {
        self.num_input_ts
    }

    fn duration(&self, position: usize) -> u64 {
        if position == 0 {
            self.duration_at_first_round
        } else {
            self.duration_at_subsequent_rounds
        }
    }

    fn physical_qubits(&self, position: usize) -> u64 {
        if position == 0 {
            self.physical_qubits_at_first_round
        } else {
            self.physical_qubits_at_subsequent_rounds
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn code_parameter(&self) -> Option<&u64> {
        Some(&self.code_distance)
    }

    fn output_error_rate(&self, input_error_rate: f64) -> f64 {
        (self.output_error_rate_formula)(
            input_error_rate,
            self.clifford_error_rate,
            self.readout_error_rate,
        )
    }

    fn failure_probability(&self, input_error_rate: f64) -> f64 {
        (self.failure_probability_formula)(
            input_error_rate,
            self.clifford_error_rate,
            self.readout_error_rate,
        )
    }
}

pub type TFactory = RoundBasedFactory<u64>;

pub fn default_t_factory(logical_qubit: &LogicalPatch<Protocol>) -> TFactory {
    let tfactory_qubit = TFactoryQubit::Logical(logical_qubit);
    let template = TFactoryDistillationUnitTemplate::create_trivial_distillation_unit_1_to_1();
    let unit = TFactoryDistillationUnit::by_template(&template, &tfactory_qubit);

    let length = 1;
    let t_error_rate = logical_qubit.logical_error_rate();
    let failure_probability_requirement = 0.0;

    let round = DistillationRound::new(&unit, failure_probability_requirement, 0);

    let rounds = vec![round];
    let input_t_error_rate_before_each_round = vec![t_error_rate; length + 1];
    let failure_probability_after_each_round: Vec<f64> =
        vec![failure_probability_requirement; length + 1];

    TFactory::new(
        length,
        failure_probability_requirement,
        rounds,
        input_t_error_rate_before_each_round,
        failure_probability_after_each_round,
    )
}

impl Serialize for TFactory {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(8))?;

        map.serialize_entry("physicalQubits", &self.physical_qubits())?;
        map.serialize_entry("runtime", &self.duration())?;
        map.serialize_entry("numTstates", &self.num_output_states())?;
        map.serialize_entry("numInputTstates", &self.num_input_states())?;
        map.serialize_entry("numRounds", &self.num_rounds())?;
        map.serialize_entry("numUnitsPerRound", &self.num_units_per_round())?;
        map.serialize_entry("unitNamePerRound", &self.unit_names())?;
        map.serialize_entry("codeDistancePerRound", &self.code_parameter_per_round())?;
        map.serialize_entry("physicalQubitsPerRound", &self.physical_qubits_per_round())?;
        map.serialize_entry("runtimePerRound", &self.duration_per_round())?;
        map.serialize_entry("logicalErrorRate", &self.output_error_rate())?;

        map.end()
    }
}
