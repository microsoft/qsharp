// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_last_statement_compute_propeties, CompilationContext};
use expect_test::expect;

#[test]
fn check_rca_for_big_int_literal() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"42L"#);
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
fn check_rca_for_bool_literal() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"true"#);
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
fn check_rca_for_double_literal() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"42.0"#);
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
fn check_rca_for_pauli_literal() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"PauliX"#);
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
fn check_rca_for_result_literal() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"Zero"#);
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
