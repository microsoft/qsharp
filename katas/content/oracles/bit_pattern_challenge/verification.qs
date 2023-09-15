namespace Kata.Verification {
    open Microsoft.Quantum.Convert;
    open Microsoft.Quantum.Arrays;

    operation ArbitraryBitPattern_Oracle_Challenge_Reference(x : Qubit[], pattern : Bool[]) : Unit is Adj + Ctl {
        within {
            for i in IndexRange(x) {
                if not pattern[i] {
                    X(x[i]);
                }
            }
        } apply {
            Controlled Z(Most(x), Tail(x));
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for N in 1..4 {
            for k in 0..((2^N)-1) {
                let pattern = IntAsBoolArray(k, N);

                let isCorrect = CheckOperationsEqualReferenced(
                    N,
                    Kata.ArbitraryBitPattern_Oracle_Challenge(_, pattern),
                    ArbitraryBitPattern_Oracle_Challenge_Reference(_, pattern));
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
