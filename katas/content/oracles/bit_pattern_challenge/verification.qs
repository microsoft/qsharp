namespace Kata.Verification {
    open Microsoft.Quantum.Convert;

    // ------------------------------------------------------
    @EntryPoint()
    operation CheckSolution(): Bool {
        for N in 1..4 {
            for k in 0..((2^N)-1) {
                let pattern = IntAsBoolArray(k, N);

                let isCorrect = CheckOperationsEqualReferenced(
                    N,
                    Kata.ArbitraryBitPattern_Oracle_Challenge(_, pattern),
                    ArbitraryBitPattern_Oracle_Challenge(_, pattern));
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
