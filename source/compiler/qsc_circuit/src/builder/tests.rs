// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::circuit::GenerationMethod;

use super::*;
use expect_test::expect;

#[test]
fn exceed_max_operations() {
    let mut builder = Builder::new(Config {
        max_operations: 2,
        loop_detection: false,
        generation_method: GenerationMethod::ClassicalEval,
    });

    let q = builder.qubit_allocate();

    builder.x(q);
    builder.x(q);
    builder.x(q);

    builder.qubit_release(q);

    let circuit = builder.finish();

    // The current behavior is to silently truncate the circuit
    // if it exceeds the maximum allowed number of operations.
    expect![[r#"
        q_0    ── X ──── X ──
    "#]]
    .assert_eq(&circuit.to_string());
}

#[test]
fn exceed_max_operations_deferred_measurements() {
    let mut builder = Builder::new(Config {
        max_operations: 2,
        loop_detection: false,
        generation_method: GenerationMethod::ClassicalEval,
    });

    let q = builder.qubit_allocate();

    builder.x(q);
    builder.m(q);
    builder.x(q);

    builder.qubit_release(q);

    let circuit = builder.finish();

    // The current behavior is to silently truncate the circuit
    // if it exceeds the maximum allowed number of operations.
    // The second X will be dropped.
    expect![[r#"
        q_0    ── X ──── M ──
                         ╘═══
    "#]]
    .assert_eq(&circuit.to_string());
}
