namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Arrays;

    operation GHZ_State_Reference (qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        
        for q in Rest(qs) {
            CNOT(qs[0], q);
        }
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.GHZ_State,
            GHZ_State_Reference,
            2)
    }
}
