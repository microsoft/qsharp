// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Microsoft.Quantum.Katas {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Intrinsic;



    operation VerifySingleQubitUnitary(unitary : (Qubit => Unit is Adj + Ctl), reference : (Qubit => Unit is Adj + Ctl))
    : Bool {
        mutable isCorrect = false;

        // Explicit scopes are used to make output from DumpMachine calls more useful.
        {
            use (control, target) = (Qubit(), Qubit());
            within {
                H(control);
            }
            apply {
                Controlled unitary([control], target);
                Adjoint Controlled reference([control], target);
            }
            set isCorrect = CheckAllZero([control, target]);
            ResetAll([control, target]);
        }

        if isCorrect {
            use target = Qubit();
            unitary(target);
            Message("Qubit state after applying the unitary operation to the |0⟩ state:");
            DumpMachine();
            Reset(target);
        } else {
            {
                use expected = Qubit();
                reference(expected);
                Message("Expected state after applying operation:");
                DumpMachine();
                Reset(expected);
            }

            {
                use actual = Qubit();
                unitary(actual);
                Message("Actual state after applying operation:");
                DumpMachine();
                Reset(actual);
            }
        }

        isCorrect
    }
}