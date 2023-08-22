namespace Kata.Verification {
    operation PrepareBasisState_Reference(qs : Qubit[]) : Unit is Adj + Ctl {
        X(qs[0]);
        X(qs[1]);
    }

    @EntryPoint()
    operation CheckSolution() : Bool {
        let isCorrect = AssertEqualOnZeroState(Kata.PrepareBasisState, PrepareBasisState_Reference);
        if isCorrect {
            Message("Correct!");
        } else {
            Message("Incorrect.");
        }
        return isCorrect;
    }
}