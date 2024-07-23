// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Sample
/// Multi File Testing Project (Refer to README for Project Overview)
///
/// # Description
/// This code builds upon the concepts explained in the README file,
/// demonstrating how to organize Q# code into multiple files for testing.
/// Here, we have separate files (`BellState.qs`, `Test_SWAP.qs`, and `OperationEquivalence.qs`)
/// containing individual operations under test.

/// The presence of a Q# manifest file (`qsharp.json`) instructs the compiler
/// to include all Q# files within the `src` directory. These operations are tested
/// by the Python wrapper script, `test_dump_operation.py`.

namespace BellState {
    /// # Summary
    /// Operation that generates all bell states for testing with `dump_operation.py`.
    ///
    /// # Input
    /// ## qs
    /// Input qubit register
    ///
    /// ## choice
    /// Bell state to construct.
    /// 0: |Φ+〉(PhiPlus)
    /// 1: |Φ-〉(PhiMinus)
    /// 2: |Ψ+〉(PsiPlus)
    /// 3: |Ψ-〉(PsiMinus)

    operation AllBellStates(qs : Qubit[], choice : Int) : Unit is Ctl + Adj {
        open Microsoft.Quantum.Convert;

        H(qs[0]);
        CNOT(qs[0], qs[1]);

        let bitmask = IntAsBoolArray(choice, 2);
        if bitmask[1] {
            X(qs[1]);
        }

        if bitmask[0] {
            Z(qs[0]);
        }

    }
}
