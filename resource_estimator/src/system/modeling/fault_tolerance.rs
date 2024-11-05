// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::estimates::{
    CodeWithThresholdAndDistance, CodeWithThresholdAndDistanceEvaluator,
    Error::{LogicalCycleTimeComputationFailed, PhysicalQubitComputationFailed},
    ErrorCorrection,
};

use super::{
    super::{
        compiled_expression::CompiledExpression,
        constants::{
            IDLE_ERROR_RATE, MAX_CODE_DISTANCE, ONE_QUBIT_GATE_ERROR_RATE, ONE_QUBIT_GATE_TIME,
            ONE_QUBIT_MEASUREMENT_PROCESS_ERROR_RATE, ONE_QUBIT_MEASUREMENT_TIME,
            TWO_QUBIT_GATE_ERROR_RATE, TWO_QUBIT_GATE_TIME,
            TWO_QUBIT_JOINT_MEASUREMENT_PROCESS_ERROR_RATE, TWO_QUBIT_JOINT_MEASUREMENT_TIME,
        },
        error::{
            InvalidInput::{
                InvalidFaultToleranceProtocol, NonPositiveLogicalCycleTime,
                NonPositivePhysicalQubitsPerLogicalQubit,
            },
            IO::CannotParseJSON,
        },
        Error,
    },
    PhysicalInstructionSet, PhysicalQubit,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "camelCase"))]
pub struct ProtocolSpecification {
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) error_correction_threshold: Option<f64>,
    #[serde(default)]
    pub(crate) crossing_prefactor: Option<f64>,
    #[serde(default)]
    pub(crate) logical_cycle_time: Option<String>,
    #[serde(default)]
    pub(crate) physical_qubits_per_logical_qubit: Option<String>,
    #[serde(default = "default_max_code_distance")]
    pub(crate) max_code_distance: u64,
}

impl Default for ProtocolSpecification {
    fn default() -> Self {
        Self {
            name: "surface_code".into(),
            crossing_prefactor: None,
            error_correction_threshold: None,
            logical_cycle_time: None,
            physical_qubits_per_logical_qubit: None,
            max_code_distance: default_max_code_distance(),
        }
    }
}

/// Fault tolerance protocol model to model a logical qubit.
///
/// The fields `logical_cycle_time`, and `physical_qubits_per_logical_qubit` can
/// be specified in terms of a functions over a physical qubit `qubit` and the
/// QEC code distance `code_distance`, in which the following variables are available:
///
/// | Variable name                      | Refers to                                           |
/// |------------------------------------|-----------------------------------------------------|
/// | `one_qubit_gate_time`              | [`PhysicalQubit::one_qubit_gate_time`]              |
/// | `two_qubit_gate_time`              | [`PhysicalQubit::two_qubit_gate_time`]              |
/// | `one_qubit_measurement_time`       | [`PhysicalQubit::one_qubit_measurement_time`]       |
/// | `two_qubit_joint_measurement_time` | [`PhysicalQubit::two_qubit_joint_measurement_time`] |
/// | `code_distance`                    | Code distance                                       |
///
/// Note that all physical qubit related variables are not available as variable
/// in formulas for `physical_qubits_per_logical_qubit`.
pub type Protocol = CodeWithThresholdAndDistance<ProtocolEvaluator>;

pub struct ProtocolEvaluator {
    logical_cycle_time_expr: String,
    logical_cycle_time: CompiledExpression,
    physical_qubits_per_logical_qubit_expr: String,
    physical_qubits_per_logical_qubit: CompiledExpression,
}

impl ProtocolEvaluator {
    /// Creates an evaluation context for formulas specified in fault-tolerance
    /// protocol files.
    ///
    /// Based on whether the formula can contain qubit or code distance values,
    /// additional values are provided in the context.
    fn create_evaluation_context(
        qubit: Option<&PhysicalQubit>,
        code_distance: u64,
    ) -> BTreeMap<String, f64> {
        let mut context = BTreeMap::new();

        if let Some(qubit) = qubit {
            context.insert(
                ONE_QUBIT_MEASUREMENT_TIME.to_string(),
                qubit.one_qubit_measurement_time() as f64,
            );

            match qubit {
                PhysicalQubit::GateBased(gate_based) => {
                    if let Some(value) = gate_based.one_qubit_gate_time {
                        context.insert(ONE_QUBIT_GATE_TIME.to_string(), value as f64);
                    }

                    if let Some(value) = gate_based.two_qubit_gate_time {
                        context.insert(TWO_QUBIT_GATE_TIME.to_string(), value as f64);
                    }
                }
                PhysicalQubit::Majorana(majorana) => {
                    if let Some(value) = majorana.two_qubit_joint_measurement_time {
                        context.insert(TWO_QUBIT_JOINT_MEASUREMENT_TIME.to_string(), value as f64);
                    }
                }
            }
        }

        context.insert("codeDistance".to_string(), code_distance as f64);

        context
    }

