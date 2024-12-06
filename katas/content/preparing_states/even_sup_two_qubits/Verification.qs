namespace Kata.Verification {
    import KatasUtils.*;

    operation AllBasisVectors_TwoQubits_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.AllBasisVectors_TwoQubits,
            AllBasisVectors_TwoQubits_Reference,
            2
        )
    }
}
