// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::*;
use expect_test::{expect, Expect};

fn check(contents: &str, expect: &Expect) {
    let actual = qviz_to_qsharp("Test".to_string(), contents.to_string());
    expect.assert_eq(&actual);
}

#[test]
fn qsharp_from_circuit() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "S", "targets": [{ "qId": 1, "type": 0 }] },
    { "gate": "X", "targets": [{ "qId": 1, "type": 0 }] }
  ],
  "qubits": [
    { "id": 0, "numChildren": 0 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                H(q0);
                Z(q0);
                S(q1);
                X(q1);
            }
        "#]],
    );
}

#[test]
fn circuit_with_controlled_gate() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "S", "targets": [{ "qId": 1, "type": 0 }] },
    {
      "gate": "X",
      "isControlled": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 1, "type": 0 }]
    }
  ],
  "qubits": [
    { "id": 0, "numChildren": 0 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                H(q0);
                Z(q0);
                S(q1);
                Controlled X([q0], q1);
            }
        "#]],
    );
}

#[test]
fn circuit_with_adjoint_gate() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "S", "targets": [{ "qId": 1, "type": 0 }] },
    {
      "gate": "X",
      "isAdjoint": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 1, "type": 0 }]
    }
  ],
  "qubits": [
    { "id": 0, "numChildren": 0 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                H(q0);
                Z(q0);
                S(q1);
                Adjoint X([q0], q1);
            }
        "#]],
    );
}

#[test]
fn circuit_with_controlled_adjoint_gate() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "S", "targets": [{ "qId": 1, "type": 0 }] },
    {
      "gate": "X",
      "isControlled": true,
      "isAdjoint": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 1, "type": 0 }]
    }
  ],
  "qubits": [
    { "id": 0, "numChildren": 0 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                H(q0);
                Z(q0);
                S(q1);
                Controlled Adjoint X([q0], q1);
            }
        "#]],
    );
}

#[test]
fn circuit_with_rz_gate() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Rz", "targets": [{ "qId": 1, "type": 0 }], "displayArgs": "1.2" }
  ],
  "qubits": [
    { "id": 0, "numChildren": 0 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                H(q0);
                Z(q0);
                Rz(1.2, q1);
            }
        "#]],
    );
}

#[test]
fn circuit_with_controlled_gate_multiple_args() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    {
      "gate": "Rz",
      "isControlled": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 1, "type": 0 }],
      "displayArgs": "1.2"
    }
  ],
  "qubits": [
    { "id": 0, "numChildren": 0 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                H(q0);
                Z(q0);
                Controlled Rz([q0], (1.2, q1));
            }
        "#]],
    );
}

#[test]
fn circuit_with_measurement_gate() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "S", "targets": [{ "qId": 1, "type": 0 }] },
    {
      "gate": "Measure",
      "isMeasurement": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 0, "type": 1, "cId": 0 }]
    }
  ],
  "qubits": [
    { "id": 0, "numChildren": 1 },
    { "id": 1, "numChildren": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Result {
                H(q0);
                Z(q0);
                S(q1);
                let c0_0 = M(q0);
                return c0_0;
            }
        "#]],
    );
}

#[test]
fn circuit_with_multiple_measurement_gates() {
    check(
        r#"
{
  "operations": [
    { "gate": "H", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "Z", "targets": [{ "qId": 0, "type": 0 }] },
    { "gate": "S", "targets": [{ "qId": 1, "type": 0 }] },
    {
      "gate": "Measure",
      "isMeasurement": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 0, "type": 1, "cId": 0 }]
    },
    {
      "gate": "Measure",
      "isMeasurement": true,
      "controls": [{ "qId": 1, "type": 0 }],
      "targets": [{ "qId": 1, "type": 1, "cId": 0 }]
    },
    {
      "gate": "Measure",
      "isMeasurement": true,
      "controls": [{ "qId": 0, "type": 0 }],
      "targets": [{ "qId": 0, "type": 1, "cId": 1 }]
    }
  ],
  "qubits": [
    { "id": 0, "numChildren": 2 },
    { "id": 1, "numChildren": 1 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Result[] {
                H(q0);
                Z(q0);
                S(q1);
                let c0_0 = M(q0);
                let c1_0 = M(q1);
                let c0_1 = M(q0);
                return [c0_0, c0_1, c1_0];
            }
        "#]],
    );
}
