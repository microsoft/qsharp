// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Quantum.Kata.SingleQubitGates {

    operation ApplyY(q : Qubit) : Unit is Adj + Ctl {
        // Apply the Pauli Y operation.
        Y(q);
    }
}
