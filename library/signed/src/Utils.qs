// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

// Util functions for internal use by the signed integer library.

import Std.Math.Min;
import Std.Diagnostics.Fact;
import Std.Arrays.Head, Std.Arrays.Tail, Std.Arrays.Most, Std.Arrays.Rest;

operation AndLadder(controls : Qubit[], targets : Qubit[]) : Unit is Adj {
    Fact(Length(controls) == Length(targets), "The number of control qubits must match the number of target qubits.");
    let controls1 = [Head(controls)] + Most(targets);
    let controls2 = Rest(controls);
    for i in 0..Length(controls1) - 1 {
        AND(controls1[i], controls2[i], targets[i]);
    }
}