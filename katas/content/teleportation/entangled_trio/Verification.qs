namespace Kata.Verification {
    import KatasUtils.*;

    operation EntangleThreeQubits_Wrapper(qs : Qubit[]) : Unit is Adj {
        Kata.EntangleThreeQubits(qs[0], qs[1], qs[2]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            EntangleThreeQubits_Wrapper,
            EntangleThreeQubitsWrapper_Reference,
            3
        );

    }
}
