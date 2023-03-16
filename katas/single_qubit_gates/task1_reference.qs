// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {
    open Microsoft.Quantum.Diagnostics;

    operation ApplyYReference(q : Qubit) : Unit is Adj + Ctl {
        body ... {
            Y(q);
        }
        adjoint self;
    }

    operation VerifyTask1() : Bool {
        let task = ApplyY;
        let taskRef = ApplyYReference;

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