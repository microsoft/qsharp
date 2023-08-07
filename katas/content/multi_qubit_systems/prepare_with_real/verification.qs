namespace Kata.Verification {
    operation PrepareState3_Reference(qs: Qubit[]): Unit is Adj+Ctl {
        H(qs[0]);
        X(qs[1]);
        H(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareState3, PrepareState3_Reference);
        if isCorrect {
            Message("All tests passed.");
        }
        return isCorrect;
    }
}