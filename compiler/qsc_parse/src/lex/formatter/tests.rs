// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;

fn check(input: &str, expect: &Expect) {
    let actual = super::format(input);
    expect.assert_debug_eq(&actual);
}

#[test]
fn test_formatting() {
    check(
        "operation   Foo   ()",
        &expect![[r#"
    [
        Edit {
            span: Span {
                lo: 9,
                hi: 12,
            },
            new_text: " ",
        },
        Edit {
            span: Span {
                lo: 15,
                hi: 18,
            },
            new_text: "",
        },
    ]
"#]],
    );
}

#[test]
fn test_indentation() {
    check(
        r#"
    /// First
/// Second
    /// Third
        namespace MyQuantumProgram {
        open Microsoft.Quantum.Diagnostics;

        @EntryPoint()
        operation Main() : Int {
            let x = 3;
            let y = 4;

            // Comment
            return 5;
        }
            }
"#,
        &expect![[r#"
            [
                Edit {
                    span: Span {
                        lo: 25,
                        hi: 30,
                    },
                    new_text: "\n",
                },
                Edit {
                    span: Span {
                        lo: 39,
                        hi: 48,
                    },
                    new_text: "\n",
                },
                Edit {
                    span: Span {
                        lo: 76,
                        hi: 85,
                    },
                    new_text: "\n    ",
                },
                Edit {
                    span: Span {
                        lo: 120,
                        hi: 130,
                    },
                    new_text: "\n\n    ",
                },
                Edit {
                    span: Span {
                        lo: 143,
                        hi: 152,
                    },
                    new_text: "\n    ",
                },
                Edit {
                    span: Span {
                        lo: 176,
                        hi: 189,
                    },
                    new_text: "\n        ",
                },
                Edit {
                    span: Span {
                        lo: 199,
                        hi: 212,
                    },
                    new_text: "\n        ",
                },
                Edit {
                    span: Span {
                        lo: 222,
                        hi: 236,
                    },
                    new_text: "\n\n        ",
                },
                Edit {
                    span: Span {
                        lo: 246,
                        hi: 259,
                    },
                    new_text: "\n        ",
                },
                Edit {
                    span: Span {
                        lo: 268,
                        hi: 277,
                    },
                    new_text: "\n    ",
                },
                Edit {
                    span: Span {
                        lo: 278,
                        hi: 291,
                    },
                    new_text: "\n",
                },
            ]
        "#]],
    );
}

#[test]
fn test_braces() {
    let code = indoc! {r#"
        operation Foo() : Unit {}
        operation Bar() : Unit {
            operation Baz() : Unit {}
        }
        "#};
    let edits = super::format(code);
    expect![[r#"
            [
                Edit {
                    span: Span {
                        lo: 24,
                        hi: 24,
                    },
                    new_text: "\n",
                },
                Edit {
                    span: Span {
                        lo: 79,
                        hi: 79,
                    },
                    new_text: "\n    ",
                },
            ]
        "#]]
    .assert_debug_eq(&edits);
}

#[test]
fn test_formatting_2() {
    let code = indoc! {r#"
        /// # Sample
        /// Joint Measurement
        ///
        /// # Description
        /// Joint measurements, also known as Pauli measurements, are a generalization
        /// of 2-outcome measurements to multiple qubits and other bases.
        namespace Sample {
            open Microsoft.Quantum.Diagnostics;

            @EntryPoint()
            operation Main() : (Result, Result[]) {
                // Prepare an entangled state.
                use qs = Qubit[2];  // |00〉
                H(qs[0]);           // 1/sqrt(2)(|00〉 + |10〉)
                CNOT(qs[0], qs[1]); // 1/sqrt(2)(|00〉 + |11〉)

                // Show the quantum state before performing the joint measurement.
                DumpMachine();

                // The below code uses a joint measurement as a way to check the parity
                // of the first two qubits. In this case, the parity measurement result
                // will always be `Zero`.
                // Notice how the state was not collapsed by the joint measurement.
                let parityResult = Measure([PauliZ, PauliZ], qs[...1]);
                DumpMachine();

                // However, if we perform a measurement just on the first qubit, we can
                // see how the state collapses.
                let firstQubitResult = M(qs[0]);
                DumpMachine();

                // Measuring the last qubit does not change the quantum state
                // since the state of the second qubit collapsed when the first qubit
                // was measured because they were entangled.
                let secondQubitResult = M(qs[1]);
                DumpMachine();

                ResetAll(qs);
                return (parityResult, [firstQubitResult, secondQubitResult]);
            }
        }
        "#};
    let edits = super::format(code);
    expect![[r#"
            []
        "#]]
    .assert_debug_eq(&edits);
}
