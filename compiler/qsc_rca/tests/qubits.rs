// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{check_last_statement_compute_properties, CompilationContext};
use expect_test::expect;

#[test]
fn check_rca_for_static_single_qubit_allcation() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(r#"use q = Qubit();"#);
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
