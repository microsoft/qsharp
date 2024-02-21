// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_last_statement_compute_propeties, CompilationContext};
use expect_test::expect;

#[test]
fn check_rca_for_udt_constructor_with_classical_values() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        Complex(0.0, 0.0)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGenerator:
                inherent: Classical
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_udt_constructor_with_a_dynamic_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        use q = Qubit();
        let r = M(q) == Zero ? 0.0 | 1.0;
        Complex(r, 0.0)"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGenerator:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicUdt | UdtConstructorUsesDynamicArg)
                    value_kind: Dynamic
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_udt_field_update_with_classical_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        mutable c = Complex(0.0, 0.0);
        set c w/= Real <- 1.0;
        c"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGenerator:
                inherent: Classical
                dynamic_param_applications: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_udt_field_update_with_dynamic_value() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        open Microsoft.Quantum.Math;
        mutable c = Complex(0.0, 0.0);
        use q = Qubit();
        let i = M(q) == Zero ? 0.0 | 1.0;
        set c w/= Imag <- i;
        c"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsGenerator:
                inherent: Quantum: QuantumProperties:
                    runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicDouble | UseOfDynamicUdt)
                    value_kind: Dynamic
                dynamic_param_applications: <empty>"#
        ],
    );
}
