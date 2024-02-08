// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::rc::Rc;

use crate::{
    estimates::LogicalQubit,
    system::{constants::FLOAT_COMPARISON_EPSILON, modeling::Protocol, Result},
};

use super::{
    super::super::constants::{
        IDLE_ERROR_RATE, INSTRUCTION_SET, ONE_QUBIT_GATE_ERROR_RATE, ONE_QUBIT_GATE_TIME,
        ONE_QUBIT_MEASUREMENT_ERROR_RATE, ONE_QUBIT_MEASUREMENT_TIME, PROCESS, READOUT,
        TWO_QUBIT_JOINT_MEASUREMENT_ERROR_RATE, T_GATE_ERROR_RATE,
    },
    GateBasedPhysicalQubit, MajoranaQubit,
};
use serde_json::json;

use super::PhysicalQubit;

fn load_qubit_from_json_string(data: &str) -> PhysicalQubit {
    serde_json::from_str(data).expect("test json should be parsable")
}

impl PhysicalQubit {
    pub fn qubit_gate_ns_e3() -> Self {
        Self::GateBased(GateBasedPhysicalQubit::qubit_gate_ns_e3())
    }

    pub fn qubit_gate_ns_e4() -> Self {
        Self::GateBased(GateBasedPhysicalQubit::qubit_gate_ns_e4())
    }

    pub fn qubit_gate_us_e3() -> Self {
        Self::GateBased(GateBasedPhysicalQubit::qubit_gate_us_e3())
    }

    pub fn qubit_gate_us_e4() -> Self {
        Self::GateBased(GateBasedPhysicalQubit::qubit_gate_us_e4())
    }

    pub fn qubit_maj_ns_e4() -> Self {
        Self::Majorana(MajoranaQubit::qubit_maj_ns_e4())
    }

    pub fn qubit_maj_ns_e6() -> Self {
        Self::Majorana(MajoranaQubit::qubit_maj_ns_e6())
    }
}

#[test]
fn test_defaults_from_names() {
    let from_func = PhysicalQubit::qubit_gate_ns_e3();
    let from_file = load_qubit_from_json_string(r#"{"name": "qubit_gate_ns_e3"}"#);
    assert_eq!(from_func, from_file);

    let from_func = PhysicalQubit::qubit_gate_ns_e4();
    let from_file = load_qubit_from_json_string(r#"{"name": "qubit_gate_ns_e4"}"#);
    assert_eq!(from_func, from_file);

    let from_func = PhysicalQubit::qubit_gate_us_e3();
    let from_file = load_qubit_from_json_string(r#"{"name": "qubit_gate_us_e3"}"#);
    assert_eq!(from_func, from_file);

    let from_func = PhysicalQubit::qubit_gate_us_e4();
    let from_file = load_qubit_from_json_string(r#"{"name": "qubit_gate_us_e4"}"#);
    assert_eq!(from_func, from_file);

    let from_func = PhysicalQubit::qubit_maj_ns_e4();
    let from_file = load_qubit_from_json_string(r#"{"name": "qubit_maj_ns_e4"}"#);
    assert_eq!(from_func, from_file);

    let from_func = PhysicalQubit::qubit_maj_ns_e6();
    let from_file = load_qubit_from_json_string(r#"{"name": "qubit_maj_ns_e6"}"#);
    assert_eq!(from_func, from_file);
}

#[test]
fn derived_defaults_gate_based() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "gateBased", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{ONE_QUBIT_GATE_TIME}": "81 ns", "{T_GATE_ERROR_RATE}": 0.05, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": 0.001, "{ONE_QUBIT_GATE_ERROR_RATE}": 0.005}}"#,
    );

    let PhysicalQubit::GateBased(qubit) = load_qubit_from_json_string(payload) else {
        unreachable!()
    };

    assert_eq!(qubit.one_qubit_measurement_time, Some(42));
    assert_eq!(qubit.one_qubit_gate_time, Some(81));
    assert_eq!(qubit.two_qubit_gate_time, Some(81));
    assert!((qubit.t_gate_error_rate - 0.05).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate - 0.001).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_gate_error_rate - 0.005).abs() <= f64::EPSILON);
    assert!((qubit.idle_error_rate - 0.001).abs() <= f64::EPSILON);
}

#[test]
fn derived_defaults_measurement_based() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "Majorana", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{T_GATE_ERROR_RATE}": 0.41, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": {{ "{PROCESS}": 0.83, "{READOUT}": 0.56 }}, "{IDLE_ERROR_RATE}": 0.005}}"#,
    );

    let PhysicalQubit::Majorana(qubit) = load_qubit_from_json_string(payload) else {
        unreachable!()
    };

    assert_eq!(qubit.one_qubit_measurement_time, Some(42));
    assert_eq!(qubit.two_qubit_joint_measurement_time, Some(42));
    assert!((qubit.t_gate_error_rate - 0.41).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate.process() - 0.83).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate.readout() - 0.56).abs() <= f64::EPSILON);
    // default from one_qubit_measurement_readout_error_rate
    assert!((qubit.two_qubit_joint_measurement_error_rate.process() - 0.56).abs() <= f64::EPSILON);
    // default from one_qubit_measurement_readout_error_rate
    assert!((qubit.two_qubit_joint_measurement_error_rate.readout() - 0.56).abs() <= f64::EPSILON);
    assert!((qubit.idle_error_rate - 0.005).abs() <= f64::EPSILON);
}

