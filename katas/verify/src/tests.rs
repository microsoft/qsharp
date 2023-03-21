// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use indoc::indoc;

use  crate::add;
use crate::verify_reference;

#[test]
fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
}

#[test]
fn verify_single_qubit_gates_kata() {
    verify_reference(
        indoc! {"
        namespace Quantum.Kata.SingleQubitGates {
            open Microsoft.Quantum.Intrinsic;
            operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
                // Apply the Pauli Y operation.
                // Y(q);
            }
        }"})
}