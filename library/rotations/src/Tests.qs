// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.Fact;
import Std.Math.HammingWeightI;

import HammingWeightPhasing.WithHammingWeight;

operation Main() : Unit {
    TestHammingWeight();
}

operation TestHammingWeight() : Unit {
    // exhaustive
    use qs = Qubit[4];

    for x in 0..2^Length(qs) - 1 {
        ApplyXorInPlace(x, qs);
        WithHammingWeight(qs, sum => {
            Fact(MeasureInteger(sum) == HammingWeightI(x), $"wrong Hamming weight computed for x = {x}");
        });
        ResetAll(qs);
    }
}
