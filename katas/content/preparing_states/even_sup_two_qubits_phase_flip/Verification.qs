namespace Kata.Verification {
    import KatasUtils.*;

    operation AllBasisVectorsWithPhaseFlip_TwoQubits_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
        CZ(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.AllBasisVectorsWithPhaseFlip_TwoQubits,
            AllBasisVectorsWithPhaseFlip_TwoQubits_Reference,
            2
        )
    }
}
