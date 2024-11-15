namespace Kata.Verification {
    import KatasUtils.*;

    operation PlusState_Reference(q : Qubit) : Unit is Adj + Ctl {
        H(q);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            ApplyToFirstCA(Kata.PlusState, _),
            ApplyToFirstCA(PlusState_Reference, _),
            1
        )
    }
}
