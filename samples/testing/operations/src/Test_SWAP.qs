// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Sample
/// Multi File Testing Project
///
/// # Description
/// This code builds upon the concepts explained in the README file,
/// demonstrating how to organize Q# code into multiple files for testing.

namespace Test_SWAP {
    /// # Summary
    /// Operation for testing with `dump_operation.py` and `TestEquivalence` operation.
    ///
    /// # Input
    /// ## qs
    /// Input qubit register
    operation ApplySWAP1(qs : Qubit[]) : Unit is Ctl + Adj {
        SWAP(qs[0], qs[1]);
    }

    // # Summary
    /// Operation for testing with `TestEquivalence` operation.
    ///
    /// # Input
    /// ## qs
    /// Input qubit register
    operation ApplySWAP2(qs : Qubit[]) : Unit is Ctl + Adj {
        CNOT(qs[0], qs[1]); // a, a xor b
        CNOT(qs[1], qs[0]); // b, a xor b
        CNOT(qs[0], qs[1]); // b, a
    }
}