#[test]
fn derived_defaults_measurement_based_override_old_format() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "Majorana", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{T_GATE_ERROR_RATE}": 0.41, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": 0.83, "{IDLE_ERROR_RATE}": 0.005}}"#,
    );

    let PhysicalQubit::Majorana(qubit) = load_qubit_from_json_string(payload) else {
        unreachable!()
    };

    assert_eq!(qubit.one_qubit_measurement_time, Some(42));
    assert_eq!(qubit.two_qubit_joint_measurement_time, Some(42));
    assert!((qubit.t_gate_error_rate - 0.41).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate.process() - 0.83).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate.readout() - 0.83).abs() <= f64::EPSILON);
    assert!((qubit.two_qubit_joint_measurement_error_rate.process() - 0.83).abs() <= f64::EPSILON);
    assert!((qubit.two_qubit_joint_measurement_error_rate.readout() - 0.83).abs() <= f64::EPSILON);
    assert!((qubit.idle_error_rate - 0.005).abs() <= f64::EPSILON);
}

#[test]
fn derived_defaults_measurement_based_override() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "Majorana", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{T_GATE_ERROR_RATE}": 0.41, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": {{ "{PROCESS}": 0.83, "{READOUT}": 0.56 }}, "{TWO_QUBIT_JOINT_MEASUREMENT_ERROR_RATE}": {{ "{PROCESS}": 0.15, "{READOUT}": 0.17 }}, "{IDLE_ERROR_RATE}": 0.005}}"#,
    );

    let PhysicalQubit::Majorana(qubit) = load_qubit_from_json_string(payload) else {
        unreachable!()
    };

    assert_eq!(qubit.one_qubit_measurement_time, Some(42));
    assert_eq!(qubit.two_qubit_joint_measurement_time, Some(42));
    assert!((qubit.t_gate_error_rate - 0.41).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate.process() - 0.83).abs() <= f64::EPSILON);
    assert!((qubit.one_qubit_measurement_error_rate.readout() - 0.56).abs() <= f64::EPSILON);
    assert!((qubit.two_qubit_joint_measurement_error_rate.process() - 0.15).abs() <= f64::EPSILON);
    assert!((qubit.two_qubit_joint_measurement_error_rate.readout() - 0.17).abs() <= f64::EPSILON);
    assert!((qubit.idle_error_rate - 0.005).abs() <= f64::EPSILON);
}

#[test]
fn update_field_for_default_gate_based() {
    let payload = &format!(r#"{{"name": "qubit_gate_ns_e3", "{T_GATE_ERROR_RATE}": 0.41}}"#);

    let qubit = load_qubit_from_json_string(payload);

    let mut expected = PhysicalQubit::qubit_gate_ns_e3();
    if let PhysicalQubit::GateBased(qubit) = &mut expected {
        qubit.t_gate_error_rate = 0.41;
    }

    assert_eq!(qubit, expected);
}

#[test]
fn update_field_for_default_majorana() {
    let payload = &format!(r#"{{"name": "qubit_maj_ns_e6", "{T_GATE_ERROR_RATE}": 0.41}}"#);

    let qubit = load_qubit_from_json_string(payload);

    let mut expected = PhysicalQubit::qubit_maj_ns_e6();
    if let PhysicalQubit::Majorana(qubit) = &mut expected {
        qubit.t_gate_error_rate = 0.41;
    }

    assert_eq!(qubit, expected);
}

#[test]
fn majorana_serialization() -> serde_json::error::Result<()> {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "Majorana", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{T_GATE_ERROR_RATE}": 0.41, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": 0.83, "{IDLE_ERROR_RATE}": 0.005}}"#,
    );

    let qubit = load_qubit_from_json_string(payload);
    let serialized = serde_json::to_value(qubit)?;

    assert_eq!(
        serialized,
        json!({
            "name": "",
            "instructionSet": "Majorana",
            "oneQubitMeasurementTime": "42 ns",
            "twoQubitJointMeasurementTime": "42 ns",
            "tGateTime": "42 ns",
            "tGateErrorRate": 0.41,
            "oneQubitMeasurementErrorRate": {
                "process": 0.83,
                "readout": 0.83
            },
            "twoQubitJointMeasurementErrorRate": {
                "process": 0.83,
                "readout": 0.83
            },
            "idleErrorRate":0.005
        })
    );

    Ok(())
}