    pub fn parse_compiled_expressions(
        logical_cycle_time_expr: &str,
        physical_qubits_per_logical_qubit_expr: &str,
    ) -> crate::system::Result<(CompiledExpression, CompiledExpression)> {
        Ok((
            CompiledExpression::from_string(logical_cycle_time_expr, "logical_cycle_time_expr")?,
            CompiledExpression::from_string(
                physical_qubits_per_logical_qubit_expr,
                "physical_qubits_per_logical_qubit_expr",
            )?,
        ))
    }
}

impl CodeWithThresholdAndDistanceEvaluator for ProtocolEvaluator {
    type Qubit = PhysicalQubit;

    fn physical_error_rate(&self, qubit: &Self::Qubit) -> f64 {
        qubit.clifford_error_rate().max(qubit.readout_error_rate())
    }

    fn physical_qubits(&self, code_distance: u64) -> Result<u64, String> {
        let mut context = Self::create_evaluation_context(None, code_distance);
        let value = self
            .physical_qubits_per_logical_qubit
            .evaluate(&mut context)
            .map_err(|err| err.to_string())?;

        if value <= 0.0 {
            Err(NonPositivePhysicalQubitsPerLogicalQubit(code_distance).to_string())
        } else {
            Ok(value as u64)
        }
    }

    fn logical_cycle_time(&self, qubit: &Self::Qubit, code_distance: u64) -> Result<u64, String> {
        let mut context = Self::create_evaluation_context(Some(qubit), code_distance);

        let result = self
            .logical_cycle_time
            .evaluate(&mut context)
            .map_err(|err| err.to_string())?;

        if result <= 0.0 {
            Err(NonPositiveLogicalCycleTime(code_distance).to_string())
        } else {
            Ok(result.round() as u64)
        }
    }
}

pub fn load_protocol_from_specification(
    model: &mut ProtocolSpecification,
    qubit: &PhysicalQubit,
) -> crate::system::Result<CodeWithThresholdAndDistance<ProtocolEvaluator>> {
    let (mut ftp, predefined) = base_protocol(model, qubit)?;

    if predefined {
        update_default_from_specification(&mut ftp, model)?;
    }

    if ftp.crossing_prefactor() > 0.5 {
        return Err(Error::InvalidValue(
            String::from("crossingPrefactor"),
            0.0,
            0.5,
        ));
    }

    // validate model with respect to qubit
    if qubit.clifford_error_rate() >= ftp.error_correction_threshold() {
        match qubit.instruction_set() {
                PhysicalInstructionSet::GateBased => {
                    return Err(Error::InvalidValue(
                        format!(
                            "{ONE_QUBIT_GATE_ERROR_RATE}, {TWO_QUBIT_GATE_ERROR_RATE}, {IDLE_ERROR_RATE}"
                        ),
                        0.0,
                        ftp.error_correction_threshold(),
                    ))
                }
                PhysicalInstructionSet::Majorana => {
                    return Err(Error::InvalidValue(
                        format!(
                            "{IDLE_ERROR_RATE}, {ONE_QUBIT_MEASUREMENT_PROCESS_ERROR_RATE}, {TWO_QUBIT_JOINT_MEASUREMENT_PROCESS_ERROR_RATE}",
                        ),
                        0.0,
                        ftp.error_correction_threshold(),
                    ))
                }
            }
    }

    // validate that formulas only yield positive values
    for code_distance in (1..=model.max_code_distance).skip(2) {
        // can you compute logical cycle time and number of physical qubits with code distance?
        ftp.logical_cycle_time(qubit, &code_distance)
            .map_err(LogicalCycleTimeComputationFailed)?;
        ftp.physical_qubits(&code_distance)
            .map_err(PhysicalQubitComputationFailed)?;
    }

    Ok(ftp)
}

