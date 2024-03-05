// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expect_test::{expect, Expect};
use indoc::indoc;

fn check(input: &str, expect: &Expect) {
    let actual = super::format_str(input);
    expect.assert_eq(&actual);
}

#[test]
fn remove_trailing_spaces() {
    let extra_spaces = "    ";
    let input = format!(
        "/// Doc Comment with trailing spaces{extra_spaces}
operation Foo() : Unit {{
    // Comment with trailing spaces{extra_spaces}
    let x = 3;   // In-line comment with trailing spaces{extra_spaces}
    let y = 4;{extra_spaces}
}}
"
    );

    check(
        input.as_str(),
        &expect![[r#"
            /// Doc Comment with trailing spaces
            operation Foo() : Unit {
                // Comment with trailing spaces
                let x = 3;   // In-line comment with trailing spaces
                let y = 4;
            }
        "#]],
    );
}

#[test]
fn correct_indentation() {
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
        "#]],
    );
}

#[test]
fn correct_empty_delimiters() {
    check(
        indoc! {r#"
        operation Foo() : Unit {
        }
        operation Bar() : Unit {
            operation Baz() : Unit {   }
            let x = {

            };
            let y : Int[] = [ ];
            let z = (

             );
        }
        "#},
        &expect![[r#"
            operation Foo() : Unit {}
            operation Bar() : Unit {
                operation Baz() : Unit {}
                let x = {};
                let y : Int[] = [];
                let z = ();
            }
        "#]],
    );
}

#[test]
fn test_sample() {
    let input = indoc! {r#"
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
    assert!(super::calculate_format_edits(input).is_empty());
}
