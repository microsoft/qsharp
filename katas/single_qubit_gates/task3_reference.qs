// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {
    open Microsoft.Quantum.Diagnostics;

    operation SignFlipOnZeroReference(q : Qubit) : Unit is Adj + Ctl {
        body ... {
            X(q);
            Z(q);
            X(q);
        }
        adjoint self;
    }

    operation VerifyTask3() : Bool {
        let task = SignFlipOnZero;
        let taskRef = SignFlipOnZeroReference;

        use (aux, target) = (Qubit(), Qubit());
        H(aux);
        CNOT(aux, target);

        task(target);
        Adjoint taskRef(target);

        CNOT(aux, target);
        H(aux);

        if CheckZero(target) {
            if CheckZero(aux) {
                task(target);
                DumpMachine();
                return true;
            }
        }

        Reset(aux);
        Reset(target);

        // Use DumpMachine to display actual vs desired state.
        task(target);
        DumpMachine();
        Reset(target);
        taskRef(target);
        DumpMachine();

        false
    }

}