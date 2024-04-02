namespace Kata.Verification {
    open Microsoft.Quantum.Katas;
    open Microsoft.Quantum.Math;


    operation BasisChange_Reference(q : Qubit) : Unit is Adj + Ctl {
        H(q);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            ApplyToFirstCA(Kata.BasisChange, _), 
            ApplyToFirstCA(BasisChange_Reference, _),
            1)
    }
        
}