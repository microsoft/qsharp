namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

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
        for q in 1 .. 5 {
            for boolVal in [false, true] {
                Message($"Testing {q} qubit(s) where isEven = {boolVal}...");
                if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                    Kata.EvenOddNumbersSuperposition(_, boolVal),
                    EvenOddNumbersSuperposition_Reference(_, boolVal),
                    q) {
                    return false;
                }
            }
        }

        return true;
    }
}
