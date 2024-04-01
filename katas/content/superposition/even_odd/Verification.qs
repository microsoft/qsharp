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
        for boolVal in [false, true] {
            Message($"Testing isEven = {boolVal}...");
            if not CheckOperationsEquivalenceOnZeroStateWithFeedback(
                Kata.EvenOddNumbersSuperposition(_, boolVal),
                EvenOddNumbersSuperposition_Reference(_, boolVal),
                2) {
                return false;
            }
        }

        return true;
    }
}
