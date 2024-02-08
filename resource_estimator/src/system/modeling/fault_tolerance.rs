// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::estimates::{
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
    #[serde(default = "Protocol::default_max_code_distance")]
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
            max_code_distance: Protocol::default_max_code_distance(),
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
#[derive(Debug)]
pub struct Protocol {
    error_correction_threshold: f64,
    crossing_prefactor: f64,
    logical_cycle_time_expr: String,
    logical_cycle_time: CompiledExpression,
    physical_qubits_per_logical_qubit_expr: String,
    physical_qubits_per_logical_qubit: CompiledExpression,
    max_code_distance: u64,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::surface_code_gate_based()
    }
}

impl Protocol {
    /// Loads and validates a fault-tolerance protocol from a fault-tolerance model
    pub(crate) fn load_from_specification(
        model: &mut ProtocolSpecification,
        qubit: &PhysicalQubit,
    ) -> crate::system::Result<Self> {
        let (mut ftp, predefined) = Self::base_protocol(model, qubit)?;

        if predefined {
            ftp.update_default_from_specification(model)?;
        }

        if ftp.crossing_prefactor > 0.5 {
            return Err(Error::InvalidValue(
                String::from("crossingPrefactor"),
                0.0,
                0.5,
            ));
        }

        // validate model with respect to qubit
        if qubit.clifford_error_rate() >= ftp.error_correction_threshold {
            match qubit.instruction_set() {
                PhysicalInstructionSet::GateBased => {
                    return Err(Error::InvalidValue(
                        format!(
                            "{ONE_QUBIT_GATE_ERROR_RATE}, {TWO_QUBIT_GATE_ERROR_RATE}, {IDLE_ERROR_RATE}"
                        ),
                        0.0,
                        ftp.error_correction_threshold,
                    ))
                }
                PhysicalInstructionSet::Majorana => {
                    return Err(Error::InvalidValue(
                        format!(
                            "{IDLE_ERROR_RATE}, {ONE_QUBIT_MEASUREMENT_PROCESS_ERROR_RATE}, {TWO_QUBIT_JOINT_MEASUREMENT_PROCESS_ERROR_RATE}",
                        ),
                        0.0,
                        ftp.error_correction_threshold,
                    ))
                }
            }
        }

        // validate that formulas only yield positive values
        for code_distance in (1..=model.max_code_distance).skip(2) {
            // can you compute logical cycle time and number of physical qubits with code distance?
            ftp.logical_cycle_time(qubit, code_distance)
                .map_err(LogicalCycleTimeComputationFailed)?;
            ftp.physical_qubits_per_logical_qubit(code_distance)
                .map_err(PhysicalQubitComputationFailed)?;
        }

        Ok(ftp)
    }

    fn base_protocol(
        model: &mut ProtocolSpecification,
        qubit: &PhysicalQubit,
    ) -> crate::system::Result<(Self, bool)> {
        if model.name == "surface_code"
            || model.name == "surfaceCode"
            || model.name == "surface-code"
        {
            match qubit.instruction_set() {
                PhysicalInstructionSet::GateBased => Ok((Self::surface_code_gate_based(), true)),
                PhysicalInstructionSet::Majorana => {
                    Ok((Self::surface_code_measurement_based(), true))
                }
            }
        } else if model.name == "floquet_code"
            || model.name == "floquetCode"
            || model.name == "floquet-code"
        {
            match qubit.instruction_set() {
                PhysicalInstructionSet::GateBased => Err(InvalidFaultToleranceProtocol.into()),
                PhysicalInstructionSet::Majorana => Ok((Self::floquet_code(), true)),
            }
        } else {
            let error_correction_threshold = model.error_correction_threshold.ok_or_else(|| {
                CannotParseJSON(serde::de::Error::missing_field("errorCorrectionThreshold"))
            })?;
            let crossing_prefactor = model.crossing_prefactor.ok_or_else(|| {
                CannotParseJSON(serde::de::Error::missing_field("crossingPrefactor"))
            })?;

            let logical_cycle_time_expr = model
                .logical_cycle_time
                .as_ref()
                .ok_or_else(|| {
                    CannotParseJSON(serde::de::Error::missing_field("logicalCycleTime"))
                })?
                .clone();

            let physical_qubits_per_logical_qubit_expr = model
                .logical_cycle_time
                .as_ref()
                .ok_or_else(|| {
                    CannotParseJSON(serde::de::Error::missing_field(
                        "physicalQubitsPerLogicalQubit",
                    ))
                })?
                .clone();

            let (logical_cycle_time, physical_qubits_per_logical_qubit) =
                Protocol::parse_compiled_expressions(
                    &logical_cycle_time_expr,
                    &physical_qubits_per_logical_qubit_expr,
                )?;

            let max_code_distance = model.max_code_distance;

            Ok((
                Self {
                    error_correction_threshold,
                    crossing_prefactor,
                    logical_cycle_time_expr,
                    logical_cycle_time,
                    physical_qubits_per_logical_qubit_expr,
                    physical_qubits_per_logical_qubit,
                    max_code_distance,
                },
                false,
            ))
        }
    }

