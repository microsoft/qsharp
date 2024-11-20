namespace Kata.Verification {
    import KatasUtils.*;

    operation PrepareWithComplex_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
        S(qs[0]);
        T(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        CheckOperationsEquivalenceOnZeroStateWithFeedback(
            Kata.PrepareWithComplex,
            PrepareWithComplex_Reference,
            2
        )
    }
}
