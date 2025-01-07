namespace Kata.Verification {
    import KatasUtils.*;

    operation ControlledRotation_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        Controlled H([qs[0]], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.ControlledRotation,
            ControlledRotation_Reference,
            2
        )
    }
}
