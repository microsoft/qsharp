// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    operation PrepareRotatedStateReference(alpha : Double, beta : Double, q : Qubit) : Unit is Adj + Ctl {
        body ... {
            let phi = ArcTan2(beta, alpha);
            Rx(2.0 * phi, q);
        }
        adjoint ... {
            let phi = ArcTan2(beta, alpha);
            Adjoint Rx(2.0 * phi, q);
        }
    }

    operation VerifyTask6() : Bool {
        let task = PrepareRotatedState;
        let taskRef = PrepareRotatedStateReference;

        use (aux, target) = (Qubit(), Qubit());
        H(aux);
        CNOT(aux, target);

        task(Cos(1.0), Sin(1.0), target);
        Adjoint taskRef(Cos(1.0), Sin(1.0), target);

        CNOT(aux, target);
        H(aux);

        if CheckZero(target) {
            if CheckZero(aux) {
                task(Cos(1.0), Sin(1.0), target);
                DumpMachine();
                return true;
            }
        }

        Reset(aux);
        Reset(target);

        // Use DumpMachine to display actual vs desired state.
        task(Cos(1.0), Sin(1.0), target);
        DumpMachine();
        Reset(target);
        taskRef(Cos(1.0), Sin(1.0), target);
        DumpMachine();

        false
    }

}