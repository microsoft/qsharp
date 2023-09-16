namespace Kata.Verification {
    open Microsoft.Quantum.Convert;

    operation ArbitraryBitPattern_Oracle_Reference(x : Qubit[], y : Qubit, pattern : Bool[]) : Unit  is Adj + Ctl {
        ApplyControlledOnBitString(pattern, X, x, y);
    }

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..4 {
            for k in 0..((2^N)-1) {
                let pattern = IntAsBoolArray(k, N);

                let isCorrect = CheckTwoOraclesAreEqual(
                    N..N,
                    Kata.ArbitraryBitPattern_Oracle(_, _, pattern),
                    ArbitraryBitPattern_Oracle_Reference(_, _, pattern));
                if not isCorrect {
                    Message($"Failed on pattern {pattern}.");
                    return false;
                }
            }
        }
        Message("All tests passed.");
        true
    }

}
