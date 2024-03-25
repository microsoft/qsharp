// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use test_utils::{check_last_statement_compute_properties, CompilationContext};

#[test]
fn check_rca_for_length_of_statically_sized_array_with_static_content() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"Length([1, 2, 3])"#);
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
fn check_rca_for_length_of_statically_sized_array_with_dynamic_content() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use qs = Qubit[2];
        Length([M(qs[0]), M(qs[1])])"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_length_of_dynamically_sized_array_with_static_content() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        let s = M(q) == Zero ? 1 | 2;
        let array = [0, size = s];
        Length(array)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | UseOfDynamicallySizedArray)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_length_of_dynamically_sized_array_with_dynamic_content() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        let r = M(q);
        let s = r == Zero ? 1 | 2;
        let array = [r, size = s];
        Length(array)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | UseOfDynamicallySizedArray)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_h() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled H([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_adjoint_h() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled Adjoint H([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_x() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled X([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_adjoint_x() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled Adjoint X([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_y() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled Y([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_adjoint_y() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled Adjoint Y([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_z() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled Z([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_controlled_adjoint_z() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use (ctl, target) = (Qubit(), Qubit());
        Controlled Adjoint Z([ctl], target)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Static)
                dynamic_param_applications: <empty>"#
        ],
    );
}
