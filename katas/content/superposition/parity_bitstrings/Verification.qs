namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation AllStatesWithParitySuperposition_Reference (qs : Qubit[], parity : Int) : Unit is Adj + Ctl {
        if Length(qs) == 1 {
            if parity == 1 {
                X(qs[0]);
            }
        } else {
            H(qs[0]);
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
