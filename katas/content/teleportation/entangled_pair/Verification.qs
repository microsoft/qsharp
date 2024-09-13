namespace Kata.Verification {
    import Std.Diagnostics.*;
    import Std.Katas.*;

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
