namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation PrepareBasisState_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        X(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckEqualOnZeroState(Kata.PrepareBasisState, PrepareBasisState_Reference)
    }
}
