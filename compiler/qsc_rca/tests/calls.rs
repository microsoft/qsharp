// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use test_utils::{check_last_statement_compute_properties, CompilationContext};

#[test]
fn check_rca_for_call_to_cyclic_function_with_classical_argument() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function GaussSum(n : Int) : Int {
            if n == 0 {
                0
            } else {
                n + GaussSum(n - 1)
            }
        }
        GaussSum(10)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Classical
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_cyclic_function_with_dynamic_argument() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        function GaussSum(n : Int) : Int {
            if n == 0 {
                0
            } else {
                n + GaussSum(n - 1)
            }
        }
        use q = Qubit();
        GaussSum(M(q) == Zero ? 10 | 20)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | CallToCyclicFunctionWithDynamicArg)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_cyclic_operation_with_classical_argument() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        operation GaussSum(n : Int) : Int {
            if n == 0 {
                0
            } else {
                n + GaussSum(n - 1)
            }
        }
        GaussSum(10)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicInt | CallToCyclicOperation)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_cyclic_operation_with_dynamic_argument() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        operation GaussSum(n : Int) : Int {
            if n == 0 {
                0
            } else {
                n + GaussSum(n - 1)
            }
        }
        use q = Qubit();
        GaussSum(M(q) == Zero ? 10 | 20)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | CallToCyclicOperation)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_static_closure_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        let f = i -> IsCoprimeI(11, i);
        f(13)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that the output of the closure function is dynamic due to closures currently considered always dynamic
    // because we are not performing detailed analysis on them.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | CallToDynamicCallee | UseOfClosure)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_dynamic_closure_function() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        use q = Qubit();
        let dynamicInt = M(q) == Zero ? 11 | 13;
        let f = i -> IsCoprimeI(dynamicInt, i);
        f(17)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that the "use of dynamic integer" runtime feature is missing because we are currently not performing
    // detailed analysis on closures.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | CallToDynamicCallee | UseOfClosure)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_static_closure_operation() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        use qubit = Qubit();
        let theta = PI();
        let f = q => Rx(theta, q);
        f(qubit)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that closures currently considered always dynamic because we are not performing detailed analysis on them.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
        ApplicationsGeneratorSet:
            inherent: Quantum: QuantumProperties:
                runtime_features: RuntimeFeatureFlags(CallToDynamicCallee | UseOfClosure)
                value_kind: Element(Static)
            dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_dynamic_closure_operation() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        use qubit = Qubit();
        let theta = M(qubit) == Zero ? PI() | PI() / 2.0;
        let f = q => Rx(theta, q);
        f(qubit)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();

    // Note that the "use of dynamic bool" and the "use of dynamic double" runtime features are missing because we are
    // currently not performing detailed analysis on closures.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
        ApplicationsGeneratorSet:
            inherent: Quantum: QuantumProperties:
                runtime_features: RuntimeFeatureFlags(CallToDynamicCallee | UseOfClosure)
                value_kind: Element(Static)
            dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_call_to_operation_with_one_classical_return_and_one_dynamic_return() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        operation Foo() : Int {
            use q = Qubit();
            if M(q) == Zero {
                return 0;
            }
            return 1;
        }
        Foo()"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | ForwardBranchingOnDynamicValue | ReturnWithinDynamicScope)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}
