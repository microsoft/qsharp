namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation WState_PowerOfTwo_Reference (qs : Qubit[]) : Unit is Adj+Ctl {
        let N = Length(qs);

        if N == 1 {
            X(qs[0]);
        } else {
            let K = N / 2;
            use anc = Qubit();
            H(anc);

            ApplyControlledOnInt(0, WState_PowerOfTwo_Reference, [anc], qs[0 .. K - 1]);
            ApplyControlledOnInt(1, WState_PowerOfTwo_Reference, [anc], qs[K .. N - 1]);

            for i in K .. N - 1 {
                CNOT(qs[i], anc);
            }
        }
    }

        function FindFirstDiff (bits1 : Bool[], bits2 : Bool[]) : Int {
        for i in 0 .. Length(bits1) - 1 {
            if bits1[i] != bits2[i] {
                return i;
            }
        }
        return -1;
    }

    operation TwoBitstringSuperposition_Reference (qs : Qubit[], bits1 : Bool[], bits2 : Bool[]) : Unit is Adj + Ctl {
        // find the index of the first bit at which the bit strings are different
        let firstDiff = FindFirstDiff(bits1, bits2);

        // Hadamard corresponding qubit to create superposition
        H(qs[firstDiff]);

        // iterate through the bit strings again setting the final state of qubits
        for i in 0 .. Length(qs) - 1 {
            if bits1[i] == bits2[i] {
                // if two bits are the same, apply X or nothing
                if bits1[i] {
                    X(qs[i]);
                }
            } else {
                // if two bits are different, set their difference using CNOT
                if i > firstDiff {
                    CNOT(qs[firstDiff], qs[i]);
                    if bits1[i] != bits1[firstDiff] {
                        X(qs[i]);
                    }
                }
            }
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.WState_PowerOfTwo,
            ApplyToEachA(X, _),
            1) {
            return false;
        }

        if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.WState_PowerOfTwo,
            TwoBitstringSuperposition_Reference(_, [false, true], [true, false]),
            2) {
            return false;
        }

        for n in [4, 8, 16] {
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.WState_PowerOfTwo,
                WState_PowerOfTwo_Reference,
                n) {
                return false;
            }
        }

        return true;
    }
}
