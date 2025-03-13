// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::*;
use expect_test::{expect, Expect};

fn check(contents: &str, expect: &Expect) {
    let actual = match serde_json::from_str::<Circuit>(contents) {
        Ok(circuit) => build_operation_def("Test".to_string(), &circuit),
        Err(e) => format!("Error: {}", e),
    };
    expect.assert_eq(&actual);
}

fn check_circuit_group(contents: &str, expect: &Expect) {
    let actual = circuits_to_qsharp("Test".to_string(), contents.to_string());
    expect.assert_eq(&actual);
}

#[test]
fn qsharp_from_circuit() {
    check_circuit_group(
        r#"
{
  "circuits": [
    {
      "componentGrid": [
        {
          "components": [
            { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] },
            { "kind": "unitary", "gate": "S", "targets": [{ "qubit": 1 }] }
          ]
        },
        {
          "components": [
            { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] },
            { "kind": "unitary", "gate": "X", "targets": [{ "qubit": 1 }] }
          ]
        }
      ],
      "qubits": [{ "id": 0 }, { "id": 1 }]
    }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit is Ctl + Adj {
                H(q0);
                S(q1);
                Z(q0);
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
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "S", "targets": [{ "qubit": 1 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        {
          "kind": "unitary",
          "gate": "X",
          "controls": [{ "qubit": 0 }],
          "targets": [{ "qubit": 1 }]
        }
      ]
    }
  ],
  "qubits": [{ "id": 0 }, { "id": 1 }]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit is Ctl + Adj {
                H(q0);
                S(q1);
                Z(q0);
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
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "S", "targets": [{ "qubit": 1 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        {
          "kind": "unitary",
          "gate": "X",
          "isAdjoint": true,
          "targets": [{ "qubit": 1 }]
        }
      ]
    }
  ],
  "qubits": [{ "id": 0 }, { "id": 1 }]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit is Ctl + Adj {
                H(q0);
                S(q1);
                Z(q0);
                Adjoint X(q1);
            }

        "#]],
    );
}

#[test]
fn circuit_with_controlled_adjoint_gate() {
    check(
        r#"
{
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "S", "targets": [{ "qubit": 1 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        {
          "kind": "unitary",
          "gate": "X",
          "isAdjoint": true,
          "controls": [{ "qubit": 0 }],
          "targets": [{ "qubit": 1 }]
        }
      ]
    }
  ],
  "qubits": [{ "id": 0 }, { "id": 1 }]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit is Ctl + Adj {
                H(q0);
                S(q1);
                Z(q0);
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
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "Rz", "targets": [{ "qubit": 1 }], "args": ["1.2"] }
      ]
    }
  ],
  "qubits": [{ "id": 0 }, { "id": 1 }]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit is Ctl + Adj {
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
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        {
          "kind": "unitary",
          "gate": "Rz",
          "controls": [{ "qubit": 0 }],
          "targets": [{ "qubit": 1 }],
          "args": ["1.2"]
        }
      ]
    }
  ],
  "qubits": [{ "id": 0 }, { "id": 1 }]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit is Ctl + Adj {
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
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "S", "targets": [{ "qubit": 1 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        {
          "kind": "measurement",
          "gate": "Measure",
          "qubits": [{ "qubit": 0 }],
          "results": [{ "qubit": 0, "result": 0 }]
        }
      ]
    }
  ],
  "qubits": [
    { "id": 0, "numResults": 1 },
    { "id": 1 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Result {
                H(q0);
                S(q1);
                Z(q0);
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
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "H", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "S", "targets": [{ "qubit": 1 }] }
      ]
    },
    {
      "components": [
        { "kind": "unitary", "gate": "Z", "targets": [{ "qubit": 0 }] }
      ]
    },
    {
      "components": [
        {
          "kind": "measurement",
          "gate": "Measure",
          "qubits": [{ "qubit": 0 }],
          "results": [{ "qubit": 0, "result": 0 }]
        },
        {
          "kind": "measurement",
          "gate": "Measure",
          "qubits": [{ "qubit": 1 }],
          "results": [{ "qubit": 1, "result": 0 }]
        }
      ]
    },
    {
      "components": [
        {
          "kind": "measurement",
          "gate": "Measure",
          "qubits": [{ "qubit": 0 }],
          "results": [{ "qubit": 0, "result": 1 }]
        }
      ]
    }
  ],
  "qubits": [
    { "id": 0, "numResults": 2 },
    { "id": 1, "numResults": 1 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Result[] {
                H(q0);
                S(q1);
                Z(q0);
                let c0_0 = M(q0);
                let c1_0 = M(q1);
                let c0_1 = M(q0);
                return [c0_0, c0_1, c1_0];
            }

        "#]],
    );
}

#[test]
fn empty_circuit() {
    check(
        r#"
{
  "componentGrid": [],
  "qubits": []
}"#,
        &expect![[r#"
            operation Test() : Unit is Ctl + Adj {
            }

        "#]],
    );
}

#[test]
fn circuit_with_qubit_missing_num_results() {
    check(
        r#"
{
  "componentGrid": [],
  "qubits": [
    { "id": 0 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit) : Unit is Ctl + Adj {
            }

        "#]],
    );
}

#[test]
fn circuit_with_ket_gates() {
    check(
        r#"
{
  "componentGrid": [
    {
      "components": [
        { "kind": "unitary", "gate": "|0〉", "targets": [{ "qubit": 0 }] },
        { "kind": "unitary", "gate": "|1〉", "targets": [{ "qubit": 1 }] }
      ]
    }
  ],
  "qubits": [
    { "id": 0 },
    { "id": 1 }
  ]
}"#,
        &expect![[r#"
            operation Test(q0 : Qubit, q1 : Qubit) : Unit {
                Reset(q0);
                Reset(q1);
                X(q1);
            }

        "#]],
    );
}
