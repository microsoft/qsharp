namespace Kata.Verification {
    operation PrepareState2_Reference(qs : Qubit[]): Unit is Adj+Ctl {
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareState2, PrepareState2_Reference);
        if isCorrect {
            Message("All tests passed.");
        }
        return isCorrect;
    }
}