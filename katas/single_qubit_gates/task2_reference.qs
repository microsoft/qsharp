// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {
    open Microsoft.Quantum.Diagnostics;

    operation GlobalPhaseIReference(q : Qubit) : Unit is Adj + Ctl {
        body ... {
            X(q);
            Z(q);
            Y(q);
        }
        adjoint ... {
            Y(q);
            Z(q);
            X(q);
        }
    }

    operation VerifyTask2() : Bool {
        let task = GlobalPhaseI;
        let taskRef = GlobalPhaseIReference;

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