// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::Value;

use crate::LogicalResources;

use super::estimate_physical_resources;

#[test]
fn estimate_single() {
    let logical_resources = LogicalResources {
        num_qubits: 100,
        t_count: 0,
        rotation_count: 112_110,
        rotation_depth: 2001,
        ccz_count: 0,
        measurement_count: 0,
    };

    let params: &str = "[{}]";
    let result = estimate_physical_resources(&logical_resources, params);

    let json_value: Vec<Value> =
        serde_json::from_str(&result.expect("result is err")).expect("Failed to parse JSON");
    assert_eq!(json_value.len(), 1);

    let map = json_value[0].as_object().expect("Failed build map");
    assert!(!map.contains_key("frontierEntries"));
    assert!(map.contains_key("logicalQubit"));
    assert!(map.contains_key("physicalCounts"));
    assert!(map.contains_key("physicalCountsFormatted"));
}

#[test]
fn estimate_frontier() {
    let logical_resources = LogicalResources {
        num_qubits: 100,
        t_count: 0,
        rotation_count: 112_110,
        rotation_depth: 2001,
        ccz_count: 0,
        measurement_count: 0,
    };

    let params: &str = r#"[{
        "estimateType": "frontier"
    }]"#;

    let result = estimate_physical_resources(&logical_resources, params);

    let json_value: Vec<Value> =
        serde_json::from_str(&result.expect("result is err")).expect("Failed to parse JSON");
    assert_eq!(json_value.len(), 1);

    let map = json_value[0].as_object().expect("Failed build map");
    assert!(map.contains_key("frontierEntries"));
    assert!(!map.contains_key("logicalQubit"));
    assert!(!map.contains_key("physicalCounts"));
    assert!(!map.contains_key("physicalCountsFormatted"));
}

#[test]
fn physical_estimates_crash() {
    let result = estimate_physical_resources(
        &LogicalResources {
            num_qubits: 9,
            t_count: 160,
            rotation_count: 0,
            rotation_depth: 0,
            ccz_count: 8,
            measurement_count: 5,
        },
        r#"[{"qubitParams": {"name": "qubit_maj_ns_e6"},
            "qecScheme": {"name": "floquet_code"},
            "errorBudget": 0.075}]"#,
    );

    assert!(result
        .expect("estimation should succeed")
        .contains(r#""status":"success"#));
}
