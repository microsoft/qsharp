namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Katas;

    operation ArbitraryBitPattern_Oracle_Reference(x : Qubit[], y : Qubit, pattern : Bool[]) : Unit  is Adj + Ctl {
        ApplyControlledOnBitString(pattern, X, x, y);
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1 .. 3 {
            for k in 0 .. 2^N - 1 {
                let pattern = IntAsBoolArray(k, N);

                let sol = ApplyOracle(_, Kata.ArbitraryBitPattern_Oracle(_, _, pattern));
                let ref = ApplyOracle(_, ArbitraryBitPattern_Oracle_Reference(_, _, pattern));
                let isCorrect = CheckOperationsEquivalenceStrict(sol, ref, N + 1);

                if not isCorrect {
                    Message("Incorrect.");
                    Message("Hint: examine how your solution transforms the given state and compare it with the expected " +
                        $"transformation for the {N}-bit oracle for the pattern {pattern}");
                    use initial = Qubit[N + 1];
                    PrepRandomState(initial[...N - 1]);
                    ShowQuantumStateComparison(initial, sol, ref);
                    ResetAll(initial);
                    return false;
                }
            }
        }
        Message("Correct!");
        true
    }

}
