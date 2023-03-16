// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    operation PrepareRotatedState_Reference (alpha : Double, beta : Double, q : Qubit) : Unit is Adj+Ctl {
        body (...) {
            let phi = ArcTan2(beta, alpha);
            Rx(2.0 * phi, q);
        }
        adjoint (...) {
            let phi = ArcTan2(beta, alpha);
            Adjoint Rx(2.0 * phi, q);
        }
    }

    operation VerifyTask6() : Bool {
        let task = PrepareRotatedState;
        let task_ref = PrepareRotatedState_Reference;

        use (aux, target) = (Qubit(), Qubit());
        H(aux);
        CNOT(aux, target);

        task(Cos(1.0), Sin(1.0), target);
        Adjoint task_ref(Cos(1.0), Sin(1.0), target);

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
        task_ref(Cos(1.0), Sin(1.0), target);
        DumpMachine();

        return false;
    }

}