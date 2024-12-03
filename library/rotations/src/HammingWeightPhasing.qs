// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

import Std.Arrays.Enumerated, Std.Arrays.Most, Std.Arrays.Partitioned, Std.Arrays.Tail;
import Std.Convert.IntAsDouble;
import Std.Diagnostics.Fact;
import Std.Math.BitSizeI, Std.Math.Floor, Std.Math.Lg, Std.Math.MaxI, Std.Math.MinI;

/// # Summary
/// Applies a Z-rotation (`Rz`) with given angle to each qubit in qs.
///
/// # Description
/// This implementation is based on Hamming-weight phasing to reduce the number
/// of rotation gates.  The technique was first presented in [1], and further
/// improved in [2] based on results in [3, 4].  Note, that the reduction of
/// rotation gates comes at a cost of additional qubits and additional quantum
/// operations to compute the Hamming-weight.
///
/// # Reference
/// - [1](https://arxiv.org/abs/1709.06648) "Halving the cost of quantum
///   addition", Craig Gidney.
/// - [2](https://arxiv.org/abs/2012.09238) "Early fault-tolerant simulations of
///   the Hubbard model", Earl T. Campbell.
/// - [3](https://cpsc.yale.edu/sites/default/files/files/tr1260.pdf) "The exact
///   multiplicative complexity of the Hamming weight function", Joan Boyar and
///   René Peralta.
/// - [4](https://arxiv.org/abs/1908.01609) "The role of multiplicative
///   complexity in compiling low T-count oracle circuits", Giulia Meuli,
///   Mathias Soeken, Earl Campbell, Martin Roetteler, Giovanni De Micheli.
operation HammingWeightPhasing(angle : Double, qs : Qubit[]) : Unit {
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
        let splitSize = 2^(BitSizeI(n - 1) - 1);
        let (leftLen, rightLen) = (n - splitSize, splitSize - 1);
        // handle corner case if n is power of 2; in that case the first
        // partition is longer than the second one, and we want to avoid that.
        let split = Partitioned([MinI(leftLen, rightLen), MaxI(leftLen, rightLen)], qs);
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
                // there is no corresponding bit in xs; this is a version of
                // Carry in which x == 0.
                CNOT(carryIn[i], ys[i]);
                AND(carryIn[i], ys[i], carryOut[i]);
                CNOT(carryIn[i], carryOut[i]);
            }
        }
    } apply {
        action(ys + [Tail(carryOut)]);
    }
}
