namespace Kata.Verification {

    operation PrepareState1_Reference(qs: Qubit[]): Unit is Adj+Ctl {
        X(qs[0]);
        X(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution(): Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareState1, PrepareState1_Reference);
        if isCorrect {
            Message("All tests passed.");
        }
        return isCorrect;
    }

}
