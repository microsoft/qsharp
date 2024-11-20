namespace Kata.Verification {
    import KatasUtils.*;

    operation PrepareSuperposition_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.PrepareSuperposition,
            PrepareSuperposition_Reference,
            2
        )
    }
}
