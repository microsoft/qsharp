// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Arrays.Enumerated, Std.Arrays.Most, Std.Arrays.Partitioned, Std.Arrays.Tail;
import Std.Convert.IntAsDouble;
import Std.Diagnostics.Fact;
import Std.Math.Floor, Std.Math.Lg;

operation HammingWeightPhasing(angle: Double, qs : Qubit[]) : Unit {
    WithHammingWeight(qs, (sum) => {
        for (i, sumQubit) in Enumerated(sum) {
            Rz(IntAsDouble(2^i) * angle, sumQubit);
        }
    });
}

internal operation WithHammingWeight(qs : Qubit[], action : Qubit[] => Unit) : Unit {
    let n = Length(qs);

    if n <= 1 {
        action(qs);
    } elif n == 2 {
        use sum = Qubit();

        within {
            AND(qs[0], qs[1], sum);
            CNOT(qs[0], qs[1]);
        } apply {
            action([qs[1], sum]);
        }
    } elif n == 3 {
        WithSum(qs[0], qs[1..1], qs[2..2], action);
    } else {
        let power = Floor(Lg(IntAsDouble(n - 1)));
        let split = Partitioned([n - 2^power, 2^power - 1], qs);
        Fact(Length(split) == 3 and Length(split[2]) == 1, $"Unexpected split for n = {n}");

        WithHammingWeight(split[0], (leftHW) => {
            WithHammingWeight(split[1], (rightHW) => {
                WithSum(split[2][0], leftHW, rightHW, action);
            });
        });
    }
}

internal operation Carry(carryIn : Qubit, x : Qubit, y : Qubit, carryOut : Qubit) : Unit is Adj {
    CNOT(carryIn, x);
    CNOT(carryIn, y);
    AND(x, y, carryOut);
    CNOT(carryIn, x);
    CNOT(x, y);
    CNOT(carryIn, carryOut);
}

internal operation WithSum(carry : Qubit, xs : Qubit[], ys : Qubit[], action : Qubit[] => Unit) : Unit {
    let n = Length(ys);
    Fact(Length(xs) <= n, "Length of xs must be less or equal to length of ys");
    Fact(n > 0, "Length must be at least 1");

    use carryOut = Qubit[n];
    let carryIn = [carry] + Most(carryOut);

    within {
        for i in 0..n-1 {
            if i < Length(xs) {
                Carry(carryIn[i], xs[i], ys[i], carryOut[i]);
            } else {
                CNOT(carryIn[i], ys[i]);
                AND(carryIn[i], ys[i], carryOut[i]);
                CNOT(carryIn[i], carryOut[i]);
            }
        }
    } apply {
        action(ys + [Tail(carryOut)]);
    }
}
