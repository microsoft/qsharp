// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {
    open Microsoft.Quantum.Intrinsic;
    open Microsoft.Quantum.Canon;
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Math;

    operation ThreeQuatersPiPhase_Reference (q : Qubit) : Unit is Adj+Ctl {
        body (...) {
            S(q);
            T(q);
        }
        adjoint (...) {
            Adjoint T(q);
            Adjoint S(q);
        }
    }

    operation VerifyTask5() : Bool {
        let task = ThreeQuatersPiPhase;
        let task_ref = ThreeQuatersPiPhase_Reference;

        use (aux, target) = (Qubit(), Qubit());
        H(aux);
        CNOT(aux, target);

        task(target);
        Adjoint task_ref(target);

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
        task_ref(target);
        DumpMachine();

        return false;
    }

}