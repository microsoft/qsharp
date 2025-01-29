// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use super::*;
use expect_test::expect;

#[test]
fn empty() {
    let c = Circuit {
        operations: vec![],
        qubits: vec![],
    };

    expect![[""]].assert_eq(&c.to_string());
}

#[test]
fn no_gates() {
    let c = Circuit {
        operations: vec![],
        qubits: vec![
            Qubit {
                id: 0,
                num_children: 0,
            },
            Qubit {
                id: 1,
                num_children: 0,
            },
        ],
    };

    expect![[r"
        q_0
        q_1
    "]]
    .assert_eq(&c.to_string());
}

#[test]
fn bell() {
    let c = Circuit {
        operations: vec![
            Operation {
                gate: "H".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![],
                targets: vec![Register::quantum(0)],
                children: vec![],
            },
            Operation {
                gate: "X".to_string(),
                display_args: None,
                is_controlled: true,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![Register::quantum(0)],
                targets: vec![Register::quantum(1)],
                children: vec![],
            },
            Operation {
                gate: "Measure".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: true,
                controls: vec![Register::quantum(0)],
                targets: vec![Register::classical(0, 0)],
                children: vec![],
            },
            Operation {
                gate: "Measure".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: true,
                controls: vec![Register::quantum(1)],
                targets: vec![Register::classical(1, 0)],
                children: vec![],
            },
        ],
        qubits: vec![
            Qubit {
                id: 0,
                num_children: 1,
            },
            Qubit {
                id: 1,
                num_children: 1,
            },
        ],
    };

    expect![[r"
        q_0    ── H ──── ● ──── M ──
                         │      ╘═══
        q_1    ───────── X ──── M ──
                                ╘═══
    "]]
    .assert_eq(&c.to_string());
}

#[test]
fn control_classical() {
    let c = Circuit {
        operations: vec![
            Operation {
                gate: "Measure".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: true,
                controls: vec![Register::quantum(0)],
                targets: vec![Register::classical(0, 0)],
                children: vec![],
            },
            Operation {
                gate: "X".to_string(),
                display_args: None,
                is_controlled: true,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![Register::classical(0, 0)],
                targets: vec![Register::quantum(2)],
                children: vec![],
            },
            Operation {
                gate: "X".to_string(),
                display_args: None,
                is_controlled: true,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![Register::quantum(0)],
                targets: vec![Register::quantum(2)],
                children: vec![],
            },
        ],
        qubits: vec![
            Qubit {
                id: 0,
                num_children: 1,
            },
            Qubit {
                id: 1,
                num_children: 0,
            },
            Qubit {
                id: 2,
                num_children: 0,
            },
        ],
    };

    expect![[r"
        q_0    ── M ─────────── ● ──
                  ╘═════ ● ═════╪═══
        q_1    ──────────┼──────┼───
        q_2    ───────── X ──── X ──
    "]]
    .assert_eq(&c.to_string());
}

#[test]
fn two_measurements() {
    let c = Circuit {
        operations: vec![
            Operation {
                gate: "Measure".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: true,
                controls: vec![Register::quantum(0)],
                targets: vec![Register::classical(0, 0)],
                children: vec![],
            },
            Operation {
                gate: "Measure".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: true,
                controls: vec![Register::quantum(0)],
                targets: vec![Register::classical(0, 1)],
                children: vec![],
            },
        ],
        qubits: vec![Qubit {
            id: 0,
            num_children: 2,
        }],
    };

    expect![[r"
        q_0    ── M ──── M ──
                  ╘══════╪═══
                         ╘═══
    "]]
    .assert_eq(&c.to_string());
}

#[test]
fn with_args() {
    let c = Circuit {
        operations: vec![Operation {
            gate: "rx".to_string(),
            display_args: Some("1.5708".to_string()),
            is_controlled: false,
            is_adjoint: false,
            is_measurement: false,
            controls: vec![],
            targets: vec![Register::quantum(0)],
            children: vec![],
        }],
        qubits: vec![Qubit {
            id: 0,
            num_children: 0,
        }],
    };

    // This looks wonky because the gate label is longer
    // than the static column width, but we can live with it.
    expect![[r"
        q_0     rx(1.5708)
    "]]
    .assert_eq(&c.to_string());
}

#[test]
fn two_targets() {
    let c = Circuit {
        operations: vec![Operation {
            gate: "rzz".to_string(),
            display_args: Some("1.0000".to_string()),
            is_controlled: false,
            is_adjoint: false,
            is_measurement: false,
            controls: vec![],
            targets: vec![Register::quantum(0), Register::quantum(2)],
            children: vec![],
        }],
        qubits: vec![
            Qubit {
                id: 0,
                num_children: 0,
            },
            Qubit {
                id: 1,
                num_children: 0,
            },
            Qubit {
                id: 2,
                num_children: 0,
            },
        ],
    };

    // This looks wonky because the gate label is longer
    // than the static column width, but we can live with it.
    expect![[r"
        q_0     rzz(1.0000)
        q_1    ───┆───
        q_2     rzz(1.0000)
    "]]
    .assert_eq(&c.to_string());
}

#[test]
fn long_gate_names() {
    let c = Circuit {
        operations: vec![
            Operation {
                gate: "rx".to_string(),
                display_args: Some("3.1416".to_string()),
                is_controlled: false,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![],
                targets: vec![Register::quantum(0)],
                children: vec![],
            },
            Operation {
                gate: "H".to_string(),
                display_args: None,
                is_controlled: false,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![],
                targets: vec![Register::quantum(1)],
                children: vec![],
            },
            Operation {
                gate: "rx".to_string(),
                display_args: Some("3.1416".to_string()),
                is_controlled: false,
                is_adjoint: false,
                is_measurement: false,
                controls: vec![],
                targets: vec![Register::quantum(2)],
                children: vec![],
            },
        ],
        qubits: vec![
            Qubit {
                id: 0,
                num_children: 0,
            },
            Qubit {
                id: 1,
                num_children: 0,
            },
            Qubit {
                id: 2,
                num_children: 0,
            },
        ],
    };

    expect![[r"
        q_0     rx(3.1416)
        q_1    ─── H ───
        q_2     rx(3.1416)
    "]]
    .assert_eq(&c.to_string());
}
