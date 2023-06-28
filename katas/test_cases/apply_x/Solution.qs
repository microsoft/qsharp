// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Kata.Solution {
    open Microsoft.Quantum.Intrinsic;

    operation ApplyX(q : Qubit) : Unit is Adj + Ctl {
        X(q);
    }
}