namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation AllStatesWithParitySuperposition_Reference (qs : Qubit[], parity : Int) : Unit is Adj + Ctl{
        // base of recursion: if N = 1, set the qubit to parity
        let N = Length(qs);
        if N == 1 {
            if parity == 1 {
                X(qs[0]);
            }
        } else {
            // split the first qubit into 0 and 1 (with equal amplitudes!)
            H(qs[0]);
            // prep 0 ⊗ state with the same parity and 1 ⊗ state with the opposite parity
            ApplyControlledOnInt(0, AllStatesWithParitySuperposition_Reference, qs[0 .. 0], (qs[1 ...], parity));
            ApplyControlledOnInt(1, AllStatesWithParitySuperposition_Reference, qs[0 .. 0], (qs[1 ...], 1 - parity));
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        Message($"Testing for N = 2...");
        for i in 1 .. 10 {
            for parity in 0 .. 1 {
                if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                    Kata.AllStatesWithParitySuperposition(_, parity),
                    AllStatesWithParitySuperposition_Reference(_, parity),
                    2
                ) {
                    return false;
                }
            }
        }

        for N in 3 .. 6 {
            Message($"Testing for N = {N}...");
            for parity in 0 .. 1 {
                if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                    Kata.AllStatesWithParitySuperposition(_, parity),
                    AllStatesWithParitySuperposition_Reference(_, parity),
                    N
                ) {
                    return false;
                }
            }
        }

        return true;
    }
}
