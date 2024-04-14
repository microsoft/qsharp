namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation BellStateChange3_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        Z(qs[0]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(Kata.BellStateChange3, 
            BellStateChange3_Reference, 2)
    }
}
