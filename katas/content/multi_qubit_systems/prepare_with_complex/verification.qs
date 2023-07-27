namespace Kata.Verification {

    operation PrepareState4_Reference(qs: Qubit[]): Unit is Adj+Ctl {
        H(qs[0]);
        H(qs[1]);
        S(qs[0]);
        T(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareState4, PrepareState4_Reference);
        if isCorrect {
            Message("All tests passed.");
        }
        return isCorrect;
    }

}
