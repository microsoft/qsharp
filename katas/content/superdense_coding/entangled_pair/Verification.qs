namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation CreateEntangledPair_Wrapper(qs : Qubit[]) : Unit is Adj {
        Kata.CreateEntangledPair(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            CreateEntangledPair_Wrapper,
            CreateEntangledPair_Reference,
            2
        );
    }
}
