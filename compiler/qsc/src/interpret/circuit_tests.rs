// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::unicode_not_nfc)]

use super::{CircuitEntryPoint, Debugger, Interpreter};
use crate::target::Profile;
use expect_test::expect;
use miette::Diagnostic;
use qsc_data_structures::language_features::LanguageFeatures;
use qsc_eval::output::GenericReceiver;
use qsc_frontend::compile::SourceMap;
use qsc_passes::PackageType;

fn interpreter(code: &str, profile: Profile) -> Interpreter {
    let sources = SourceMap::new([("test.qs".into(), code.into())], None);
    let (std_id, store) = crate::compile::package_store_with_stdlib(profile.into());
    Interpreter::new(
        sources,
        PackageType::Exe,
        profile.into(),
        LanguageFeatures::default(),
        store,
        &[(std_id, None)],
    )
    .expect("interpreter creation should succeed")
}

#[test]
fn empty() {
    let mut interpreter = interpreter(
        r#"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    Message("hi");
                }
            }
        "#,
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![].assert_eq(&circ.to_string());
}

#[test]
fn one_gate() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    H(q);
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn measure_same_qubit_twice() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                @EntryPoint()
                operation Main() : Result[] {
                    use q = Qubit();
                    H(q);
                    let r1 = M(q);
                    let r2 = M(q);
                    [r1, r2]
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![["
        q_0    ── H ──── M ──── M ──
                         ╘══════╪═══
                                ╘═══
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn toffoli() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit[3];
                    CCNOT(q[0], q[1], q[2]);
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── ● ──
        q_1    ── ● ──
        q_2    ── X ──
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn rotation_gate() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    Rx(Microsoft.Quantum.Math.PI()/2.0, q);
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    // The wire isn't visible here since the gate label is longer
    // than the static column width, but we can live with it.
    expect![[r"
        q_0     rx(1.5708)
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn classical_for_loop() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                @EntryPoint()
                operation Main() : Unit {
                    use q = Qubit();
                    for i in 0..5 {
                        X(q);
                    }
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── X ──── X ──── X ──── X ──── X ──── X ──
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn m_base_profile() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                import Std.Measurement.*;
                @EntryPoint()
                operation Main() : Result[] {
                    use q = Qubit();
                    H(q);
                    [M(q)]
                }
            }
        ",
        Profile::Base,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── Z ────────────────
        q_1    ── H ──── ● ──── H ──── M ──
                                       ╘═══
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn m_unrestricted_profile() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                import Std.Measurement.*;
                @EntryPoint()
                operation Main() : Result[] {
                    use q = Qubit();
                    H(q);
                    [M(q)]
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── M ──
                         ╘═══
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn mresetz_unrestricted_profile() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                import Std.Measurement.*;
                @EntryPoint()
                operation Main() : Result[] {
                    use q = Qubit();
                    H(q);
                    [MResetZ(q)]
                }
            }
        ",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── M ─── |0〉 ─
                         ╘══════════
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn mresetz_base_profile() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                import Std.Measurement.*;
                @EntryPoint()
                operation Main() : Result[] {
                    use q = Qubit();
                    H(q);
                    [MResetZ(q)]
                }
            }
        ",
        Profile::Base,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── M ──
                         ╘═══
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn unrestricted_profile_result_comparison() {
    let mut interpreter = interpreter(
        r"
            namespace Test {
                import Std.Measurement.*;
                @EntryPoint()
                operation Main() : Result[] {
                    use q1 = Qubit();
                    use q2 = Qubit();
                    H(q1);
                    H(q2);
                    let r1 = M(q1);
                    let r2 = M(q2);
                    if (r1 == r2) {
                        X(q1);
                    }
                    ResetAll([q1, q2]);
                    [r1, r2]
                }
            }
        ",
        Profile::Unrestricted,
    );

    interpreter.set_quantum_seed(Some(2));

    let circuit_err = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect_err("circuit should return error")
        .pop()
        .expect("error should exist");

    expect!["Qsc.Eval.ResultComparisonUnsupported"].assert_eq(
        &circuit_err
            .code()
            .expect("error code should exist")
            .to_string(),
    );

    let circuit = interpreter.get_circuit();
    expect![""].assert_eq(&circuit.to_string());

    let mut out = std::io::sink();
    let mut r = GenericReceiver::new(&mut out);

    // Result comparisons are okay when tracing
    // circuit with the simulator.
    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, true)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── M ──── X ─── |0〉 ─
                         ╘═════════════════
        q_1    ── H ──── M ─── |0〉 ────────
                         ╘═════════════════
    "]]
    .assert_eq(&circ.to_string());

    // Result comparisons are also okay if calling
    // get_circuit() after incremental evaluation,
    // because we're using the current simulator
    // state.
    interpreter
        .eval_fragments(&mut r, "Test.Main();")
        .expect("eval should succeed");

    let circuit = interpreter.get_circuit();
    expect![[r"
        q_0    ── H ──── M ──── X ─── |0〉 ─
                         ╘═════════════════
        q_1    ── H ──── M ─── |0〉 ────────
                         ╘═════════════════
    "]]
    .assert_eq(&circuit.to_string());
}

#[test]
fn custom_intrinsic() {
    let mut interpreter = interpreter(
        r"
    namespace Test {
        operation foo(q: Qubit): Unit {
            body intrinsic;
        }

        @EntryPoint()
        operation Main() : Unit {
            use q = Qubit();
            foo(q);
        }
    }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ─ foo ─
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn custom_intrinsic_classical_arg() {
    let mut interpreter = interpreter(
        r"
    namespace Test {
        operation foo(n: Int): Unit {
            body intrinsic;
        }

        @EntryPoint()
        operation Main() : Unit {
            use q = Qubit();
            X(q);
            foo(4);
        }
    }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    // A custom intrinsic that doesn't take qubits just doesn't
    // show up on the circuit.
    expect![[r"
        q_0    ── X ──
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn custom_intrinsic_one_classical_arg() {
    let mut interpreter = interpreter(
        r"
    namespace Test {
        operation foo(n: Int, q: Qubit): Unit {
            body intrinsic;
        }

        @EntryPoint()
        operation Main() : Unit {
            use q = Qubit();
            X(q);
            foo(4, q);
        }
    }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    // A custom intrinsic that doesn't take qubits just doesn't
    // show up on the circuit.
    expect![[r"
        q_0    ── X ── foo(4)
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn custom_intrinsic_mixed_args() {
    let mut interpreter = interpreter(
        r"
    namespace Test {
        import Std.ResourceEstimation.*;

        @EntryPoint()
        operation Main() : Unit {
            use qs = Qubit[10];
            AccountForEstimates(
                [
                    AuxQubitCount(1),
                    TCount(2),
                    RotationCount(3),
                    RotationDepth(4),
                    CczCount(5),
                    MeasurementCount(6),
                ],
                PSSPCLayout(),
                qs);
        }
    }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::EntryPoint, false)
        .expect("circuit generation should succeed");

    // This is one gate that spans ten target wires, even though the
    // text visualization doesn't convey that clearly.
    expect![[r"
        q_0     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_1     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_2     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_3     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_4     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_5     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_6     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_7     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_8     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
        q_9     AccountForEstimatesInternal([(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)], 1)
    "]]
    .assert_eq(&circ.to_string());

    assert_eq!(circ.operations.len(), 1);
}

#[test]
fn operation_with_qubits() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            operation Test(q1: Qubit, q2: Qubit) : Result[] {
                H(q1);
                CNOT(q1, q2);
                [M(q1), M(q2)]
            }

        }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::Operation("Test.Test".into()), false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── ● ──── M ──
                         │      ╘═══
        q_1    ───────── X ──── M ──
                                ╘═══
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn operation_with_qubits_base_profile() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            operation Test(q1: Qubit, q2: Qubit) : Result[] {
                H(q1);
                CNOT(q1, q2);
                [M(q1), M(q2)]
            }

        }",
        Profile::Base,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::Operation("Test.Test".into()), false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── ● ──── Z ──────────────────────────────
        q_1    ───────── X ─────┼──────────── Z ────────────────
        q_2    ── H ─────────── ● ──── H ─────┼───── M ─────────
                                              │      ╘══════════
        q_3    ── H ───────────────────────── ● ──── H ──── M ──
                                                            ╘═══
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn operation_with_qubit_arrays() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            import Std.Measurement.*;
            operation Test(q1: Qubit[], q2: Qubit[][], q3: Qubit[][][], q: Qubit) : Result[] {
                for q in q1 {
                    H(q);
                }
                for qs in q2 {
                    for q in qs {
                        X(q);
                    }
                }
                for qss in q3 {
                    for qs in qss {
                        for q in qs {
                            Y(q);
                        }
                    }
                }
                X(q);
                MeasureEachZ(q1)
            }
        }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::Operation("Test.Test".into()), false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──── M ──
                         ╘═══
        q_1    ── H ──── M ──
                         ╘═══
        q_2    ── X ─────────
        q_3    ── X ─────────
        q_4    ── X ─────────
        q_5    ── X ─────────
        q_6    ── Y ─────────
        q_7    ── Y ─────────
        q_8    ── Y ─────────
        q_9    ── Y ─────────
        q_10   ── Y ─────────
        q_11   ── Y ─────────
        q_12   ── Y ─────────
        q_13   ── Y ─────────
        q_14   ── X ─────────
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn adjoint_operation() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            operation Foo (q : Qubit) : Unit
                is Adj + Ctl {

                body (...) {
                    X(q);
                }

                adjoint (...) {
                    Y(q);
                }

                controlled (cs, ...) {
                }
            }

        }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(
            CircuitEntryPoint::Operation("Adjoint Test.Foo".into()),
            false,
        )
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── Y ──
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn lambda() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }
        }",
        Profile::Unrestricted,
    );

    let circ = interpreter
        .circuit(CircuitEntryPoint::Operation("q => H(q)".into()), false)
        .expect("circuit generation should succeed");

    expect![[r"
        q_0    ── H ──
    "]]
    .assert_eq(&circ.to_string());
}

#[test]
fn controlled_operation() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            operation SWAP (q1 : Qubit, q2 : Qubit) : Unit
                is Adj + Ctl {

                body (...) {
                    CNOT(q1, q2);
                    CNOT(q2, q1);
                    CNOT(q1, q2);
                }

                adjoint (...) {
                    SWAP(q1, q2);
                }

                controlled (cs, ...) {
                    CNOT(q1, q2);
                    Controlled CNOT(cs, (q2, q1));
                    CNOT(q1, q2);
                }
            }

        }",
        Profile::Unrestricted,
    );

    let circ_err = interpreter
        .circuit(
            CircuitEntryPoint::Operation("Controlled Test.SWAP".into()),
            false,
        )
        .expect_err("circuit generation should fail");

    // Controlled operations are not supported at the moment.
    // We don't generate an accurate call signature with the tuple arguments.
    expect![[r"
        [
            Circuit(
                ControlledUnsupported,
            ),
        ]
    "]]
    .assert_debug_eq(&circ_err);
}

#[test]
#[allow(clippy::too_many_lines)]
fn internal_operation() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            internal operation Test(q1: Qubit, q2: Qubit) : Result[] {
                H(q1);
                CNOT(q1, q2);
                [M(q1), M(q2)]
            }
        }",
        Profile::Unrestricted,
    );

    let circ_err = interpreter
        .circuit(CircuitEntryPoint::Operation("Test.Test".into()), false)
        .expect("circuit generation should not fail");

    expect![[r#"
        Circuit {
            operations: [
                Operation {
                    gate: "H",
                    display_args: None,
                    is_controlled: false,
                    is_adjoint: false,
                    is_measurement: false,
                    controls: [],
                    targets: [
                        Register {
                            q_id: 0,
                            type: 0,
                            c_id: None,
                        },
                    ],
                    children: [],
                },
                Operation {
                    gate: "X",
                    display_args: None,
                    is_controlled: true,
                    is_adjoint: false,
                    is_measurement: false,
                    controls: [
                        Register {
                            q_id: 0,
                            type: 0,
                            c_id: None,
                        },
                    ],
                    targets: [
                        Register {
                            q_id: 1,
                            type: 0,
                            c_id: None,
                        },
                    ],
                    children: [],
                },
                Operation {
                    gate: "Measure",
                    display_args: None,
                    is_controlled: false,
                    is_adjoint: false,
                    is_measurement: true,
                    controls: [
                        Register {
                            q_id: 0,
                            type: 0,
                            c_id: None,
                        },
                    ],
                    targets: [
                        Register {
                            q_id: 0,
                            type: 1,
                            c_id: Some(
                                0,
                            ),
                        },
                    ],
                    children: [],
                },
                Operation {
                    gate: "Measure",
                    display_args: None,
                    is_controlled: false,
                    is_adjoint: false,
                    is_measurement: true,
                    controls: [
                        Register {
                            q_id: 1,
                            type: 0,
                            c_id: None,
                        },
                    ],
                    targets: [
                        Register {
                            q_id: 1,
                            type: 1,
                            c_id: Some(
                                0,
                            ),
                        },
                    ],
                    children: [],
                },
            ],
            qubits: [
                Qubit {
                    id: 0,
                    num_children: 1,
                },
                Qubit {
                    id: 1,
                    num_children: 1,
                },
            ],
        }
    "#]]
    .assert_debug_eq(&circ_err);
}

#[test]
fn operation_with_non_qubit_args() {
    let mut interpreter = interpreter(
        r"
        namespace Test {
            @EntryPoint()
            operation Main() : Result[] { [] }

            operation Test(q1: Qubit, q2: Qubit, i: Int) : Unit {
            }

        }",
        Profile::Unrestricted,
    );

    let circ_err = interpreter
        .circuit(CircuitEntryPoint::Operation("Test.Test".into()), false)
        .expect_err("circuit generation should fail");

    expect![[r"
        [
            Circuit(
                NoQubitParameters,
            ),
        ]
    "]]
    .assert_debug_eq(&circ_err);
}

/// Tests that invoke circuit generation throught the debugger.
mod debugger_stepping {
    use super::Debugger;
    use crate::target::Profile;
    use expect_test::expect;
    use qsc_data_structures::language_features::LanguageFeatures;
    use qsc_data_structures::line_column::Encoding;
    use qsc_eval::{output::GenericReceiver, StepAction, StepResult};
    use qsc_frontend::compile::SourceMap;
    use std::fmt::Write;

    /// Steps through the code in the debugger and collects the
    /// circuit representation at each step.
    fn generate_circuit_steps(code: &str, profile: Profile) -> String {
        let sources = SourceMap::new([("test.qs".into(), code.into())], None);
        let (std_id, store) = crate::compile::package_store_with_stdlib(profile.into());
        let mut debugger = Debugger::new(
            sources,
            profile.into(),
            Encoding::Utf8,
            LanguageFeatures::default(),
            store,
            &[(std_id, None)],
        )
        .expect("debugger creation should succeed");

        debugger.interpreter.set_quantum_seed(Some(2));

        let mut out = std::io::sink();
        let mut r = GenericReceiver::new(&mut out);

        let mut circs = String::new();
        let mut result = debugger
            .eval_step(&mut r, &[], StepAction::In)
            .expect("step should succeed");

        write!(&mut circs, "step:\n{}", debugger.circuit()).expect("write should succeed");
        while !matches!(result, StepResult::Return(_)) {
            result = debugger
                .eval_step(&mut r, &[], StepAction::Next)
                .expect("step should succeed");

            write!(&mut circs, "step:\n{}", debugger.circuit()).expect("write should succeed");
        }
        circs
    }

    #[test]
    fn base_profile() {
        let circs = generate_circuit_steps(
            r"
                namespace Test {
                    import Std.Measurement.*;
                    @EntryPoint()
                    operation Main() : Result[] {
                        use q = Qubit();
                        H(q);
                        let r = M(q);
                        Reset(q);
                        [r]
                    }
                }
            ",
            Profile::Base,
        );

        // Surprising but expected: Reset gates would *not* normally
        // be generated in Base Profile, but they are here, since
        // when running in tandem with the simulator, the resulting
        // circuit is intended to match the calls into the simulator.
        //
        // Note the circuit still looks different than what would be
        // generated in Unrestricted Profile for the same code,
        // due to conditional compilation in the standard library.
        expect![["
            step:
            step:
            q_0
            step:
            q_0    ── H ──
            step:
            q_0    ── H ──── Z ───────────────────────
            q_1    ── H ──── ● ──── H ──── M ─── |0〉 ─
                                           ╘══════════
            step:
            q_0    ── H ──── Z ─── |0〉 ───────────────
            q_1    ── H ──── ● ──── H ──── M ─── |0〉 ─
                                           ╘══════════
            step:
            q_0    ── H ──── Z ─── |0〉 ───────────────
            q_1    ── H ──── ● ──── H ──── M ─── |0〉 ─
                                           ╘══════════
        "]]
        .assert_eq(&circs);
    }

    #[test]
    fn unrestricted_profile() {
        let circs = generate_circuit_steps(
            r"
                namespace Test {
                    import Std.Measurement.*;
                    @EntryPoint()
                    operation Main() : Result[] {
                        use q = Qubit();
                        H(q);
                        let r = M(q);
                        Reset(q);
                        [r]
                    }
                }
            ",
            Profile::Unrestricted,
        );

        expect![[r"
            step:
            step:
            q_0
            step:
            q_0    ── H ──
            step:
            q_0    ── H ──── M ──
                             ╘═══
            step:
            q_0    ── H ──── M ─── |0〉 ─
                             ╘══════════
            step:
            q_0    ── H ──── M ─── |0〉 ─
                             ╘══════════
        "]]
        .assert_eq(&circs);
    }

    #[test]
    fn unrestricted_profile_result_comparison() {
        let circs = generate_circuit_steps(
            r"
                namespace Test {
                    import Std.Measurement.*;
                    @EntryPoint()
                    operation Main() : Result[] {
                        use q = Qubit();
                        H(q);
                        let r = M(q);
                        if (r == One) {
                            X(q);
                        }
                        [r]
                    }
                }
            ",
            Profile::Unrestricted,
        );

        // We set the random seed in the test to account for
        // the nondeterministic output. Since the debugger is running
        // the real simulator, the circuit is going to vary from run to run
        // depending on measurement outcomes.
        expect![[r"
            step:
            step:
            q_0
            step:
            q_0    ── H ──
            step:
            q_0    ── H ──── M ──
                             ╘═══
            step:
            q_0    ── H ──── M ──
                             ╘═══
            step:
            q_0    ── H ──── M ──── X ──
                             ╘══════════
            step:
            q_0    ── H ──── M ──── X ──
                             ╘══════════
        "]]
        .assert_eq(&circs);
    }
}
