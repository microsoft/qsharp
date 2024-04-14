namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation BellStateChange2_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(Kata.BellStateChange2, 
            BellStateChange2_Reference, 2)
    }
}
