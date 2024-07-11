// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

/// # Sample
/// Multi File Testing Project
///
/// # Description
/// Organizing code into multiple Q# source files is an important part of
/// writing readable and maintainable code. In this project, we have `SWAP.qs`,
/// and `BellState.qs`, which contain the operation to be tested.
/// The presence of a Q# manifest file (`qsharp.json`) tells the compiler
/// to include all Q# files under `src/`.
/// These will be tested by Python wrapper, `test_dump_operation.py`

namespace BellState {
    operation AllBellStates(qs : Qubit[], choice: Int) : Unit is Ctl + Adj {
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
