// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_last_statement_compute_propeties, CompilationContext};
use expect_test::expect;

#[test]
fn check_rca_for_array_with_classical_elements() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"[1, 2, 3, 4, 5]"#);
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: Classical
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_with_one_dynamic_element() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        [One, M(q), One]"#);
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicResult)
                    value_kind: Static
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_with_dynamic_elements() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (a, b, c) = (Qubit(), Qubit(), Qubit());
        [M(a), M(b), M(c)]"#);
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicResult)
                    value_kind: Static
                dynamic_param_applications: <empty>"#
        ],
    );
}