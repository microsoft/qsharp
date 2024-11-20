namespace Kata.Verification {
    import KatasUtils.*;

    operation PrepareBasisState_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        X(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.PrepareBasisState,
            PrepareBasisState_Reference,
            2
        )
    }
}
