// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_last_statement_compute_propeties, CompilationContext};
use expect_test::expect;

#[test]
fn check_rca_for_static_single_measurement() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            use q = Qubit();
            M(q)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Dynamic
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_static_single_measurement_and_reset() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            open Microsoft.Quantum.Measurement;
            use q = Qubit();
            MResetZ(q)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Dynamic
                dynamic_param_applications: <empty>"#
        ],
    );
}
