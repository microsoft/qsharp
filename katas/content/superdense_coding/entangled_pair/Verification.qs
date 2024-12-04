namespace Kata.Verification {
    import KatasUtils.*;

    operation CreateEntangledPair_Wrapper(qs : Qubit[]) : Unit is Adj {
        Kata.CreateEntangledPair(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {

        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            CreateEntangledPair_Wrapper,
            CreateEntangledPairWrapper_Reference,
            2
        );
    }
}
