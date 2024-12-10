// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Diagnostics.CheckOperationsAreEqual, Std.Diagnostics.Fact;
import Std.Math.HammingWeightI, Std.Math.PI;

import HammingWeightPhasing.HammingWeightPhasing, HammingWeightPhasing.WithHammingWeight;

operation Main() : Unit {
    TestHammingWeight();
    TestPhasing();
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

    // some explicit cases
    for (width, number) in [
        (1, 1),
        (2, 0),
        (2, 3),
        (8, 10),
        (7, 99)
    ] {
        use qs = Qubit[width];

        ApplyXorInPlace(number, qs);
        WithHammingWeight(qs, sum => {
            Fact(MeasureInteger(sum) == HammingWeightI(number), $"wrong Hamming weight computed for number = {number}");
        });
        ResetAll(qs);
    }
}

operation TestPhasing() : Unit {
    for theta in [1.0, 2.0, 0.0, -0.5, 5.0 * PI()] {
        for numQubits in 1..6 {
            Fact(CheckOperationsAreEqual(numQubits, qs => HammingWeightPhasing(theta, qs), qs => ApplyToEachA(Rz(theta, _), qs)), "Operations are not equal");
        }
    }
}
