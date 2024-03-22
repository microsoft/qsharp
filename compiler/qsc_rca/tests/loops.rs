// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

pub mod test_utils;

use expect_test::expect;
use test_utils::{check_last_statement_compute_properties, CompilationContext};

#[test]
fn check_rca_for_classical_for_loop() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        let loop = for i in 0..5 { };
        loop"#,
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
fn check_rca_for_dynamic_for_loop() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
        use q = Qubit();
        let end = M(q) == Zero ? 5 | 10;
        let loop = for i in 0..end { };
        loop"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    check_last_statement_compute_properties(
        package_store_compute_properties,
        &expect![
            r#"
        ApplicationsGeneratorSet:
            inherent: Quantum: QuantumProperties:
                runtime_features: RuntimeFeatureFlags(UseOfDynamicBool | UseOfDynamicInt | UseOfDynamicRange | LoopWithDynamicCondition)
                value_kind: Element(Static)
            dynamic_param_applications: <empty>"#
        ],
    );
}
