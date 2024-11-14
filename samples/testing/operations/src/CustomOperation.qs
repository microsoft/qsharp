// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Summary
/// CNOT based SWAP operation for testing with `TestEquivalence` operation.
///
/// # Input
/// ## q1
/// First input qubit
/// ## q2
/// Second input qubit
operation ApplySWAP(q1 : Qubit, q2 : Qubit) : Unit is Ctl + Adj {
    CNOT(q1, q2); // q1, q1 xor q2
    CNOT(q2, q1); // q2, q1 xor q2
    CNOT(q1, q2); // q2, q1
}
