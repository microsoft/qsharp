namespace Kata.Verification {
    open Microsoft.Quantum.Diagnostics;
    open Microsoft.Quantum.Katas;

    operation Entangle_Wrapper (qs : Qubit[]) : Unit is Adj {
        Kata.Entangle(qs[0], qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        
        return CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Entangle_Wrapper,
            EntangleWrapper_Reference,
            2
        );
        
    }
}
