// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use test_utils::{check_last_statement_compute_properties, CompilationContext};

#[test]
fn check_rca_for_array_with_classical_elements() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"[1.0, 2.0, 3.0, 4.0, 5.0]"#);
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
fn check_rca_for_array_with_dynamic_results() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (a, b, c) = (Qubit(), Qubit(), Qubit());
        [M(a), M(b), M(c)]"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    // Even though results are dynamic, they do not require any special runtime features to exist.
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Array(Content: Dynamic, Size: Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_with_dynamic_bools() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Convert;
        use (a, b, c) = (Qubit(), Qubit(), Qubit());
        [ResultAsBool(M(a)), ResultAsBool(M(b)), ResultAsBool(M(c))]"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool)
                    value_kind: Array(Content: Dynamic, Size: Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_repeat_with_classical_value_and_classical_size() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"[1L, size = 11]"#);
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
fn check_rca_for_array_repeat_with_dynamic_result_value_and_classical_size() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        [M(q), size = 11]"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Array(Content: Dynamic, Size: Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_repeat_with_dynamic_bool_value_and_classical_size() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Convert;
        use q = Qubit();
        [ResultAsBool(M(q)), size = 11]"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool)
                    value_kind: Array(Content: Dynamic, Size: Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_repeat_with_classical_value_and_dynamic_size() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        let s = M(q) == Zero ? 5 | 10;
        [Zero, size = s]"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | UseOfDynamicallySizedArray)
                    value_kind: Array(Content: Static, Size: Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_array_repeat_with_dynamic_double_value_and_dynamic_size() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Convert;
        use q = Qubit();
        let r = M(q);
        let s = r == Zero ? 5 | 10;
        let d = IntAsDouble(s);
        [d, size = s]"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | UseOfDynamicDouble | UseOfDynamicallySizedArray)
                    value_kind: Array(Content: Dynamic, Size: Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_mutable_array_statically_appended() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        mutable arr = [];
        use q = Qubit();
        for i in 0..10 {
            set arr += [M(q)];
        }
        arr"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![[r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Array(Content: Dynamic, Size: Static)
                dynamic_param_applications: <empty>"#]],
    );
}

#[test]
fn check_rca_for_mutable_array_dynamically_appended() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        mutable arr = [0, 1];
        use q = Qubit();
        if M(q) == Zero {
            set arr += [2];
        }
        arr"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![[r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicInt | UseOfDynamicallySizedArray)
                    value_kind: Array(Content: Dynamic, Size: Dynamic)
                dynamic_param_applications: <empty>"#]],
    );
}
