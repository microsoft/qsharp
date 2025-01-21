// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::*;
use expect_test::expect;

#[test]
fn exceed_max_operations() {
    let mut builder = Builder::new(Config {
        base_profile: false,
        max_operations: 2,
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
        base_profile: true, // deferred measurements
        max_operations: 2,
    });

    let q = builder.qubit_allocate();

    builder.x(q);
    builder.m(q);
    builder.x(q);

    builder.qubit_release(q);

    let circuit = builder.finish();

    // The current behavior is to silently truncate the circuit
    // if it exceeds the maximum allowed number of operations.
    // The measurement will be dropped.
    expect![[r#"
        q_0    ── X ──

        q_1    ── X ──
    "#]]
    .assert_eq(&circuit.to_string());
}
