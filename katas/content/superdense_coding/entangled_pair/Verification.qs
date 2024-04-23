namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation CreateEntangledPair(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        CNOT(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.CreateEntangledPair,
            CreateEntangledPair,
            2);
    }
}
