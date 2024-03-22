// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use test_utils::{
    check_callable_compute_properties, check_last_statement_compute_properties, CompilationContext,
};

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

#[test]
fn check_rca_for_operation_with_one_classical_return_and_one_dynamic_return() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        operation Foo() : Int {
            use q = Qubit();
            if M(q) == Zero {
                return 0;
            }
            return 1;
        }"#,
    );
    check_callable_compute_properties(
        &compilation_context.fir_store,
        compilation_context.get_compute_properties(),
        "Foo",
        &expect![
            r#"
            Callable: CallableComputeProperties:
                body: ApplicationsGeneratorSet:
                    inherent: Quantum: QuantumProperties:
                        runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | ForwardBranchingOnDynamicValue | ReturnWithinDynamicScope)
                        value_kind: Element(Dynamic)
                    dynamic_param_applications: <empty>
                adj: <none>
                ctl: <none>
                ctl-adj: <none>"#
        ],
    );
}
