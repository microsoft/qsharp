namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;
    open Microsoft.Quantum.Random;

    operation FourBitstringSuperposition_Reference (qs : Qubit[], bits : Bool[][]) : Unit {
        use anc = Qubit[2];
        ApplyToEachA(H, anc);

        for i in 0 .. 3 {
            for j in 0 .. Length(qs) - 1 {
                if bits[i][j] {
                    ApplyControlledOnInt(i, X, anc, qs[j]);
                }
            }
        }

        for i in 0 .. 3 {
            if i % 2 == 1 {
                ApplyControlledOnBitString(bits[i], X, qs, anc[0]);
            }
            if i / 2 == 1 {
                ApplyControlledOnBitString(bits[i], X, qs, anc[1]);
            }
        }
    }

    operation WState_Arbitrary_Reference (qs : Qubit[]) : Unit is Adj + Ctl {

        let N = Length(qs);

        if N ==1 {
            // base case of recursion: |1⟩
            X(qs[0]);
        } else {
            // |W_N⟩ = |0⟩|W_(N-1)⟩ + |1⟩|0...0⟩
            // do a rotation on the first qubit to split it into |0⟩ and |1⟩ with proper weights
            // |0⟩ -> sqrt((N-1)/N) |0⟩ + 1/sqrt(N) |1⟩
            let theta = ArcSin(1.0 / Sqrt(IntAsDouble(N)));
            Ry(2.0 * theta, qs[0]);

            // do a zero-controlled W-state generation for qubits 1..N-1
            X(qs[0]);
            Controlled WState_Arbitrary_Reference(qs[0 .. 0], qs[1 .. N - 1]);
            X(qs[0]);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        // cross-tests
        mutable bits = [[false, false], [false, true], [true, false], [true, true]];
        Message($"Testing for bits = {bits}...");
        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.FourBitstringSuperposition(_, bits),
            ApplyToEachA(H, _),
            2
        ) {
            return false;
        }

        set bits = [[false, false, false, true], [false, false, true, false], [false, true, false, false], [true, false, false, false]];
        Message($"Testing for bits = {bits}...");
        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.FourBitstringSuperposition(_, bits),
            WState_Arbitrary_Reference(_),
            4
        ) {
            return false;
        }

        return true;
    }
}
