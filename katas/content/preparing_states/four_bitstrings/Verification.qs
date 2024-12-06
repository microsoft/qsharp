namespace Kata.Verification {
    import Std.Convert.*;
    import KatasUtils.*;
    import Std.Math.*;
    import Std.Random.*;

    operation FourBitstringSuperposition_Reference(qs : Qubit[], bits : Bool[][]) : Unit is Adj {
        use anc = Qubit[2];
        ApplyToEachA(H, anc);

        for i in 0..3 {
            for j in 0..Length(qs) - 1 {
                if bits[i][j] {
                    ApplyControlledOnInt(i, X, anc, qs[j]);
                }
            }
        }

        for i in 0..3 {
            if i % 2 == 1 {
                ApplyControlledOnBitString(bits[i], X, qs, anc[0]);
            }
            if i / 2 == 1 {
                ApplyControlledOnBitString(bits[i], X, qs, anc[1]);
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        let bits = [[false, false], [false, true], [true, false], [true, true]];
        Message($"Testing for bits = {bits}...");
        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.FourBitstringSuperposition(_, bits),
            ApplyToEachA(H, _),
            2
        ) {
            return false;
        }

        let bitstrings = [
            [[false, true, false], [true, false, false], [false, false, true], [true, true, false]],
            [[true, false, false], [false, false, true], [false, true, false], [true, true, true]],
            [[false, false, false], [false, true, false], [true, true, false], [true, false, true]]
        ];

        for bitstring in bitstrings {
            Message($"Testing for bits = {bitstring}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.FourBitstringSuperposition(_, bitstring),
                FourBitstringSuperposition_Reference(_, bitstring),
                3
            ) {
                return false;
            }
        }

        return true;
    }
}