fn base_protocol(
    model: &mut ProtocolSpecification,
    qubit: &PhysicalQubit,
) -> crate::system::Result<(CodeWithThresholdAndDistance<ProtocolEvaluator>, bool)> {
    if model.name == "surface_code" || model.name == "surfaceCode" || model.name == "surface-code" {
        match qubit.instruction_set() {
            PhysicalInstructionSet::GateBased => Ok((surface_code_gate_based(), true)),
            PhysicalInstructionSet::Majorana => Ok((surface_code_measurement_based(), true)),
        }
    } else if model.name == "floquet_code"
        || model.name == "floquetCode"
        || model.name == "floquet-code"
    {
        match qubit.instruction_set() {
            PhysicalInstructionSet::GateBased => Err(InvalidFaultToleranceProtocol.into()),
            PhysicalInstructionSet::Majorana => Ok((floquet_code(), true)),
        }
    } else {
        let error_correction_threshold = model.error_correction_threshold.ok_or_else(|| {
            CannotParseJSON(serde::de::Error::missing_field("errorCorrectionThreshold"))
        })?;
        let crossing_prefactor = model
            .crossing_prefactor
            .ok_or_else(|| CannotParseJSON(serde::de::Error::missing_field("crossingPrefactor")))?;

        let logical_cycle_time_expr = model
            .logical_cycle_time
            .as_ref()
            .ok_or_else(|| CannotParseJSON(serde::de::Error::missing_field("logicalCycleTime")))?
            .clone();

        let physical_qubits_per_logical_qubit_expr = model
            .physical_qubits_per_logical_qubit
            .as_ref()
            .ok_or_else(|| {
                CannotParseJSON(serde::de::Error::missing_field(
                    "physicalQubitsPerLogicalQubit",
                ))
            })?
            .clone();

        let (logical_cycle_time, physical_qubits_per_logical_qubit) =
            ProtocolEvaluator::parse_compiled_expressions(
                &logical_cycle_time_expr,
                &physical_qubits_per_logical_qubit_expr,
            )?;

        let max_code_distance = model.max_code_distance;

        let evaluator = ProtocolEvaluator {
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
        };

        Ok((
            CodeWithThresholdAndDistance::with_max_code_distance(
                evaluator,
                crossing_prefactor,
                error_correction_threshold,
                max_code_distance,
            ),
            false,
        ))
    }
}

/// Assumes that self is initialized to a default model based on a
/// predefined specification; then either updates `self` based on
/// additionally set values in `model`, or initializes non-assigned values
/// in `model` from the predefined ones.
fn update_default_from_specification(
    code: &mut CodeWithThresholdAndDistance<ProtocolEvaluator>,
    model: &mut ProtocolSpecification,
) -> crate::system::Result<()> {
    if let Some(error_correction_threshold) = model.error_correction_threshold {
        code.set_error_correction_threshold(error_correction_threshold);
    } else {
        model.error_correction_threshold = Some(code.error_correction_threshold());
    }

    if let Some(crossing_prefactor) = model.crossing_prefactor {
        code.set_crossing_prefactor(crossing_prefactor);
    } else {
        model.crossing_prefactor = Some(code.crossing_prefactor());
    }

    if let Some(logical_cycle_time) = model.logical_cycle_time.as_ref() {
        code.evaluator_mut().logical_cycle_time =
            CompiledExpression::from_string(logical_cycle_time, "logicalCycleTime")?;
    } else {
        model.logical_cycle_time = Some(code.evaluator().logical_cycle_time_expr.clone());
    }

    if let Some(physical_qubits_per_logical_qubit) =
        model.physical_qubits_per_logical_qubit.as_ref()
    {
        code.evaluator_mut().physical_qubits_per_logical_qubit = CompiledExpression::from_string(
            physical_qubits_per_logical_qubit,
            "physicalQubitsPerLogicalQubit",
        )?;
    } else {
        model.physical_qubits_per_logical_qubit = Some(
            code.evaluator()
                .physical_qubits_per_logical_qubit_expr
                .clone(),
        );
    }

    code.set_max_code_distance(model.max_code_distance);

    Ok(())
}

fn default_max_code_distance() -> u64 {
    MAX_CODE_DISTANCE
}

