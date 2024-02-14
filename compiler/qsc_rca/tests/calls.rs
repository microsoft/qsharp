// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub mod common;

use common::{
    check_last_statement_compute_propeties, write_compute_properties_to_files,
    write_fir_store_to_files, CompilationContext,
};
use expect_test::expect;

#[ignore = "work in progress"]
#[test]
fn check_rca_for_doubly_controlled_call_to_pauli_x() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            use ctl1 = Qubit();
            use ctl2 = Qubit();
            use target = Qubit();
            Controlled Controlled X([ctl2], ([ctl1], target));"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    write_fir_store_to_files(&compilation_context.fir_store);
    write_compute_properties_to_files(package_store_compute_properties);
    check_last_statement_compute_propeties(package_store_compute_properties, &expect![r#""#]);
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_call_to_closure_operation() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            use control = Qubit();
            let o = (theta, q) => Rxx(theta, control, q);
            use target = Qubit();
            o(3.14, target);"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    write_fir_store_to_files(&compilation_context.fir_store);
    write_compute_properties_to_files(package_store_compute_properties);
    //check_last_statement_compute_propeties(package_store_compute_properties, &expect![r#""#]);
}

#[ignore = "work in progress"]
#[test]
fn check_rca_for_call_to_operation_with_callable_argument() {
    let mut compilation_context = CompilationContext::new();
    compilation_context.update(
        r#"
            open Microsoft.Quantum.Canon;
            use register = Qubit[2];
            ApplyToEach(H, register);"#,
    );
    let package_store_compute_properties = compilation_context.get_compute_properties();
    write_fir_store_to_files(&compilation_context.fir_store);
    write_compute_properties_to_files(package_store_compute_properties);
    //check_last_statement_compute_propeties(package_store_compute_properties, &expect![r#""#]);
}
