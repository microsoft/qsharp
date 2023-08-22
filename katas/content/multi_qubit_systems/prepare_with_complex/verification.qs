namespace Kata.Verification {
    operation PrepareWithComplex_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        H(qs[0]);
        H(qs[1]);
        S(qs[0]);
        T(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareWithComplex, PrepareWithComplex_Reference);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }
        return isCorrect;
    }
}