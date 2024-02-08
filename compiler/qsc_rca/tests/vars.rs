// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{
    check_last_statement_compute_propeties, write_compute_properties_to_files,
    write_fir_store_to_files, CompilationContext,
};
use expect_test::expect;

#[test]
fn check_rca_for_callable_var() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"H"#);
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: ComputeProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    dynamism_sources: <empty>
                dynamic_params_properties: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_udt_var() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            open Microsoft.Quantum.Math;
            Complex"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: ComputeProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    dynamism_sources: <empty>
                dynamic_params_properties: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_static_int_var() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            let a = 42;
            a"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: ComputeProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    dynamism_sources: <empty>
                dynamic_params_properties: <empty>"#
        ],
    );
}

#[test]
fn check_rca_for_static_qubit_var() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            use q = Qubit();
            q"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_propeties(
        package_store_compute_properties,
        &expect![
            r#"
            ApplicationsTable:
                inherent: ComputeProperties:
                    runtime_features: RuntimeFeatureFlags(0x0)
                    dynamism_sources: <empty>
                dynamic_params_properties: <empty>"#
        ],
    );
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_dynamic_result_var() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            use q = Qubit();
            let r = M(q);
            r"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    write_fir_store_to_files(&compilation_context.fir_store);
    write_compute_properties_to_files(package_store_compute_properties);
    check_last_statement_compute_propeties(package_store_compute_properties, &expect![r#""#]);
}