    /// Assumes that self is initialized to a default model based on a
    /// predefined specification; then either updates `self` based on
    /// additionally set values in `model`, or initializes non-assigned values
    /// in `model` from the predefined ones.
    fn update_default_from_specification(
        &mut self,
        model: &mut ProtocolSpecification,
    ) -> crate::system::Result<()> {
        if let Some(error_correction_threshold) = model.error_correction_threshold {
            self.error_correction_threshold = error_correction_threshold;
        } else {
            model.error_correction_threshold = Some(self.error_correction_threshold);
        }

        if let Some(crossing_prefactor) = model.crossing_prefactor {
            self.crossing_prefactor = crossing_prefactor;
        } else {
            model.crossing_prefactor = Some(self.crossing_prefactor);
        }

        if let Some(logical_cycle_time) = model.logical_cycle_time.as_ref() {
            self.logical_cycle_time =
                CompiledExpression::from_string(logical_cycle_time, "logicalCycleTime")?;
        } else {
            model.logical_cycle_time = Some(self.logical_cycle_time_expr.clone());
        }

        if let Some(physical_qubits_per_logical_qubit) =
            model.physical_qubits_per_logical_qubit.as_ref()
        {
            self.physical_qubits_per_logical_qubit = CompiledExpression::from_string(
                physical_qubits_per_logical_qubit,
                "physicalQubitsPerLogicalQubit",
            )?;
        } else {
            model.physical_qubits_per_logical_qubit =
                Some(self.physical_qubits_per_logical_qubit_expr.clone());
        }

        self.max_code_distance = model.max_code_distance;

        Ok(())
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
    pub(crate) fn surface_code_gate_based() -> Self {
        // [arXiv:1208.0928, Eq. (13)]
        // [arXiv:1009.3686, Figs. 6-7]
        let error_correction_threshold = 0.01;
        // [arXiv:1208.0928, Eq. (11)]
        let crossing_prefactor = 0.03;

        let logical_cycle_time_expr = format!(
            "(4 * {TWO_QUBIT_GATE_TIME} + 2 * {ONE_QUBIT_MEASUREMENT_TIME}) * codeDistance"
        );
        let physical_qubits_per_logical_qubit_expr =
            String::from("2 * codeDistance * codeDistance");

        let (logical_cycle_time, physical_qubits_per_logical_qubit) =
            Protocol::parse_compiled_expressions(
                &logical_cycle_time_expr,
                &physical_qubits_per_logical_qubit_expr,
            )
            .expect("could not parse expressions");

        Self {
            error_correction_threshold,
            crossing_prefactor,
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
            max_code_distance: Self::default_max_code_distance(),
        }
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
    pub(crate) fn surface_code_measurement_based() -> Self {
        // [arXiv:2007.00307, Eq. (1)]
        let error_correction_threshold = 0.0015;
        let crossing_prefactor = 0.08;

        let logical_cycle_time_expr = format!("20 * {ONE_QUBIT_MEASUREMENT_TIME} * codeDistance");
        let physical_qubits_per_logical_qubit_expr =
            String::from("2 * codeDistance * codeDistance");

        let (logical_cycle_time, physical_qubits_per_logical_qubit) =
            Protocol::parse_compiled_expressions(
                &logical_cycle_time_expr,
                &physical_qubits_per_logical_qubit_expr,
            )
            .expect("could not parse expressions");

        Self {
            error_correction_threshold,
            crossing_prefactor,
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
            max_code_distance: Self::default_max_code_distance(),
        }
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
    pub(crate) fn floquet_code() -> Self {
        let error_correction_threshold = 0.01;
        let crossing_prefactor = 0.07;
        let logical_cycle_time_expr = format!("3 * {ONE_QUBIT_MEASUREMENT_TIME} * codeDistance");
        // [arXiv:2202.11829, Table 2]
        let physical_qubits_per_logical_qubit_expr =
            String::from("4 * codeDistance * codeDistance + 8 * (codeDistance - 1)");

        let (logical_cycle_time, physical_qubits_per_logical_qubit) =
            Protocol::parse_compiled_expressions(
                &logical_cycle_time_expr,
                &physical_qubits_per_logical_qubit_expr,
            )
            .expect("could not parse expressions");

        Self {
            error_correction_threshold,
            crossing_prefactor,
            logical_cycle_time_expr,
            logical_cycle_time,
            physical_qubits_per_logical_qubit_expr,
            physical_qubits_per_logical_qubit,
            max_code_distance: Self::default_max_code_distance(),
        }
    }

    /// Returns the fault-tolerance protocol's crossing prefactor.
    fn crossing_prefactor(&self) -> f64 {
        self.crossing_prefactor
    }

    /// Returns the fault-tolerance protocol's error-correction threshold.
    fn error_correction_threshold(&self) -> f64 {
        self.error_correction_threshold
    }

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

    fn parse_compiled_expressions(
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

    #[inline]
    fn default_max_code_distance() -> u64 {
        MAX_CODE_DISTANCE
    }
}

impl ErrorCorrection for Protocol {
    type Qubit = PhysicalQubit;

    fn max_code_distance(&self) -> u64 {
        self.max_code_distance
    }

    /// Computes the number of physical qubits required for one logical qubit
    ///
    /// The formula for this field has a default value of `2 * code_distance *
    /// code_distance`.
    fn physical_qubits_per_logical_qubit(&self, code_distance: u64) -> Result<u64, String> {
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

    /// Returns the time of one logical cycle.
    ///
    /// The logical cycle time is the time it takes to perform `code_distance`
    /// rounds of quantum error correction.  The latter, also called syndrome
    /// extraction time, is based on physical operation times specified in the
    /// qubit and usually some factor based on the choice of stabilizer
    /// extraction circuit.
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

    /// Computes the logical failure probability.
    ///
    /// Computes the logical failure probability based on a physical error rate
    /// and a code distance
    fn logical_failure_probability(
        &self,
        qubit: &Self::Qubit,
        code_distance: u64,
    ) -> Result<f64, String> {
        let physical_error_rate = qubit.clifford_error_rate().max(qubit.readout_error_rate());

        if physical_error_rate > self.error_correction_threshold() {
            Err(Error::InvalidValue(
                String::from("physical_error_rate"),
                0.0,
                self.error_correction_threshold,
            )
            .to_string())
        } else {
            #[allow(clippy::cast_possible_truncation)]
            Ok(self.crossing_prefactor()
                * ((physical_error_rate / self.error_correction_threshold())
                    .powi((code_distance as i32 + 1) / 2)))
        }
    }

    // Compute code distance d (Equation (E2) in paper)
    fn compute_code_distance(
        &self,
        qubit: &Self::Qubit,
        required_logical_qubit_error_rate: f64,
    ) -> u64 {
        let physical_error_rate = qubit.clifford_error_rate().max(qubit.readout_error_rate());
        let numerator = 2.0 * (self.crossing_prefactor() / required_logical_qubit_error_rate).ln();
        let denominator = (self.error_correction_threshold() / physical_error_rate).ln();

        (((numerator / denominator) - 1.0).ceil() as u64) | 0x1
    }
}
