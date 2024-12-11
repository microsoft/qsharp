namespace Kata.Verification {
    import KatasUtils.*;

    operation EvenOddNumbersSuperposition_Reference(qs : Qubit[], isEven : Bool) : Unit is Adj + Ctl {
        let N = Length(qs);

        for i in 0..N-2 {
            H(qs[i]);
        }

        if not isEven {
            X(qs[N-1]);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        for i in 1..5 {
            for boolVal in [false, true] {
                Message($"Testing {i} qubit(s) where isEven = {boolVal}...");
                if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                    Kata.EvenOddNumbersSuperposition(_, boolVal),
                    EvenOddNumbersSuperposition_Reference(_, boolVal),
                    i
                ) {
                    return false;
                }
            }
        }

        return true;
    }
}
