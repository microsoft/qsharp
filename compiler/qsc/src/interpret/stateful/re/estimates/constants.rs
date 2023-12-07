// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Maximum allowed error code distance
pub const MAX_CODE_DISTANCE: u64 = 50;

/// Maximum number of distillation rounds
pub const MAX_DISTILLATION_ROUNDS: usize = 3;

/// Maximum number of extra distillation rounds in case none is found for [`MAX_DISTILLATION_ROUNDS`]
pub const MAX_EXTRA_DISTILLATION_ROUNDS: usize = 4;

/// (Γ_R in paper for layout)
#[allow(clippy::doc_markdown)]
pub const NUM_MEASUREMENTS_PER_R: u64 = 1;

/// (Γ_Tof in paper for layout)
#[allow(clippy::doc_markdown)]
pub const NUM_MEASUREMENTS_PER_TOF: u64 = 3;

/// A coefficient in Ts per rotation
pub const NUM_TS_PER_ROTATION_A_COEFFICIENT: f64 = 0.53;

/// A coefficient in Ts per rotation
pub const NUM_TS_PER_ROTATION_B_COEFFICIENT: f64 = 5.3;

// Physical qubit field names
pub const INSTRUCTION_SET: &str = "instructionSet";
pub const ONE_QUBIT_MEASUREMENT_TIME: &str = "oneQubitMeasurementTime";
pub const TWO_QUBIT_JOINT_MEASUREMENT_TIME: &str = "twoQubitJointMeasurementTime";
pub const ONE_QUBIT_MEASUREMENT_ERROR_RATE: &str = "oneQubitMeasurementErrorRate";
pub const ONE_QUBIT_MEASUREMENT_PROCESS_ERROR_RATE: &str = "oneQubitMeasurementErrorRate.process";
#[cfg(test)]
pub const TWO_QUBIT_JOINT_MEASUREMENT_ERROR_RATE: &str = "twoQubitJointMeasurementErrorRate";
pub const TWO_QUBIT_JOINT_MEASUREMENT_PROCESS_ERROR_RATE: &str =
    "twoQubitJointMeasurementErrorRate.process";
pub const ONE_QUBIT_GATE_TIME: &str = "oneQubitGateTime";
pub const ONE_QUBIT_GATE_ERROR_RATE: &str = "oneQubitGateErrorRate";
pub const TWO_QUBIT_GATE_TIME: &str = "twoQubitGateTime";
pub const TWO_QUBIT_GATE_ERROR_RATE: &str = "twoQubitGateErrorRate";
pub const T_GATE_ERROR_RATE: &str = "tGateErrorRate";
pub const IDLE_ERROR_RATE: &str = "idleErrorRate";
#[cfg(test)]
pub const READOUT: &str = "readout";
#[cfg(test)]
pub const PROCESS: &str = "process";

#[cfg(test)]
pub const FLOAT_COMPARISON_EPSILON: f64 = 0.000_000_000_1;
