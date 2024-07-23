// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Sample
/// Multi File Testing Project
///
/// # Description
/// This code builds upon the concepts explained in the README file,
/// demonstrating how to organize Q# code into multiple files for testing.

namespace Test_SWAP {
    // # Summary
    /// CNOT based operation for testing with `TestEquivalence` operation.
    ///
    /// # Input
    /// ## q1
    /// First input qubit
    /// ## q2
    /// Second input qubit
    operation ApplySWAP(q1: Qubit, q2: Qubit) : Unit is Ctl + Adj {
        CNOT(q1, q2); // q1, q1 xor q2
        CNOT(q2, q1); // q2, q1 xor q2
        CNOT(q1, q2); // q2, q1
    }
}