/// Default floquet code FTP for gate based qubits
///
/// ```yaml
/// name: "surface_code"
/// instruction_set: "gate_based"
/// # [arXiv:1208.0928, Eq. (13)]
/// # [arXiv:1009.3686, Figs. 6-7]
/// error_correction_threshold: 0.01
/// # [arXiv:1208.0928, Eq. (11)]
/// crossing_prefactor: 0.03
/// logical_cycle_time: "(4 * twoQubitGateTime + 2 * oneQubitMeasurementTime) * codeDistance"
/// physical_qubits_per_logical_qubit: "2 * codeDistance * codeDistance"
/// ```
#[must_use]
pub fn surface_code_gate_based() -> CodeWithThresholdAndDistance<ProtocolEvaluator> {
    // [arXiv:1208.0928, Eq. (13)]
    // [arXiv:1009.3686, Figs. 6-7]
    let error_correction_threshold = 0.01;
    // [arXiv:1208.0928, Eq. (11)]
    let crossing_prefactor = 0.03;

    let logical_cycle_time_expr =
        format!("(4 * {TWO_QUBIT_GATE_TIME} + 2 * {ONE_QUBIT_MEASUREMENT_TIME}) * codeDistance");
    let physical_qubits_per_logical_qubit_expr = String::from("2 * codeDistance * codeDistance");

    let (logical_cycle_time, physical_qubits_per_logical_qubit) =
        ProtocolEvaluator::parse_compiled_expressions(
            &logical_cycle_time_expr,
            &physical_qubits_per_logical_qubit_expr,
        )
        .expect("could not parse expressions");

    CodeWithThresholdAndDistance::with_max_code_distance(
        ProtocolEvaluator {
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
        },
        crossing_prefactor,
        error_correction_threshold,
        MAX_CODE_DISTANCE,
    )
}

/// Default floquet code FTP for measurement based qubits
///
/// ```yaml
/// name: "surface_code"
/// instruction_set: "measurement_based"
/// # [arXiv:2007.00307, Eq. (1)]
/// error_correction_threshold: 0.0015
/// crossing_prefactor: 0.08
/// logical_cycle_time: "20 * oneQubitMeasurementTime * codeDistance"
/// physical_qubits_per_logical_qubit: "2 * codeDistance * codeDistance"
/// ```
#[must_use]
pub fn surface_code_measurement_based() -> CodeWithThresholdAndDistance<ProtocolEvaluator> {
    // [arXiv:2007.00307, Eq. (1)]
    let error_correction_threshold = 0.0015;
    let crossing_prefactor = 0.08;

    let logical_cycle_time_expr = format!("20 * {ONE_QUBIT_MEASUREMENT_TIME} * codeDistance");
    let physical_qubits_per_logical_qubit_expr = String::from("2 * codeDistance * codeDistance");

    let (logical_cycle_time, physical_qubits_per_logical_qubit) =
        ProtocolEvaluator::parse_compiled_expressions(
            &logical_cycle_time_expr,
            &physical_qubits_per_logical_qubit_expr,
        )
        .expect("could not parse expressions");

    CodeWithThresholdAndDistance::with_max_code_distance(
        ProtocolEvaluator {
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
        },
        crossing_prefactor,
        error_correction_threshold,
        MAX_CODE_DISTANCE,
    )
}

/// Default floquet code FTP for measurement based qubits
///
/// ```yaml
/// name: "floquet_code"
/// instruction_set: "measurement_based"
/// error_correction_threshold: 0.01
/// crossing_prefactor: 0.07
/// logical_cycle_time: "3 * oneQubitMeasurementTime * codeDistance"
/// # [arXiv:2202.11829, Table 2]
/// physical_qubits_per_logical_qubit: "4 * codeDistance * codeDistance + 8 * (codeDistance - 1)"
/// ```
#[must_use]
pub fn floquet_code() -> CodeWithThresholdAndDistance<ProtocolEvaluator> {
    let error_correction_threshold = 0.01;
    let crossing_prefactor = 0.07;
    let logical_cycle_time_expr = format!("3 * {ONE_QUBIT_MEASUREMENT_TIME} * codeDistance");
    // [arXiv:2202.11829, Table 2]
    let physical_qubits_per_logical_qubit_expr =
        String::from("4 * codeDistance * codeDistance + 8 * (codeDistance - 1)");

    let (logical_cycle_time, physical_qubits_per_logical_qubit) =
        ProtocolEvaluator::parse_compiled_expressions(
            &logical_cycle_time_expr,
            &physical_qubits_per_logical_qubit_expr,
        )
        .expect("could not parse expressions");

    CodeWithThresholdAndDistance::with_max_code_distance(
        ProtocolEvaluator {
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
        },
        crossing_prefactor,
        error_correction_threshold,
        MAX_CODE_DISTANCE,
    )
}
