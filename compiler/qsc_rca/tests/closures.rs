// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_last_statement_compute_properties, CompilationContext};
use expect_test::expect;

#[test]
fn check_rca_for_closure_function_with_classical_captured_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        let f = i -> IsCoprimeI(11, i);
        f"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that a closure is always considered dynamic because we are currently not performing detailed analysis on
    // them.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfClosure)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_closure_function_with_dynamic_captured_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        use q = Qubit();
        let dynamicInt = M(q) == Zero ? 11 | 13;
        let f = i -> IsCoprimeI(dynamicInt, i);
        f"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that only the "use of closure" runtime feature appears because we are currently not performing detailed
    // analysis on closures.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfClosure)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_closure_operation_with_classical_captured_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        let theta = PI();
        let f = q => Rx(theta, q);
        f"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that a closure is always considered dynamic because we are currently not performing detailed analysis on
    // them.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfClosure)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_closure_operation_with_dynamic_captured_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        use qubit = Qubit();
        let theta = M(qubit) == Zero ? PI() | PI() / 2.0;
        let f = q => Rx(theta, q);
        f"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that only the "use of closure" runtime feature appears because we are currently not performing detailed
    // analysis on closures.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfClosure)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}
