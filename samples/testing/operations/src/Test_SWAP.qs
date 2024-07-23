// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Sample
/// Multi File Testing Project
///
/// # Description
/// Organizing code into multiple Q# source files is an important part of
/// writing readable and maintainable code. In this project, we have `BellState.qs`, `Test_SWAP.qs`,
/// and `OperationEquivalence.qs`, which contain the operation to be tested.
/// The presence of a Q# manifest file (`qsharp.json`) tells the compiler
/// to include all Q# files under `src/`.
/// These will be tested by Python wrapper, `test_dump_operation.py`

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
