namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation BellStateChange1_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(Kata.BellStateChange1, 
            BellStateChange1_Reference, 2)
    }
}
