// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

namespace Kata {
    open Microsoft.Quantum.Intrinsic;

    operation ApplyX(q : Qubit) : Unit is Adj + Ctl {
        X(q);
    }
}