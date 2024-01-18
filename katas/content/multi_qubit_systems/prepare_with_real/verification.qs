namespace Kata.Verification {
    open Microsoft.Quantum.Katas;

    operation PrepareWithReal_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckEqualOnZeroState(Kata.PrepareWithReal, PrepareWithReal_Reference)
    }
}
