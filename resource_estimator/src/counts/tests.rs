// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#![allow(clippy::needless_raw_string_hashes)]

use std::convert::Into;

use expect_test::{expect, Expect};
use indoc::indoc;
use qsc::{
    interpret::{GenericReceiver, Interpreter},
    target::Profile,
    PackageType, SourceMap,
};

use super::LogicalCounter;

fn verify_logical_counts(source: &str, entry: Option<&str>, expect: &Expect) {
    let source_map = SourceMap::new([("test".into(), source.into())], entry.map(Into::into));
    let mut interpreter = Interpreter::new(
        true,
        source_map,
        PackageType::Exe,
        Profile::Unrestricted.into(),
    )
    .expect("compilation should succeed");
    let mut counter = LogicalCounter::default();
    let mut stdout = std::io::sink();
    let mut out = GenericReceiver::new(&mut stdout);
    interpreter
        .eval_entry_with_sim(&mut counter, &mut out)
        .expect("evaluation should succeed");
    expect.assert_debug_eq(&counter.logical_resources());
}

#[test]
fn gates_are_counted() {
    verify_logical_counts(
        indoc! {"
            namespace Test {
                open Microsoft.Quantum.Measurement;

                operation Rotate(qs: Qubit[]) : Unit {
                    for q in qs {
                        Rx(1.0, q);
                        Ry(1.0, q);
                        Rz(1.0, q);
                    }
                }

                @EntryPoint()
                operation Main() : Result[] {
                    use qs = Qubit[10];
                    within {
                        T(qs[0]);
                        CCNOT(qs[0], qs[1], qs[2]);
                    }
                    apply {
                        Rotate(qs);
                    }
                    MResetEachZ(qs)
                }
            }
        "},
        None,
        &expect![["
            LogicalResources {
                num_qubits: 10,
                t_count: 2,
                rotation_count: 30,
                rotation_depth: 5,
                ccz_count: 2,
                measurement_count: 10,
            }
        "]],
    );
}

#[test]
fn estimate_caching_works() {
    verify_logical_counts(
        indoc! {r#"
            namespace Test {
                open Microsoft.Quantum.ResourceEstimation;

                operation Rotate(qs: Qubit[]) : Unit {
                    for q in qs {
                        Rx(1.0, q);
                        Ry(1.0, q);
                        Rz(1.0, q);
                    }
                }

                @EntryPoint()
                operation Main() : Unit {
                    use qs = Qubit[10];
                    mutable count = 0;
                    for _ in 1..10 {
                        if BeginEstimateCaching("Rotate", SingleVariant()) {
                            Rotate(qs);
                            set count += 1;
                            EndEstimateCaching();
                        }
                    }
                    for _ in 1..count {
                        T(qs[0]);
                    }
                }
            }
        "#},
        None,
        &expect![["
            LogicalResources {
                num_qubits: 10,
                t_count: 1,
                rotation_count: 300,
                rotation_depth: 30,
                ccz_count: 0,
                measurement_count: 0,
            }
        "]],
    );
}

#[test]
fn estimate_repeat_works() {
    verify_logical_counts(
        indoc! {r#"
            namespace Test {
                open Microsoft.Quantum.ResourceEstimation;

                operation Rotate(qs: Qubit[]) : Unit {
                    for q in qs {
                        Rx(1.0, q);
                        Ry(1.0, q);
                        Rz(1.0, q);
                    }
                }

                @EntryPoint()
                operation Main() : Unit {
                    use qs = Qubit[10];
                    mutable count = 0;
                    within {
                        RepeatEstimates(10);
                    }
                    apply {
                        Rotate(qs);
                        set count += 1;
                    }
                    for _ in 1..count {
                        T(qs[0]);
                    }
                }
            }
        "#},
        None,
        &expect![[r#"
            LogicalResources {
                num_qubits: 10,
                t_count: 1,
                rotation_count: 300,
                rotation_depth: 30,
                ccz_count: 0,
                measurement_count: 0,
            }
        "#]],
    );
}

#[test]
fn account_for_estimates_works() {
    verify_logical_counts(
        indoc! {"
            namespace Test {
                open Microsoft.Quantum.ResourceEstimation;

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
            }
        "},
        None,
        &expect![["
            LogicalResources {
                num_qubits: 11,
                t_count: 2,
                rotation_count: 3,
                rotation_depth: 1,
                ccz_count: 5,
                measurement_count: 6,
            }
        "]],
    );
}
