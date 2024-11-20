namespace Kata.Verification {
    import KatasUtils.*;

    operation PrepareWithReal_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.PrepareWithReal,
            PrepareWithReal_Reference,
            2
        )
    }
}
