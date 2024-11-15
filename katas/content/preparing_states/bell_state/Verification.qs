namespace Kata.Verification {
    import KatasUtils.*;

    operation BellState_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.BellState,
            BellState_Reference,
            2
        )
    }
}
