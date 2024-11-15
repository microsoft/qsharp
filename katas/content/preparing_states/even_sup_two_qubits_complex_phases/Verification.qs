namespace Kata.Verification {
    import KatasUtils.*;

    operation AllBasisVectorsWithComplexPhases_TwoQubits_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        // Qubit 0 is taken into |+⟩ and then z-rotated into |-⟩.
        H(qs[0]);
        Z(qs[0]);

        // Qubit 1 is taken into |+⟩ and then z-rotated into |i⟩.
        H(qs[1]);
        S(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.AllBasisVectorsWithComplexPhases_TwoQubits,
            AllBasisVectorsWithComplexPhases_TwoQubits_Reference,
            2
        )
    }
}
