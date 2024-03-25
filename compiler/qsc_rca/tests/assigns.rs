// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use test_utils::{check_last_statement_compute_properties, CompilationContext};

#[test]
fn check_rca_for_classical_int_assign_to_local() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        mutable i = 0;
        set i = 1;
        i"#,
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
fn check_rca_for_dynamic_result_assign_to_local() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        mutable r = Zero;
        set r = M(q);
        r"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_dynamic_bool_assign_to_local() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Convert;
        use q = Qubit();
        mutable b = false;
        set b = ResultAsBool(M(q));
        b"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_dynamic_int_assign_to_local() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Convert;
        open Microsoft.Quantum.Measurement;
        use register = Qubit[8];
        let results = MeasureEachZ(register);
        mutable i = 0;
        set i = ResultArrayAsInt(results);
        i"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_dynamic_double_assign_to_local() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Convert;
        open Microsoft.Quantum.Measurement;
        use register = Qubit[8];
        let results = MeasureEachZ(register);
        let i = ResultArrayAsInt(results);
        mutable d = 0.0;
        set d = IntAsDouble(i);
        d"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGeneratorSet:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | UseOfDynamicDouble)
                    value_kind: Element(Dynamic)
                dynamic_param_applications: <empty>"#
        ],
    );
}