#[test]
fn gate_based_serialization() -> serde_json::error::Result<()> {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "gateBased", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{ONE_QUBIT_GATE_TIME}": "81 ns", "{T_GATE_ERROR_RATE}": 0.05, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": 0.001, "{ONE_QUBIT_GATE_ERROR_RATE}": 0.005}}"#,
    );

    let qubit = load_qubit_from_json_string(payload);

    let serialized = serde_json::to_value(qubit)?;

    assert_eq!(
        serialized,
        json!({
            "name": "",
            "instructionSet": "GateBased",
            "oneQubitMeasurementTime": "42 ns",
            "oneQubitGateTime": "81 ns",
            "twoQubitGateTime": "81 ns",
            "tGateTime": "81 ns",
            "oneQubitGateErrorRate": 0.005,
            "twoQubitGateErrorRate": 0.005,
            "tGateErrorRate": 0.05,
            "oneQubitMeasurementErrorRate": 0.001,
            "idleErrorRate": 0.001
        })
    );

    Ok(())
}

#[test]
fn unknown_default_model_name() {
    let payload = r#"{"name": "unknown"}"#;

    let error = serde_json::from_str::<PhysicalQubit>(payload)
        .expect_err("expected deserialization to fail");

    assert_eq!(error.to_string(), "missing field `instructionSet`");
}

#[test]
fn majorana_field_not_supported_in_gate_based_1() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "gateBased", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{ONE_QUBIT_GATE_TIME}": "81 ns", "{T_GATE_ERROR_RATE}": 0.05, "{ONE_QUBIT_GATE_ERROR_RATE}": 0.005, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": {{ "{PROCESS}": 0.15, "{READOUT}": 0.17 }}}}"#,
    );

    let error = serde_json::from_str::<PhysicalQubit>(payload)
        .expect_err("expected deserialization to fail");

    assert_eq!(error.to_string(), "invalid type: map, expected f64");
}

#[test]
fn majorana_field_not_supported_in_gate_based_2() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "gateBased", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{ONE_QUBIT_GATE_TIME}": "81 ns", "{T_GATE_ERROR_RATE}": 0.05, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": 0.001, "{ONE_QUBIT_GATE_ERROR_RATE}": 0.005, "{TWO_QUBIT_JOINT_MEASUREMENT_ERROR_RATE}": {{ "{PROCESS}": 0.15, "{READOUT}": 0.17 }}}}"#,
    );

    let error = serde_json::from_str::<PhysicalQubit>(payload)
        .expect_err("expected deserialization to fail");

    assert_eq!(
        error.to_string(),
        "unknown field `twoQubitJointMeasurementErrorRate`, expected one of `name`, `oneQubitMeasurementTime`, `oneQubitGateTime`, `twoQubitGateTime`, `tGateTime`, `oneQubitMeasurementErrorRate`, `oneQubitGateErrorRate`, `twoQubitGateErrorRate`, `tGateErrorRate`, `idleErrorRate`"
    );
}

#[test]
fn gate_based_field_not_supported_in_majorana() {
    let payload = &format!(
        r#"{{"{INSTRUCTION_SET}": "Majorana", "{ONE_QUBIT_MEASUREMENT_TIME}": "42 ns", "{T_GATE_ERROR_RATE}": 0.41, "{ONE_QUBIT_MEASUREMENT_ERROR_RATE}": 0.83, "{ONE_QUBIT_GATE_TIME}": "1ns", "{IDLE_ERROR_RATE}": 0.005}}"#,
    );

    let error = serde_json::from_str::<PhysicalQubit>(payload)
        .expect_err("expected deserialization to fail");

    assert_eq!(
        error.to_string(),
        "unknown field `oneQubitGateTime`, expected one of `name`, `oneQubitMeasurementTime`, `twoQubitJointMeasurementTime`, `tGateTime`, `oneQubitMeasurementErrorRate`, `twoQubitJointMeasurementErrorRate`, `tGateErrorRate`, `idleErrorRate`"
    );
}

#[test]
fn load_physical_qubit_from_file() {
    let qubit = load_qubit_from_json_string(r#"{"name": "qubit_gate_ns_e3"}"#);

    assert_eq!(qubit, PhysicalQubit::qubit_gate_ns_e3());
}

#[test]
fn logical_qubit_from_fast_gate_based_and_surface_code() -> Result<()> {
    let physical_qubit = Rc::new(PhysicalQubit::default());

    let ftp = Protocol::default();

    let logical_qubit = LogicalQubit::new(&ftp, 7, physical_qubit)?;

    assert_eq!(logical_qubit.physical_qubits(), 98);
    assert!(
        (logical_qubit.logical_error_rate() - 3.000_000_000_000_001_3e-6).abs()
            < FLOAT_COMPARISON_EPSILON
    );

    Ok(())
}
