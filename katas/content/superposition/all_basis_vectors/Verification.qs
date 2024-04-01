namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation AllBasisVectorsSuperposition_Reference (qs : Qubit[]) : Unit is Adj + Ctl {
        for q in qs {
            H(q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.AllBasisVectorsSuperposition,
            AllBasisVectorsSuperposition_Reference,
            2)
    }
}
