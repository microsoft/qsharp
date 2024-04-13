namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation FourBitstringSuperposition_Reference (qs : Qubit[], bits : Bool[][]) : Unit is Adj {
        use anc = Qubit[2];
        ApplyToEachA(H, anc);

        for i in 0 .. 3 {
            for j in 0 .. Length(qs) - 1 {
                if bits[i][j] {
                    (ApplyControlledOnInt(i, X))(anc, qs[j]);
                }
            }
        }

        for i in 0 .. 3 {
            if i % 2 == 1 {
                (ApplyControlledOnBitString(bits[i], X))(qs, anc[0]);
            }
            if i / 2 == 1 {
                (ApplyControlledOnBitString(bits[i], X))(qs, anc[1]);
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        // cross-tests
        mutable bits = [[false, false], [false, true], [true, false], [true, true]];
        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            FourBitstringSuperposition(_, bits),
            ApplyToEachA(H, _),
            2
        ) {
            return false;
        }

        // random tests
        for N in 3 .. 10 {
            // generate 4 distinct numbers corresponding to the bit strings
            mutable numbers = [0, size = 4];

            repeat {
                mutable ok = true;
                for i in 0 .. 3 {
                    set numbers w/= i <- DrawRandomInt(0, 1 <<< N - 1);
                    for j in 0 .. i - 1 {
                        if numbers[i] == numbers[j] {
                            set ok = false;
                        }
                    }
                }
            }
            until (ok);

            // convert numbers to bit strings
            for i in 0 .. 3 {
                set bits w/= i <- IntAsBoolArray(numbers[i], N);
            }

            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                FourBitstringSuperposition(_, bits),
                FourBitstringSuperposition_Reference(_, bits),
                N
            ) {
                return false;
            }
        }
    }
}
