// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

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
    import Std.Convert.*;

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
