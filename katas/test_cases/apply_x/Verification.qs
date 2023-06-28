// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Kata {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;

    operation VerifyExercise() : Bool {
        VerifySingleQubitOperation(ApplyX, Kata.Solution.ApplyX)
    }

    operation VerifySingleQubitOperation(
        op : (Qubit => Unit is Adj + Ctl),
        reference : (Qubit => Unit is Adj + Ctl))
    : Bool {
        use (control, target) = (Qubit(), Qubit());
        within {
            H(control);
        }
        apply {
            Controlled op([control], target);
            Adjoint Controlled reference([control], target);
        }
        let isCorrect = CheckAllZero([control, target]);
        ResetAll([control, target]);

        isCorrect
    }
}